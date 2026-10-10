use super::super::MAX_PROJECT_FILE_BYTES;
use super::super::capabilities::SnapshotEntryIdentity;
use super::super::capabilities::directory_identity;
use super::super::capabilities::snapshot_metadata_identity;
use super::super::capabilities::snapshot_metadata_is_link;
use super::super::manifest_paths::normalize_snapshot_input;
#[cfg(debug_assertions)]
use super::super::test_barriers::wait_for_mcp_snapshot_directory_test_barrier;
use super::SnapshotBudget;
use super::inputs::read_snapshot_project_file_from_directory;
use cap_std::fs::Dir;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;

const SNAPSHOT_IGNORED_DIRS: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    "node_modules",
    "target",
    "vendor",
    ".next",
    ".nuxt",
    ".output",
    ".cache",
    ".turbo",
    "__pycache__",
    ".mypy_cache",
    ".pytest_cache",
    ".tox",
    ".venv",
    "venv",
    ".dart_tool",
    ".gradle",
    "Pods",
    "obj",
];

fn snapshot_input_overlaps(input: &Path, child: &Path) -> bool {
    let input = normalize_snapshot_input(input);
    input.as_os_str().is_empty() || input.starts_with(child) || child.starts_with(input)
}

struct RetainedSnapshotDirectory {
    directory: Dir,
    identity: SnapshotEntryIdentity,
}

struct SnapshotDirectoryEntry {
    name: std::ffi::OsString,
    child: PathBuf,
    directory_identity: Option<SnapshotEntryIdentity>,
}

pub(super) fn copy_snapshot_directory(
    source: &Dir,
    destination: &Dir,
    relative: &Path,
    configured_exclusions: &HashSet<String>,
    configured_inputs: &[PathBuf],
    budget: &mut SnapshotBudget,
) -> Result<(), String> {
    copy_snapshot_directory_with_enumeration_hook(
        source,
        destination,
        relative,
        configured_exclusions,
        configured_inputs,
        budget,
        &mut |enumerated| {
            #[cfg(debug_assertions)]
            wait_for_mcp_snapshot_directory_test_barrier(enumerated)?;
            #[cfg(not(debug_assertions))]
            let _ = enumerated;
            Ok(())
        },
    )
}

#[cfg(all(test, unix))]
fn copy_snapshot_directory_with_hook(
    source: &Dir,
    destination: &Dir,
    relative: &Path,
    configured_exclusions: &HashSet<String>,
    configured_inputs: &[PathBuf],
    budget: &mut SnapshotBudget,
    mut after_enumeration: impl FnMut(&Path) -> Result<(), String>,
) -> Result<(), String> {
    copy_snapshot_directory_with_enumeration_hook(
        source,
        destination,
        relative,
        configured_exclusions,
        configured_inputs,
        budget,
        &mut after_enumeration,
    )
}

fn copy_snapshot_directory_with_enumeration_hook(
    source: &Dir,
    destination: &Dir,
    relative: &Path,
    configured_exclusions: &HashSet<String>,
    configured_inputs: &[PathBuf],
    budget: &mut SnapshotBudget,
    after_enumeration: &mut impl FnMut(&Path) -> Result<(), String>,
) -> Result<(), String> {
    destination.create_dir_all(relative).map_err(|error| {
        format!(
            "Cannot create MCP snapshot directory {}: {error}",
            relative.display()
        )
    })?;
    let entries = source.read_dir(".").map_err(|error| {
        format!(
            "Cannot read MCP project directory {} through its root capability: {error}",
            relative.display()
        )
    })?;

    let mut retained_entries = Vec::new();
    for entry in entries {
        budget.charge_entries(1, "MCP project snapshot")?;
        let entry = entry.map_err(|error| {
            format!(
                "Cannot inspect MCP project entry beneath {}: {error}",
                relative.display()
            )
        })?;
        let child = if relative == Path::new(".") {
            PathBuf::from(entry.file_name())
        } else {
            relative.join(entry.file_name())
        };
        let name = entry.file_name();
        if name
            .to_str()
            .is_some_and(|name| name.eq_ignore_ascii_case(".git"))
        {
            continue;
        }
        let metadata = source.symlink_metadata(&name).map_err(|error| {
            format!(
                "Cannot inspect MCP project input type {}: {error}",
                child.display()
            )
        })?;
        let file_type = metadata.file_type();
        if file_type.is_dir() || file_type.is_symlink() {
            let is_configured_input = configured_inputs
                .iter()
                .any(|input| snapshot_input_overlaps(input, &child));
            let has_direct_configured_input = configured_inputs.iter().any(|input| {
                let input = normalize_snapshot_input(input);
                !input.as_os_str().is_empty() && input.starts_with(&child)
            });
            let is_fixed_ignored = name
                .to_str()
                .is_some_and(|name| SNAPSHOT_IGNORED_DIRS.contains(&name));
            let is_configured_excluded = name
                .to_str()
                .is_some_and(|name| configured_exclusions.contains(name));
            if (is_configured_excluded && !has_direct_configured_input)
                || (is_fixed_ignored && !is_configured_input)
            {
                continue;
            }
        }

        if budget.copied_paths.contains(&child) {
            continue;
        }

        let directory_identity = if file_type.is_dir() {
            Some(snapshot_metadata_identity(&metadata).map_err(|error| {
                format!(
                    "Cannot identify MCP project directory {} during enumeration: {error}",
                    child.display()
                )
            })?)
        } else {
            None
        };
        retained_entries.push(SnapshotDirectoryEntry {
            name,
            child,
            directory_identity,
        });
    }

    after_enumeration(relative)?;

    for entry in retained_entries {
        if let Some(expected_identity) = entry.directory_identity {
            let retained =
                retain_snapshot_directory(source, &entry.name, &entry.child, expected_identity)?;
            verify_retained_snapshot_directory(source, &entry.name, &entry.child, &retained)?;
            copy_snapshot_directory_with_enumeration_hook(
                &retained.directory,
                destination,
                &entry.child,
                configured_exclusions,
                configured_inputs,
                budget,
                after_enumeration,
            )?;
            verify_retained_snapshot_directory(source, &entry.name, &entry.child, &retained)?;
            continue;
        }

        let metadata = source.symlink_metadata(&entry.name).map_err(|error| {
            format!(
                "Cannot inspect MCP project input {} through its retained parent capability: {error}",
                entry.child.display()
            )
        })?;
        if snapshot_metadata_is_link(&metadata) || !metadata.is_file() {
            return Err(format!(
                "MCP project input must be a regular file or directory and must not be a symlink or reparse point: {}",
                entry.child.display()
            ));
        }
        if metadata.len() > MAX_PROJECT_FILE_BYTES {
            return Err(format!(
                "MCP project input exceeds the {} MiB per-file limit: {}",
                MAX_PROJECT_FILE_BYTES / (1024 * 1024),
                entry.child.display()
            ));
        }
        let bytes = read_snapshot_project_file_from_directory(
            source,
            Path::new(&entry.name),
            &entry.child,
        )?;
        budget.charge_file(&entry.child, bytes.len() as u64)?;
        destination.write(&entry.child, bytes).map_err(|error| {
            format!(
                "Cannot copy MCP project input {} into the bounded snapshot: {error}",
                entry.child.display()
            )
        })?;
        budget.copied_paths.insert(entry.child);
    }

    Ok(())
}

fn retain_snapshot_directory(
    parent: &Dir,
    name: &OsStr,
    relative: &Path,
    expected_identity: SnapshotEntryIdentity,
) -> Result<RetainedSnapshotDirectory, String> {
    let before = parent.symlink_metadata(name).map_err(|error| {
        format!(
            "Cannot inspect MCP project directory {} through its retained parent capability: {error}",
            relative.display()
        )
    })?;
    if snapshot_metadata_is_link(&before) || !before.is_dir() {
        return Err(format!(
            "MCP project input must be a regular directory and must not be a symlink or reparse point: {}",
            relative.display()
        ));
    }
    let expected = snapshot_metadata_identity(&before).map_err(|error| {
        format!(
            "Cannot identify MCP project directory {}: {error}",
            relative.display()
        )
    })?;
    if expected != expected_identity {
        return Err(format!(
            "MCP project directory changed during snapshot traversal: {}",
            relative.display()
        ));
    }
    let directory = parent.open_dir(name).map_err(|error| {
        format!(
            "Cannot open MCP project directory {} through its retained parent capability: {error}",
            relative.display()
        )
    })?;
    let opened = directory_identity(&directory).map_err(|error| {
        format!(
            "Cannot identify opened MCP project directory {}: {error}",
            relative.display()
        )
    })?;
    let retained = RetainedSnapshotDirectory {
        directory,
        identity: expected,
    };
    if opened != expected {
        return Err(format!(
            "MCP project directory changed during enumeration: {}",
            relative.display()
        ));
    }
    verify_retained_snapshot_directory(parent, name, relative, &retained)?;
    Ok(retained)
}

fn verify_retained_snapshot_directory(
    parent: &Dir,
    name: &OsStr,
    relative: &Path,
    retained: &RetainedSnapshotDirectory,
) -> Result<(), String> {
    let current = parent.symlink_metadata(name).map_err(|error| {
        format!(
            "Cannot re-inspect MCP project directory {} through its retained parent capability: {error}",
            relative.display()
        )
    })?;
    if snapshot_metadata_is_link(&current)
        || !current.is_dir()
        || snapshot_metadata_identity(&current).ok() != Some(retained.identity)
        || directory_identity(&retained.directory).ok() != Some(retained.identity)
    {
        return Err(format!(
            "MCP project directory changed during snapshot traversal: {}",
            relative.display()
        ));
    }
    Ok(())
}

#[cfg(all(test, unix))]
mod tests;
