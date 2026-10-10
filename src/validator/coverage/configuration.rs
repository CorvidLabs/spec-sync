use super::CoverageTraversalBudget;
use super::access::{
    coverage_metadata_is_link, open_retained_coverage_directory, read_coverage_file,
    verify_retained_coverage_directory,
};
use super::sources::retained_source_dirs;
use crate::config::{CONFIG_PATH_CANDIDATES, parse_config_content_checked_with_source_dirs};
use crate::types::SpecSyncConfig;
use cap_std::fs::Dir;
use std::io;
use std::path::Path;

pub(super) fn retained_config(
    project: &Dir,
    root: &Path,
    budget: &mut CoverageTraversalBudget,
) -> Result<SpecSyncConfig, String> {
    retained_config_with_hook(project, root, budget, |_| {})
}

fn retained_config_with_hook<BeforeRead>(
    project: &Dir,
    root: &Path,
    budget: &mut CoverageTraversalBudget,
    mut before_read: BeforeRead,
) -> Result<SpecSyncConfig, String>
where
    BeforeRead: FnMut(&Path),
{
    for configured in CONFIG_PATH_CANDIDATES {
        let relative = Path::new(configured);
        let parent = relative.parent().unwrap_or_else(|| Path::new(""));
        let Some(retained_directory) = open_retained_coverage_directory(project, parent)? else {
            continue;
        };
        let directory = &retained_directory.directory;
        let name = relative.file_name().ok_or_else(|| {
            format!(
                "Configuration path {} has no terminal filename",
                relative.display()
            )
        })?;
        let metadata = match directory.symlink_metadata(name) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(format!(
                    "Cannot inspect retained configuration file {}: {error}",
                    relative.display()
                ));
            }
        };
        if coverage_metadata_is_link(&metadata) || !metadata.is_file() {
            return Err(format!(
                "Configuration file {} must be a regular file beneath the retained project root",
                relative.display()
            ));
        }
        verify_retained_coverage_directory(project, &retained_directory)?;
        before_read(relative);
        let bytes = read_coverage_file(
            directory,
            name,
            relative,
            budget.limits.max_file_bytes,
            budget.remaining_bytes(),
            budget.limits.max_input_bytes,
        )?;
        verify_retained_coverage_directory(project, &retained_directory)?;
        budget.charge_input_file(relative, bytes.len() as u64)?;
        let content = match std::str::from_utf8(&bytes) {
            Ok(content) => content,
            Err(_) => {
                eprintln!(
                    "Warning: config file {} exists but could not be read (not valid UTF-8 or unreadable); \
                     using built-in defaults — its settings (enforcement, required sections, etc.) are NOT applied",
                    root.join(relative).display()
                );
                let source_dirs = retained_source_dirs(project, root, budget)?;
                return Ok(SpecSyncConfig {
                    source_dirs,
                    config_path: Some(root.join(relative)),
                    // The project configured rules that are now NOT in effect.
                    // Warning on stderr and carrying on meant stdout reported
                    // `✓ All required sections present` over a section list that
                    // had been thrown away (#570).
                    load_error: Some(format!(
                        "config file {} exists but could not be loaded; built-in defaults are in use",
                        root.join(relative).display()
                    )),
                    ..SpecSyncConfig::default()
                });
            }
        };
        let parsed = parse_config_content_checked_with_source_dirs(
            &root.join(relative),
            content,
            root,
            Some(Vec::new()),
        );
        let mut config = match parsed {
            Ok(config) => config,
            Err(error) => {
                eprintln!(
                    "Warning: failed to parse config file {}: {error}; using built-in defaults",
                    root.join(relative).display()
                );
                let source_dirs = retained_source_dirs(project, root, budget)?;
                return Ok(SpecSyncConfig {
                    source_dirs,
                    config_path: Some(root.join(relative)),
                    // The project configured rules that are now NOT in effect.
                    // Warning on stderr and carrying on meant stdout reported
                    // `✓ All required sections present` over a section list that
                    // had been thrown away (#570).
                    load_error: Some(format!(
                        "config file {} exists but could not be loaded; built-in defaults are in use",
                        root.join(relative).display()
                    )),
                    ..SpecSyncConfig::default()
                });
            }
        };
        // One predicate decides both the fallback and the record of it. If the
        // flag were left to the parser while the override were decided here,
        // the two could disagree and coverage would treat a SCANNED list as a
        // stated one — degrading a manifest error over a guess, which is the
        // one thing the flag exists to prevent (#723).
        config.source_dirs_set = retained_config_has_source_dirs(relative, content)?;
        if !config.source_dirs_set {
            config.source_dirs = retained_source_dirs(project, root, budget)?;
        }
        return Ok(config);
    }

    let source_dirs = retained_source_dirs(project, root, budget)?;
    Ok(SpecSyncConfig {
        source_dirs,
        ..SpecSyncConfig::default()
    })
}

fn retained_config_has_source_dirs(relative: &Path, content: &str) -> Result<bool, String> {
    let content = content.trim_start_matches('\u{feff}');
    if relative
        .extension()
        .and_then(|extension| extension.to_str())
        == Some("toml")
    {
        let table = toml::from_str::<toml::Table>(content).map_err(|error| error.to_string())?;
        return Ok(table.contains_key("source_dirs"));
    }
    let value =
        serde_json::from_str::<serde_json::Value>(content).map_err(|error| error.to_string())?;
    let object = value
        .as_object()
        .ok_or_else(|| "configuration root must be an object".to_string())?;
    Ok(object.contains_key("sourceDirs"))
}

#[cfg(test)]
#[path = "tests/configuration.rs"]
mod tests;
