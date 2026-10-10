mod inputs;
mod manifests;
mod traversal;

use super::MAX_CONFINEMENT_ENTRIES;
use super::MAX_PROJECT_INPUT_BYTES;
use crate::config::load_config;
use cap_std::ambient_authority;
use cap_std::fs::Dir;
use inputs::copy_snapshot_configuration;
use manifests::collect_snapshot_manifest_inputs;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use traversal::copy_snapshot_directory;

pub(super) struct ProjectSnapshot {
    directory: tempfile::TempDir,
    git_freshness_available: bool,
}

impl ProjectSnapshot {
    #[cfg(test)]
    pub(super) fn create(root: &Path) -> Result<Self, String> {
        let source = Dir::open_ambient_dir(root, ambient_authority()).map_err(|error| {
            format!(
                "Cannot open MCP server root capability {}: {error}",
                root.display()
            )
        })?;
        Self::create_from_directory(&source)
    }

    pub(super) fn create_from_directory(source: &Dir) -> Result<Self, String> {
        Self::create_from_directory_in(source, None)
    }

    #[cfg(test)]
    pub(super) fn create_in(root: &Path, temp_parent: &Path) -> Result<Self, String> {
        let source = Dir::open_ambient_dir(root, ambient_authority()).map_err(|error| {
            format!(
                "Cannot open MCP server root capability {}: {error}",
                root.display()
            )
        })?;
        Self::create_from_directory_in(&source, Some(temp_parent))
    }

    fn create_from_directory_in(source: &Dir, temp_parent: Option<&Path>) -> Result<Self, String> {
        let mut builder = tempfile::Builder::new();
        builder.prefix("specsync-mcp-");
        let directory = if let Some(parent) = temp_parent {
            builder.tempdir_in(parent)
        } else {
            builder.tempdir()
        }
        .map_err(|error| format!("Cannot create bounded MCP project snapshot: {error}"))?;
        let destination = Dir::open_ambient_dir(directory.path(), ambient_authority())
            .map_err(|error| format!("Cannot open MCP snapshot capability: {error}"))?;
        let mut budget = SnapshotBudget::default();
        copy_snapshot_configuration(source, &destination, directory.path(), &mut budget)?;
        let snapshot_config = load_config(directory.path());
        let configured_exclusions: HashSet<String> =
            snapshot_config.exclude_dirs.iter().cloned().collect();
        let mut configured_inputs: Vec<PathBuf> = snapshot_config
            .source_dirs
            .iter()
            .map(PathBuf::from)
            .collect();
        configured_inputs.push(PathBuf::from(&snapshot_config.specs_dir));
        if let Some(schema_dir) = snapshot_config.schema_dir.as_deref() {
            configured_inputs.push(PathBuf::from(schema_dir));
        }
        configured_inputs.extend(
            snapshot_config
                .modules
                .values()
                .flat_map(|module| module.files.iter().map(PathBuf::from)),
        );
        collect_snapshot_manifest_inputs(source, &mut configured_inputs, &mut budget)?;
        copy_preloaded_snapshot_files(&destination, &mut budget)?;
        if configured_inputs.iter().any(|path| {
            path.components().any(|component| {
                component
                    .as_os_str()
                    .to_str()
                    .is_some_and(|name| name.eq_ignore_ascii_case(".git"))
            })
        }) {
            return Err(
                "MCP project configuration must not use Git metadata as a project input"
                    .to_string(),
            );
        }
        copy_snapshot_directory(
            source,
            &destination,
            Path::new("."),
            &configured_exclusions,
            &configured_inputs,
            &mut budget,
        )?;
        Ok(Self {
            directory,
            git_freshness_available: false,
        })
    }

    pub(super) fn root(&self) -> &Path {
        self.directory.path()
    }

    pub(super) fn git_freshness_available(&self) -> bool {
        self.git_freshness_available
    }
}

/// Pin git discovery to `parent(snapshot)` for the duration of `f`.
///
/// Snapshots copy the project without `.git`. If the tempfile sits inside a
/// host worktree (`TMPDIR` pointing at the repo), default walk-up would find
/// that host. Empirically ceiling=`parent(snapshot)` stops the walk; ceiling
/// equal to the snapshot path itself does not. CLI callers never enter here.
pub(super) fn with_isolated_snapshot_git<T>(
    snapshot: &ProjectSnapshot,
    f: impl FnOnce() -> T,
) -> T {
    match snapshot.root().parent() {
        Some(ceiling) => crate::git_utils::with_discovery_ceiling(ceiling, f),
        None => f(),
    }
}

pub(super) struct SnapshotBudget {
    bytes: u64,
    max_bytes: u64,
    entries: usize,
    charged_paths: HashSet<PathBuf>,
    copied_paths: HashSet<PathBuf>,
    preloaded_files: HashMap<PathBuf, Vec<u8>>,
}

impl Default for SnapshotBudget {
    fn default() -> Self {
        Self {
            bytes: 0,
            max_bytes: MAX_PROJECT_INPUT_BYTES,
            entries: 0,
            charged_paths: HashSet::new(),
            copied_paths: HashSet::new(),
            preloaded_files: HashMap::new(),
        }
    }
}

fn copy_preloaded_snapshot_files(
    destination: &Dir,
    budget: &mut SnapshotBudget,
) -> Result<(), String> {
    let mut preloaded: Vec<(PathBuf, Vec<u8>)> = budget.preloaded_files.drain().collect();
    preloaded.sort_by(|left, right| left.0.cmp(&right.0));
    for (relative, bytes) in preloaded {
        if let Some(parent) = relative.parent()
            && !parent.as_os_str().is_empty()
        {
            destination.create_dir_all(parent).map_err(|error| {
                format!(
                    "Cannot create MCP snapshot manifest directory {}: {error}",
                    parent.display()
                )
            })?;
        }
        destination.write(&relative, bytes).map_err(|error| {
            format!(
                "Cannot copy preflighted MCP snapshot manifest {}: {error}",
                relative.display()
            )
        })?;
        budget.copied_paths.insert(relative);
    }
    Ok(())
}

impl SnapshotBudget {
    fn charge_entries(&mut self, count: usize, label: &str) -> Result<(), String> {
        self.entries = self.entries.saturating_add(count);
        if self.entries > MAX_CONFINEMENT_ENTRIES {
            return Err(format!("{label} exceeds {MAX_CONFINEMENT_ENTRIES} entries"));
        }
        Ok(())
    }

    fn charge_file(&mut self, relative: &Path, bytes: u64) -> Result<(), String> {
        if !self.charged_paths.insert(relative.to_path_buf()) {
            return Ok(());
        }
        self.bytes = self.bytes.saturating_add(bytes);
        if self.bytes > self.max_bytes {
            let limit = if self.max_bytes.is_multiple_of(1024 * 1024) {
                format!("{} MiB", self.max_bytes / (1024 * 1024))
            } else {
                format!("{} bytes", self.max_bytes)
            };
            return Err(format!(
                "MCP project inputs exceed the {limit} cumulative limit"
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
