//! Retained manifest capabilities, bounded reads, and workspace listings.

use cap_std::fs::Dir;
use std::collections::HashMap;
use std::ffi::OsString;
#[cfg(test)]
use std::fs;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

use super::identity::{
    GradleFilesystemIdentity, gradle_filesystem_identity, gradle_metadata_is_link,
    gradle_read_only_nofollow_options,
};

const MAX_RETAINED_MANIFEST_BYTES: u64 = 8 * 1024 * 1024;
const MAX_RETAINED_MANIFEST_INPUT_BYTES: u64 = 64 * 1024 * 1024;
pub(super) const MAX_RETAINED_MANIFEST_ENTRIES: usize = 100_000;
const MAX_RETAINED_MANIFEST_DEPTH: usize = 256;

pub(super) trait ManifestAccess {
    fn read_text(&mut self, relative: &Path, label: &str) -> Result<Option<String>, String>;
    fn directory_exists(&mut self, relative: &Path, label: &str) -> Result<bool, String>;
    fn child_directories(&mut self, relative: &Path, label: &str) -> Result<Vec<String>, String>;
    fn read_enumerated_child_text(
        &mut self,
        parent: &Path,
        child: &str,
        relative: &Path,
        label: &str,
    ) -> Result<Option<String>, String> {
        self.read_text(&parent.join(child).join(relative), label)
    }
    fn enumerated_child_directory_exists(
        &mut self,
        parent: &Path,
        child: &str,
        relative: &Path,
        label: &str,
    ) -> Result<bool, String> {
        self.directory_exists(&parent.join(child).join(relative), label)
    }
    fn verify_enumerated_child(
        &mut self,
        _parent: &Path,
        _child: &str,
        _label: &str,
    ) -> Result<(), String> {
        Ok(())
    }
    fn verify_child_directories(&mut self, _relative: &Path, _label: &str) -> Result<(), String> {
        Ok(())
    }
    fn release_child_directories(&mut self, _relative: &Path, _label: &str) -> Result<(), String> {
        Ok(())
    }
    fn charge_entries(&mut self, _count: usize, _label: &str) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
pub(super) struct AmbientManifestAccess<'a> {
    pub(super) root: &'a Path,
}

#[cfg(test)]
impl ManifestAccess for AmbientManifestAccess<'_> {
    fn read_text(&mut self, relative: &Path, _label: &str) -> Result<Option<String>, String> {
        Ok(fs::read_to_string(self.root.join(relative)).ok())
    }

    fn directory_exists(&mut self, relative: &Path, _label: &str) -> Result<bool, String> {
        Ok(self.root.join(relative).is_dir())
    }

    fn child_directories(&mut self, relative: &Path, _label: &str) -> Result<Vec<String>, String> {
        let mut names = Vec::new();
        let Ok(entries) = fs::read_dir(self.root.join(relative)) else {
            return Ok(names);
        };
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                names.push(entry.file_name().to_string_lossy().into_owned());
            }
        }
        names.sort();
        Ok(names)
    }
}

pub(super) struct RetainedManifestAccess<'a> {
    root: &'a Dir,
    bytes: u64,
    entries: usize,
    texts: HashMap<PathBuf, Option<String>>,
    directories: HashMap<PathBuf, bool>,
    children: HashMap<PathBuf, RetainedDirectoryListing>,
}

struct RetainedDirectoryListing {
    names: Vec<String>,
    directory: Dir,
    child_identities: HashMap<String, GradleFilesystemIdentity>,
}

impl<'a> RetainedManifestAccess<'a> {
    pub(super) fn new(root: &'a Dir) -> Self {
        Self {
            root,
            bytes: 0,
            entries: 0,
            texts: HashMap::new(),
            directories: HashMap::new(),
            children: HashMap::new(),
        }
    }

    fn child_directories_with_hook<AfterTraversal>(
        &mut self,
        relative: &Path,
        label: &str,
        after_traversal: AfterTraversal,
    ) -> Result<Vec<String>, String>
    where
        AfterTraversal: FnOnce(),
    {
        let relative = normalize_retained_manifest_path(relative, label)?;
        if let Some(cached) = self.children.get(&relative) {
            verify_retained_manifest_directory_edge(
                self.root,
                &relative,
                &cached.directory,
                label,
            )?;
            return Ok(cached.names.clone());
        }
        let Some(directory) = open_retained_manifest_directory(self.root, &relative, label)? else {
            return Ok(Vec::new());
        };
        let mut names: Vec<OsString> = Vec::new();
        let entries = directory.read_dir(".").map_err(|error| {
            format!(
                "Cannot read retained {label} directory {}: {error}",
                display_manifest_path(&relative)
            )
        })?;
        for entry in entries {
            let entry = entry.map_err(|error| {
                format!(
                    "Cannot inspect retained {label} directory {}: {error}",
                    display_manifest_path(&relative)
                )
            })?;
            self.charge_entries(1, label)?;
            names.push(entry.file_name());
        }
        names.sort();
        let mut children = Vec::new();
        let mut child_identities = HashMap::new();
        for name in names {
            let name_text = name.to_str().ok_or_else(|| {
                format!(
                    "Retained {label} path beneath {} is not valid UTF-8",
                    display_manifest_path(&relative)
                )
            })?;
            let child = relative.join(&name);
            let metadata = directory.symlink_metadata(&name).map_err(|error| {
                format!(
                    "Cannot inspect retained {label} path {}: {error}",
                    child.display()
                )
            })?;
            if gradle_metadata_is_link(&metadata) {
                return Err(format!(
                    "Retained {label} path {} must not be a symlink or reparse point",
                    child.display()
                ));
            }
            if metadata.is_dir() {
                let identity = gradle_filesystem_identity(&metadata).map_err(|error| {
                    format!(
                        "Cannot identify retained {label} directory {} during enumeration: {error}",
                        child.display()
                    )
                })?;
                children.push(name_text.to_string());
                child_identities.insert(name_text.to_string(), identity);
            }
        }
        after_traversal();
        verify_retained_manifest_directory_edge(self.root, &relative, &directory, label)?;
        self.children.insert(
            relative,
            RetainedDirectoryListing {
                names: children.clone(),
                directory,
                child_identities,
            },
        );
        Ok(children)
    }

    fn enumerated_child_directory(
        &self,
        parent: &Path,
        child: &str,
        label: &str,
    ) -> Result<Dir, String> {
        let parent = normalize_retained_manifest_path(parent, label)?;
        let listing = self.children.get(&parent).ok_or_else(|| {
            format!(
                "Retained {label} directory {} was not enumerated",
                display_manifest_path(&parent)
            )
        })?;
        let expected_identity = listing.child_identities.get(child).ok_or_else(|| {
            format!(
                "Retained {label} child {} was not present in enumerated directory {}",
                parent.join(child).display(),
                display_manifest_path(&parent)
            )
        })?;
        let relative = parent.join(child);
        let before = listing.directory.symlink_metadata(child).map_err(|error| {
            format!(
                "Cannot re-inspect enumerated {label} child {}: {error}",
                relative.display()
            )
        })?;
        if gradle_metadata_is_link(&before)
            || !before.is_dir()
            || gradle_filesystem_identity(&before)? != *expected_identity
        {
            return Err(format!(
                "Retained {label} directory {} changed after enumeration",
                relative.display()
            ));
        }
        let directory = listing.directory.open_dir(child).map_err(|error| {
            format!(
                "Cannot open enumerated {label} child {}: {error}",
                relative.display()
            )
        })?;
        let opened = directory.dir_metadata().map_err(|error| {
            format!(
                "Cannot inspect opened enumerated {label} child {}: {error}",
                relative.display()
            )
        })?;
        let after = listing.directory.symlink_metadata(child).map_err(|error| {
            format!(
                "Cannot re-inspect opened {label} child {}: {error}",
                relative.display()
            )
        })?;
        if gradle_metadata_is_link(&after)
            || !after.is_dir()
            || gradle_filesystem_identity(&opened)? != *expected_identity
            || gradle_filesystem_identity(&after)? != *expected_identity
        {
            return Err(format!(
                "Retained {label} directory {} changed during confined open",
                relative.display()
            ));
        }
        Ok(directory)
    }

    fn record_text(
        &mut self,
        relative: PathBuf,
        text: Option<String>,
    ) -> Result<Option<String>, String> {
        if let Some(content) = &text {
            self.bytes = self.bytes.saturating_add(content.len() as u64);
            if self.bytes > MAX_RETAINED_MANIFEST_INPUT_BYTES {
                return Err(format!(
                    "Retained manifest inputs exceed the {MAX_RETAINED_MANIFEST_INPUT_BYTES} byte cumulative limit"
                ));
            }
        }
        self.texts.insert(relative, text.clone());
        Ok(text)
    }
}

impl ManifestAccess for RetainedManifestAccess<'_> {
    fn read_text(&mut self, relative: &Path, label: &str) -> Result<Option<String>, String> {
        let relative = normalize_retained_manifest_path(relative, label)?;
        if let Some(cached) = self.texts.get(&relative) {
            return Ok(cached.clone());
        }
        let remaining = MAX_RETAINED_MANIFEST_INPUT_BYTES.saturating_sub(self.bytes);
        let text = read_retained_manifest_text(
            self.root,
            &relative,
            label,
            MAX_RETAINED_MANIFEST_BYTES,
            remaining,
        )?;
        self.record_text(relative, text)
    }

    fn directory_exists(&mut self, relative: &Path, label: &str) -> Result<bool, String> {
        let relative = normalize_retained_manifest_path(relative, label)?;
        if relative.as_os_str().is_empty() {
            return Ok(true);
        }
        if let Some(cached) = self.directories.get(&relative) {
            return Ok(*cached);
        }
        let exists = open_retained_manifest_directory(self.root, &relative, label)?.is_some();
        self.directories.insert(relative, exists);
        Ok(exists)
    }

    fn child_directories(&mut self, relative: &Path, label: &str) -> Result<Vec<String>, String> {
        self.child_directories_with_hook(relative, label, || {})
    }

    fn read_enumerated_child_text(
        &mut self,
        parent: &Path,
        child: &str,
        relative: &Path,
        label: &str,
    ) -> Result<Option<String>, String> {
        let full_path =
            normalize_retained_manifest_path(&parent.join(child).join(relative), label)?;
        if let Some(cached) = self.texts.get(&full_path) {
            return Ok(cached.clone());
        }
        let child_directory = self.enumerated_child_directory(parent, child, label)?;
        let remaining = MAX_RETAINED_MANIFEST_INPUT_BYTES.saturating_sub(self.bytes);
        let text = read_retained_manifest_text(
            &child_directory,
            relative,
            label,
            MAX_RETAINED_MANIFEST_BYTES,
            remaining,
        )?;
        self.record_text(full_path, text)
    }

    fn enumerated_child_directory_exists(
        &mut self,
        parent: &Path,
        child: &str,
        relative: &Path,
        label: &str,
    ) -> Result<bool, String> {
        let full_path =
            normalize_retained_manifest_path(&parent.join(child).join(relative), label)?;
        if let Some(cached) = self.directories.get(&full_path) {
            return Ok(*cached);
        }
        let child_directory = self.enumerated_child_directory(parent, child, label)?;
        let exists = open_retained_manifest_directory(&child_directory, relative, label)?.is_some();
        self.directories.insert(full_path, exists);
        Ok(exists)
    }

    fn verify_enumerated_child(
        &mut self,
        parent: &Path,
        child: &str,
        label: &str,
    ) -> Result<(), String> {
        let parent = normalize_retained_manifest_path(parent, label)?;
        let listing = self.children.get(&parent).ok_or_else(|| {
            format!(
                "Retained {label} directory {} was not enumerated",
                display_manifest_path(&parent)
            )
        })?;
        let child_directory = self.enumerated_child_directory(&parent, child, label)?;
        verify_retained_manifest_directory_edge(
            &listing.directory,
            Path::new(child),
            &child_directory,
            label,
        )
    }

    fn verify_child_directories(&mut self, relative: &Path, label: &str) -> Result<(), String> {
        let relative = normalize_retained_manifest_path(relative, label)?;
        if let Some(listing) = self.children.get(&relative) {
            verify_retained_manifest_directory_edge(
                self.root,
                &relative,
                &listing.directory,
                label,
            )?;
        }
        Ok(())
    }

    fn release_child_directories(&mut self, relative: &Path, label: &str) -> Result<(), String> {
        let relative = normalize_retained_manifest_path(relative, label)?;
        self.children.remove(&relative);
        Ok(())
    }

    fn charge_entries(&mut self, count: usize, _label: &str) -> Result<(), String> {
        if self.entries.saturating_add(count) > MAX_RETAINED_MANIFEST_ENTRIES {
            return Err(format!(
                "Retained manifest discovery exceeds the {MAX_RETAINED_MANIFEST_ENTRIES}-entry limit"
            ));
        }
        self.entries = self.entries.saturating_add(count);
        Ok(())
    }
}

pub(super) fn normalize_retained_manifest_path(
    relative: &Path,
    label: &str,
) -> Result<PathBuf, String> {
    let mut normalized = PathBuf::new();
    for component in relative.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(name) => normalized.push(name),
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(format!(
                        "Retained {label} path {} escapes the project root",
                        relative.display()
                    ));
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(format!(
                    "Retained {label} path {} must remain project-relative",
                    relative.display()
                ));
            }
        }
    }
    if normalized
        .components()
        .filter(|component| matches!(component, Component::Normal(_)))
        .count()
        > MAX_RETAINED_MANIFEST_DEPTH
    {
        return Err(format!(
            "Retained {label} path {} exceeds the {MAX_RETAINED_MANIFEST_DEPTH}-component depth limit",
            relative.display()
        ));
    }
    Ok(normalized)
}

pub(super) fn display_manifest_path(path: &Path) -> String {
    if path.as_os_str().is_empty() {
        ".".to_string()
    } else {
        path.display().to_string()
    }
}

pub(super) fn manifest_path_text(path: &Path, label: &str) -> Result<String, String> {
    let normalized = normalize_retained_manifest_path(path, label)?;
    let mut components = Vec::new();
    for component in normalized.components() {
        let Component::Normal(name) = component else {
            continue;
        };
        components.push(name.to_str().ok_or_else(|| {
            format!(
                "Retained {label} path {} is not valid UTF-8",
                normalized.display()
            )
        })?);
    }
    Ok(components.join("/"))
}

fn open_retained_manifest_directory(
    root: &Dir,
    relative: &Path,
    label: &str,
) -> Result<Option<Dir>, String> {
    let relative = normalize_retained_manifest_path(relative, label)?;
    let mut directory = root
        .try_clone()
        .map_err(|error| format!("Cannot retain the manifest project root: {error}"))?;
    let mut inspected = PathBuf::new();
    for component in relative.components() {
        let Component::Normal(name) = component else {
            continue;
        };
        inspected.push(name);
        let before = match directory.symlink_metadata(name) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(format!(
                    "Cannot inspect retained {label} directory {}: {error}",
                    inspected.display()
                ));
            }
        };
        if gradle_metadata_is_link(&before) {
            return Err(format!(
                "Retained {label} directory {} must not traverse a symlink or reparse point",
                inspected.display()
            ));
        }
        if !before.is_dir() {
            return Ok(None);
        }
        let before_identity = gradle_filesystem_identity(&before).map_err(|error| {
            format!(
                "Cannot identify retained {label} directory {} before open: {error}",
                inspected.display()
            )
        })?;
        let next = directory.open_dir(name).map_err(|error| {
            format!(
                "Cannot open retained {label} directory {}: {error}",
                inspected.display()
            )
        })?;
        let opened = next.dir_metadata().map_err(|error| {
            format!(
                "Cannot inspect opened retained {label} directory {}: {error}",
                inspected.display()
            )
        })?;
        let after = directory.symlink_metadata(name).map_err(|error| {
            format!(
                "Cannot re-inspect retained {label} directory {}: {error}",
                inspected.display()
            )
        })?;
        if gradle_metadata_is_link(&after)
            || !after.is_dir()
            || before_identity != gradle_filesystem_identity(&opened)?
            || before_identity != gradle_filesystem_identity(&after)?
        {
            return Err(format!(
                "Retained {label} directory {} changed during confined open",
                inspected.display()
            ));
        }
        directory = next;
    }
    Ok(Some(directory))
}

fn verify_retained_manifest_directory_edge(
    root: &Dir,
    relative: &Path,
    expected: &Dir,
    label: &str,
) -> Result<(), String> {
    let Some(observed) = open_retained_manifest_directory(root, relative, label)? else {
        return Err(format!(
            "Retained {label} directory {} changed during confined read",
            display_manifest_path(relative)
        ));
    };
    let expected_metadata = expected.dir_metadata().map_err(|error| {
        format!(
            "Cannot identify retained {label} directory {}: {error}",
            display_manifest_path(relative)
        )
    })?;
    let observed_metadata = observed.dir_metadata().map_err(|error| {
        format!(
            "Cannot identify re-opened retained {label} directory {}: {error}",
            display_manifest_path(relative)
        )
    })?;
    if gradle_filesystem_identity(&expected_metadata)?
        != gradle_filesystem_identity(&observed_metadata)?
    {
        return Err(format!(
            "Retained {label} directory {} changed during confined read",
            display_manifest_path(relative)
        ));
    }
    Ok(())
}

fn read_retained_manifest_text(
    root: &Dir,
    relative: &Path,
    label: &str,
    max_file_bytes: u64,
    remaining_input_bytes: u64,
) -> Result<Option<String>, String> {
    read_retained_manifest_text_with_hook(
        root,
        relative,
        label,
        max_file_bytes,
        remaining_input_bytes,
        || {},
    )
}

fn read_retained_manifest_text_with_hook<AfterOpen>(
    root: &Dir,
    relative: &Path,
    label: &str,
    max_file_bytes: u64,
    remaining_input_bytes: u64,
    after_open: AfterOpen,
) -> Result<Option<String>, String>
where
    AfterOpen: FnOnce(),
{
    let parent = relative.parent().unwrap_or_else(|| Path::new(""));
    let name = relative.file_name().ok_or_else(|| {
        format!(
            "Retained {label} path {} has no filename",
            relative.display()
        )
    })?;
    let Some(directory) = open_retained_manifest_directory(root, parent, label)? else {
        return Ok(None);
    };
    let before = match directory.symlink_metadata(name) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "Cannot inspect retained {label} file {}: {error}",
                relative.display()
            ));
        }
    };
    if gradle_metadata_is_link(&before) {
        return Err(format!(
            "Retained {label} file {} must not be a symlink or reparse point",
            relative.display()
        ));
    }
    if !before.is_file() {
        return Err(format!(
            "Retained {label} file {} must be a regular file",
            relative.display()
        ));
    }
    if before.len() > max_file_bytes {
        return Err(format!(
            "Retained {label} file {} exceeds the {max_file_bytes} byte limit",
            relative.display()
        ));
    }
    if before.len() > remaining_input_bytes {
        return Err(format!(
            "Retained manifest inputs exceed the {MAX_RETAINED_MANIFEST_INPUT_BYTES} byte cumulative limit"
        ));
    }
    let before_identity = gradle_filesystem_identity(&before).map_err(|error| {
        format!(
            "Cannot identify retained {label} file {} before open: {error}",
            relative.display()
        )
    })?;
    let mut file = directory
        .open_with(name, &gradle_read_only_nofollow_options())
        .map_err(|error| {
            format!(
                "Cannot open retained {label} file {} as a non-blocking regular file: {error}",
                relative.display()
            )
        })?;
    let opened = file.metadata().map_err(|error| {
        format!(
            "Cannot inspect opened retained {label} file {}: {error}",
            relative.display()
        )
    })?;
    let opened_identity = gradle_filesystem_identity(&opened).map_err(|error| {
        format!(
            "Cannot identify opened retained {label} file {}: {error}",
            relative.display()
        )
    })?;
    after_open();
    verify_retained_manifest_directory_edge(root, parent, &directory, label)?;
    let after_open = directory.symlink_metadata(name).map_err(|error| {
        format!(
            "Cannot re-inspect retained {label} file {} after open: {error}",
            relative.display()
        )
    })?;
    if !opened.is_file()
        || gradle_metadata_is_link(&opened)
        || gradle_metadata_is_link(&after_open)
        || !after_open.is_file()
        || before_identity != opened_identity
        || before_identity != gradle_filesystem_identity(&after_open)?
    {
        return Err(format!(
            "Retained {label} file {} changed during confined open",
            relative.display()
        ));
    }
    let mut bytes = Vec::new();
    file.by_ref()
        .take(max_file_bytes.min(remaining_input_bytes).saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| {
            format!(
                "Cannot read retained {label} file {}: {error}",
                relative.display()
            )
        })?;
    if bytes.len() as u64 > max_file_bytes {
        return Err(format!(
            "Retained {label} file {} exceeds the {max_file_bytes} byte limit",
            relative.display()
        ));
    }
    if bytes.len() as u64 > remaining_input_bytes {
        return Err(format!(
            "Retained manifest inputs exceed the {MAX_RETAINED_MANIFEST_INPUT_BYTES} byte cumulative limit"
        ));
    }
    let after_read = directory.symlink_metadata(name).map_err(|error| {
        format!(
            "Cannot re-inspect retained {label} file {} after read: {error}",
            relative.display()
        )
    })?;
    let opened_after_read = file.metadata().map_err(|error| {
        format!(
            "Cannot re-inspect opened retained {label} file {} after read: {error}",
            relative.display()
        )
    })?;
    verify_retained_manifest_directory_edge(root, parent, &directory, label)?;
    if gradle_metadata_is_link(&after_read)
        || !after_read.is_file()
        || before_identity != gradle_filesystem_identity(&after_read)?
        || !opened_after_read.is_file()
        || gradle_metadata_is_link(&opened_after_read)
        || opened_identity != gradle_filesystem_identity(&opened_after_read)?
    {
        return Err(format!(
            "Retained {label} file {} changed during confined read",
            relative.display()
        ));
    }
    String::from_utf8(bytes).map(Some).map_err(|_| {
        format!(
            "Retained {label} file {} is not valid UTF-8",
            relative.display()
        )
    })
}

#[cfg(test)]
mod tests;
