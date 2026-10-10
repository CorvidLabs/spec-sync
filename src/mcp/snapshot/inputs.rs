use super::super::MAX_PROJECT_FILE_BYTES;
use super::super::capabilities::SnapshotEntryIdentity;
use super::super::capabilities::open_snapshot_configuration_directory;
use super::super::capabilities::read_only_nofollow_options;
use super::super::capabilities::snapshot_file_identity;
use super::super::capabilities::snapshot_metadata_identity;
use super::super::capabilities::snapshot_metadata_is_link;
use super::super::manifest_paths::normalize_snapshot_input;
#[cfg(debug_assertions)]
use super::super::test_barriers::wait_for_mcp_snapshot_file_test_barrier;
use super::SnapshotBudget;
use crate::config::parse_config_content_checked;
use cap_std::fs::Dir;
use serde_json::Value;
use std::io;
use std::io::Read;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

pub(super) fn read_capability_text_if_exists(
    source: &Dir,
    relative: &Path,
    budget: &mut SnapshotBudget,
) -> Result<Option<String>, String> {
    read_capability_text_if_exists_with_limit_and_hook(
        source,
        relative,
        budget,
        MAX_PROJECT_FILE_BYTES,
        || {},
    )
}

#[cfg(test)]
fn read_capability_text_if_exists_with_hook(
    source: &Dir,
    relative: &Path,
    budget: &mut SnapshotBudget,
    after_retained_open: impl FnOnce(),
) -> Result<Option<String>, String> {
    read_capability_text_if_exists_with_limit_and_hook(
        source,
        relative,
        budget,
        MAX_PROJECT_FILE_BYTES,
        after_retained_open,
    )
}

pub(super) fn read_capability_text_if_exists_with_limit(
    source: &Dir,
    relative: &Path,
    budget: &mut SnapshotBudget,
    max_bytes: u64,
) -> Result<Option<String>, String> {
    read_capability_text_if_exists_with_limit_and_hook(source, relative, budget, max_bytes, || {})
}

fn read_capability_text_if_exists_with_limit_and_hook(
    source: &Dir,
    relative: &Path,
    budget: &mut SnapshotBudget,
    max_bytes: u64,
    after_retained_open: impl FnOnce(),
) -> Result<Option<String>, String> {
    let Some(bytes) = read_retained_snapshot_file_if_exists_with_hook(
        source,
        relative,
        relative,
        "MCP snapshot manifest",
        max_bytes,
        || {
            after_retained_open();
            Ok(())
        },
    )?
    else {
        return Ok(None);
    };
    let normalized = normalize_snapshot_input(relative);
    budget.charge_file(&normalized, bytes.len() as u64)?;
    let content = String::from_utf8(bytes.clone()).map_err(|_| {
        format!(
            "MCP snapshot manifest is not valid UTF-8: {}",
            relative.display()
        )
    })?;
    budget.preloaded_files.insert(normalized, bytes);
    Ok(Some(content))
}

fn read_retained_snapshot_file_if_exists_with_hook(
    source: &Dir,
    relative: &Path,
    diagnostic_relative: &Path,
    label: &str,
    max_bytes: u64,
    after_retained_open: impl FnOnce() -> Result<(), String>,
) -> Result<Option<Vec<u8>>, String> {
    let before = match source.symlink_metadata(relative) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "Cannot inspect {label} {}: {error}",
                diagnostic_relative.display()
            ));
        }
    };
    if snapshot_metadata_is_link(&before) || !before.is_file() {
        return Err(format!(
            "{label} must be a regular file and must not be a symlink or reparse point: {}",
            diagnostic_relative.display()
        ));
    }
    let expected_identity = snapshot_metadata_identity(&before).map_err(|error| {
        format!(
            "Cannot identify {label} {}: {error}",
            diagnostic_relative.display()
        )
    })?;

    let options = read_only_nofollow_options();
    let mut file = source.open_with(relative, &options).map_err(|error| {
        format!(
            "Cannot open {label} {} as a no-follow, non-blocking regular file: {error}",
            diagnostic_relative.display()
        )
    })?;
    let opened_metadata = file.metadata().map_err(|error| {
        format!(
            "Cannot inspect opened {label} {}: {error}",
            diagnostic_relative.display()
        )
    })?;
    if snapshot_metadata_is_link(&opened_metadata) || !opened_metadata.is_file() {
        return Err(format!(
            "{label} must be a regular file and must not be a symlink or reparse point: {}",
            diagnostic_relative.display()
        ));
    }
    let opened_identity = snapshot_file_identity(&file).map_err(|error| {
        format!(
            "Cannot identify opened {label} {}: {error}",
            diagnostic_relative.display()
        )
    })?;
    if opened_identity != expected_identity {
        return Err(format!(
            "{label} changed during inspection: {}",
            diagnostic_relative.display()
        ));
    }

    after_retained_open()?;
    let after_open = source.symlink_metadata(relative).map_err(|error| {
        format!(
            "Cannot re-inspect {label} {}: {error}",
            diagnostic_relative.display()
        )
    })?;
    if snapshot_metadata_is_link(&after_open)
        || !after_open.is_file()
        || snapshot_metadata_identity(&after_open).ok() != Some(expected_identity)
        || snapshot_file_identity(&file).ok() != Some(opened_identity)
    {
        return Err(format!(
            "{label} changed during inspection: {}",
            diagnostic_relative.display()
        ));
    }

    let mut bytes = Vec::with_capacity(before.len().min(max_bytes) as usize);
    Read::by_ref(&mut file)
        .take(max_bytes + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| {
            format!(
                "Cannot read {label} {}: {error}",
                diagnostic_relative.display()
            )
        })?;
    if bytes.len() as u64 > max_bytes {
        return Err(format!(
            "{label} exceeds the {} MiB per-file limit: {}",
            max_bytes / (1024 * 1024),
            diagnostic_relative.display()
        ));
    }

    let after_read = source.symlink_metadata(relative).map_err(|error| {
        format!(
            "Cannot re-inspect {label} {} after reading: {error}",
            diagnostic_relative.display()
        )
    })?;
    if snapshot_metadata_is_link(&after_read)
        || !after_read.is_file()
        || snapshot_metadata_identity(&after_read).ok() != Some(expected_identity)
        || snapshot_file_identity(&file).ok() != Some(opened_identity)
    {
        return Err(format!(
            "{label} changed while it was being read: {}",
            diagnostic_relative.display()
        ));
    }
    Ok(Some(bytes))
}

pub(super) fn copy_snapshot_configuration(
    source: &Dir,
    destination: &Dir,
    destination_root: &Path,
    budget: &mut SnapshotBudget,
) -> Result<(), String> {
    for relative in [
        ".specsync/config.toml",
        ".specsync/config.json",
        ".specsync.toml",
        "specsync.json",
    ] {
        let path = Path::new(relative);
        let Some(bytes) = read_snapshot_configuration(source, path)? else {
            continue;
        };
        budget.charge_file(path, bytes.len() as u64)?;
        validate_snapshot_configuration(destination_root, relative, &bytes)?;
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            destination.create_dir_all(parent).map_err(|error| {
                format!("Cannot create MCP snapshot configuration directory: {error}")
            })?;
        }
        destination.write(path, bytes).map_err(|error| {
            format!("Cannot copy MCP configuration {relative} into the snapshot: {error}")
        })?;
        budget.copied_paths.insert(path.to_path_buf());
        break;
    }
    Ok(())
}

fn revalidate_snapshot_configuration_parents(
    source: &Dir,
    parents: &[(PathBuf, SnapshotEntryIdentity)],
) -> Result<(), String> {
    let mut parent = source
        .try_clone()
        .map_err(|error| format!("Cannot clone MCP server root capability: {error}"))?;
    for (relative, expected) in parents {
        let Some(name) = relative.file_name() else {
            return Err(format!(
                "Selected MCP configuration parent path is invalid: {}",
                relative.display()
            ));
        };
        let Some((directory, observed)) =
            open_snapshot_configuration_directory(&parent, name, relative)?
        else {
            return Err(format!(
                "Selected MCP configuration directory changed while the configuration was being read: {}",
                relative.display()
            ));
        };
        if &observed != expected {
            return Err(format!(
                "Selected MCP configuration directory changed while the configuration was being read: {}",
                relative.display()
            ));
        }
        parent = directory;
    }
    Ok(())
}

fn read_snapshot_configuration(source: &Dir, relative: &Path) -> Result<Option<Vec<u8>>, String> {
    read_snapshot_configuration_with_hook(source, relative, || {})
}

fn read_snapshot_configuration_with_hook(
    source: &Dir,
    relative: &Path,
    after_retained_open: impl FnOnce(),
) -> Result<Option<Vec<u8>>, String> {
    let components = relative.components().collect::<Vec<_>>();
    let Some((file_name, parent_components)) = components.split_last() else {
        return Err("Selected MCP configuration path is empty".to_string());
    };
    let Component::Normal(file_name) = file_name else {
        return Err(format!(
            "Selected MCP configuration path is not project-relative: {}",
            relative.display()
        ));
    };

    let mut parent = source
        .try_clone()
        .map_err(|error| format!("Cannot clone MCP server root capability: {error}"))?;
    let mut traversed = PathBuf::new();
    let mut parents = Vec::new();
    for component in parent_components {
        let Component::Normal(name) = component else {
            return Err(format!(
                "Selected MCP configuration path is not project-relative: {}",
                relative.display()
            ));
        };
        traversed.push(name);
        parent = match open_snapshot_configuration_directory(&parent, name, &traversed)? {
            Some((directory, identity)) => {
                parents.push((traversed.clone(), identity));
                directory
            }
            None => return Ok(None),
        };
    }

    let bytes = read_retained_snapshot_file_if_exists_with_hook(
        &parent,
        Path::new(file_name),
        relative,
        "Selected MCP configuration",
        MAX_PROJECT_FILE_BYTES,
        || {
            after_retained_open();
            Ok(())
        },
    )?;
    revalidate_snapshot_configuration_parents(source, &parents)?;
    Ok(bytes)
}

fn validate_snapshot_configuration(
    destination_root: &Path,
    relative: &str,
    bytes: &[u8],
) -> Result<(), String> {
    let content = std::str::from_utf8(bytes)
        .map_err(|_| format!("Selected MCP configuration {relative} is not valid UTF-8"))?;
    let content = content.trim_start_matches('\u{feff}');
    if relative.ends_with(".toml") {
        let value = toml::from_str::<toml::Value>(content).map_err(|error| {
            format!("Selected MCP configuration {relative} is malformed TOML: {error}")
        })?;
        validate_toml_snapshot_path_selectors(relative, &value)?;
    } else {
        let value = serde_json::from_str::<Value>(content).map_err(|error| {
            format!("Selected MCP configuration {relative} is malformed JSON: {error}")
        })?;
        if !value.is_object() {
            return Err(format!(
                "Selected MCP configuration {relative} is malformed JSON: root must be an object"
            ));
        }
        validate_json_snapshot_path_selectors(relative, &value)?;
    }
    parse_config_content_checked(&destination_root.join(relative), content, destination_root)
        .map(|_| ())
        .map_err(|error| {
            let format = if relative.ends_with(".toml") {
                "TOML"
            } else {
                "JSON"
            };
            format!("Selected MCP configuration {relative} is malformed {format}: {error}")
        })
}

fn validate_json_snapshot_path_selectors(relative: &str, config: &Value) -> Result<(), String> {
    if config
        .get("specsDir")
        .is_some_and(|value| !value.is_string())
    {
        return Err(format!(
            "Selected MCP configuration {relative} path selector `specsDir` must be a string"
        ));
    }
    if config.get("sourceDirs").is_some_and(|value| {
        !value
            .as_array()
            .is_some_and(|entries| entries.iter().all(Value::is_string))
    }) {
        return Err(format!(
            "Selected MCP configuration {relative} path selector `sourceDirs` must be an array of strings"
        ));
    }
    Ok(())
}

fn validate_toml_snapshot_path_selectors(
    relative: &str,
    config: &toml::Value,
) -> Result<(), String> {
    if config.get("specs_dir").is_some_and(|value| !value.is_str()) {
        return Err(format!(
            "Selected MCP configuration {relative} path selector `specs_dir` must be a string"
        ));
    }
    if config.get("source_dirs").is_some_and(|value| {
        !value
            .as_array()
            .is_some_and(|entries| entries.iter().all(toml::Value::is_str))
    }) {
        return Err(format!(
            "Selected MCP configuration {relative} path selector `source_dirs` must be an array of strings"
        ));
    }
    Ok(())
}

pub(super) fn read_snapshot_project_file_from_directory(
    source: &Dir,
    relative: &Path,
    diagnostic_relative: &Path,
) -> Result<Vec<u8>, String> {
    read_snapshot_project_file_from_directory_with_result_hook(
        source,
        relative,
        diagnostic_relative,
        || {
            #[cfg(debug_assertions)]
            wait_for_mcp_snapshot_file_test_barrier(diagnostic_relative)?;
            Ok(())
        },
    )
}

#[cfg(all(test, unix))]
fn read_snapshot_project_file_with_hook(
    source: &Dir,
    relative: &Path,
    after_retained_open: impl FnOnce(),
) -> Result<Vec<u8>, String> {
    read_snapshot_project_file_with_result_hook(source, relative, || {
        after_retained_open();
        Ok(())
    })
}

#[cfg(all(test, unix))]
fn read_snapshot_project_file_with_result_hook(
    source: &Dir,
    relative: &Path,
    after_retained_open: impl FnOnce() -> Result<(), String>,
) -> Result<Vec<u8>, String> {
    read_snapshot_project_file_from_directory_with_result_hook(
        source,
        relative,
        relative,
        after_retained_open,
    )
}

fn read_snapshot_project_file_from_directory_with_result_hook(
    source: &Dir,
    relative: &Path,
    diagnostic_relative: &Path,
    after_retained_open: impl FnOnce() -> Result<(), String>,
) -> Result<Vec<u8>, String> {
    read_retained_snapshot_file_if_exists_with_hook(
        source,
        relative,
        diagnostic_relative,
        "MCP project input",
        MAX_PROJECT_FILE_BYTES,
        after_retained_open,
    )?
    .ok_or_else(|| {
        format!(
            "MCP project input changed during inspection: {}",
            diagnostic_relative.display()
        )
    })
}

#[cfg(test)]
mod tests;
