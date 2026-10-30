//! Bounded, identity-checked copy of a project into a temporary snapshot.

#[cfg(windows)]
use super::confinement::windows_handle_identity;
use super::confinement::{
    MAX_CONFINEMENT_ENTRIES, MAX_PROJECT_FILE_BYTES, MAX_PROJECT_INPUT_BYTES, directory_identity,
    read_only_nofollow_options,
};
#[cfg(debug_assertions)]
use super::protocol::MCP_STARTUP_TEST_BARRIER_TIMEOUT;
use super::snapshot_manifest::{
    collect_snapshot_manifest_inputs, normalize_snapshot_input, snapshot_input_overlaps,
};
use crate::config::{load_config, parse_config_content_checked};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, Metadata};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
#[cfg(debug_assertions)]
use std::fs;
#[cfg(debug_assertions)]
use std::io::Write;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

#[cfg(debug_assertions)]
const MCP_SNAPSHOT_FILE_TEST_BARRIER_ENV: &str = "SPECSYNC_TEST_MCP_SNAPSHOT_FILE_IDENTITY_BARRIER";

#[cfg(debug_assertions)]
const MCP_SNAPSHOT_FILE_TEST_PATH_ENV: &str = "SPECSYNC_TEST_MCP_SNAPSHOT_FILE_PATH";

#[cfg(debug_assertions)]
const MCP_SNAPSHOT_DIRECTORY_TEST_BARRIER_ENV: &str =
    "SPECSYNC_TEST_MCP_SNAPSHOT_DIRECTORY_IDENTITY_BARRIER";

#[cfg(debug_assertions)]
const MCP_SNAPSHOT_DIRECTORY_TEST_PATH_ENV: &str = "SPECSYNC_TEST_MCP_SNAPSHOT_DIRECTORY_PATH";

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

#[cfg(debug_assertions)]
fn wait_for_mcp_snapshot_file_test_barrier(relative: &Path) -> Result<(), String> {
    let Some(target) = std::env::var_os(MCP_SNAPSHOT_FILE_TEST_PATH_ENV) else {
        return Ok(());
    };
    if Path::new(&target) != relative {
        return Ok(());
    }
    let Some(barrier_directory) = std::env::var_os(MCP_SNAPSHOT_FILE_TEST_BARRIER_ENV) else {
        return Err("MCP snapshot file test barrier path is not configured".to_string());
    };
    let barrier_directory = PathBuf::from(barrier_directory);
    let ready_path = barrier_directory.join("retained-open");
    let resume_path = barrier_directory.join("resume");
    let mut ready_file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&ready_path)
        .map_err(|error| format!("Cannot create MCP snapshot file test barrier: {error}"))?;
    ready_file
        .write_all(b"retained-open\n")
        .and_then(|_| ready_file.sync_all())
        .map_err(|error| format!("Cannot publish MCP snapshot file test barrier: {error}"))?;
    drop(ready_file);

    let started = std::time::Instant::now();
    loop {
        match fs::metadata(&resume_path) {
            Ok(metadata) if metadata.is_file() => return Ok(()),
            Ok(_) => {
                return Err(
                    "MCP snapshot file test barrier resume marker is not a file".to_string()
                );
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "Cannot inspect MCP snapshot file test barrier resume marker: {error}"
                ));
            }
        }
        if started.elapsed() >= MCP_STARTUP_TEST_BARRIER_TIMEOUT {
            return Err("Timed out waiting for MCP snapshot file test barrier".to_string());
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[cfg(debug_assertions)]
fn wait_for_mcp_snapshot_directory_test_barrier(relative: &Path) -> Result<(), String> {
    let Some(target) = std::env::var_os(MCP_SNAPSHOT_DIRECTORY_TEST_PATH_ENV) else {
        return Ok(());
    };
    if Path::new(&target) != relative {
        return Ok(());
    }
    let Some(barrier_directory) = std::env::var_os(MCP_SNAPSHOT_DIRECTORY_TEST_BARRIER_ENV) else {
        return Err("MCP snapshot directory test barrier path is not configured".to_string());
    };
    let barrier_directory = PathBuf::from(barrier_directory);
    let ready_path = barrier_directory.join("enumerated-open");
    let resume_path = barrier_directory.join("resume");
    let mut ready_file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&ready_path)
        .map_err(|error| format!("Cannot create MCP snapshot directory test barrier: {error}"))?;
    ready_file
        .write_all(b"enumerated-open\n")
        .and_then(|_| ready_file.sync_all())
        .map_err(|error| format!("Cannot publish MCP snapshot directory test barrier: {error}"))?;
    drop(ready_file);

    let started = std::time::Instant::now();
    loop {
        match fs::metadata(&resume_path) {
            Ok(metadata) if metadata.is_file() => return Ok(()),
            Ok(_) => {
                return Err(
                    "MCP snapshot directory test barrier resume marker is not a file".to_string(),
                );
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "Cannot inspect MCP snapshot directory test barrier resume marker: {error}"
                ));
            }
        }
        if started.elapsed() >= MCP_STARTUP_TEST_BARRIER_TIMEOUT {
            return Err("Timed out waiting for MCP snapshot directory test barrier".to_string());
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

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
pub(super) fn read_capability_text_if_exists_with_hook(
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

pub(super) fn read_capability_text_if_exists_with_limit_and_hook(
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

fn copy_snapshot_configuration(
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

#[cfg(unix)]
pub(super) type SnapshotEntryIdentity = (u64, u64);

#[cfg(windows)]
pub(super) type SnapshotEntryIdentity = (u32, u64);

#[cfg(not(any(unix, windows)))]
pub(super) type SnapshotEntryIdentity = (u64, Option<std::time::SystemTime>);

#[cfg(unix)]
fn snapshot_metadata_identity(metadata: &Metadata) -> io::Result<SnapshotEntryIdentity> {
    use cap_std::fs::MetadataExt;

    Ok((metadata.dev(), metadata.ino()))
}

#[cfg(windows)]
fn snapshot_metadata_identity(metadata: &Metadata) -> io::Result<SnapshotEntryIdentity> {
    use cap_primitives::fs::_WindowsByHandle;

    let volume = metadata.volume_serial_number().ok_or_else(|| {
        io::Error::other("Windows metadata does not expose a volume serial number")
    })?;
    let index = metadata
        .file_index()
        .ok_or_else(|| io::Error::other("Windows metadata does not expose a file index"))?;
    Ok((volume, index))
}

#[cfg(not(any(unix, windows)))]
fn snapshot_metadata_identity(metadata: &Metadata) -> io::Result<SnapshotEntryIdentity> {
    Ok((metadata.len(), metadata.modified().ok()))
}

#[cfg(unix)]
fn snapshot_file_identity(file: &cap_std::fs::File) -> io::Result<SnapshotEntryIdentity> {
    snapshot_metadata_identity(&file.metadata()?)
}

#[cfg(windows)]
fn snapshot_file_identity(file: &cap_std::fs::File) -> io::Result<SnapshotEntryIdentity> {
    use std::os::windows::io::AsRawHandle;

    windows_handle_identity(file.as_raw_handle().cast())
}

#[cfg(not(any(unix, windows)))]
fn snapshot_file_identity(file: &cap_std::fs::File) -> io::Result<SnapshotEntryIdentity> {
    snapshot_metadata_identity(&file.metadata()?)
}

#[cfg(windows)]
fn snapshot_metadata_is_link(metadata: &Metadata) -> bool {
    use cap_std::fs::MetadataExt;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn snapshot_metadata_is_link(metadata: &Metadata) -> bool {
    metadata.file_type().is_symlink()
}

pub(super) fn snapshot_configuration_identities_match<Identity: PartialEq>(
    expected: &Identity,
    observed: &[Option<Identity>],
) -> bool {
    observed
        .iter()
        .all(|identity| identity.as_ref() == Some(expected))
}

#[cfg(not(windows))]
pub(super) fn open_snapshot_configuration_directory(
    parent: &Dir,
    name: &OsStr,
    relative: &Path,
) -> Result<Option<(Dir, SnapshotEntryIdentity)>, String> {
    open_snapshot_configuration_directory_with_hook(parent, name, relative, || {})
}

#[cfg(not(windows))]
pub(super) fn open_snapshot_configuration_directory_with_hook(
    parent: &Dir,
    name: &OsStr,
    relative: &Path,
    after_pre_open_identity: impl FnOnce(),
) -> Result<Option<(Dir, SnapshotEntryIdentity)>, String> {
    let before = match parent.symlink_metadata(name) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "Cannot inspect MCP configuration directory {}: {error}",
                relative.display()
            ));
        }
    };
    if snapshot_metadata_is_link(&before) || !before.is_dir() {
        return Err(format!(
            "Selected MCP configuration {} must not traverse a symlink and must use regular directories",
            relative.display()
        ));
    }
    let expected = snapshot_metadata_identity(&before).map_err(|error| {
        format!(
            "Cannot identify MCP configuration directory {}: {error}",
            relative.display()
        )
    })?;
    after_pre_open_identity();
    let directory = parent.open_dir(name).map_err(|error| {
        format!(
            "Cannot open MCP configuration directory {}: {error}",
            relative.display()
        )
    })?;
    let opened =
        snapshot_metadata_identity(&directory.dir_metadata().map_err(|error| {
            format!("Cannot inspect opened MCP configuration directory: {error}")
        })?)
        .map_err(|error| format!("Cannot identify opened MCP configuration directory: {error}"))?;
    let after = parent.symlink_metadata(name).map_err(|error| {
        format!(
            "Cannot re-inspect MCP configuration directory {}: {error}",
            relative.display()
        )
    })?;
    if snapshot_metadata_is_link(&after)
        || !after.is_dir()
        || !snapshot_configuration_identities_match(
            &expected,
            &[Some(opened), snapshot_metadata_identity(&after).ok()],
        )
    {
        return Err(format!(
            "Selected MCP configuration directory changed during inspection: {}",
            relative.display()
        ));
    }
    Ok(Some((directory, expected)))
}

#[cfg(windows)]
pub(super) fn open_snapshot_configuration_directory(
    parent: &Dir,
    name: &OsStr,
    relative: &Path,
) -> Result<Option<(Dir, SnapshotEntryIdentity)>, String> {
    open_snapshot_configuration_directory_with_hook(parent, name, relative, || {})
}

#[cfg(windows)]
pub(super) fn open_snapshot_configuration_directory_with_hook(
    parent: &Dir,
    name: &OsStr,
    relative: &Path,
    after_pre_open_identity: impl FnOnce(),
) -> Result<Option<(Dir, SnapshotEntryIdentity)>, String> {
    let before = match parent.symlink_metadata(name) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "Cannot inspect MCP configuration directory {}: {error}",
                relative.display()
            ));
        }
    };
    if snapshot_metadata_is_link(&before) || !before.is_dir() {
        return Err(format!(
            "Selected MCP configuration {} must not traverse a reparse point and must use regular directories",
            relative.display()
        ));
    }
    let expected = snapshot_metadata_identity(&before).map_err(|error| {
        format!(
            "Cannot identify MCP configuration directory {}: {error}",
            relative.display()
        )
    })?;
    after_pre_open_identity();
    let directory = parent.open_dir(name).map_err(|error| {
        format!(
            "Cannot open MCP configuration directory {}: {error}",
            relative.display()
        )
    })?;
    let opened = directory_identity(&directory)
        .map_err(|error| format!("Cannot identify opened MCP configuration directory: {error}"))?;
    let observed = parent.open_dir(name).map_err(|error| {
        format!(
            "Cannot re-open MCP configuration directory {}: {error}",
            relative.display()
        )
    })?;
    let after = parent.symlink_metadata(name).map_err(|error| {
        format!(
            "Cannot re-inspect MCP configuration directory {}: {error}",
            relative.display()
        )
    })?;
    if snapshot_metadata_is_link(&after)
        || !after.is_dir()
        || !snapshot_configuration_identities_match(
            &expected,
            &[
                Some(opened),
                snapshot_metadata_identity(&after).ok(),
                directory_identity(&observed).ok(),
            ],
        )
    {
        return Err(format!(
            "Selected MCP configuration directory changed during inspection: {}",
            relative.display()
        ));
    }
    Ok(Some((directory, expected)))
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

pub(super) fn read_snapshot_configuration_with_hook(
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

pub(super) fn copy_preloaded_snapshot_files(
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
    pub(super) fn charge_entries(&mut self, count: usize, label: &str) -> Result<(), String> {
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

fn read_snapshot_project_file_from_directory(
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
pub(super) fn read_snapshot_project_file_with_hook(
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
pub(super) fn copy_snapshot_directory_with_hook(
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

#[cfg(test)]
mod tests;
