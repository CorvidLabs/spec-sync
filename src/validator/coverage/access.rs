use super::sources::{DiscoveredCoverageDirectory, display_coverage_byte_limit};
use cap_primitives::fs::FollowSymlinks;
use cap_std::ambient_authority;
use cap_std::fs::{Dir, Metadata, OpenOptions};
use std::ffi::{OsStr, OsString};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

pub(super) struct RetainedCoverageDirectory {
    root: Dir,
    pub(super) directory: Dir,
    edges: Vec<RetainedCoverageDirectoryEdge>,
}

struct RetainedCoverageDirectoryEdge {
    parent: Dir,
    name: OsString,
    relative: PathBuf,
    child: Dir,
}

pub(super) fn open_retained_coverage_directory(
    root: &Dir,
    relative: &Path,
) -> Result<Option<RetainedCoverageDirectory>, String> {
    let retained_root = root
        .try_clone()
        .map_err(|error| format!("Cannot retain coverage project root: {error}"))?;
    if relative.as_os_str().is_empty() {
        let directory = root
            .try_clone()
            .map_err(|error| format!("Cannot retain coverage project root: {error}"))?;
        return Ok(Some(RetainedCoverageDirectory {
            root: retained_root,
            directory,
            edges: Vec::new(),
        }));
    }
    let mut directory = root
        .try_clone()
        .map_err(|error| format!("Cannot retain coverage project root: {error}"))?;
    let mut edges = Vec::new();
    let mut inspected = PathBuf::new();
    for component in relative.components() {
        let std::path::Component::Normal(name) = component else {
            return Err(format!(
                "Coverage source directory must remain beneath the project root: {}",
                relative.display()
            ));
        };
        inspected.push(name);
        let before = match directory.symlink_metadata(name) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(format!(
                    "Cannot inspect coverage source directory {}: {error}",
                    inspected.display()
                ));
            }
        };
        // Deliberately still fatal, unlike the discovery walks (#546). This
        // opens a path the project *configured* — a `source_dirs` entry — not
        // one discovery happened to meet. Skipping an incidentally-encountered
        // link loses nothing; skipping a configured source directory silently
        // drops everything the author asked to be measured, which is the
        // failure this whole change exists to prevent.
        if coverage_metadata_is_link(&before) {
            return Err(format!(
                "Coverage source directory {} must not traverse a symlink or reparse point",
                inspected.display()
            ));
        }
        if !before.is_dir() {
            if !before.is_file() {
                return Err(format!(
                    "Coverage source directory {} must be a regular directory",
                    inspected.display()
                ));
            }
            return Ok(None);
        }
        let child = open_coverage_child_directory(&directory, name, &inspected)?;
        edges.push(RetainedCoverageDirectoryEdge {
            parent: directory.try_clone().map_err(|error| {
                format!(
                    "Cannot retain coverage source directory {}: {error}",
                    display_coverage_path(inspected.parent().unwrap_or_else(|| Path::new("")))
                )
            })?,
            name: name.to_os_string(),
            relative: inspected.clone(),
            child: child.try_clone().map_err(|error| {
                format!(
                    "Cannot retain coverage source directory {}: {error}",
                    inspected.display()
                )
            })?,
        });
        directory = child;
    }
    Ok(Some(RetainedCoverageDirectory {
        root: retained_root,
        directory,
        edges,
    }))
}

pub(super) fn verify_retained_coverage_directory(
    project: &Dir,
    retained: &RetainedCoverageDirectory,
) -> Result<(), String> {
    verify_coverage_directory_edge(project, Path::new(""), &retained.root)?;
    for edge in &retained.edges {
        verify_coverage_child_directory(&edge.parent, &edge.name, &edge.relative, &edge.child)?;
    }
    Ok(())
}

fn open_coverage_directory(root: &Dir, relative: &Path) -> Result<Option<Dir>, String> {
    open_retained_coverage_directory(root, relative)
        .map(|retained| retained.map(|retained| retained.directory))
}

pub(super) fn open_coverage_child_directory(
    parent: &Dir,
    name: &std::ffi::OsStr,
    display: &Path,
) -> Result<Dir, String> {
    let child = parent.open_dir(name).map_err(|error| {
        format!(
            "Cannot open retained coverage source directory {}: {error}",
            display.display()
        )
    })?;
    verify_coverage_child_directory(parent, name, display, &child)?;
    Ok(child)
}

pub(super) fn reopen_discovered_coverage_directory(
    parent: &Dir,
    discovered: &DiscoveredCoverageDirectory,
) -> Result<Dir, String> {
    let metadata = parent.symlink_metadata(&discovered.name).map_err(|error| {
        format!(
            "Cannot re-inspect coverage source directory {}: {error}",
            discovered.relative.display()
        )
    })?;
    if coverage_metadata_is_link(&metadata) || !metadata.is_dir() {
        return Err(format!(
            "Coverage source directory {} changed during retained traversal",
            discovered.relative.display()
        ));
    }
    let identity = coverage_metadata_identity(&metadata).map_err(|error| {
        format!(
            "Cannot identify re-inspected coverage source directory {}: {error}",
            discovered.relative.display()
        )
    })?;
    if identity != discovered.identity {
        return Err(format!(
            "Coverage source directory {} changed during retained traversal",
            discovered.relative.display()
        ));
    }
    open_coverage_child_directory(parent, &discovered.name, &discovered.relative)
}

pub(super) fn verify_coverage_directory_edge(
    project: &Dir,
    relative: &Path,
    expected: &Dir,
) -> Result<(), String> {
    if relative.as_os_str().is_empty() {
        let observed = project
            .try_clone()
            .map_err(|error| format!("Cannot re-open coverage project root: {error}"))?;
        let observed_identity = coverage_directory_identity(&observed)
            .map_err(|error| format!("Cannot identify re-opened coverage project root: {error}"))?;
        let expected_identity = coverage_directory_identity(expected)
            .map_err(|error| format!("Cannot identify retained coverage project root: {error}"))?;
        if observed_identity != expected_identity {
            return Err("Coverage project root changed during retained traversal".to_string());
        }
        return Ok(());
    }
    let parent = relative.parent().unwrap_or_else(|| Path::new(""));
    let name = relative
        .file_name()
        .ok_or_else(|| "Coverage source directory has no terminal component".to_string())?;
    let Some(parent_directory) = open_coverage_directory(project, parent)? else {
        return Err(format!(
            "Coverage source directory {} changed during retained traversal",
            relative.display()
        ));
    };
    verify_coverage_child_directory(&parent_directory, name, relative, expected)
}

pub(super) fn verify_coverage_child_directory(
    parent: &Dir,
    name: &std::ffi::OsStr,
    display: &Path,
    expected: &Dir,
) -> Result<(), String> {
    let metadata = parent.symlink_metadata(name).map_err(|error| {
        format!(
            "Cannot re-inspect coverage source directory {}: {error}",
            display.display()
        )
    })?;
    if coverage_metadata_is_link(&metadata) || !metadata.is_dir() {
        return Err(format!(
            "Coverage source directory {} changed during retained traversal",
            display.display()
        ));
    }
    let observed = parent.open_dir(name).map_err(|error| {
        format!(
            "Cannot re-open coverage source directory {}: {error}",
            display.display()
        )
    })?;
    let observed_identity = coverage_directory_identity(&observed).map_err(|error| {
        format!(
            "Cannot identify re-opened coverage source directory {}: {error}",
            display.display()
        )
    })?;
    let expected_identity = coverage_directory_identity(expected).map_err(|error| {
        format!(
            "Cannot identify retained coverage source directory {}: {error}",
            display.display()
        )
    })?;
    if observed_identity != expected_identity {
        return Err(format!(
            "Coverage source directory {} changed during retained traversal",
            display.display()
        ));
    }
    Ok(())
}

pub(super) fn read_coverage_file(
    directory: &Dir,
    name: &OsStr,
    display: &Path,
    max_file_bytes: u64,
    remaining_input_bytes: u64,
    max_input_bytes: u64,
) -> Result<Vec<u8>, String> {
    read_coverage_file_with_hook(
        directory,
        name,
        display,
        max_file_bytes,
        remaining_input_bytes,
        max_input_bytes,
        || {},
    )
}

fn read_coverage_file_with_hook<BeforeOpen>(
    directory: &Dir,
    name: &OsStr,
    display: &Path,
    max_file_bytes: u64,
    remaining_input_bytes: u64,
    max_input_bytes: u64,
    before_open: BeforeOpen,
) -> Result<Vec<u8>, String>
where
    BeforeOpen: FnOnce(),
{
    read_coverage_file_with_expected_identity_and_hook(
        directory,
        name,
        display,
        max_file_bytes,
        remaining_input_bytes,
        max_input_bytes,
        None,
        before_open,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn read_coverage_file_with_expected_identity_and_hook<BeforeOpen>(
    directory: &Dir,
    name: &OsStr,
    display: &Path,
    max_file_bytes: u64,
    remaining_input_bytes: u64,
    max_input_bytes: u64,
    expected_identity: Option<CoverageEntryIdentity>,
    before_open: BeforeOpen,
) -> Result<Vec<u8>, String>
where
    BeforeOpen: FnOnce(),
{
    let before = directory.symlink_metadata(name).map_err(|error| {
        format!(
            "Cannot inspect retained coverage source file {} before open: {error}",
            display.display()
        )
    })?;
    if coverage_metadata_is_link(&before) || !before.is_file() {
        return Err(format!(
            "Coverage source file {} must be a regular non-link file",
            display.display()
        ));
    }
    if before.len() > max_file_bytes {
        return Err(format!(
            "Coverage source file {} exceeds the {} per-file limit",
            display.display(),
            display_coverage_byte_limit(max_file_bytes)
        ));
    }
    if before.len() > remaining_input_bytes {
        return Err(format!(
            "Coverage source inputs exceed the {} cumulative limit",
            display_coverage_byte_limit(max_input_bytes)
        ));
    }
    let before_identity = coverage_metadata_identity(&before).map_err(|error| {
        format!(
            "Cannot identify retained coverage source file {} before open: {error}",
            display.display()
        )
    })?;
    if expected_identity.is_some_and(|expected| expected != before_identity) {
        return Err(format!(
            "Coverage source file {} changed after retained inventory",
            display.display()
        ));
    }
    before_open();
    let mut options = OpenOptions::new();
    options
        .read(true)
        ._cap_fs_ext_nonblock(true)
        ._cap_fs_ext_follow(FollowSymlinks::No);
    let mut file = directory.open_with(name, &options).map_err(|error| {
        format!(
            "Cannot open retained coverage source file {}: {error}",
            display.display()
        )
    })?;
    let identity = coverage_file_identity(&file).map_err(|error| {
        format!(
            "Cannot identify retained coverage source file {}: {error}",
            display.display()
        )
    })?;
    if identity != before_identity {
        return Err(format!(
            "Coverage source file {} changed during retained open",
            display.display()
        ));
    }
    verify_coverage_file_edge(directory, name, display, identity)?;
    let mut bytes = Vec::new();
    let read_limit = max_file_bytes.min(remaining_input_bytes);
    Read::by_ref(&mut file)
        .take(read_limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| {
            format!(
                "Cannot read retained coverage source file {}: {error}",
                display.display()
            )
        })?;
    if bytes.len() as u64 > max_file_bytes {
        return Err(format!(
            "Coverage source file {} exceeds the {} per-file limit",
            display.display(),
            display_coverage_byte_limit(max_file_bytes)
        ));
    }
    if bytes.len() as u64 > remaining_input_bytes {
        return Err(format!(
            "Coverage source inputs exceed the {} cumulative limit",
            display_coverage_byte_limit(max_input_bytes)
        ));
    }
    if coverage_file_identity(&file).ok() != Some(identity) {
        return Err(format!(
            "Coverage source file {} changed while it was being read",
            display.display()
        ));
    }
    verify_coverage_file_edge(directory, name, display, identity)?;
    Ok(bytes)
}

fn verify_coverage_file_edge(
    directory: &Dir,
    name: &std::ffi::OsStr,
    display: &Path,
    expected: CoverageEntryIdentity,
) -> Result<(), String> {
    let metadata = directory.symlink_metadata(name).map_err(|error| {
        format!(
            "Cannot re-inspect coverage source file {}: {error}",
            display.display()
        )
    })?;
    if coverage_metadata_is_link(&metadata) || !metadata.is_file() {
        return Err(format!(
            "Coverage source file {} changed during retained traversal",
            display.display()
        ));
    }
    let mut options = OpenOptions::new();
    options
        .read(true)
        ._cap_fs_ext_nonblock(true)
        ._cap_fs_ext_follow(FollowSymlinks::No);
    let observed = directory.open_with(name, &options).map_err(|error| {
        format!(
            "Cannot re-open coverage source file {}: {error}",
            display.display()
        )
    })?;
    if coverage_file_identity(&observed).ok() != Some(expected) {
        return Err(format!(
            "Coverage source file {} changed during retained traversal",
            display.display()
        ));
    }
    Ok(())
}

pub(super) fn display_coverage_path(path: &Path) -> String {
    if path.as_os_str().is_empty() {
        ".".to_string()
    } else {
        path.display().to_string()
    }
}

pub(super) fn coverage_relative_path_text(path: &Path) -> Result<String, String> {
    let mut components = Vec::new();
    for component in path.components() {
        let std::path::Component::Normal(name) = component else {
            return Err(format!(
                "Coverage source path must remain relative to the project root: {}",
                path.display()
            ));
        };
        let name = name
            .to_str()
            .ok_or_else(|| format!("Coverage source path {} is not valid UTF-8", path.display()))?;
        components.push(name);
    }
    Ok(components.join("/"))
}

pub(super) fn open_coverage_project_root(root: &Path) -> Result<Dir, String> {
    Dir::open_ambient_dir(root, ambient_authority()).map_err(|error| {
        format!(
            "Cannot open coverage project root {} as a retained directory: {error}",
            root.display()
        )
    })
}

pub(super) fn verify_coverage_project_root(root: &Path, retained: &Dir) -> Result<(), String> {
    let expected = coverage_directory_identity(retained)
        .map_err(|error| format!("Cannot identify retained coverage project root: {error}"))?;
    let observed = Dir::open_ambient_dir(root, ambient_authority()).map_err(|error| {
        format!(
            "Coverage project root {} changed during retained traversal: {error}",
            root.display()
        )
    })?;
    let observed = coverage_directory_identity(&observed)
        .map_err(|error| format!("Cannot identify re-opened coverage project root: {error}"))?;
    if expected != observed {
        return Err(format!(
            "Coverage project root {} changed during retained traversal",
            root.display()
        ));
    }
    Ok(())
}

#[cfg(unix)]
pub(super) type CoverageEntryIdentity = (u64, u64);

#[cfg(windows)]
pub(super) type CoverageEntryIdentity = (u32, u64);

#[cfg(not(any(unix, windows)))]
pub(super) type CoverageEntryIdentity = (u64, Option<std::time::SystemTime>);

#[cfg(unix)]
pub(super) fn coverage_directory_identity(directory: &Dir) -> io::Result<CoverageEntryIdentity> {
    use cap_std::fs::MetadataExt;

    let metadata = directory.dir_metadata()?;
    Ok((metadata.dev(), metadata.ino()))
}

#[cfg(windows)]
pub(super) fn coverage_directory_identity(directory: &Dir) -> io::Result<CoverageEntryIdentity> {
    use std::os::windows::io::AsRawHandle;

    let file = directory.try_clone()?.into_std_file();
    coverage_windows_handle_identity(file.as_raw_handle().cast())
}

#[cfg(not(any(unix, windows)))]
pub(super) fn coverage_directory_identity(directory: &Dir) -> io::Result<CoverageEntryIdentity> {
    let metadata = directory.dir_metadata()?;
    Ok((metadata.len(), metadata.modified().ok()))
}

#[cfg(unix)]
fn coverage_file_identity(file: &cap_std::fs::File) -> io::Result<CoverageEntryIdentity> {
    use cap_std::fs::MetadataExt;

    let metadata = file.metadata()?;
    Ok((metadata.dev(), metadata.ino()))
}

#[cfg(unix)]
pub(super) fn coverage_metadata_identity(
    metadata: &cap_std::fs::Metadata,
) -> io::Result<CoverageEntryIdentity> {
    use cap_std::fs::MetadataExt;

    Ok((metadata.dev(), metadata.ino()))
}

#[cfg(windows)]
fn coverage_file_identity(file: &cap_std::fs::File) -> io::Result<CoverageEntryIdentity> {
    use std::os::windows::io::AsRawHandle;

    coverage_windows_handle_identity(file.as_raw_handle().cast())
}

#[cfg(windows)]
pub(super) fn coverage_metadata_identity(
    metadata: &cap_std::fs::Metadata,
) -> io::Result<CoverageEntryIdentity> {
    use cap_primitives::fs::_WindowsByHandle;

    let volume = metadata
        .volume_serial_number()
        .ok_or_else(|| io::Error::other("Windows volume serial number is unavailable"))?;
    let index = metadata
        .file_index()
        .ok_or_else(|| io::Error::other("Windows file index is unavailable"))?;
    Ok((volume, index))
}

#[cfg(not(any(unix, windows)))]
fn coverage_file_identity(file: &cap_std::fs::File) -> io::Result<CoverageEntryIdentity> {
    let metadata = file.metadata()?;
    Ok((metadata.len(), metadata.modified().ok()))
}

#[cfg(not(any(unix, windows)))]
pub(super) fn coverage_metadata_identity(
    metadata: &cap_std::fs::Metadata,
) -> io::Result<CoverageEntryIdentity> {
    Ok((metadata.len(), metadata.modified().ok()))
}

#[cfg(windows)]
fn coverage_windows_handle_identity(
    handle: *mut std::ffi::c_void,
) -> io::Result<CoverageEntryIdentity> {
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
    // SAFETY: `handle` is owned by the live retained file/directory for the duration of the call,
    // and `information` has the exact writable Win32 structure layout.
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

#[cfg(windows)]
pub(super) fn coverage_metadata_is_link(metadata: &Metadata) -> bool {
    use cap_std::fs::MetadataExt;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
pub(super) fn coverage_metadata_is_link(metadata: &Metadata) -> bool {
    metadata.file_type().is_symlink()
}

#[cfg(test)]
#[path = "tests/access.rs"]
mod tests;
