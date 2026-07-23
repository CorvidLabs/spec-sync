use cap_std::ambient_authority;
use cap_std::fs::{Dir, Metadata};
use colored::Colorize;
use std::collections::{HashMap, HashSet};
use std::ffi::{OsStr, OsString};
use std::fmt::Write;
use std::fs;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};
use std::process;

use crate::config::load_config;
use crate::github;
use crate::ignore::IgnoreRules;
use crate::parser;
use crate::schema;
use crate::types;
use crate::types::SpecStatus;
use crate::validator::{
    SourceSnapshot, get_schema_table_names, normalize_source_mapping,
    validate_spec_content_with_sources,
};

use super::{ValidationErrors, build_schema_columns, create_drift_issues_with_diagnostics};

type IssueReferences = (String, Vec<u64>, Vec<u64>);

const MAX_SPEC_SNAPSHOT_BYTES: u64 = 4 * 1024 * 1024;
const MAX_SPEC_SNAPSHOT_TOTAL_BYTES: u64 = 64 * 1024 * 1024;
const MAX_SPEC_SNAPSHOT_FILES: usize = 10_000;
const MAX_SPEC_SNAPSHOT_ENTRIES: usize = 100_000;

struct OpenedSpecsDirectory {
    project: Dir,
    directory: Dir,
    project_relative: PathBuf,
}

struct SpecSnapshot {
    relative_path: String,
    content: String,
}

#[derive(Debug, PartialEq, Eq)]
enum SpecInspectionFindingKind {
    ConfigurationError,
    DiscoveryError,
    ReadError,
    MalformedFrontmatter,
}

impl SpecInspectionFindingKind {
    fn as_str(&self) -> &'static str {
        match self {
            Self::ConfigurationError => "configuration_error",
            Self::DiscoveryError => "discovery_error",
            Self::ReadError => "read_error",
            Self::MalformedFrontmatter => "malformed_frontmatter",
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::ConfigurationError => {
                "Configured specs directory is not confined to the project."
            }
            Self::DiscoveryError => "Unable to inspect spec path.",
            Self::ReadError => "Unable to read spec file.",
            Self::MalformedFrontmatter => "Malformed or missing spec frontmatter.",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct SpecInspectionFinding {
    spec: String,
    kind: SpecInspectionFindingKind,
}

fn issue_text_summary(
    reference_specs: usize,
    valid: usize,
    closed: usize,
    not_found: usize,
    errors: usize,
    inspection_findings: usize,
) -> Option<String> {
    (reference_specs > 0 || inspection_findings > 0).then(|| {
        let issue_summary = format!(
            "Issue references: {valid} valid, {closed} closed, {not_found} not found, {errors} errors"
        );
        if inspection_findings > 0 {
            format!("{issue_summary}; {inspection_findings} spec inspection findings")
        } else {
            issue_summary
        }
    })
}

fn slash_normalized_relative_path(path: &Path) -> Option<String> {
    let path = path.to_str()?;
    #[cfg(windows)]
    {
        Some(path.replace('\\', "/"))
    }
    #[cfg(not(windows))]
    {
        Some(path.to_string())
    }
}

fn relative_path_finding(path: &Path, kind: SpecInspectionFindingKind) -> SpecInspectionFinding {
    SpecInspectionFinding {
        spec: slash_normalized_relative_path(path)
            .unwrap_or_else(|| "<non-utf8-spec-path>".to_string()),
        kind,
    }
}

fn non_utf8_spec_discovery_finding() -> SpecInspectionFinding {
    SpecInspectionFinding {
        spec: "<non-utf8-spec-path>".to_string(),
        kind: SpecInspectionFindingKind::DiscoveryError,
    }
}

fn specs_dir_configuration_finding() -> SpecInspectionFinding {
    SpecInspectionFinding {
        spec: "<configured-specs-dir>".to_string(),
        kind: SpecInspectionFindingKind::ConfigurationError,
    }
}

#[cfg(unix)]
type FileSystemIdentity = (u64, u64);

#[cfg(unix)]
fn metadata_identity(metadata: &Metadata) -> io::Result<FileSystemIdentity> {
    use cap_std::fs::MetadataExt;

    Ok((metadata.dev(), metadata.ino()))
}

#[cfg(windows)]
type FileSystemIdentity = (u32, u64);

#[cfg(windows)]
fn windows_handle_identity(handle: *mut std::ffi::c_void) -> io::Result<FileSystemIdentity> {
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
    // SAFETY: the caller retains a valid file or directory handle for this call, and
    // `information` is writable storage for the exact Win32 output structure.
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
type FileSystemIdentity = (u64, Option<std::time::SystemTime>);

#[cfg(not(any(unix, windows)))]
fn metadata_identity(metadata: &Metadata) -> io::Result<FileSystemIdentity> {
    Ok((metadata.len(), metadata.modified().ok()))
}

#[cfg(unix)]
fn directory_identity(directory: &Dir) -> io::Result<FileSystemIdentity> {
    metadata_identity(&directory.dir_metadata()?)
}

#[cfg(windows)]
fn directory_identity(directory: &Dir) -> io::Result<FileSystemIdentity> {
    use std::os::windows::io::AsRawHandle;

    let file = directory.try_clone()?.into_std_file();
    windows_handle_identity(file.as_raw_handle().cast())
}

#[cfg(not(any(unix, windows)))]
fn directory_identity(directory: &Dir) -> io::Result<FileSystemIdentity> {
    metadata_identity(&directory.dir_metadata()?)
}

#[cfg(windows)]
fn file_identity(file: &cap_std::fs::File) -> io::Result<FileSystemIdentity> {
    use std::os::windows::io::AsRawHandle;

    windows_handle_identity(file.as_raw_handle().cast())
}

#[cfg(windows)]
fn is_reparse_point(metadata: &Metadata) -> bool {
    use cap_std::fs::MetadataExt;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_link_or_reparse_point(metadata: &Metadata) -> bool {
    metadata.file_type().is_symlink()
}

#[cfg(windows)]
fn is_link_or_reparse_point(metadata: &Metadata) -> bool {
    is_reparse_point(metadata)
}

fn invalid_entry_type() -> io::Error {
    io::Error::new(
        io::ErrorKind::PermissionDenied,
        "untrusted filesystem entry",
    )
}

#[cfg(not(windows))]
fn open_verified_directory(parent: &Dir, name: &OsStr) -> io::Result<Dir> {
    let before = parent.symlink_metadata(name)?;
    if before.file_type().is_symlink() || !before.is_dir() {
        return Err(invalid_entry_type());
    }
    let before_identity = metadata_identity(&before)?;

    let directory = parent.open_dir(name)?;
    let opened_identity = metadata_identity(&directory.dir_metadata()?)?;
    let after = parent.symlink_metadata(name)?;
    if after.file_type().is_symlink()
        || !after.is_dir()
        || metadata_identity(&after)? != before_identity
        || opened_identity != before_identity
    {
        return Err(invalid_entry_type());
    }

    Ok(directory)
}

#[cfg(windows)]
fn open_verified_directory(parent: &Dir, name: &OsStr) -> io::Result<Dir> {
    let before = parent.symlink_metadata(name)?;
    if is_reparse_point(&before) || !before.is_dir() {
        return Err(invalid_entry_type());
    }

    let directory = parent.open_dir(name)?;
    let opened_identity = directory_identity(&directory)?;
    let after_open = parent.symlink_metadata(name)?;
    if is_reparse_point(&after_open) || !after_open.is_dir() {
        return Err(invalid_entry_type());
    }
    let observed = parent.open_dir(name)?;
    let after_observed = parent.symlink_metadata(name)?;
    if is_reparse_point(&after_observed)
        || !after_observed.is_dir()
        || directory_identity(&observed)? != opened_identity
    {
        return Err(invalid_entry_type());
    }

    Ok(directory)
}

#[cfg(not(windows))]
fn discovered_file_identity(
    _parent: &Dir,
    _name: &OsStr,
    metadata: &Metadata,
) -> io::Result<FileSystemIdentity> {
    metadata_identity(metadata)
}

#[cfg(windows)]
fn discovered_file_identity(
    parent: &Dir,
    name: &OsStr,
    metadata: &Metadata,
) -> io::Result<FileSystemIdentity> {
    if is_reparse_point(metadata) || !metadata.is_file() {
        return Err(invalid_entry_type());
    }
    let file = parent.open(name)?;
    let identity = file_identity(&file)?;
    let observed = parent.open(name)?;
    let after_observed = parent.symlink_metadata(name)?;
    if is_reparse_point(&after_observed)
        || !after_observed.is_file()
        || file_identity(&observed)? != identity
    {
        return Err(invalid_entry_type());
    }
    Ok(identity)
}

fn snapshot_limit_error() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "spec snapshot exceeds the inspection limit",
    )
}

#[cfg(not(windows))]
fn read_verified_bytes(
    parent: &Dir,
    name: &OsStr,
    expected_identity: FileSystemIdentity,
    max_bytes: u64,
) -> io::Result<Vec<u8>> {
    let before = parent.symlink_metadata(name)?;
    if before.file_type().is_symlink() || !before.is_file() {
        return Err(invalid_entry_type());
    }
    let before_identity = metadata_identity(&before)?;
    if before_identity != expected_identity {
        return Err(invalid_entry_type());
    }

    let mut file = parent.open(name)?;
    let opened_identity = metadata_identity(&file.metadata()?)?;
    let after_open = parent.symlink_metadata(name)?;
    if after_open.file_type().is_symlink()
        || !after_open.is_file()
        || metadata_identity(&after_open)? != before_identity
        || opened_identity != before_identity
    {
        return Err(invalid_entry_type());
    }

    let mut bytes = Vec::new();
    file.by_ref().take(max_bytes + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max_bytes {
        return Err(snapshot_limit_error());
    }

    let after_read = parent.symlink_metadata(name)?;
    if after_read.file_type().is_symlink()
        || !after_read.is_file()
        || metadata_identity(&after_read)? != before_identity
        || metadata_identity(&file.metadata()?)? != opened_identity
    {
        return Err(invalid_entry_type());
    }

    Ok(bytes)
}

#[cfg(windows)]
fn read_verified_bytes(
    parent: &Dir,
    name: &OsStr,
    expected_identity: FileSystemIdentity,
    max_bytes: u64,
) -> io::Result<Vec<u8>> {
    let before = parent.symlink_metadata(name)?;
    if is_reparse_point(&before) || !before.is_file() {
        return Err(invalid_entry_type());
    }

    let mut file = parent.open(name)?;
    let opened_identity = file_identity(&file)?;
    if opened_identity != expected_identity {
        return Err(invalid_entry_type());
    }
    let after_open = parent.symlink_metadata(name)?;
    if is_reparse_point(&after_open) || !after_open.is_file() {
        return Err(invalid_entry_type());
    }
    let observed = parent.open(name)?;
    let after_observed = parent.symlink_metadata(name)?;
    if is_reparse_point(&after_observed)
        || !after_observed.is_file()
        || file_identity(&observed)? != opened_identity
    {
        return Err(invalid_entry_type());
    }

    let mut bytes = Vec::new();
    file.by_ref().take(max_bytes + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max_bytes {
        return Err(snapshot_limit_error());
    }

    let after_read = parent.symlink_metadata(name)?;
    if is_reparse_point(&after_read) || !after_read.is_file() {
        return Err(invalid_entry_type());
    }
    let observed = parent.open(name)?;
    let after_observed = parent.symlink_metadata(name)?;
    if is_reparse_point(&after_observed)
        || !after_observed.is_file()
        || file_identity(&observed)? != opened_identity
        || file_identity(&file)? != opened_identity
    {
        return Err(invalid_entry_type());
    }

    Ok(bytes)
}

fn read_verified_file(
    parent: &Dir,
    name: &OsStr,
    expected_identity: FileSystemIdentity,
    max_bytes: u64,
) -> io::Result<String> {
    String::from_utf8(read_verified_bytes(
        parent,
        name,
        expected_identity,
        max_bytes,
    )?)
    .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "spec is not valid UTF-8"))
}

fn open_verified_root(root: &Path) -> io::Result<Dir> {
    let directory = Dir::open_ambient_dir(root, ambient_authority())?;
    let expected_identity = directory_identity(&directory)?;
    let canonical_root = fs::canonicalize(root)?;
    let observed = match (canonical_root.parent(), canonical_root.file_name()) {
        (Some(parent), Some(name)) => {
            let parent = Dir::open_ambient_dir(parent, ambient_authority())?;
            parent.open_dir(name)?
        }
        _ => Dir::open_ambient_dir(&canonical_root, ambient_authority())?,
    };
    if directory_identity(&observed)? != expected_identity {
        return Err(invalid_entry_type());
    }

    Ok(directory)
}

fn open_specs_directory(
    root: &Path,
    configured: &str,
) -> Result<Option<OpenedSpecsDirectory>, SpecInspectionFinding> {
    let configured = Path::new(configured);
    if configured.is_absolute()
        || configured
            .components()
            .any(|component| !matches!(component, Component::CurDir | Component::Normal(_)))
    {
        return Err(specs_dir_configuration_finding());
    }

    let components = configured
        .components()
        .filter_map(|component| match component {
            Component::Normal(component) => Some(component.to_os_string()),
            Component::CurDir => None,
            _ => unreachable!("non-confined components were rejected above"),
        })
        .collect::<Vec<_>>();
    if components.is_empty() {
        return Err(specs_dir_configuration_finding());
    }

    let project = open_verified_root(root).map_err(|_| specs_dir_configuration_finding())?;
    let mut directory = project
        .try_clone()
        .map_err(|_| specs_dir_configuration_finding())?;
    for component in &components {
        match open_verified_directory(&directory, component) {
            Ok(child) => directory = child,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err(specs_dir_configuration_finding()),
        }
    }

    Ok(Some(OpenedSpecsDirectory {
        project,
        directory,
        project_relative: components.iter().collect(),
    }))
}

fn is_spec_shaped_file_name(file_name: &OsStr) -> bool {
    if let Some(file_name) = file_name.to_str() {
        return file_name.ends_with(".spec.md");
    }

    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;

        file_name.as_bytes().ends_with(b".spec.md")
    }

    #[cfg(not(unix))]
    {
        false
    }
}

fn find_spec_snapshots_checked(
    specs: &OpenedSpecsDirectory,
) -> (Vec<SpecSnapshot>, Vec<SpecInspectionFinding>) {
    find_spec_snapshots_checked_with_hook(specs, |_| {})
}

fn find_spec_snapshots_checked_with_hook<Hook>(
    specs: &OpenedSpecsDirectory,
    after_discovery: Hook,
) -> (Vec<SpecSnapshot>, Vec<SpecInspectionFinding>)
where
    Hook: FnMut(&Path),
{
    find_spec_snapshots_checked_with_limits(
        specs,
        after_discovery,
        MAX_SPEC_SNAPSHOT_BYTES,
        MAX_SPEC_SNAPSHOT_TOTAL_BYTES,
        MAX_SPEC_SNAPSHOT_FILES,
        MAX_SPEC_SNAPSHOT_ENTRIES,
    )
}

fn find_spec_snapshots_checked_with_limits<Hook>(
    specs: &OpenedSpecsDirectory,
    mut after_discovery: Hook,
    max_file_bytes: u64,
    max_total_bytes: u64,
    max_files: usize,
    max_entries: usize,
) -> (Vec<SpecSnapshot>, Vec<SpecInspectionFinding>)
where
    Hook: FnMut(&Path),
{
    let mut snapshots = Vec::new();
    let mut findings = Vec::new();
    let mut total_bytes = 0u64;
    let mut total_files = 0usize;
    let mut total_entries = 0usize;
    collect_spec_snapshots(
        &specs.directory,
        &specs.project_relative,
        Path::new(""),
        &mut after_discovery,
        &mut snapshots,
        &mut findings,
        &mut total_bytes,
        &mut total_files,
        &mut total_entries,
        max_file_bytes,
        max_total_bytes,
        max_files,
        max_entries,
    );
    snapshots.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    findings.sort_by(|left, right| left.spec.cmp(&right.spec));
    (snapshots, findings)
}

#[allow(clippy::too_many_arguments)]
fn collect_spec_snapshots<Hook>(
    directory: &Dir,
    specs_relative: &Path,
    directory_relative: &Path,
    after_discovery: &mut Hook,
    snapshots: &mut Vec<SpecSnapshot>,
    findings: &mut Vec<SpecInspectionFinding>,
    total_bytes: &mut u64,
    total_files: &mut usize,
    total_entries: &mut usize,
    max_file_bytes: u64,
    max_total_bytes: u64,
    max_files: usize,
    max_entries: usize,
) where
    Hook: FnMut(&Path),
{
    let entries = match directory.entries() {
        Ok(entries) => entries,
        Err(_) => {
            findings.push(relative_path_finding(
                &specs_relative.join(directory_relative),
                SpecInspectionFindingKind::DiscoveryError,
            ));
            return;
        }
    };
    let mut names = Vec::<OsString>::new();
    for result in entries {
        if *total_entries >= max_entries {
            findings.push(relative_path_finding(
                &specs_relative.join(directory_relative),
                SpecInspectionFindingKind::DiscoveryError,
            ));
            return;
        }
        *total_entries += 1;
        match result {
            Ok(entry) => names.push(entry.file_name()),
            Err(_) => findings.push(relative_path_finding(
                &specs_relative.join(directory_relative),
                SpecInspectionFindingKind::DiscoveryError,
            )),
        }
    }
    names.sort();

    for name in names {
        let relative_entry = directory_relative.join(&name);
        let project_relative = specs_relative.join(&relative_entry);
        if *total_files >= max_files {
            findings.push(relative_path_finding(
                &project_relative,
                SpecInspectionFindingKind::ReadError,
            ));
            return;
        }
        let metadata = match directory.symlink_metadata(&name) {
            Ok(metadata) => metadata,
            Err(_) => {
                findings.push(relative_path_finding(
                    &project_relative,
                    SpecInspectionFindingKind::DiscoveryError,
                ));
                continue;
            }
        };
        let discovered_identity = if metadata.is_file() && is_spec_shaped_file_name(&name) {
            discovered_file_identity(directory, &name, &metadata).ok()
        } else {
            None
        };
        after_discovery(&project_relative);

        if is_link_or_reparse_point(&metadata) {
            findings.push(relative_path_finding(
                &project_relative,
                SpecInspectionFindingKind::DiscoveryError,
            ));
            continue;
        }

        if metadata.is_dir() {
            if is_spec_shaped_file_name(&name) {
                findings.push(relative_path_finding(
                    &project_relative,
                    SpecInspectionFindingKind::DiscoveryError,
                ));
                continue;
            }
            match open_verified_directory(directory, &name) {
                Ok(child) => collect_spec_snapshots(
                    &child,
                    specs_relative,
                    &relative_entry,
                    after_discovery,
                    snapshots,
                    findings,
                    total_bytes,
                    total_files,
                    total_entries,
                    max_file_bytes,
                    max_total_bytes,
                    max_files,
                    max_entries,
                ),
                Err(_) => findings.push(relative_path_finding(
                    &project_relative,
                    SpecInspectionFindingKind::DiscoveryError,
                )),
            }
            continue;
        }

        if !is_spec_shaped_file_name(&name) {
            continue;
        }
        let Some(relative_path) = slash_normalized_relative_path(&project_relative) else {
            findings.push(non_utf8_spec_discovery_finding());
            continue;
        };
        if !metadata.is_file() {
            findings.push(relative_path_finding(
                &project_relative,
                SpecInspectionFindingKind::DiscoveryError,
            ));
            continue;
        }

        let expected_identity = match discovered_identity {
            Some(identity) => identity,
            None => {
                findings.push(relative_path_finding(
                    &project_relative,
                    SpecInspectionFindingKind::ReadError,
                ));
                continue;
            }
        };
        *total_files += 1;

        match read_verified_file(directory, &name, expected_identity, max_file_bytes) {
            Ok(content)
                if total_bytes
                    .checked_add(content.len() as u64)
                    .is_some_and(|total| total <= max_total_bytes) =>
            {
                *total_bytes += content.len() as u64;
                snapshots.push(SpecSnapshot {
                    relative_path,
                    content,
                });
            }
            Ok(_) => findings.push(relative_path_finding(
                &project_relative,
                SpecInspectionFindingKind::ReadError,
            )),
            Err(_) => findings.push(relative_path_finding(
                &project_relative,
                SpecInspectionFindingKind::ReadError,
            )),
        }
    }
}

fn inspect_spec(snapshot: &SpecSnapshot) -> Result<Option<IssueReferences>, SpecInspectionFinding> {
    let (implements, tracks) =
        parser::parse_checked_issue_references(&snapshot.content).map_err(|_| {
            SpecInspectionFinding {
                spec: snapshot.relative_path.clone(),
                kind: SpecInspectionFindingKind::MalformedFrontmatter,
            }
        })?;

    if implements.is_empty() && tracks.is_empty() {
        return Ok(None);
    }

    Ok(Some((snapshot.relative_path.clone(), implements, tracks)))
}

type SnapshotValidationDiagnostics = (ValidationErrors, Vec<String>, Vec<String>);

fn snapshot_source_file(project: &Dir, mapping: &str, max_bytes: u64) -> SourceSnapshot {
    let path = Path::new(mapping);
    if path.is_absolute()
        || mapping.contains('\\')
        || path
            .components()
            .any(|component| !matches!(component, Component::CurDir | Component::Normal(_)))
    {
        return SourceSnapshot::Rejected;
    }
    let components = path
        .components()
        .filter_map(|component| match component {
            Component::Normal(component) => Some(component),
            Component::CurDir => None,
            _ => None,
        })
        .collect::<Vec<_>>();
    let Some((name, parents)) = components.split_last() else {
        return SourceSnapshot::Rejected;
    };

    let mut directory = match project.try_clone() {
        Ok(directory) => directory,
        Err(_) => return SourceSnapshot::Unreadable,
    };
    for component in parents {
        directory = match open_verified_directory(&directory, component) {
            Ok(child) => child,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return SourceSnapshot::Missing;
            }
            Err(_) => return SourceSnapshot::Rejected,
        };
    }

    let metadata = match directory.symlink_metadata(name) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return SourceSnapshot::Missing,
        Err(_) => return SourceSnapshot::Unreadable,
    };
    if is_link_or_reparse_point(&metadata) || !metadata.is_file() {
        return SourceSnapshot::Rejected;
    }
    let identity = match discovered_file_identity(&directory, name, &metadata) {
        Ok(identity) => identity,
        Err(_) => return SourceSnapshot::Rejected,
    };
    match read_verified_bytes(&directory, name, identity, max_bytes) {
        Ok(bytes) => SourceSnapshot::Present(bytes),
        Err(error) if error.kind() == io::ErrorKind::NotFound => SourceSnapshot::Missing,
        Err(error) if error.kind() == io::ErrorKind::PermissionDenied => SourceSnapshot::Rejected,
        Err(_) => SourceSnapshot::Unreadable,
    }
}

fn snapshot_mapped_sources(
    project: &Dir,
    snapshots: &[SpecSnapshot],
) -> HashMap<String, SourceSnapshot> {
    let mut mappings = HashSet::new();
    for snapshot in snapshots {
        let normalized = if snapshot.content.contains("\r\n") {
            std::borrow::Cow::Owned(snapshot.content.replace("\r\n", "\n"))
        } else {
            std::borrow::Cow::Borrowed(snapshot.content.as_str())
        };
        if let Some(parsed) = parser::parse_frontmatter(&normalized) {
            mappings.extend(parsed.frontmatter.files);
        }
    }

    let mut mappings = mappings.into_iter().collect::<Vec<_>>();
    mappings.sort();
    let mut total_bytes = 0u64;
    let mut sources = HashMap::new();
    for (index, mapping) in mappings.into_iter().enumerate() {
        if index >= MAX_SPEC_SNAPSHOT_FILES {
            sources.insert(mapping, SourceSnapshot::Unreadable);
            continue;
        }
        let remaining = MAX_SPEC_SNAPSHOT_TOTAL_BYTES.saturating_sub(total_bytes);
        if remaining == 0 {
            sources.insert(mapping, SourceSnapshot::Unreadable);
            continue;
        }
        let source =
            snapshot_source_file(project, &mapping, remaining.min(MAX_SPEC_SNAPSHOT_BYTES));
        if let SourceSnapshot::Present(bytes) = &source {
            total_bytes += bytes.len() as u64;
        }
        sources.insert(mapping, source);
    }
    sources
}

fn collect_snapshot_validation(
    project: &Dir,
    root: &Path,
    snapshots: &[SpecSnapshot],
    config: &types::SpecSyncConfig,
) -> SnapshotValidationDiagnostics {
    collect_snapshot_validation_with_hook(project, root, snapshots, config, || {})
}

fn collect_snapshot_validation_with_hook<Hook>(
    project: &Dir,
    root: &Path,
    snapshots: &[SpecSnapshot],
    config: &types::SpecSyncConfig,
    after_snapshot: Hook,
) -> SnapshotValidationDiagnostics
where
    Hook: FnOnce(),
{
    let source_snapshots = snapshot_mapped_sources(project, snapshots);
    after_snapshot();

    let schema_tables = get_schema_table_names(root, config);
    let schema_columns = build_schema_columns(root, config);
    let ignore_rules = IgnoreRules::default();
    let mut file_owners: HashMap<String, Vec<String>> = HashMap::new();
    let mut spec_files_by_path: HashMap<&str, HashSet<String>> = HashMap::new();

    for snapshot in snapshots {
        let normalized = if snapshot.content.contains("\r\n") {
            std::borrow::Cow::Owned(snapshot.content.replace("\r\n", "\n"))
        } else {
            std::borrow::Cow::Borrowed(snapshot.content.as_str())
        };
        let Some(parsed) = parser::parse_frontmatter(&normalized) else {
            continue;
        };
        if parsed.frontmatter.parsed_status() == Some(SpecStatus::Archived) {
            continue;
        }

        let owner = snapshot.relative_path.replace('\\', "/");
        let mut existing_files = HashSet::new();
        for file in &parsed.frontmatter.files {
            if matches!(source_snapshots.get(file), Some(SourceSnapshot::Present(_)))
                && !file.contains('\\')
                && let Some(normalized_file) = normalize_source_mapping(file)
            {
                let owners = file_owners.entry(normalized_file.clone()).or_default();
                if !owners.contains(&owner) {
                    owners.push(owner.clone());
                }
                existing_files.insert(normalized_file);
            }
        }
        spec_files_by_path.insert(&snapshot.relative_path, existing_files);
    }

    let mut all_errors = ValidationErrors::default();
    let mut all_warnings = Vec::new();
    let mut all_notices = Vec::new();
    for snapshot in snapshots {
        let logical_path = root.join(&snapshot.relative_path);
        let mut result = validate_spec_content_with_sources(
            &logical_path,
            &snapshot.content,
            root,
            &schema_tables,
            &schema_columns,
            config,
            &source_snapshots,
        );
        let owner = snapshot.relative_path.replace('\\', "/");
        for file in spec_files_by_path
            .get(snapshot.relative_path.as_str())
            .into_iter()
            .flat_map(|files| files.iter())
        {
            if let Some(owners) = file_owners.get(file).filter(|owners| owners.len() > 1) {
                let others = owners
                    .iter()
                    .filter(|candidate| *candidate != &owner)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ");
                result.errors.push(format!(
                    "Source file has duplicate spec ownership: {file} (also mapped by {others})"
                ));
            }
        }

        let inline_ignores = IgnoreRules::parse_inline(&snapshot.content);
        let filtered_warnings = result
            .warnings
            .iter()
            .filter(|warning| {
                !ignore_rules.is_suppressed(warning, &result.spec_path, &inline_ignores)
            })
            .collect::<Vec<_>>();
        let prefix = &result.spec_path;
        for error in &result.errors {
            all_errors.push_for_spec(prefix, error);
        }
        all_warnings.extend(
            filtered_warnings
                .iter()
                .map(|warning| format!("{prefix}: {warning}")),
        );
        all_notices.extend(
            result
                .notices
                .iter()
                .map(|notice| format!("{prefix}: {notice}")),
        );
    }

    if let Some(directory) = &config.schema_dir {
        for error in schema::schema_read_errors(&root.join(directory)) {
            all_errors.push_unattributed(error);
        }
    }

    (all_errors, all_warnings, all_notices)
}

fn is_unsafe_diagnostic_character(character: char) -> bool {
    character.is_control()
        || matches!(
            character,
            '\u{061c}'
                | '\u{200e}'
                | '\u{200f}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202a}'..='\u{202e}'
                | '\u{2066}'..='\u{2069}'
        )
}

pub(super) fn safe_diagnostic(value: &str) -> String {
    let mut safe = String::with_capacity(value.len());
    for character in value.chars() {
        if is_unsafe_diagnostic_character(character) {
            write!(&mut safe, "\\u{{{:04X}}}", character as u32)
                .expect("writing to a String cannot fail");
        } else {
            safe.push(character);
        }
    }
    safe
}

fn markdown_cell(value: &str) -> String {
    safe_diagnostic(value)
        .replace('\\', "\\\\")
        .replace('|', "\\|")
}

fn markdown_code_span(value: &str) -> String {
    let value = markdown_cell(value);
    let longest_backtick_run = value
        .split(|character| character != '`')
        .map(str::len)
        .max()
        .unwrap_or(0);
    let delimiter = "`".repeat(longest_backtick_run + 1);
    if value.starts_with('`') || value.ends_with('`') {
        format!("{delimiter} {value} {delimiter}")
    } else {
        format!("{delimiter}{value}{delimiter}")
    }
}

fn issue_verification_json(verification: &github::IssueVerification) -> serde_json::Value {
    serde_json::json!({
        "spec": safe_diagnostic(&verification.spec_path),
        "valid": verification.valid.iter().map(|issue| serde_json::json!({
            "number": issue.number,
            "title": safe_diagnostic(&issue.title),
            "state": safe_diagnostic(&issue.state),
        })).collect::<Vec<_>>(),
        "closed": verification.closed.iter().map(|issue| serde_json::json!({
            "number": issue.number,
            "title": safe_diagnostic(&issue.title),
        })).collect::<Vec<_>>(),
        "not_found": verification.not_found,
        "errors": verification.errors.iter().map(|error| safe_diagnostic(error)).collect::<Vec<_>>(),
    })
}

pub fn cmd_issues(root: &Path, format: types::OutputFormat, create: bool) {
    let config = load_config(root);
    let (opened_specs, snapshots, mut inspection_findings) =
        match open_specs_directory(root, &config.specs_dir) {
            Ok(Some(specs)) => {
                let (snapshots, findings) = find_spec_snapshots_checked(&specs);
                (Some(specs), snapshots, findings)
            }
            Ok(None) => (None, Vec::new(), Vec::new()),
            Err(finding) => (None, Vec::new(), vec![finding]),
        };

    let mut total_valid = 0usize;
    let mut total_closed = 0usize;
    let mut total_not_found = 0usize;
    let mut total_errors = 0usize;
    let mut json_results: Vec<serde_json::Value> = Vec::new();
    let mut references = Vec::new();

    for snapshot in &snapshots {
        match inspect_spec(snapshot) {
            Ok(Some(reference)) => references.push(reference),
            Ok(None) => {}
            Err(finding) => inspection_findings.push(finding),
        }
    }

    let repo_config = config.github.as_ref().and_then(|g| g.repo.as_deref());
    let repo = match (repo_config, references.is_empty()) {
        (None, true) => None,
        _ => {
            let resolved = match github::resolve_repo(repo_config, root) {
                Ok(repo) => repo,
                Err(error) => {
                    eprintln!("{} {}", "error:".red().bold(), safe_diagnostic(&error));
                    process::exit(1);
                }
            };
            Some(resolved)
        }
    };

    if snapshots.is_empty() && inspection_findings.is_empty() {
        println!("No spec files found.");
        return;
    }

    if matches!(format, types::OutputFormat::Text)
        && let Some(repo) = repo.as_deref()
        && !references.is_empty()
    {
        println!(
            "Verifying issue references against {}...\n",
            safe_diagnostic(repo)
        );
    }

    let verifications = repo
        .as_deref()
        .filter(|_| !references.is_empty())
        .map(|repo| github::verify_issue_batch(repo, &references))
        .unwrap_or_default();

    for verification in verifications {
        let rel_path = verification.spec_path.clone();

        total_valid += verification.valid.len();
        total_closed += verification.closed.len();
        total_not_found += verification.not_found.len();
        total_errors += verification.errors.len();

        match format {
            types::OutputFormat::Text | types::OutputFormat::Table | types::OutputFormat::Csv => {
                if !verification.valid.is_empty()
                    || !verification.closed.is_empty()
                    || !verification.not_found.is_empty()
                    || !verification.errors.is_empty()
                {
                    println!("  {}", safe_diagnostic(&rel_path).bold());

                    for issue in &verification.valid {
                        println!(
                            "    {} #{} — {} (open)",
                            "✓".green(),
                            issue.number,
                            safe_diagnostic(&issue.title)
                        );
                    }
                    for issue in &verification.closed {
                        println!(
                            "    {} #{} — {} (closed — spec may need updating)",
                            "⚠".yellow(),
                            issue.number,
                            safe_diagnostic(&issue.title)
                        );
                    }
                    for num in &verification.not_found {
                        println!("    {} #{num} — not found", "✗".red());
                    }
                    for err in &verification.errors {
                        println!("    {} {}", "✗".red(), safe_diagnostic(err));
                    }
                    println!();
                }
            }
            types::OutputFormat::Json
            | types::OutputFormat::Markdown
            | types::OutputFormat::Github => {
                json_results.push(issue_verification_json(&verification));
            }
        }
    }

    if create && let Some(specs) = opened_specs {
        let (all_errors, _, _) =
            collect_snapshot_validation(&specs.project, root, &snapshots, &config);
        if !all_errors.is_empty() {
            create_drift_issues_with_diagnostics(root, &config, &all_errors, format);
        }
    }

    match format {
        types::OutputFormat::Json => {
            let findings = inspection_findings
                .iter()
                .map(|finding| {
                    serde_json::json!({
                        "spec": safe_diagnostic(&finding.spec),
                        "kind": finding.kind.as_str(),
                        "message": finding.kind.message(),
                    })
                })
                .collect::<Vec<_>>();
            let safe_repo = repo.as_deref().map(safe_diagnostic);
            let output = serde_json::json!({
                "repo": safe_repo,
                "valid": total_valid,
                "closed": total_closed,
                "not_found": total_not_found,
                "errors": total_errors,
                "inspection_findings": inspection_findings.len(),
                "findings": findings,
                "specs": json_results,
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        types::OutputFormat::Markdown | types::OutputFormat::Github => {
            if let Some(repo) = repo.as_deref() {
                println!("## Issue Verification — {}\n", markdown_code_span(repo));
            } else {
                println!("## Issue Verification\n");
            }
            println!("| Metric | Count |");
            println!("|--------|-------|");
            println!("| Valid (open) | {total_valid} |");
            println!("| Closed | {total_closed} |");
            println!("| Not found | {total_not_found} |");
            println!("| Errors | {total_errors} |");
            println!("| Inspection findings | {} |", inspection_findings.len());

            if !inspection_findings.is_empty() {
                println!("\n### Spec Inspection Findings\n");
                println!("| Spec | Finding |");
                println!("|------|---------|");
                for finding in &inspection_findings {
                    println!(
                        "| {} | {} |",
                        markdown_code_span(&finding.spec),
                        finding.kind.message()
                    );
                }
            }
        }
        types::OutputFormat::Text | types::OutputFormat::Table | types::OutputFormat::Csv => {
            for finding in &inspection_findings {
                println!("  {}", safe_diagnostic(&finding.spec).bold());
                println!("    {} {}", "✗".red(), finding.kind.message());
                println!();
            }

            if let Some(summary) = issue_text_summary(
                references.len(),
                total_valid,
                total_closed,
                total_not_found,
                total_errors,
                inspection_findings.len(),
            ) {
                println!("{summary}");
            } else {
                println!(
                    "{}",
                    "No issue references found in spec frontmatter.".cyan()
                );
                println!(
                    "Add `implements: [42]` or `tracks: [10]` to spec frontmatter to link issues."
                );
            }
        }
    }

    if total_not_found > 0 || total_errors > 0 || !inspection_findings.is_empty() {
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        SpecInspectionFinding, SpecInspectionFindingKind, SpecSnapshot, inspect_spec,
        issue_text_summary, issue_verification_json, markdown_code_span, safe_diagnostic,
        slash_normalized_relative_path,
    };
    #[cfg(unix)]
    use super::{
        collect_snapshot_validation_with_hook, find_spec_snapshots_checked_with_hook,
        find_spec_snapshots_checked_with_limits, is_spec_shaped_file_name, open_specs_directory,
        snapshot_mapped_sources,
    };
    use crate::github::{GitHubIssue, IssueVerification};
    #[cfg(unix)]
    use crate::validator::SourceSnapshot;
    #[cfg(unix)]
    use std::fs;

    #[test]
    fn all_error_batches_report_errors_instead_of_no_reference_guidance() {
        let summary = issue_text_summary(1, 0, 0, 0, 2, 0)
            .expect("a batch with references must produce a summary");

        assert_eq!(
            summary,
            "Issue references: 0 valid, 0 closed, 0 not found, 2 errors"
        );
        assert!(issue_text_summary(0, 0, 0, 0, 0, 0).is_none());
    }

    #[test]
    fn inspection_findings_suppress_no_reference_guidance() {
        assert_eq!(
            issue_text_summary(0, 0, 0, 0, 0, 2).as_deref(),
            Some(
                "Issue references: 0 valid, 0 closed, 0 not found, 0 errors; 2 spec inspection findings"
            )
        );
    }

    #[test]
    fn malformed_snapshots_are_retained_as_findings_without_parser_details() {
        let snapshot = SpecSnapshot {
            relative_path: "specs/missing/missing.spec.md".to_string(),
            content: "not frontmatter\nSECRET_CONTENT".to_string(),
        };

        assert_eq!(
            inspect_spec(&snapshot),
            Err(SpecInspectionFinding {
                spec: "specs/missing/missing.spec.md".to_string(),
                kind: SpecInspectionFindingKind::MalformedFrontmatter,
            })
        );
    }

    #[test]
    fn crlf_snapshot_issue_references_are_retained() {
        let snapshot = SpecSnapshot {
            relative_path: "specs/crlf/crlf.spec.md".to_string(),
            content: "---\r\nmodule: crlf\r\nimplements: [41, 42]\r\ntracks:\r\n  - 43\r\n---\r\n\r\n# CRLF\r\n"
                .to_string(),
        };

        assert_eq!(
            inspect_spec(&snapshot),
            Ok(Some((
                "specs/crlf/crlf.spec.md".to_string(),
                vec![41, 42],
                vec![43],
            )))
        );
    }

    #[cfg(unix)]
    #[test]
    fn unix_non_utf8_spec_suffix_is_recognized_without_lossy_conversion() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        assert!(is_spec_shaped_file_name(&OsString::from_vec(
            b"opaque-\xff.spec.md".to_vec()
        )));
        assert!(!is_spec_shaped_file_name(&OsString::from_vec(
            b"opaque-\xff.md".to_vec()
        )));
    }

    #[cfg(unix)]
    #[test]
    fn discovery_to_read_replacement_hook_rejects_symlink_without_reading_target() {
        use std::os::unix::fs::symlink;

        let temporary = tempfile::TempDir::new().unwrap();
        let root = temporary.path().join("project");
        let spec_dir = root.join("specs/auth");
        fs::create_dir_all(&spec_dir).unwrap();
        let spec_path = spec_dir.join("auth.spec.md");
        fs::write(
            &spec_path,
            "---\nmodule: auth\nimplements: [42]\n---\n\n# Auth\n",
        )
        .unwrap();
        let outside_path = temporary.path().join("outside.spec.md");
        let outside_bytes = b"OUTSIDE_REPLACEMENT_SECRET\xff";
        fs::write(&outside_path, outside_bytes).unwrap();

        let opened = open_specs_directory(&root, "specs")
            .expect("configuration must be confined")
            .expect("specs directory must exist");
        let expected_relative = std::path::Path::new("specs/auth/auth.spec.md");
        let (snapshots, findings) = find_spec_snapshots_checked_with_hook(&opened, |relative| {
            if relative == expected_relative {
                fs::rename(&spec_path, spec_path.with_extension("original")).unwrap();
                symlink(&outside_path, &spec_path).unwrap();
            }
        });

        assert!(snapshots.is_empty());
        assert_eq!(
            findings,
            vec![SpecInspectionFinding {
                spec: "specs/auth/auth.spec.md".to_string(),
                kind: SpecInspectionFindingKind::ReadError,
            }]
        );
        assert_eq!(fs::read(&outside_path).unwrap(), outside_bytes);
    }

    #[cfg(unix)]
    #[test]
    fn discovery_to_read_replacement_hook_rejects_regular_file_replacement() {
        let temporary = tempfile::TempDir::new().unwrap();
        let root = temporary.path().join("project");
        let spec_dir = root.join("specs/auth");
        fs::create_dir_all(&spec_dir).unwrap();
        let spec_path = spec_dir.join("auth.spec.md");
        fs::write(
            &spec_path,
            "---\nmodule: auth\nimplements: [42]\n---\n\n# Auth\n",
        )
        .unwrap();

        let opened = open_specs_directory(&root, "specs")
            .expect("configuration must be confined")
            .expect("specs directory must exist");
        let expected_relative = std::path::Path::new("specs/auth/auth.spec.md");
        let (snapshots, findings) = find_spec_snapshots_checked_with_hook(&opened, |relative| {
            if relative == expected_relative {
                fs::rename(&spec_path, spec_path.with_extension("original")).unwrap();
                fs::write(
                    &spec_path,
                    "---\nmodule: replacement\nimplements: [999]\n---\n",
                )
                .unwrap();
            }
        });

        assert!(snapshots.is_empty());
        assert_eq!(
            findings,
            vec![SpecInspectionFinding {
                spec: "specs/auth/auth.spec.md".to_string(),
                kind: SpecInspectionFindingKind::ReadError,
            }]
        );
    }

    #[cfg(unix)]
    #[test]
    fn discovery_to_read_replacement_hook_rejects_hardlink_replacement() {
        let temporary = tempfile::TempDir::new().unwrap();
        let root = temporary.path().join("project");
        let spec_dir = root.join("specs/auth");
        fs::create_dir_all(&spec_dir).unwrap();
        let spec_path = spec_dir.join("auth.spec.md");
        fs::write(
            &spec_path,
            "---\nmodule: auth\nimplements: [42]\n---\n\n# Auth\n",
        )
        .unwrap();
        let replacement = temporary.path().join("replacement.spec.md");
        fs::write(
            &replacement,
            "---\nmodule: replacement\nimplements: [999]\n---\n",
        )
        .unwrap();

        let opened = open_specs_directory(&root, "specs")
            .expect("configuration must be confined")
            .expect("specs directory must exist");
        let expected_relative = std::path::Path::new("specs/auth/auth.spec.md");
        let (snapshots, findings) = find_spec_snapshots_checked_with_hook(&opened, |relative| {
            if relative == expected_relative {
                fs::rename(&spec_path, spec_path.with_extension("original")).unwrap();
                fs::hard_link(&replacement, &spec_path).unwrap();
            }
        });

        assert!(snapshots.is_empty());
        assert_eq!(
            findings,
            vec![SpecInspectionFinding {
                spec: "specs/auth/auth.spec.md".to_string(),
                kind: SpecInspectionFindingKind::ReadError,
            }]
        );
    }

    #[cfg(unix)]
    #[test]
    fn snapshot_discovery_enforces_per_file_cumulative_and_file_count_limits() {
        let temporary = tempfile::TempDir::new().unwrap();
        let root = temporary.path().join("project");
        let spec_dir = root.join("specs");
        fs::create_dir_all(&spec_dir).unwrap();
        for index in 0..4 {
            fs::write(
                spec_dir.join(format!("{index}.spec.md")),
                format!("---\nmodule: m{index}\n---\n{}", "x".repeat(48)),
            )
            .unwrap();
        }
        let opened = open_specs_directory(&root, "specs")
            .expect("configuration must be confined")
            .expect("specs directory must exist");

        let (snapshots, findings) =
            find_spec_snapshots_checked_with_limits(&opened, |_| {}, 96, 150, 2, usize::MAX);

        assert_eq!(snapshots.len(), 2);
        assert_eq!(findings.len(), 1);
        assert!(
            findings
                .iter()
                .all(|finding| finding.kind == SpecInspectionFindingKind::ReadError)
        );

        let (oversized, oversized_findings) =
            find_spec_snapshots_checked_with_limits(&opened, |_| {}, 32, 1_000, 10, usize::MAX);
        assert!(oversized.is_empty());
        assert_eq!(oversized_findings.len(), 4);
    }

    #[cfg(unix)]
    #[test]
    fn snapshot_discovery_bounds_huge_non_spec_inventories_before_accumulation() {
        let temporary = tempfile::TempDir::new().unwrap();
        let root = temporary.path().join("project");
        let spec_dir = root.join("specs");
        fs::create_dir_all(&spec_dir).unwrap();
        for index in 0..256 {
            fs::write(
                spec_dir.join(format!("inventory-{index:03}.txt")),
                b"not a spec",
            )
            .unwrap();
        }
        fs::write(
            spec_dir.join("zzzz.spec.md"),
            "---\nmodule: unreachable\n---\n",
        )
        .unwrap();
        let opened = open_specs_directory(&root, "specs")
            .expect("configuration must be confined")
            .expect("specs directory must exist");

        let (snapshots, findings) =
            find_spec_snapshots_checked_with_limits(&opened, |_| {}, 96, 1_000, 10, 32);

        assert!(snapshots.is_empty());
        assert_eq!(
            findings,
            vec![SpecInspectionFinding {
                spec: "specs/".to_string(),
                kind: SpecInspectionFindingKind::DiscoveryError,
            }]
        );
    }

    #[cfg(unix)]
    #[test]
    fn mapped_sources_use_original_root_capability_after_root_symlink_replacement() {
        use std::os::unix::fs::symlink;

        let temporary = tempfile::TempDir::new().unwrap();
        let root = temporary.path().join("project");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("specs/auth")).unwrap();
        let original_source = b"pub fn authenticate() {}\n";
        fs::write(root.join("src/auth.rs"), original_source).unwrap();
        fs::write(
            root.join("specs/auth/auth.spec.md"),
            concat!(
                "---\nmodule: auth\nversion: 1\nstatus: active\nfiles:\n",
                "  - src/auth.rs\n---\n"
            ),
        )
        .unwrap();

        let opened = open_specs_directory(&root, "specs")
            .expect("configuration must be confined")
            .expect("specs directory must exist");
        let (snapshots, findings) = find_spec_snapshots_checked_with_hook(&opened, |_| {});
        assert!(findings.is_empty());

        let renamed_root = temporary.path().join("project-original");
        let replacement_root = temporary.path().join("project-replacement");
        fs::rename(&root, &renamed_root).unwrap();
        fs::create_dir_all(replacement_root.join("src")).unwrap();
        fs::write(
            replacement_root.join("src/auth.rs"),
            b"pub fn attacker_controlled() {}\n",
        )
        .unwrap();
        symlink(&replacement_root, &root).unwrap();

        let sources = snapshot_mapped_sources(&opened.project, &snapshots);

        assert_eq!(
            sources.get("src/auth.rs"),
            Some(&SourceSnapshot::Present(original_source.to_vec()))
        );
        assert_eq!(
            fs::read(root.join("src/auth.rs")).unwrap(),
            b"pub fn attacker_controlled() {}\n"
        );
    }

    #[cfg(unix)]
    #[test]
    fn snapshot_validation_ignores_post_read_symlink_replacement() {
        use std::os::unix::fs::symlink;

        let temporary = tempfile::TempDir::new().unwrap();
        let root = temporary.path().join("project");
        let spec_dir = root.join("specs/auth");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(&spec_dir).unwrap();
        fs::write(root.join("src/auth.rs"), "pub fn authenticate() {}\n").unwrap();
        let spec_path = spec_dir.join("auth.spec.md");
        fs::write(
            &spec_path,
            "---\nmodule: auth\nversion: 1\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n",
        )
        .unwrap();
        let outside_dir = temporary.path().join("outside");
        fs::create_dir_all(&outside_dir).unwrap();
        let outside_spec_path = outside_dir.join("auth.spec.md");
        let outside_spec_bytes = b"OUTSIDE_VALIDATION_SECRET\xff";
        fs::write(&outside_spec_path, outside_spec_bytes).unwrap();
        let outside_companion_path = outside_dir.join("context.md");
        let outside_companion_secret = "OUTSIDE_COMPANION_SECRET";
        fs::write(
            &outside_companion_path,
            format!(
                "<!-- Describe the context and motivation for this module. -->\n{outside_companion_secret}\n"
            ),
        )
        .unwrap();

        let opened = open_specs_directory(&root, "specs")
            .expect("configuration must be confined")
            .expect("specs directory must exist");
        let (snapshots, findings) = find_spec_snapshots_checked_with_hook(&opened, |_| {});
        assert_eq!(snapshots.len(), 1);
        assert!(findings.is_empty());

        let config = crate::config::load_config(&root);
        let (errors, warnings, notices) = collect_snapshot_validation_with_hook(
            &opened.project,
            &root,
            &snapshots,
            &config,
            || {
                fs::rename(&spec_dir, root.join("specs/auth-original")).unwrap();
                symlink(&outside_dir, &spec_dir).unwrap();
            },
        );

        assert!(errors.is_empty(), "{errors:?}");
        assert!(
            warnings
                .iter()
                .all(|warning| !warning.contains("Unfilled companion scaffold marker")),
            "{warnings:?}"
        );
        let diagnostics = [errors.to_vec(), warnings, notices].concat().join("\n");
        assert!(!diagnostics.contains("OUTSIDE_VALIDATION_SECRET"));
        assert!(!diagnostics.contains(outside_companion_secret));
        assert!(!diagnostics.contains(&outside_dir.to_string_lossy().to_string()));
        assert_eq!(fs::read(&outside_spec_path).unwrap(), outside_spec_bytes);
        assert_eq!(
            fs::read_to_string(&outside_companion_path).unwrap(),
            format!(
                "<!-- Describe the context and motivation for this module. -->\n{outside_companion_secret}\n"
            )
        );
    }

    #[cfg(unix)]
    #[test]
    fn snapshot_validation_never_reopens_replaced_mapped_source() {
        use std::os::unix::fs::symlink;

        let temporary = tempfile::TempDir::new().unwrap();
        let root = temporary.path().join("project");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("specs/auth")).unwrap();
        let source_path = root.join("src/auth.rs");
        fs::write(&source_path, "pub fn authenticate() {}\n").unwrap();
        let spec_path = root.join("specs/auth/auth.spec.md");
        fs::write(
            &spec_path,
            concat!(
                "---\nmodule: auth\nversion: 1\nstatus: active\nfiles:\n",
                "  - src/auth.rs\n---\n\n",
                "## Purpose\nAuth.\n\n## Requirements\nAuthenticate.\n\n",
                "## Public API\n| Name | Description |\n|---|---|\n",
                "| `authenticate` | Authenticate. |\n\n",
                "## Invariants\nSafe.\n\n## Behavioral Examples\nWorks.\n\n",
                "## Error Cases\nNone.\n\n## Dependencies\nNone.\n\n",
                "## Change Log\n- Initial.\n",
            ),
        )
        .unwrap();
        let outside = temporary.path().join("outside.rs");
        let outside_bytes = b"OUTSIDE_SOURCE_SECRET\xff";
        fs::write(&outside, outside_bytes).unwrap();

        let opened = open_specs_directory(&root, "specs")
            .expect("configuration must be confined")
            .expect("specs directory must exist");
        let (snapshots, findings) = find_spec_snapshots_checked_with_hook(&opened, |_| {});
        assert!(findings.is_empty());

        let config = crate::config::load_config(&root);
        let (errors, warnings, notices) = collect_snapshot_validation_with_hook(
            &opened.project,
            &root,
            &snapshots,
            &config,
            || {
                fs::rename(&source_path, source_path.with_extension("original")).unwrap();
                symlink(&outside, &source_path).unwrap();
            },
        );
        assert!(errors.is_empty(), "{errors:?}");
        let combined = [errors.to_vec(), warnings, notices].concat().join("\n");

        assert!(!combined.contains("OUTSIDE_SOURCE_SECRET"));
        assert!(!combined.contains(&outside.to_string_lossy().to_string()));
        assert_eq!(fs::read(&outside).unwrap(), outside_bytes);
    }

    #[test]
    fn untrusted_diagnostics_are_safe_in_text_json_and_markdown() {
        let path =
            "specs/bad``tick|line\n\u{1b}]8;;https://example.invalid\u{7}\u{202e}\u{2028}.spec.md";

        assert_eq!(
            safe_diagnostic(path),
            "specs/bad``tick|line\\u{000A}\\u{001B}]8;;https://example.invalid\\u{0007}\\u{202E}\\u{2028}.spec.md"
        );
        assert_eq!(
            markdown_code_span(path),
            "```specs/bad``tick\\|line\\\\u{000A}\\\\u{001B}]8;;https://example.invalid\\\\u{0007}\\\\u{202E}\\\\u{2028}.spec.md```"
        );

        let verification = IssueVerification {
            spec_path: path.to_string(),
            valid: vec![GitHubIssue {
                number: 42,
                title: "title\n## heading|table\u{1b}]8;;evil\u{7}\u{202d}\u{2029}".to_string(),
                state: "open\u{2067}spoof\u{2069}".to_string(),
                labels: Vec::new(),
                url: String::new(),
            }],
            closed: Vec::new(),
            not_found: Vec::new(),
            errors: vec!["provider error\r\n# heading\u{1b}[31m\u{202e}".to_string()],
        };
        let json = issue_verification_json(&verification);
        assert_eq!(
            json["valid"][0]["title"],
            "title\\u{000A}## heading|table\\u{001B}]8;;evil\\u{0007}\\u{202D}\\u{2029}"
        );
        assert_eq!(json["valid"][0]["state"], "open\\u{2067}spoof\\u{2069}");
        assert_eq!(
            json["errors"][0],
            "provider error\\u{000D}\\u{000A}# heading\\u{001B}[31m\\u{202E}"
        );
        let serialized = serde_json::to_string(&json).unwrap();
        for unsafe_character in [
            '\n', '\r', '\u{1b}', '\u{7}', '\u{202d}', '\u{202e}', '\u{2029}',
        ] {
            assert!(!serialized.contains(unsafe_character));
        }
    }

    #[test]
    fn markdown_code_spans_pad_leading_and_trailing_backticks() {
        assert_eq!(markdown_code_span("`spec.md"), "`` `spec.md ``");
        assert_eq!(markdown_code_span("spec.md`"), "`` spec.md` ``");
    }

    #[test]
    fn relative_paths_use_slashes_on_every_platform() {
        let path = std::path::Path::new("specs")
            .join("adversarial")
            .join("bad``tick.spec.md");

        assert_eq!(
            slash_normalized_relative_path(&path).as_deref(),
            Some("specs/adversarial/bad``tick.spec.md")
        );
    }
}
