//! Bounded, identity-continuous Gradle manifest reads and source probes.

use cap_std::fs::Dir;
use std::io::{self, Read};
use std::path::{Component, Path};

use super::super::MAX_GRADLE_MANIFEST_BYTES;
use super::super::identity::{
    gradle_filesystem_identity, gradle_metadata_is_link, gradle_read_only_nofollow_options,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GradleManifestReadCheckpoint {
    PreOpen,
    Opened,
    AfterOpen,
    AfterRead,
}

pub(super) fn gradle_confined_manifest_text(
    root: &Dir,
    name: &str,
    label: &str,
) -> Result<Option<String>, String> {
    gradle_confined_manifest_text_with_checkpoint(root, name, label, |_| {})
}

fn gradle_confined_manifest_text_with_checkpoint<Checkpoint>(
    root: &Dir,
    name: &str,
    label: &str,
    mut checkpoint: Checkpoint,
) -> Result<Option<String>, String>
where
    Checkpoint: FnMut(GradleManifestReadCheckpoint),
{
    let before = match root.symlink_metadata(name) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "Cannot inspect confined Gradle {label} manifest {name}: {error}"
            ));
        }
    };
    if gradle_metadata_is_link(&before) {
        return Err(format!(
            "Gradle {label} manifest {name} must not be a symlink or reparse point"
        ));
    }
    if !before.is_file() {
        return Err(format!(
            "Gradle {label} manifest {name} must be a regular file"
        ));
    }
    let before_identity = gradle_filesystem_identity(&before).map_err(|error| {
        format!("Cannot identify confined Gradle {label} manifest {name} before open: {error}")
    })?;
    checkpoint(GradleManifestReadCheckpoint::PreOpen);

    let options = gradle_read_only_nofollow_options();
    let mut file = root.open_with(name, &options).map_err(|error| {
        format!(
            "Cannot open confined Gradle {label} manifest {name} as a non-blocking regular file: {error}"
        )
    })?;
    let opened = file.metadata().map_err(|error| {
        format!("Cannot inspect opened Gradle {label} manifest {name}: {error}")
    })?;
    let opened_identity = gradle_filesystem_identity(&opened).map_err(|error| {
        format!("Cannot identify opened Gradle {label} manifest {name}: {error}")
    })?;
    checkpoint(GradleManifestReadCheckpoint::Opened);
    let after_open = root
        .symlink_metadata(name)
        .map_err(|error| format!("Cannot re-inspect Gradle {label} manifest {name}: {error}"))?;
    let after_open_identity = gradle_filesystem_identity(&after_open).map_err(|error| {
        format!("Cannot identify Gradle {label} manifest {name} after open: {error}")
    })?;
    if !opened.is_file()
        || gradle_metadata_is_link(&opened)
        || gradle_metadata_is_link(&after_open)
        || !after_open.is_file()
        || before_identity != opened_identity
        || before_identity != after_open_identity
    {
        return Err(format!(
            "Gradle {label} manifest {name} changed during confined open"
        ));
    }
    checkpoint(GradleManifestReadCheckpoint::AfterOpen);

    let mut bytes = Vec::new();
    file.by_ref()
        .take(MAX_GRADLE_MANIFEST_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("Cannot read Gradle {label} manifest {name}: {error}"))?;
    if bytes.len() as u64 > MAX_GRADLE_MANIFEST_BYTES {
        return Err(format!(
            "Gradle {label} manifest {name} exceeds the {} byte limit",
            MAX_GRADLE_MANIFEST_BYTES
        ));
    }
    checkpoint(GradleManifestReadCheckpoint::AfterRead);
    let after_read = root.symlink_metadata(name).map_err(|error| {
        format!("Cannot re-inspect Gradle {label} manifest {name} after read: {error}")
    })?;
    let after_read_identity = gradle_filesystem_identity(&after_read).map_err(|error| {
        format!("Cannot identify Gradle {label} manifest {name} after read: {error}")
    })?;
    let opened_after_read = file.metadata().map_err(|error| {
        format!("Cannot re-inspect opened Gradle {label} manifest {name} after read: {error}")
    })?;
    let opened_after_read_identity =
        gradle_filesystem_identity(&opened_after_read).map_err(|error| {
            format!("Cannot re-identify opened Gradle {label} manifest {name} after read: {error}")
        })?;
    if gradle_metadata_is_link(&after_read)
        || !after_read.is_file()
        || before_identity != after_read_identity
        || !opened_after_read.is_file()
        || gradle_metadata_is_link(&opened_after_read)
        || opened_identity != opened_after_read_identity
    {
        return Err(format!(
            "Gradle {label} manifest {name} changed during confined read"
        ));
    }

    String::from_utf8(bytes)
        .map(Some)
        .map_err(|_| format!("Gradle {label} manifest {name} is not valid UTF-8"))
}

pub(super) fn gradle_confined_directory_exists(root: &Dir, relative: &str) -> Result<bool, String> {
    let mut directory = root
        .try_clone()
        .map_err(|error| format!("Cannot retain the Gradle project root: {error}"))?;
    let mut inspected = Vec::new();

    for component in Path::new(relative).components() {
        let name = match component {
            Component::CurDir => continue,
            Component::Normal(name) => name,
            _ => {
                return Err(format!(
                    "Gradle source directory {relative} must remain beneath the project root"
                ));
            }
        };
        inspected.push(name.to_string_lossy().into_owned());
        let display = inspected.join("/");
        let before = match directory.symlink_metadata(name) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
            Err(error) => {
                return Err(format!(
                    "Cannot inspect confined Gradle source directory {display}: {error}"
                ));
            }
        };
        if gradle_metadata_is_link(&before) {
            return Err(format!(
                "Gradle source directory {display} must not traverse a symlink or reparse point"
            ));
        }
        if !before.is_dir() {
            return Ok(false);
        }
        let next = directory.open_dir(name).map_err(|error| {
            format!("Cannot open confined Gradle source directory {display}: {error}")
        })?;
        let after = directory.symlink_metadata(name).map_err(|error| {
            format!("Cannot re-inspect confined Gradle source directory {display}: {error}")
        })?;
        if gradle_metadata_is_link(&after) || !after.is_dir() {
            return Err(format!(
                "Gradle source directory {display} changed during confined inspection"
            ));
        }
        directory = next;
    }

    Ok(true)
}

#[cfg(test)]
mod tests;
