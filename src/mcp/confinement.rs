//! Retained server-root capability, filesystem identity, and path confinement.

use super::preflight::{
    AUTODETECT_IGNORED_DIRS, validate_autodetection_tree, validate_manifest_inputs,
};
#[cfg(debug_assertions)]
use super::protocol::MCP_STARTUP_TEST_BARRIER_TIMEOUT;
use super::snapshot::{SnapshotEntryIdentity, open_snapshot_configuration_directory};
use super::tools::MAX_GENERATED_OUTPUT_BYTES;
use crate::config::load_config;
use crate::types::SpecSyncConfig;
use cap_primitives::fs::FollowSymlinks;
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
#[cfg(debug_assertions)]
use std::io::Write;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Component, Path, PathBuf};
use walkdir::WalkDir;

pub(super) const MAX_PROJECT_FILE_BYTES: u64 = 8 * 1024 * 1024;

pub(super) const MAX_PROJECT_INPUT_BYTES: u64 = 64 * 1024 * 1024;

pub(super) const MAX_CONFINEMENT_ENTRIES: usize = 100_000;

pub(super) const MAX_CONFIGURED_PATHS: usize = 1_000;

#[cfg(debug_assertions)]
const MCP_READ_ROOT_REVALIDATION_TEST_BARRIER_ENV: &str =
    "SPECSYNC_TEST_MCP_READ_ROOT_REVALIDATION_BARRIER";

pub(super) fn read_only_nofollow_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options
        .read(true)
        ._cap_fs_ext_nonblock(true)
        ._cap_fs_ext_follow(FollowSymlinks::No);
    options
}

pub(super) fn open_server_root_capability(
    root: &Path,
    after_identity_bound: impl FnOnce() -> Result<(), String>,
) -> Result<(PathBuf, Dir), String> {
    let server_directory = Dir::open_ambient_dir(root, ambient_authority()).map_err(|error| {
        if error.kind() == io::ErrorKind::NotADirectory {
            format!("MCP server root is not a directory: {}", root.display())
        } else {
            format!(
                "Cannot open initial MCP server root capability {}: {error}",
                root.display()
            )
        }
    })?;
    let expected_identity = directory_identity(&server_directory).map_err(|error| {
        format!(
            "Cannot bind initial MCP server root identity {}: {error}",
            root.display()
        )
    })?;
    after_identity_bound()?;

    let server_root = root
        .canonicalize()
        .map_err(|error| format!("Cannot resolve MCP server root {}: {error}", root.display()))?;
    let observed_directory = match (server_root.parent(), server_root.file_name()) {
        (Some(parent), Some(name)) => {
            let parent_directory =
                Dir::open_ambient_dir(parent, ambient_authority()).map_err(|error| {
                    format!(
                        "Cannot open MCP server-root parent capability {}: {error}",
                        parent.display()
                    )
                })?;
            parent_directory.open_dir(name).map_err(|error| {
                format!(
                    "Cannot reopen MCP server root through its parent capability {}: {error}",
                    server_root.display()
                )
            })?
        }
        _ => Dir::open_ambient_dir(&server_root, ambient_authority()).map_err(|error| {
            format!(
                "Cannot reopen filesystem-root MCP capability {}: {error}",
                server_root.display()
            )
        })?,
    };
    let observed_identity = directory_identity(&observed_directory).map_err(|error| {
        format!(
            "Cannot verify MCP server root identity {}: {error}",
            server_root.display()
        )
    })?;
    if expected_identity != observed_identity {
        return Err(format!(
            "MCP server root changed while its identity was being resolved: {}",
            server_root.display()
        ));
    }

    Ok((server_root, server_directory))
}

#[cfg(debug_assertions)]
fn wait_for_mcp_read_root_revalidation_test_barrier() -> Result<(), String> {
    let Some(barrier_directory) = std::env::var_os(MCP_READ_ROOT_REVALIDATION_TEST_BARRIER_ENV)
    else {
        return Ok(());
    };
    let barrier_directory = PathBuf::from(barrier_directory);
    let ready_path = barrier_directory.join("operation-complete");
    let resume_path = barrier_directory.join("resume");
    let mut ready_file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&ready_path)
        .map_err(|error| format!("Cannot create MCP read-root test barrier: {error}"))?;
    ready_file
        .write_all(b"operation-complete\n")
        .and_then(|_| ready_file.sync_all())
        .map_err(|error| format!("Cannot publish MCP read-root test barrier: {error}"))?;
    drop(ready_file);

    let started = std::time::Instant::now();
    loop {
        match fs::metadata(&resume_path) {
            Ok(metadata) if metadata.is_file() => return Ok(()),
            Ok(_) => {
                return Err("MCP read-root test barrier resume marker is not a file".to_string());
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "Cannot inspect MCP read-root test barrier resume marker: {error}"
                ));
            }
        }
        if started.elapsed() >= MCP_STARTUP_TEST_BARRIER_TIMEOUT {
            return Err("Timed out waiting for MCP read-root test barrier".to_string());
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[cfg(unix)]
pub(super) type DirectoryIdentity = (u64, u64);

#[cfg(unix)]
pub(super) type FileIdentity = (u64, u64, [u8; 32]);

#[cfg(unix)]
pub(super) fn directory_identity(directory: &Dir) -> io::Result<DirectoryIdentity> {
    use cap_std::fs::MetadataExt;

    let metadata = directory.dir_metadata()?;
    Ok((metadata.dev(), metadata.ino()))
}

#[cfg(unix)]
pub(super) fn file_identity(file: &cap_std::fs::File) -> io::Result<FileIdentity> {
    use cap_std::fs::MetadataExt;

    let metadata = file.metadata()?;
    Ok((metadata.dev(), metadata.ino(), file_content_digest(file)?))
}

#[cfg(windows)]
pub(super) type DirectoryIdentity = (u32, u64);

#[cfg(windows)]
pub(super) type FileIdentity = (u32, u64, [u8; 32]);

#[cfg(windows)]
pub(super) fn directory_identity(directory: &Dir) -> io::Result<DirectoryIdentity> {
    use std::os::windows::io::AsRawHandle;

    let file = directory.try_clone()?.into_std_file();
    windows_handle_identity(file.as_raw_handle().cast())
}

#[cfg(windows)]
pub(super) fn file_identity(file: &cap_std::fs::File) -> io::Result<FileIdentity> {
    use std::os::windows::io::AsRawHandle;

    let (volume, index) = windows_handle_identity(file.as_raw_handle().cast())?;
    Ok((volume, index, file_content_digest(file)?))
}

#[cfg(windows)]
pub(super) fn windows_handle_identity(handle: *mut std::ffi::c_void) -> io::Result<(u32, u64)> {
    use std::ffi::c_void;
    use std::mem::MaybeUninit;

    #[repr(C)]
    struct FileTime {
        low: u32,
        high: u32,
    }

    #[repr(C)]
    struct ByHandleFileInformation {
        file_attributes: u32,
        creation_time: FileTime,
        last_access_time: FileTime,
        last_write_time: FileTime,
        volume_serial_number: u32,
        file_size_high: u32,
        file_size_low: u32,
        number_of_links: u32,
        file_index_high: u32,
        file_index_low: u32,
    }

    unsafe extern "system" {
        fn GetFileInformationByHandle(
            file: *mut c_void,
            information: *mut ByHandleFileInformation,
        ) -> i32;
    }

    let mut information = MaybeUninit::<ByHandleFileInformation>::uninit();
    // SAFETY: the caller owns a valid file or directory handle for the duration of the call, and
    // `information` points to writable storage of the exact Win32 structure layout.
    let success = unsafe { GetFileInformationByHandle(handle, information.as_mut_ptr()) };
    if success == 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: a successful GetFileInformationByHandle call initializes every field.
    let information = unsafe { information.assume_init() };
    let file_index =
        (u64::from(information.file_index_high) << 32) | u64::from(information.file_index_low);
    Ok((information.volume_serial_number, file_index))
}

#[cfg(not(any(unix, windows)))]
pub(super) type DirectoryIdentity = (u64, Option<std::time::SystemTime>);

#[cfg(not(any(unix, windows)))]
pub(super) type FileIdentity = (u64, Option<std::time::SystemTime>, [u8; 32]);

#[cfg(not(any(unix, windows)))]
pub(super) fn directory_identity(directory: &Dir) -> io::Result<DirectoryIdentity> {
    let metadata = directory.dir_metadata()?;
    Ok((metadata.len(), metadata.modified().ok()))
}

#[cfg(not(any(unix, windows)))]
pub(super) fn file_identity(file: &cap_std::fs::File) -> io::Result<FileIdentity> {
    let metadata = file.metadata()?;
    Ok((
        metadata.len(),
        metadata.modified().ok(),
        file_content_digest(file)?,
    ))
}

fn file_content_digest(file: &cap_std::fs::File) -> io::Result<[u8; 32]> {
    file_content_digest_with_limit(file, MAX_GENERATED_OUTPUT_BYTES)
}

pub(super) fn file_content_digest_with_limit(
    file: &cap_std::fs::File,
    maximum_bytes: u64,
) -> io::Result<[u8; 32]> {
    let mut reader = file.try_clone()?;
    reader.seek(SeekFrom::Start(0))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8 * 1024];
    let mut total_bytes = 0_u64;
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        total_bytes = total_bytes.saturating_add(read as u64);
        if total_bytes > maximum_bytes {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("file identity input exceeds the {maximum_bytes}-byte limit"),
            ));
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher.finalize().into())
}

pub(super) fn resolve_read_root(
    server_root: &Path,
    requested_server_root: &Path,
    requested_root: Option<&str>,
) -> Result<PathBuf, String> {
    let Some(requested_root) = requested_root else {
        return Ok(PathBuf::from("."));
    };
    let requested_path = Path::new(requested_root);
    let components: Vec<Component<'_>> = requested_path.components().collect();
    if components.contains(&Component::ParentDir) {
        return Err("Read root override must not contain parent traversal".to_string());
    }
    if !requested_path.is_absolute()
        && components
            .iter()
            .any(|component| matches!(*component, Component::RootDir | Component::Prefix(_)))
    {
        return Err("Read root override must not use a rooted or drive-relative path".to_string());
    }

    let relative = if requested_path.is_absolute() {
        relative_read_root_suffix_from_aliases(server_root, requested_server_root, requested_path)
            .ok_or_else(|| "Read root override escapes the configured server root".to_string())?
    } else {
        requested_path.to_path_buf()
    };
    if relative.components().any(|component| {
        matches!(
            component,
            Component::Normal(name)
                if name
                    .to_str()
                    .is_some_and(|name| name.eq_ignore_ascii_case(".git"))
        )
    }) {
        return Err("Read root override must not select Git metadata".to_string());
    }
    Ok(if relative.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        relative
    })
}

fn relative_read_root_suffix_from_aliases(
    canonical_root: &Path,
    requested_root: &Path,
    candidate: &Path,
) -> Option<PathBuf> {
    // Windows canonicalization can expand an operator-supplied DOS 8.3 root spelling. Accept the
    // candidate beneath either startup spelling, but never canonicalize the candidate before
    // authorization. The derived suffix is still opened only through the retained root capability.
    relative_read_root_suffix(canonical_root, candidate)
        .or_else(|| relative_read_root_suffix(requested_root, candidate))
}

#[cfg(not(windows))]
fn relative_read_root_suffix(root: &Path, candidate: &Path) -> Option<PathBuf> {
    candidate.strip_prefix(root).ok().map(Path::to_path_buf)
}

#[cfg(windows)]
fn relative_read_root_suffix(root: &Path, candidate: &Path) -> Option<PathBuf> {
    let (root_prefix, root_components) = parse_windows_absolute_path_native(root)?;
    let (candidate_prefix, candidate_components) = parse_windows_absolute_path_native(candidate)?;
    if !windows_prefixes_equal(&root_prefix, &candidate_prefix)
        || candidate_components.len() < root_components.len()
        || !root_components
            .iter()
            .zip(&candidate_components)
            .all(|(root, candidate)| windows_ordinal_ignore_case(root, candidate))
    {
        return None;
    }
    Some(
        candidate_components[root_components.len()..]
            .iter()
            .collect(),
    )
}

#[cfg(windows)]
enum WindowsPathPrefix {
    Disk(u8),
    Unc(std::ffi::OsString, std::ffi::OsString),
}

#[cfg(windows)]
fn parse_windows_absolute_path_native(
    path: &Path,
) -> Option<(WindowsPathPrefix, Vec<std::ffi::OsString>)> {
    use std::path::Prefix;

    let mut components = path.components();
    let Component::Prefix(prefix) = components.next()? else {
        return None;
    };
    let prefix = match prefix.kind() {
        Prefix::Disk(drive) | Prefix::VerbatimDisk(drive) => WindowsPathPrefix::Disk(drive),
        Prefix::UNC(server, share) | Prefix::VerbatimUNC(server, share) => {
            WindowsPathPrefix::Unc(server.to_os_string(), share.to_os_string())
        }
        Prefix::Verbatim(_) | Prefix::DeviceNS(_) => return None,
    };
    if !matches!(components.next(), Some(Component::RootDir)) {
        return None;
    }
    let mut relative = Vec::new();
    for component in components {
        match component {
            Component::Normal(component) => relative.push(component.to_os_string()),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    Some((prefix, relative))
}

#[cfg(windows)]
fn windows_prefixes_equal(left: &WindowsPathPrefix, right: &WindowsPathPrefix) -> bool {
    match (left, right) {
        (WindowsPathPrefix::Disk(left), WindowsPathPrefix::Disk(right)) => {
            left.eq_ignore_ascii_case(right)
        }
        (
            WindowsPathPrefix::Unc(left_server, left_share),
            WindowsPathPrefix::Unc(right_server, right_share),
        ) => {
            windows_ordinal_ignore_case(left_server, right_server)
                && windows_ordinal_ignore_case(left_share, right_share)
        }
        _ => false,
    }
}

#[cfg(windows)]
fn windows_ordinal_ignore_case(left: &std::ffi::OsStr, right: &std::ffi::OsStr) -> bool {
    use std::os::windows::ffi::OsStrExt;

    unsafe extern "system" {
        fn CompareStringOrdinal(
            left: *const u16,
            left_len: i32,
            right: *const u16,
            right_len: i32,
            ignore_case: i32,
        ) -> i32;
    }

    let left: Vec<u16> = left.encode_wide().collect();
    let right: Vec<u16> = right.encode_wide().collect();
    let (Ok(left_len), Ok(right_len)) = (i32::try_from(left.len()), i32::try_from(right.len()))
    else {
        return false;
    };
    // SAFETY: both pointers reference initialized UTF-16 buffers for the exact lengths supplied;
    // CompareStringOrdinal does not retain or mutate either buffer.
    unsafe { CompareStringOrdinal(left.as_ptr(), left_len, right.as_ptr(), right_len, 1) == 2 }
}

#[cfg(test)]
pub(super) fn windows_relative_suffix_text(root: &str, candidate: &str) -> Option<PathBuf> {
    let root = parse_windows_absolute_path(root)?;
    let candidate = parse_windows_absolute_path(candidate)?;
    if !windows_text_ignore_case(&root.prefix, &candidate.prefix)
        || candidate.components.len() < root.components.len()
        || !root
            .components
            .iter()
            .zip(&candidate.components)
            .all(|(root, candidate)| windows_text_ignore_case(root, candidate))
    {
        return None;
    }
    Some(
        candidate.components[root.components.len()..]
            .iter()
            .collect(),
    )
}

#[cfg(test)]
fn windows_text_ignore_case(left: &str, right: &str) -> bool {
    left.to_lowercase() == right.to_lowercase()
}

#[cfg(test)]
struct WindowsAbsolutePath {
    prefix: String,
    components: Vec<String>,
}

#[cfg(test)]
fn parse_windows_absolute_path(path: &str) -> Option<WindowsAbsolutePath> {
    let mut path = path.replace('/', "\\");
    if let Some(without_prefix) = strip_ascii_case_prefix(&path, r"\\?\UNC\") {
        path = format!(r"\\{without_prefix}");
    } else if let Some(without_prefix) = strip_ascii_case_prefix(&path, r"\\?\") {
        path = without_prefix.to_string();
    }

    if let Some(unc) = path.strip_prefix(r"\\") {
        let mut components = unc.split('\\').filter(|component| !component.is_empty());
        let server = components.next()?;
        let share = components.next()?;
        return Some(WindowsAbsolutePath {
            prefix: format!(r"\\{server}\{share}"),
            components: components.map(str::to_string).collect(),
        });
    }

    let bytes = path.as_bytes();
    if bytes.len() < 3 || bytes[1] != b':' || bytes[2] != b'\\' {
        return None;
    }
    Some(WindowsAbsolutePath {
        prefix: path[..2].to_string(),
        components: path[3..]
            .split('\\')
            .filter(|component| !component.is_empty())
            .map(str::to_string)
            .collect(),
    })
}

#[cfg(test)]
fn strip_ascii_case_prefix<'a>(value: &'a str, prefix: &str) -> Option<&'a str> {
    let candidate = value.get(..prefix.len())?;
    candidate
        .eq_ignore_ascii_case(prefix)
        .then(|| &value[prefix.len()..])
}

pub(super) fn load_confined_config(root: &Path) -> Result<SpecSyncConfig, String> {
    validate_known_config_files(root)?;
    validate_manifest_inputs(root)?;
    if config_requires_source_autodetection(root) {
        validate_autodetection_tree(root)?;
    }
    let config = load_config(root);

    let configured_path_count = 1usize
        + config.source_dirs.len()
        + usize::from(config.schema_dir.is_some())
        + config
            .modules
            .values()
            .map(|module| module.files.len())
            .sum::<usize>();
    if configured_path_count > MAX_CONFIGURED_PATHS {
        return Err(format!(
            "MCP configuration exceeds {MAX_CONFIGURED_PATHS} path entries"
        ));
    }

    let no_exclusions = HashSet::new();
    let source_exclusions: HashSet<String> = config.exclude_dirs.iter().cloned().collect();
    let mut confinement_entries_seen = 0usize;

    validate_configured_path_with_budget(
        root,
        &config.specs_dir,
        "specs_dir",
        Some(&no_exclusions),
        &mut confinement_entries_seen,
    )?;
    for source_dir in &config.source_dirs {
        validate_configured_path_with_budget(
            root,
            source_dir,
            "source_dirs entry",
            Some(&source_exclusions),
            &mut confinement_entries_seen,
        )?;
    }
    if let Some(schema_dir) = config.schema_dir.as_deref()
        && !schema_dir.is_empty()
    {
        validate_configured_path_with_budget(
            root,
            schema_dir,
            "schema_dir",
            Some(&no_exclusions),
            &mut confinement_entries_seen,
        )?;
    }
    for (module_name, module) in &config.modules {
        validate_module_name(module_name)?;
        for file in &module.files {
            validate_configured_path_with_budget(
                root,
                file,
                "configured module file",
                Some(&source_exclusions),
                &mut confinement_entries_seen,
            )?;
        }
    }

    validate_project_input_budget(root, &config)?;

    Ok(config)
}

pub(super) fn validate_known_config_files(root: &Path) -> Result<(), String> {
    for relative in [
        ".specsync/config.toml",
        ".specsync/config.json",
        ".specsync/config.local.toml",
        ".specsync.toml",
        "specsync.json",
        ".specsync/hashes.json",
        ".git",
        "Cargo.toml",
        "Package.swift",
        "build.gradle.kts",
        "build.gradle",
        "settings.gradle.kts",
        "settings.gradle",
        "package.json",
        "pubspec.yaml",
        "go.mod",
        "pyproject.toml",
    ] {
        let candidate = root.join(relative);
        match fs::symlink_metadata(&candidate) {
            Ok(_) => validate_existing_path(root, &candidate, "project metadata file", None)?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "Cannot inspect MCP configuration path {}: {error}",
                    candidate.display()
                ));
            }
        }
    }
    Ok(())
}

fn selected_config_path(root: &Path) -> Option<PathBuf> {
    [
        ".specsync/config.toml",
        ".specsync/config.json",
        ".specsync.toml",
        "specsync.json",
    ]
    .iter()
    .map(|relative| root.join(relative))
    .find(|path| path.exists())
}

fn config_requires_source_autodetection(root: &Path) -> bool {
    let Some(config_path) = selected_config_path(root) else {
        return true;
    };
    let Some(content) = crate::config::read_config_file(&config_path) else {
        return false;
    };
    if config_path
        .extension()
        .and_then(|extension| extension.to_str())
        == Some("json")
    {
        return serde_json::from_str::<Value>(&content).is_ok()
            && !content.contains("\"sourceDirs\"");
    }

    let mut top_level = true;
    for line in content.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            top_level = false;
            continue;
        }
        if top_level
            && line
                .split_once('=')
                .is_some_and(|(key, _)| key.trim() == "source_dirs")
        {
            return false;
        }
    }
    true
}

pub(super) fn validate_configured_path_with_budget(
    root: &Path,
    configured: &str,
    label: &str,
    tree_exclusions: Option<&HashSet<String>>,
    confinement_entries_seen: &mut usize,
) -> Result<(), String> {
    let path = Path::new(configured);
    if configured.is_empty() || path.is_absolute() {
        return Err(format!(
            "MCP {label} must be a non-empty project-relative path: {configured}"
        ));
    }
    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(format!(
            "MCP {label} must not contain traversal or an absolute prefix: {configured}"
        ));
    }

    validate_path_or_ancestor_with_budget(
        root,
        &root.join(path),
        label,
        tree_exclusions,
        confinement_entries_seen,
    )
}

pub(super) fn validate_path_or_ancestor(
    root: &Path,
    candidate: &Path,
    label: &str,
    tree_exclusions: Option<&HashSet<String>>,
) -> Result<(), String> {
    let mut confinement_entries_seen = 0usize;
    validate_path_or_ancestor_with_budget(
        root,
        candidate,
        label,
        tree_exclusions,
        &mut confinement_entries_seen,
    )
}

fn validate_path_or_ancestor_with_budget(
    root: &Path,
    candidate: &Path,
    label: &str,
    tree_exclusions: Option<&HashSet<String>>,
    confinement_entries_seen: &mut usize,
) -> Result<(), String> {
    let mut current = candidate;
    loop {
        match fs::symlink_metadata(current) {
            Ok(_) => {
                let exclusions = (current == candidate).then_some(tree_exclusions).flatten();
                validate_existing_path_with_budget(
                    root,
                    current,
                    label,
                    exclusions,
                    confinement_entries_seen,
                )?;
                return Ok(());
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                current = current.parent().ok_or_else(|| {
                    format!(
                        "MCP {label} has no resolvable ancestor: {}",
                        candidate.display()
                    )
                })?;
            }
            Err(error) => {
                return Err(format!(
                    "Cannot inspect MCP {label} {}: {error}",
                    current.display()
                ));
            }
        }
    }
}

pub(super) fn validate_existing_path(
    root: &Path,
    candidate: &Path,
    label: &str,
    tree_exclusions: Option<&HashSet<String>>,
) -> Result<(), String> {
    let mut confinement_entries_seen = 0usize;
    validate_existing_path_with_budget(
        root,
        candidate,
        label,
        tree_exclusions,
        &mut confinement_entries_seen,
    )
}

fn validate_existing_path_with_budget(
    root: &Path,
    candidate: &Path,
    label: &str,
    tree_exclusions: Option<&HashSet<String>>,
    confinement_entries_seen: &mut usize,
) -> Result<(), String> {
    let canonical_root = root.canonicalize().map_err(|error| {
        format!(
            "Cannot resolve MCP server root {} while validating {label}: {error}",
            root.display()
        )
    })?;
    let canonical = candidate.canonicalize().map_err(|error| {
        format!(
            "Cannot resolve MCP {label} {} (including symlink targets): {error}",
            candidate.display()
        )
    })?;
    if !canonical.starts_with(&canonical_root) {
        return Err(format!(
            "MCP {label} escapes the configured server root: {}",
            candidate.display()
        ));
    }
    let metadata = fs::metadata(&canonical).map_err(|error| {
        format!(
            "Cannot inspect MCP {label} {} after canonicalization: {error}",
            candidate.display()
        )
    })?;
    if metadata.is_file() && metadata.len() > MAX_PROJECT_FILE_BYTES {
        return Err(format!(
            "MCP {label} exceeds the {} MiB project-file limit: {}",
            MAX_PROJECT_FILE_BYTES / (1024 * 1024),
            candidate.display()
        ));
    }
    if let Some(exclusions) = tree_exclusions
        && canonical.is_dir()
    {
        validate_tree_confinement(
            &canonical_root,
            &canonical,
            label,
            exclusions,
            confinement_entries_seen,
        )?;
    }
    Ok(())
}

pub(super) fn read_file_bounded(root: &Path, path: &Path, label: &str) -> Result<String, String> {
    validate_existing_path(root, path, label, None)?;
    let metadata = fs::metadata(path)
        .map_err(|error| format!("Cannot inspect MCP {label} {}: {error}", path.display()))?;
    if !metadata.is_file() {
        return Err(format!("MCP {label} is not a file: {}", path.display()));
    }
    if metadata.len() > MAX_PROJECT_FILE_BYTES {
        return Err(format!(
            "MCP {label} exceeds the {} MiB project-file limit: {}",
            MAX_PROJECT_FILE_BYTES / (1024 * 1024),
            path.display()
        ));
    }

    let file = fs::File::open(path)
        .map_err(|error| format!("Cannot read MCP {label} {}: {error}", path.display()))?;
    let mut bytes = Vec::new();
    file.take(MAX_PROJECT_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("Cannot read MCP {label} {}: {error}", path.display()))?;
    if bytes.len() as u64 > MAX_PROJECT_FILE_BYTES {
        return Err(format!(
            "MCP {label} exceeds the {} MiB project-file limit: {}",
            MAX_PROJECT_FILE_BYTES / (1024 * 1024),
            path.display()
        ));
    }
    String::from_utf8(bytes)
        .map_err(|_| format!("MCP {label} is not valid UTF-8: {}", path.display()))
}

fn validate_project_input_budget(root: &Path, config: &SpecSyncConfig) -> Result<(), String> {
    let mut candidates = vec![root.join(&config.specs_dir)];
    candidates.extend(config.source_dirs.iter().map(|path| root.join(path)));
    if let Some(schema_dir) = config.schema_dir.as_deref()
        && !schema_dir.is_empty()
    {
        candidates.push(root.join(schema_dir));
    }
    for module in config.modules.values() {
        candidates.extend(module.files.iter().map(|path| root.join(path)));
    }

    let mut exclusions: HashSet<String> = AUTODETECT_IGNORED_DIRS
        .iter()
        .map(|name| (*name).to_string())
        .collect();
    exclusions.extend(config.exclude_dirs.iter().cloned());
    let canonical_root = root
        .canonicalize()
        .map_err(|error| format!("Cannot resolve MCP server root {}: {error}", root.display()))?;
    let mut seen_files = HashSet::new();
    let mut total_bytes = 0u64;
    let mut entries_seen = 0usize;

    for candidate in candidates {
        if !candidate.exists() {
            continue;
        }
        let entries = WalkDir::new(&candidate)
            .follow_links(false)
            .into_iter()
            .filter_entry(|entry| {
                entry.depth() == 0
                    || !entry.file_type().is_dir()
                    || !entry
                        .file_name()
                        .to_str()
                        .is_some_and(|name| exclusions.contains(name))
            });
        for entry in entries {
            entries_seen += 1;
            if entries_seen > MAX_CONFINEMENT_ENTRIES {
                return Err(format!(
                    "MCP project input scan exceeds {MAX_CONFINEMENT_ENTRIES} entries"
                ));
            }
            let entry =
                entry.map_err(|error| format!("Cannot inspect MCP project input: {error}"))?;
            if !entry.file_type().is_file() {
                continue;
            }
            let canonical = entry.path().canonicalize().map_err(|error| {
                format!(
                    "Cannot resolve MCP project input {}: {error}",
                    entry.path().display()
                )
            })?;
            if !canonical.starts_with(&canonical_root) {
                return Err(format!(
                    "MCP project input escapes the configured server root: {}",
                    entry.path().display()
                ));
            }
            if !seen_files.insert(canonical) {
                continue;
            }
            let bytes = entry
                .metadata()
                .map_err(|error| {
                    format!(
                        "Cannot inspect MCP project input {}: {error}",
                        entry.path().display()
                    )
                })?
                .len();
            if bytes > MAX_PROJECT_FILE_BYTES {
                return Err(format!(
                    "MCP project input exceeds the {} MiB per-file limit: {}",
                    MAX_PROJECT_FILE_BYTES / (1024 * 1024),
                    entry.path().display()
                ));
            }
            total_bytes = total_bytes.saturating_add(bytes);
            if total_bytes > MAX_PROJECT_INPUT_BYTES {
                return Err(format!(
                    "MCP project inputs exceed the {} MiB cumulative limit",
                    MAX_PROJECT_INPUT_BYTES / (1024 * 1024)
                ));
            }
        }
    }

    Ok(())
}

fn validate_tree_confinement(
    root: &Path,
    start: &Path,
    label: &str,
    exclusions: &HashSet<String>,
    entries_seen: &mut usize,
) -> Result<(), String> {
    let entries = WalkDir::new(start)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            entry.depth() == 0
                || !entry.file_type().is_dir()
                || !entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| exclusions.contains(name))
        });

    for entry in entries {
        *entries_seen += 1;
        if *entries_seen > MAX_CONFINEMENT_ENTRIES {
            return Err(format!(
                "MCP {label} confinement scan exceeds {MAX_CONFINEMENT_ENTRIES} entries"
            ));
        }
        let entry =
            entry.map_err(|error| format!("Cannot inspect entry beneath MCP {label}: {error}"))?;
        if !entry.file_type().is_symlink() {
            continue;
        }
        let canonical = entry.path().canonicalize().map_err(|error| {
            format!(
                "Cannot resolve symlink beneath MCP {label} {}: {error}",
                entry.path().display()
            )
        })?;
        if !canonical.starts_with(root) {
            return Err(format!(
                "Symlink beneath MCP {label} escapes the configured server root: {}",
                entry.path().display()
            ));
        }
    }

    Ok(())
}

pub(super) fn validate_spec_file_mappings(
    root: &Path,
    spec_files: &[PathBuf],
    exclude_dirs: &[String],
) -> Result<(), String> {
    let mut confinement_entries_seen = 0usize;
    let exclusions: HashSet<String> = exclude_dirs.iter().cloned().collect();
    for spec_file in spec_files {
        validate_existing_path(root, spec_file, "spec file", None)?;
        let content = read_file_bounded(root, spec_file, "spec file")?;
        let Some(parsed) = crate::parser::parse_frontmatter(&content) else {
            continue;
        };
        for file in &parsed.frontmatter.files {
            validate_configured_path_with_budget(
                root,
                file,
                "spec file mapping",
                Some(&exclusions),
                &mut confinement_entries_seen,
            )?;
        }
        for dependency in &parsed.frontmatter.depends_on {
            if crate::validator::is_cross_project_ref(dependency) {
                continue;
            }
            if !dependency.contains('/') && !dependency.contains('.') {
                validate_module_name(dependency)?;
            } else {
                validate_configured_path_with_budget(
                    root,
                    dependency,
                    "dependency reference",
                    None,
                    &mut confinement_entries_seen,
                )?;
            }
        }
        validate_consumed_by_paths(root, &parsed.body, &mut confinement_entries_seen)?;
    }
    Ok(())
}

fn validate_consumed_by_paths(
    root: &Path,
    body: &str,
    confinement_entries_seen: &mut usize,
) -> Result<(), String> {
    let mut in_section = false;
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed == "### Consumed By" {
            in_section = true;
            continue;
        }
        if in_section && (trimmed.starts_with("## ") || trimmed.starts_with("### ")) {
            break;
        }
        if !in_section || !trimmed.starts_with('|') {
            continue;
        }
        for (index, value) in line.split('`').enumerate() {
            if index % 2 == 1 && value.rsplit_once('.').is_some() {
                validate_configured_path_with_budget(
                    root,
                    value,
                    "Consumed By file reference",
                    None,
                    confinement_entries_seen,
                )?;
            }
        }
    }
    Ok(())
}

fn validate_module_name(module_name: &str) -> Result<(), String> {
    crate::commands::validate_module_name(module_name)
        .map_err(|error| format!("MCP module name is unsafe: {error}"))
}

pub(super) fn validate_generated_module_names(module_names: &[String]) -> Result<(), String> {
    for module_name in module_names {
        validate_module_name(module_name)?;
    }
    Ok(())
}

pub(super) struct ConfinedReadRoot<'root> {
    root: &'root Dir,
    root_identity: DirectoryIdentity,
    directory: Dir,
    components: Vec<(PathBuf, SnapshotEntryIdentity)>,
}

impl ConfinedReadRoot<'_> {
    pub(super) fn directory(&self) -> &Dir {
        &self.directory
    }

    pub(super) fn revalidate_before_success(&self) -> Result<(), String> {
        #[cfg(debug_assertions)]
        wait_for_mcp_read_root_revalidation_test_barrier()?;

        let observed_root = directory_identity(self.root)
            .map_err(|error| format!("Cannot reidentify MCP read-root authority: {error}"))?;
        if observed_root != self.root_identity {
            return Err("MCP read-root authority changed before operation success".to_string());
        }

        let retained_identity = directory_identity(&self.directory)
            .map_err(|error| format!("Cannot reidentify retained MCP read root: {error}"))?;
        let expected_final = self
            .components
            .last()
            .map(|(_, identity)| *identity)
            .unwrap_or(self.root_identity);
        if retained_identity != expected_final {
            return Err("Retained MCP read root changed before operation success".to_string());
        }

        let mut directory = self
            .root
            .try_clone()
            .map_err(|error| format!("Cannot clone MCP read-root authority: {error}"))?;
        for (relative, expected) in &self.components {
            let Some(name) = relative.file_name() else {
                return Err(format!(
                    "MCP read-root route is invalid before operation success: {}",
                    relative.display()
                ));
            };
            let Some((next, observed)) = open_snapshot_configuration_directory(
                &directory, name, relative,
            )
            .map_err(|error| {
                format!(
                    "MCP read-root route changed before operation success at {}: {error}",
                    relative.display()
                )
            })?
            else {
                return Err(format!(
                    "MCP read-root route changed before operation success at {}",
                    relative.display()
                ));
            };
            if observed != *expected {
                return Err(format!(
                    "MCP read-root route changed before operation success at {}",
                    relative.display()
                ));
            }
            directory = next;
        }
        Ok(())
    }
}

pub(super) fn open_confined_read_root<'root>(
    server_directory: &'root Dir,
    relative: &Path,
) -> Result<ConfinedReadRoot<'root>, String> {
    let root_identity = directory_identity(server_directory)
        .map_err(|error| format!("Cannot identify MCP read-root authority: {error}"))?;
    let mut directory = server_directory
        .try_clone()
        .map_err(|error| format!("Cannot clone MCP server root capability: {error}"))?;
    let mut display = PathBuf::new();
    let mut components = Vec::new();
    for component in relative.components() {
        match component {
            Component::CurDir => continue,
            Component::Normal(name) => {
                display.push(name);
                let Some((next, identity)) =
                    open_snapshot_configuration_directory(&directory, name, &display)
                        .map_err(|error| {
                            format!(
                                "Read root override does not resolve through regular confined directories {}: {error}",
                                display.display()
                            )
                        })?
                else {
                    return Err(format!(
                        "Read root override does not resolve to a confined existing directory {}",
                        display.display()
                    ));
                };
                directory = next;
                components.push((display.clone(), identity));
            }
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(format!(
                    "Read root override must remain beneath the MCP server root: {}",
                    relative.display()
                ));
            }
        }
    }
    Ok(ConfinedReadRoot {
        root: server_directory,
        root_identity,
        directory,
        components,
    })
}

pub(super) fn validate_confined_relative(relative: &Path, label: &str) -> Result<(), String> {
    if relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(format!(
            "MCP {label} must be a project-relative path without traversal: {}",
            relative.display()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
