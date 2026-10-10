//! Shared no-follow options and native filesystem identity checks.

use cap_primitives::fs::FollowSymlinks;
use cap_std::ambient_authority;
use cap_std::fs::{Dir, Metadata, OpenOptions};
use std::path::Path;

pub(super) fn gradle_read_only_nofollow_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options
        .read(true)
        ._cap_fs_ext_nonblock(true)
        ._cap_fs_ext_follow(FollowSymlinks::No);
    options
}

#[cfg(windows)]
pub(super) fn gradle_metadata_is_link(metadata: &Metadata) -> bool {
    use cap_std::fs::MetadataExt;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
pub(super) fn gradle_metadata_is_link(metadata: &Metadata) -> bool {
    metadata.file_type().is_symlink()
}

#[cfg(unix)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct GradleFilesystemIdentity {
    device: u64,
    inode: u64,
}

#[cfg(windows)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct GradleFilesystemIdentity {
    volume_serial_number: u32,
    file_index: u64,
}

#[cfg(unix)]
pub(super) fn gradle_filesystem_identity(
    metadata: &Metadata,
) -> Result<GradleFilesystemIdentity, String> {
    use cap_std::fs::MetadataExt;

    Ok(GradleFilesystemIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
    })
}

#[cfg(windows)]
pub(super) fn gradle_filesystem_identity(
    metadata: &Metadata,
) -> Result<GradleFilesystemIdentity, String> {
    use cap_primitives::fs::_WindowsByHandle;

    let volume_serial_number = metadata
        .volume_serial_number()
        .ok_or_else(|| "Windows volume serial number is unavailable".to_string())?;
    let file_index = metadata
        .file_index()
        .ok_or_else(|| "Windows file index is unavailable".to_string())?;
    Ok(GradleFilesystemIdentity {
        volume_serial_number,
        file_index,
    })
}

#[cfg(not(any(unix, windows)))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct GradleFilesystemIdentity;

#[cfg(not(any(unix, windows)))]
pub(super) fn gradle_filesystem_identity(
    _metadata: &Metadata,
) -> Result<GradleFilesystemIdentity, String> {
    Err("filesystem identity is unavailable on this platform".to_string())
}

pub(super) fn verify_retained_project_root(
    root: &Path,
    retained: &Dir,
    phase: &str,
) -> Result<(), String> {
    let retained_metadata = retained
        .dir_metadata()
        .map_err(|error| format!("Cannot inspect retained project root {phase}: {error}"))?;
    let retained_identity = gradle_filesystem_identity(&retained_metadata)
        .map_err(|error| format!("Cannot identify retained project root {phase}: {error}"))?;
    let ambient = Dir::open_ambient_dir(root, ambient_authority()).map_err(|error| {
        format!(
            "Cannot reopen ambient project root {} {phase}: {error}",
            root.display()
        )
    })?;
    let ambient_metadata = ambient
        .dir_metadata()
        .map_err(|error| format!("Cannot inspect ambient project root {phase}: {error}"))?;
    let ambient_identity = gradle_filesystem_identity(&ambient_metadata)
        .map_err(|error| format!("Cannot identify ambient project root {phase}: {error}"))?;
    if retained_identity != ambient_identity {
        return Err(format!(
            "Ambient project root {} does not match the retained project root {phase}; project root changed during retained traversal",
            root.display()
        ));
    }
    Ok(())
}
