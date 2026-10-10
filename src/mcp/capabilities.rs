use super::MAX_GENERATED_OUTPUT_BYTES;
use cap_primitives::fs::FollowSymlinks;
use cap_std::ambient_authority;
use cap_std::fs::Dir;
use cap_std::fs::Metadata;
use cap_std::fs::OpenOptions;
use sha2::Digest;
use sha2::Sha256;
use std::ffi::OsStr;
use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::Path;
use std::path::PathBuf;

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
fn windows_handle_identity(handle: *mut std::ffi::c_void) -> io::Result<(u32, u64)> {
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

fn file_content_digest_with_limit(
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

#[cfg(unix)]
pub(super) type SnapshotEntryIdentity = (u64, u64);

#[cfg(windows)]
pub(super) type SnapshotEntryIdentity = (u32, u64);

#[cfg(not(any(unix, windows)))]
pub(super) type SnapshotEntryIdentity = (u64, Option<std::time::SystemTime>);

#[cfg(unix)]
pub(super) fn snapshot_metadata_identity(metadata: &Metadata) -> io::Result<SnapshotEntryIdentity> {
    use cap_std::fs::MetadataExt;

    Ok((metadata.dev(), metadata.ino()))
}

#[cfg(windows)]
pub(super) fn snapshot_metadata_identity(metadata: &Metadata) -> io::Result<SnapshotEntryIdentity> {
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
pub(super) fn snapshot_metadata_identity(metadata: &Metadata) -> io::Result<SnapshotEntryIdentity> {
    Ok((metadata.len(), metadata.modified().ok()))
}

#[cfg(unix)]
pub(super) fn snapshot_file_identity(
    file: &cap_std::fs::File,
) -> io::Result<SnapshotEntryIdentity> {
    snapshot_metadata_identity(&file.metadata()?)
}

#[cfg(windows)]
pub(super) fn snapshot_file_identity(
    file: &cap_std::fs::File,
) -> io::Result<SnapshotEntryIdentity> {
    use std::os::windows::io::AsRawHandle;

    windows_handle_identity(file.as_raw_handle().cast())
}

#[cfg(not(any(unix, windows)))]
pub(super) fn snapshot_file_identity(
    file: &cap_std::fs::File,
) -> io::Result<SnapshotEntryIdentity> {
    snapshot_metadata_identity(&file.metadata()?)
}

#[cfg(windows)]
pub(super) fn snapshot_metadata_is_link(metadata: &Metadata) -> bool {
    use cap_std::fs::MetadataExt;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
pub(super) fn snapshot_metadata_is_link(metadata: &Metadata) -> bool {
    metadata.file_type().is_symlink()
}

fn snapshot_configuration_identities_match<Identity: PartialEq>(
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
fn open_snapshot_configuration_directory_with_hook(
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
fn open_snapshot_configuration_directory_with_hook(
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

#[cfg(test)]
mod tests;
