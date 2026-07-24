//! Manifest-aware module detection.
//!
//! Parses language-specific manifest files (Package.swift, Cargo.toml,
//! build.gradle.kts, package.json, etc.) to discover targets, source paths,
//! and module names instead of relying on directory scanning alone.

use cap_primitives::fs::FollowSymlinks;
use cap_std::ambient_authority;
use cap_std::fs::{Dir, Metadata, OpenOptions};
use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
#[cfg(test)]
use std::fs;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

pub(crate) const MAX_GRADLE_MANIFEST_BYTES: u64 = 4 * 1024 * 1024;
const MAX_RETAINED_MANIFEST_BYTES: u64 = 8 * 1024 * 1024;
const MAX_RETAINED_MANIFEST_INPUT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_RETAINED_MANIFEST_ENTRIES: usize = 100_000;
const MAX_RETAINED_MANIFEST_DEPTH: usize = 256;

fn gradle_read_only_nofollow_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options
        .read(true)
        ._cap_fs_ext_nonblock(true)
        ._cap_fs_ext_follow(FollowSymlinks::No);
    options
}

/// A module discovered from a manifest file.
#[derive(Debug, Clone)]
pub struct ManifestModule {
    /// Module/target name.
    pub name: String,
    /// Source paths relative to project root.
    #[allow(dead_code)]
    pub source_paths: Vec<String>,
    /// Dependencies (other module names).
    pub dependencies: Vec<String>,
}

/// Result of parsing all manifest files in a project.
#[derive(Debug, Default)]
pub struct ManifestDiscovery {
    /// Modules discovered from manifest files, keyed by name.
    pub modules: HashMap<String, ManifestModule>,
    /// Source directories discovered from manifests.
    pub source_dirs: Vec<String>,
}

/// A Gradle module name and its effective project-relative directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GradleSettingsModule {
    pub(crate) name: String,
    pub(crate) path: String,
}

/// Discover modules from all supported manifest files in the project root.
#[allow(dead_code)]
pub fn discover_from_manifests(root: &Path) -> ManifestDiscovery {
    discover_from_manifests_checked(root).unwrap_or_default()
}

/// Discover modules while surfacing malformed manifest inputs.
pub fn discover_from_manifests_checked(root: &Path) -> Result<ManifestDiscovery, String> {
    let project_root = Dir::open_ambient_dir(root, ambient_authority()).map_err(|error| {
        format!(
            "Cannot open manifest project root {} as a confined directory: {error}",
            root.display()
        )
    })?;
    discover_from_manifests_checked_with_root(root, &project_root)
}

/// Discover modules using a caller-retained project-root capability.
///
/// Every recognized manifest, workspace directory, and source-path probe is acquired through
/// `project_root`. The ambient path is consulted only after discovery to reject a replaced
/// project-root name.
pub(crate) fn discover_from_manifests_checked_with_root(
    root: &Path,
    project_root: &Dir,
) -> Result<ManifestDiscovery, String> {
    let mut access = RetainedManifestAccess::new(project_root);
    let mut discovery = ManifestDiscovery::default();

    if let Some(d) = parse_cargo_toml_with_access(
        &mut access,
        Path::new(""),
        &mut HashSet::new(),
        &mut HashSet::new(),
    )? {
        merge_discovery(&mut discovery, d);
    }
    if let Some(d) = parse_package_swift_with_access(&mut access)? {
        merge_discovery(&mut discovery, d);
    }
    if let Some(d) = parse_gradle_checked_with_root(project_root)? {
        merge_discovery(&mut discovery, d);
    }
    if let Some(d) = parse_package_json_with_access(&mut access)? {
        merge_discovery(&mut discovery, d);
    }
    if let Some(d) = parse_pubspec_yaml_with_access(&mut access)? {
        merge_discovery(&mut discovery, d);
    }
    if let Some(d) = parse_go_mod_with_access(&mut access)? {
        merge_discovery(&mut discovery, d);
    }
    if let Some(d) = parse_pyproject_toml_with_access(&mut access)? {
        merge_discovery(&mut discovery, d);
    }

    verify_retained_project_root(root, project_root, "after manifest discovery")?;
    Ok(discovery)
}

fn merge_discovery(target: &mut ManifestDiscovery, source: ManifestDiscovery) {
    for (name, module) in source.modules {
        target.modules.entry(name).or_insert(module);
    }
    for dir in source.source_dirs {
        if !target.source_dirs.contains(&dir) {
            target.source_dirs.push(dir);
        }
    }
}

trait ManifestAccess {
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
struct AmbientManifestAccess<'a> {
    root: &'a Path,
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

struct RetainedManifestAccess<'a> {
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
    fn new(root: &'a Dir) -> Self {
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

fn normalize_retained_manifest_path(relative: &Path, label: &str) -> Result<PathBuf, String> {
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

fn display_manifest_path(path: &Path) -> String {
    if path.as_os_str().is_empty() {
        ".".to_string()
    } else {
        path.display().to_string()
    }
}

fn manifest_path_text(path: &Path, label: &str) -> Result<String, String> {
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

// ─── Cargo.toml (Rust) ──────────────────────────────────────────────────

#[cfg(test)]
fn parse_cargo_toml(root: &Path) -> Option<ManifestDiscovery> {
    let mut access = AmbientManifestAccess { root };
    parse_cargo_toml_with_access(
        &mut access,
        Path::new(""),
        &mut HashSet::new(),
        &mut HashSet::new(),
    )
    .ok()
    .flatten()
}

fn parse_cargo_toml_with_access(
    access: &mut dyn ManifestAccess,
    relative_root: &Path,
    active: &mut HashSet<PathBuf>,
    completed: &mut HashSet<PathBuf>,
) -> Result<Option<ManifestDiscovery>, String> {
    let relative_root = normalize_retained_manifest_path(relative_root, "Cargo workspace")?;
    if completed.contains(&relative_root) {
        return Ok(None);
    }
    if !active.insert(relative_root.clone()) {
        return Err(format!(
            "Retained Cargo workspace cycle revisits {}",
            display_manifest_path(&relative_root)
        ));
    }
    let result = parse_cargo_toml_with_access_inner(access, &relative_root, active, completed);
    active.remove(&relative_root);
    if result.is_ok() {
        completed.insert(relative_root);
    }
    result
}

fn parse_cargo_toml_with_access_inner(
    access: &mut dyn ManifestAccess,
    relative_root: &Path,
    active: &mut HashSet<PathBuf>,
    completed: &mut HashSet<PathBuf>,
) -> Result<Option<ManifestDiscovery>, String> {
    let manifest_path = relative_root.join("Cargo.toml");
    let Some(content) = access.read_text(&manifest_path, "Cargo manifest")? else {
        return Ok(None);
    };
    let document = toml::from_str::<toml::Table>(&content).map_err(|error| {
        format!(
            "Cannot parse retained Cargo manifest {} as TOML: {error}",
            display_manifest_path(&manifest_path)
        )
    })?;
    let mut discovery = ManifestDiscovery::default();

    // Extract package name
    if let Some(name) = extract_toml_value(&content, "name", Some("[package]")) {
        let src_path = manifest_path_text(&relative_root.join("src"), "Cargo source")?;
        discovery.modules.insert(
            name.clone(),
            ManifestModule {
                name,
                source_paths: vec![src_path.clone()],
                dependencies: Vec::new(),
            },
        );
        if !discovery.source_dirs.contains(&src_path) {
            discovery.source_dirs.push(src_path);
        }
    }

    // Extract [[bin]] targets
    for section in split_toml_array_sections(&content, "[[bin]]") {
        if let Some(name) = extract_toml_value(&section, "name", None) {
            let path = extract_toml_value(&section, "path", None)
                .unwrap_or_else(|| format!("src/bin/{name}.rs"));
            let local_dir = Path::new(&path)
                .parent()
                .unwrap_or_else(|| Path::new("src"));
            let dir = manifest_path_text(&relative_root.join(local_dir), "Cargo binary source")?;
            discovery.modules.insert(
                name.clone(),
                ManifestModule {
                    name,
                    source_paths: vec![dir.clone()],
                    dependencies: Vec::new(),
                },
            );
            if !discovery.source_dirs.contains(&dir) {
                discovery.source_dirs.push(dir);
            }
        }
    }

    // Check for workspace members using the same TOML semantics as security preflight.
    if let Some(members) = cargo_workspace_members(&document, &manifest_path, access)? {
        let mut expanded_members = HashSet::new();
        for member in members {
            // Workspace members are subdirectories with their own Cargo.toml
            let member_root = normalize_retained_manifest_path(
                &relative_root.join(&member),
                "Cargo workspace member",
            )?;
            if !expanded_members.insert(member_root.clone()) {
                continue;
            }
            if let Some(sub) =
                parse_cargo_toml_with_access(access, &member_root, active, completed)?
            {
                for (_, module) in sub.modules {
                    discovery
                        .modules
                        .insert(module.name.clone(), module.clone());
                }
                let member = manifest_path_text(&member_root, "Cargo workspace member")?;
                if !discovery.source_dirs.contains(&member) {
                    discovery.source_dirs.push(member);
                }
            }
        }
    }

    // Extract [dependencies] as dependency names
    if let Some(deps_section) = extract_section(&content, "[dependencies]") {
        let dep_names: Vec<String> = deps_section
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') || line.starts_with('[') {
                    return None;
                }
                line.split('=').next().map(|k| k.trim().to_string())
            })
            .filter(|k| !k.is_empty())
            .collect();

        // Assign deps to the main package
        if let Some(pkg_name) = extract_toml_value(&content, "name", Some("[package]"))
            && let Some(module) = discovery.modules.get_mut(&pkg_name)
        {
            module.dependencies = dep_names;
        }
    }

    if discovery.modules.is_empty() {
        Ok(None)
    } else {
        Ok(Some(discovery))
    }
}

// ─── Package.swift (Swift) ───────────────────────────────────────────────

#[cfg(test)]
fn parse_package_swift(root: &Path) -> Option<ManifestDiscovery> {
    let mut access = AmbientManifestAccess { root };
    parse_package_swift_with_access(&mut access).ok().flatten()
}

fn parse_package_swift_with_access(
    access: &mut dyn ManifestAccess,
) -> Result<Option<ManifestDiscovery>, String> {
    let Some(content) = access.read_text(Path::new("Package.swift"), "Swift package manifest")?
    else {
        return Ok(None);
    };
    let mut discovery = ManifestDiscovery::default();

    // Parse .target and .executableTarget declarations
    // Pattern: .target(name: "TargetName", ..., path: "Sources/TargetName", ...)
    // or .target(name: "TargetName", dependencies: [...])
    let target_patterns = [
        ".target(",
        ".executableTarget(",
        ".testTarget(",
        ".systemLibrary(",
    ];

    for pattern in &target_patterns {
        let is_test = *pattern == ".testTarget(";
        let mut search_from = 0;
        while let Some(start) = content[search_from..].find(pattern) {
            let abs_start = search_from + start;
            // Find the matching closing paren (handle nested parens)
            if let Some(block) = extract_balanced_parens(&content[abs_start + pattern.len()..]) {
                let name = extract_swift_string_param(&block, "name");
                let explicit_path = extract_swift_string_param(&block, "path");

                if let Some(name) = name
                    && !is_test
                {
                    let source_path = explicit_path.unwrap_or_else(|| format!("Sources/{name}"));

                    discovery.modules.insert(
                        name.clone(),
                        ManifestModule {
                            name: name.clone(),
                            source_paths: vec![source_path.clone()],
                            dependencies: extract_swift_dependencies(&block),
                        },
                    );

                    if !discovery.source_dirs.contains(&source_path) {
                        discovery.source_dirs.push(source_path);
                    }
                }

                search_from = abs_start + pattern.len() + block.len();
            } else {
                search_from = abs_start + pattern.len();
            }
        }
    }

    // Default: if no targets found, check for Sources/ directory
    if discovery.modules.is_empty()
        && access.directory_exists(Path::new("Sources"), "Swift source")?
    {
        discovery.source_dirs.push("Sources".to_string());
    }

    if discovery.modules.is_empty() && discovery.source_dirs.is_empty() {
        Ok(None)
    } else {
        Ok(Some(discovery))
    }
}

/// Extract the content within balanced parentheses.
fn extract_balanced_parens(s: &str) -> Option<String> {
    let mut depth = 1;
    let mut end = 0;
    for (i, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    end = i;
                    break;
                }
            }
            _ => {}
        }
    }
    if depth == 0 {
        Some(s[..end].to_string())
    } else {
        None
    }
}

/// Extract a named string parameter from a Swift function call body.
/// e.g. `name: "Foo"` → Some("Foo")
fn extract_swift_string_param(block: &str, param: &str) -> Option<String> {
    let pattern = format!("{param}:");
    let start = block.find(&pattern)?;
    let after = &block[start + pattern.len()..];
    let quote_start = after.find('"')?;
    let rest = &after[quote_start + 1..];
    let quote_end = rest.find('"')?;
    Some(rest[..quote_end].to_string())
}

/// Extract dependency names from a Swift target block.
fn extract_swift_dependencies(block: &str) -> Vec<String> {
    let mut deps = Vec::new();
    if let Some(start) = block.find("dependencies:") {
        let after = &block[start..];
        if let Some(bracket_start) = after.find('[') {
            let rest = &after[bracket_start + 1..];
            if let Some(bracket_end) = rest.find(']') {
                let deps_str = &rest[..bracket_end];
                // Parse both string deps and .target/.product deps
                for dep in deps_str.split(',') {
                    let dep = dep.trim();
                    // .target(name: "Foo") or .product(name: "Foo", ...)
                    if let Some(name) = extract_swift_string_param(dep, "name") {
                        deps.push(name);
                    }
                    // Simple string dependency: "Foo"
                    else if dep.starts_with('"') && dep.ends_with('"') && dep.len() > 2 {
                        deps.push(dep[1..dep.len() - 1].to_string());
                    }
                }
            }
        }
    }
    deps
}

// ─── build.gradle.kts / build.gradle (Kotlin/Java) ──────────────────────

#[allow(dead_code)]
fn parse_gradle(root: &Path) -> Option<ManifestDiscovery> {
    parse_gradle_checked(root).ok().flatten()
}

fn parse_gradle_checked(root: &Path) -> Result<Option<ManifestDiscovery>, String> {
    let project_root = Dir::open_ambient_dir(root, ambient_authority()).map_err(|error| {
        format!(
            "Cannot open Gradle project root {} as a confined directory: {error}",
            root.display()
        )
    })?;
    parse_gradle_checked_with_root(&project_root)
}

fn parse_gradle_checked_with_root(project_root: &Dir) -> Result<Option<ManifestDiscovery>, String> {
    // Try Kotlin DSL first, then Groovy. A settings manifest is independently sufficient for a
    // multi-project Gradle workspace; do not require a root build script before parsing it. Every
    // recognized variant is preflighted before precedence is selected so a shadowed unsafe entry
    // cannot evade the retained reader.
    let build_kotlin = gradle_confined_manifest_text(project_root, "build.gradle.kts", "build")?;
    let build_groovy = gradle_confined_manifest_text(project_root, "build.gradle", "build")?;
    let settings_kotlin =
        gradle_confined_manifest_text(project_root, "settings.gradle.kts", "settings")?;
    let settings_groovy =
        gradle_confined_manifest_text(project_root, "settings.gradle", "settings")?;
    let build = build_kotlin
        .map(|content| ("build.gradle.kts", content))
        .or_else(|| build_groovy.map(|content| ("build.gradle", content)));
    let settings = settings_kotlin
        .map(|content| ("settings.gradle.kts", content))
        .or_else(|| settings_groovy.map(|content| ("settings.gradle", content)));
    if build.is_none() && settings.is_none() {
        return Ok(None);
    }
    let content = build.map_or_else(String::new, |(_, content)| content);
    let modules = if let Some((settings_name, settings)) = settings {
        parse_gradle_settings(&settings).map_err(|error| {
            format!("Cannot parse Gradle settings manifest {settings_name}: {error}")
        })?
    } else {
        Vec::new()
    };
    let mut discovery = ManifestDiscovery::default();

    // Detect Android project vs plain Kotlin/Java
    let is_android = content.contains("android {") || content.contains("android{");

    if is_android {
        // Android: source in app/src/main/java or app/src/main/kotlin
        for dir in &[
            "app/src/main/java",
            "app/src/main/kotlin",
            "src/main/java",
            "src/main/kotlin",
        ] {
            if gradle_confined_directory_exists(project_root, dir)? {
                discovery.source_dirs.push(dir.to_string());
            }
        }
    } else {
        // Standard Gradle: src/main/kotlin or src/main/java
        for dir in &["src/main/kotlin", "src/main/java", "src/main/scala"] {
            if gradle_confined_directory_exists(project_root, dir)? {
                discovery.source_dirs.push(dir.to_string());
            }
        }
    }

    for module in modules {
        let module_src = format!("{}/src/main", module.path);
        let kotlin_source = format!("{}/src/main/kotlin", module.path);
        let java_source = format!("{}/src/main/java", module.path);
        let source_path = if gradle_confined_directory_exists(project_root, &kotlin_source)? {
            kotlin_source
        } else if gradle_confined_directory_exists(project_root, &java_source)? {
            java_source
        } else {
            module_src
        };

        discovery.modules.insert(
            module.name.clone(),
            ManifestModule {
                name: module.name,
                source_paths: vec![source_path.clone()],
                dependencies: Vec::new(),
            },
        );
        if !discovery.source_dirs.contains(&source_path) {
            discovery.source_dirs.push(source_path);
        }
    }

    if discovery.modules.is_empty() && discovery.source_dirs.is_empty() {
        Ok(None)
    } else {
        Ok(Some(discovery))
    }
}

/// Parse Groovy or Kotlin Gradle settings into effective module directories.
///
/// Supports parenthesized and bare `include` forms, single or double quotes,
/// multiline declarations, nested `:module:name` paths, and the common
/// `project(...).projectDir = ...` and `project(...).setProjectDir(...)`
/// overrides with `file(...)` / `new File(rootDir, ...)` path expressions.
pub(crate) fn parse_gradle_settings(content: &str) -> Result<Vec<GradleSettingsModule>, String> {
    let content = strip_gradle_comments(content)?;
    reject_non_leading_gradle_includes(&content)?;
    reject_unsupported_gradle_project_dir_mutations(&content)?;
    let mut included = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut index = 0usize;
    while index < lines.len() {
        let trimmed = lines[index].trim_start();
        if !is_gradle_include_start(trimmed) {
            index += 1;
            continue;
        }

        let mut statement = trimmed.to_string();
        let mut balance = gradle_paren_balance(trimmed);
        while index + 1 < lines.len() && (balance > 0 || statement.trim_end().ends_with(',')) {
            index += 1;
            statement.push('\n');
            statement.push_str(lines[index].trim());
            balance += gradle_paren_balance(lines[index]);
        }
        if balance != 0 {
            return Err("Gradle include declaration has unbalanced parentheses".to_string());
        }
        for value in parse_gradle_include_statement(&statement)? {
            if !value.trim().trim_start_matches(':').is_empty() {
                included.push(normalize_gradle_module_path(&value, "module path")?);
            }
        }
        index += 1;
    }

    let overrides = parse_gradle_project_dir_overrides(&content)?;

    included.sort();
    included.dedup();
    Ok(included
        .into_iter()
        .map(|name| GradleSettingsModule {
            path: overrides
                .get(&name)
                .cloned()
                .unwrap_or_else(|| name.clone()),
            name,
        })
        .collect())
}

fn parse_gradle_project_dir_overrides(content: &str) -> Result<HashMap<String, String>, String> {
    let mut overrides = HashMap::new();
    let mut search_start = 0usize;

    while let Some((project_dir_index, syntax)) =
        find_next_gradle_project_dir_override(content, search_start)
    {
        let marker = syntax.marker();
        let after_marker = content[project_dir_index + marker.len()..].trim_start();
        if matches!(syntax, GradleProjectDirSyntax::Assignment)
            && !is_gradle_simple_assignment(after_marker)?
        {
            search_start = project_dir_index + marker.len();
            continue;
        }
        let before = &content[..project_dir_index];
        let Some(project_index) = find_gradle_project_call(before) else {
            return Err("Unsupported indirect Gradle projectDir mutation".to_string());
        };
        let line_start = before[..project_index]
            .rfind('\n')
            .map_or(0, |index| index + 1);
        if !before[line_start..project_index].trim().is_empty() {
            return Err(
                "Unsupported qualified or conditional Gradle projectDir mutation".to_string(),
            );
        }
        if gradle_brace_depth(before)? != 0 {
            return Err("Unsupported block-scoped Gradle projectDir mutation".to_string());
        }
        if gradle_follows_detached_control_header(content, project_index)? {
            return Err("Unsupported conditional Gradle projectDir mutation".to_string());
        }

        let project_call = before[project_index + "project".len()..].trim_start();
        let (project_arguments, project_remainder) = gradle_parenthesized(project_call)?;
        if !project_remainder.trim().is_empty() {
            return Err("Unsupported indirect Gradle projectDir mutation".to_string());
        }
        let module_values = gradle_string_arguments(project_arguments)?;
        if module_values.len() != 1 {
            return Err("Gradle projectDir assignment must identify one module".to_string());
        }

        let expression = match syntax {
            GradleProjectDirSyntax::Assignment => {
                let Some(right_hand_side) = after_marker.strip_prefix('=') else {
                    return Err("Gradle projectDir assignment is missing '='".to_string());
                };
                right_hand_side.trim_start()
            }
            GradleProjectDirSyntax::Setter => {
                let (arguments, remainder) = gradle_parenthesized(after_marker)?;
                require_gradle_expression_end(remainder)?;
                arguments.trim()
            }
        };
        let path = parse_gradle_project_dir_path(expression)?;

        let module = normalize_gradle_module_path(&module_values[0], "projectDir module path")?;
        let path = normalize_gradle_project_relative_path(&path, "projectDir path", true)?;
        overrides.insert(module, path);
        search_start = project_dir_index + marker.len();
    }

    Ok(overrides)
}

#[derive(Clone, Copy)]
enum GradleProjectDirSyntax {
    Assignment,
    Setter,
}

impl GradleProjectDirSyntax {
    fn marker(self) -> &'static str {
        match self {
            Self::Assignment => ".projectDir",
            Self::Setter => ".setProjectDir",
        }
    }
}

fn is_gradle_simple_assignment(after_marker: &str) -> Result<bool, String> {
    if let Some(remainder) = after_marker.strip_prefix('=') {
        return Ok(!remainder.starts_with('=') && !remainder.starts_with('~'));
    }
    let mutation_prefix = after_marker
        .chars()
        .take_while(|character| !character.is_whitespace())
        .collect::<String>();
    if mutation_prefix.ends_with('=')
        && mutation_prefix.chars().any(|character| {
            matches!(
                character,
                '+' | '-' | '*' | '/' | '%' | '?' | '&' | '|' | '^'
            )
        })
    {
        return Err("Unsupported Gradle projectDir mutation operator".to_string());
    }
    Ok(false)
}

fn find_next_gradle_project_dir_override(
    content: &str,
    search_start: usize,
) -> Option<(usize, GradleProjectDirSyntax)> {
    [
        GradleProjectDirSyntax::Assignment,
        GradleProjectDirSyntax::Setter,
    ]
    .into_iter()
    .filter_map(|syntax| {
        find_unquoted_gradle_fragment(content, syntax.marker(), search_start)
            .map(|index| (index, syntax))
    })
    .min_by_key(|(index, _)| *index)
}

fn parse_gradle_project_dir_path(expression: &str) -> Result<String, String> {
    if let Some(file_call) = expression.strip_prefix("file") {
        let (arguments, remainder) = gradle_parenthesized(file_call.trim_start())?;
        require_gradle_expression_end(remainder)?;
        let path_values = gradle_string_arguments(arguments)?;
        if path_values.len() != 1 {
            return Err("Gradle projectDir assignment must contain one path".to_string());
        }
        Ok(path_values[0].clone())
    } else if let Some(file_call) = expression.strip_prefix("new") {
        if file_call
            .chars()
            .next()
            .is_none_or(|character| !character.is_whitespace())
        {
            return Err("Unsupported Gradle projectDir assignment".to_string());
        }
        let Some(file_call) = file_call.trim_start().strip_prefix("File") else {
            return Err("Unsupported Gradle projectDir assignment".to_string());
        };
        let (arguments, remainder) = gradle_parenthesized(file_call.trim_start())?;
        require_gradle_expression_end(remainder)?;
        parse_gradle_root_dir_file_arguments(arguments)
    } else {
        Err("Unsupported Gradle projectDir assignment".to_string())
    }
}

fn normalize_gradle_module_path(value: &str, label: &str) -> Result<String, String> {
    let trimmed = value.trim();
    let rooted_gradle_identity = trimmed.starts_with(':');
    let value = trimmed.trim_start_matches(':');
    let bytes = value.as_bytes();
    if bytes.get(1) == Some(&b':')
        && bytes.first().is_some_and(u8::is_ascii_alphabetic)
        && (!rooted_gradle_identity
            || bytes
                .get(2)
                .is_some_and(|byte| matches!(byte, b'/' | b'\\')))
    {
        return Err(format!(
            "Gradle {label} must remain beneath the project root"
        ));
    }
    normalize_gradle_project_relative_path(&value.replace(':', "/"), label, false)
}

fn normalize_gradle_project_relative_path(
    value: &str,
    label: &str,
    allow_project_root: bool,
) -> Result<String, String> {
    let normalized = value.replace('\\', "/");
    if normalized.starts_with('/') {
        return Err(format!(
            "Gradle {label} must remain beneath the project root"
        ));
    }

    let mut components = Vec::new();
    let mut requires_normalization = false;
    for component in normalized.split('/') {
        if component.is_empty() {
            continue;
        }
        if component == "." {
            requires_normalization = true;
            continue;
        }
        if component == ".." {
            requires_normalization = true;
            if components.pop().is_none() {
                return Err(format!(
                    "Gradle {label} must remain beneath the project root"
                ));
            }
            continue;
        }
        if components.is_empty()
            && component.as_bytes().get(1) == Some(&b':')
            && component
                .as_bytes()
                .first()
                .is_some_and(u8::is_ascii_alphabetic)
        {
            return Err(format!(
                "Gradle {label} must remain beneath the project root"
            ));
        }
        components.push(component);
    }

    if components.is_empty() {
        if allow_project_root {
            return Ok(".".to_string());
        }
        return Err(format!("Gradle {label} must identify a project path"));
    }
    Ok(if requires_normalization {
        components.join("/")
    } else {
        normalized
    })
}

#[cfg(windows)]
fn gradle_metadata_is_link(metadata: &Metadata) -> bool {
    use cap_std::fs::MetadataExt;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn gradle_metadata_is_link(metadata: &Metadata) -> bool {
    metadata.file_type().is_symlink()
}

#[cfg(unix)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GradleFilesystemIdentity {
    device: u64,
    inode: u64,
}

#[cfg(windows)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GradleFilesystemIdentity {
    volume_serial_number: u32,
    file_index: u64,
}

#[cfg(unix)]
fn gradle_filesystem_identity(metadata: &Metadata) -> Result<GradleFilesystemIdentity, String> {
    use cap_std::fs::MetadataExt;

    Ok(GradleFilesystemIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
    })
}

#[cfg(windows)]
fn gradle_filesystem_identity(metadata: &Metadata) -> Result<GradleFilesystemIdentity, String> {
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
struct GradleFilesystemIdentity;

#[cfg(not(any(unix, windows)))]
fn gradle_filesystem_identity(_metadata: &Metadata) -> Result<GradleFilesystemIdentity, String> {
    Err("filesystem identity is unavailable on this platform".to_string())
}

fn verify_retained_project_root(root: &Path, retained: &Dir, phase: &str) -> Result<(), String> {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GradleManifestReadCheckpoint {
    PreOpen,
    Opened,
    AfterOpen,
    AfterRead,
}

fn gradle_confined_manifest_text(
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

fn gradle_confined_directory_exists(root: &Dir, relative: &str) -> Result<bool, String> {
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

fn find_gradle_project_call(content: &str) -> Option<usize> {
    let mut search_start = 0usize;
    let mut found = None;
    while let Some(index) = find_unquoted_gradle_fragment(content, "project", search_start) {
        let before_is_boundary = content[..index]
            .chars()
            .next_back()
            .is_none_or(|character| !character.is_alphanumeric() && character != '_');
        let after = content[index + "project".len()..].trim_start();
        let after_is_boundary = content[index + "project".len()..]
            .chars()
            .next()
            .is_none_or(|character| !character.is_alphanumeric() && character != '_');
        if before_is_boundary && after_is_boundary && after.starts_with('(') {
            found = Some(index);
        }
        search_start = index + "project".len();
    }
    found
}

fn gradle_parenthesized(value: &str) -> Result<(&str, &str), String> {
    let Some(value) = value.strip_prefix('(') else {
        return Err("Gradle declaration is missing an opening parenthesis".to_string());
    };
    let mut depth = 1usize;
    let mut quote = None;
    let mut escaped = false;
    for (index, character) in value.char_indices() {
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == delimiter {
                quote = None;
            }
            continue;
        }
        if matches!(character, '\'' | '"') {
            quote = Some(character);
            continue;
        }
        match character {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Ok((&value[..index], &value[index + character.len_utf8()..]));
                }
            }
            _ => {}
        }
    }
    Err("Gradle declaration has unbalanced parentheses".to_string())
}

fn parse_gradle_include_statement(statement: &str) -> Result<Vec<String>, String> {
    let Some(arguments) = statement.strip_prefix("include") else {
        return Err("Gradle include declaration is missing 'include'".to_string());
    };
    let arguments = arguments.trim_start();
    let values = if arguments.starts_with('(') {
        let (arguments, remainder) = gradle_parenthesized(arguments)?;
        require_gradle_complete_remainder(remainder, "include declaration")?;
        gradle_string_arguments(arguments)
    } else {
        let arguments = strip_gradle_statement_terminator(arguments, "include declaration")?;
        gradle_string_arguments(arguments)
    }?;
    if values.is_empty() {
        return Err("Gradle include declaration must contain a literal module".to_string());
    }
    Ok(values)
}

fn parse_gradle_root_dir_file_arguments(arguments: &str) -> Result<String, String> {
    let Some(arguments) = arguments.trim_start().strip_prefix("rootDir") else {
        return Err("Gradle new File projectDir base must be rootDir".to_string());
    };
    let Some(arguments) = arguments.trim_start().strip_prefix(',') else {
        return Err("Gradle new File projectDir base must be exactly rootDir".to_string());
    };
    let path_values = gradle_string_arguments(arguments)?;
    if path_values.len() != 1 {
        return Err("Gradle new File projectDir assignment must contain one path".to_string());
    }
    Ok(path_values[0].clone())
}

fn require_gradle_expression_end(remainder: &str) -> Result<(), String> {
    let statement_remainder = remainder
        .split_once('\n')
        .map_or(remainder, |(line, _)| line);
    require_gradle_complete_remainder(statement_remainder, "projectDir assignment")
}

fn require_gradle_complete_remainder(remainder: &str, context: &str) -> Result<(), String> {
    let remainder = remainder.trim();
    if remainder.is_empty() || remainder == ";" {
        Ok(())
    } else {
        Err(format!("Unsupported trailing Gradle {context} expression"))
    }
}

fn strip_gradle_statement_terminator<'a>(
    statement: &'a str,
    context: &str,
) -> Result<&'a str, String> {
    let statement = statement.trim_end();
    let Some(statement) = statement.strip_suffix(';') else {
        return Ok(statement);
    };
    if statement.trim_end().ends_with(';') {
        return Err(format!("Unsupported trailing Gradle {context} expression"));
    }
    Ok(statement.trim_end())
}

fn strip_gradle_comments(content: &str) -> Result<String, String> {
    let mut cleaned = String::with_capacity(content.len());
    let characters = content.chars().collect::<Vec<_>>();
    let mut index = 0usize;

    while index < characters.len() {
        let character = characters[index];
        if matches!(character, '\'' | '"')
            && characters.get(index + 1) == Some(&character)
            && characters.get(index + 2) == Some(&character)
        {
            cleaned.extend([' ', ' ', ' ']);
            index += 3;
            let mut terminated = false;
            while index < characters.len() {
                if characters.get(index) == Some(&character)
                    && characters.get(index + 1) == Some(&character)
                    && characters.get(index + 2) == Some(&character)
                {
                    cleaned.extend([' ', ' ', ' ']);
                    index += 3;
                    terminated = true;
                    break;
                }
                cleaned.push(if characters[index] == '\n' { '\n' } else { ' ' });
                index += 1;
            }
            if !terminated {
                return Err("Gradle settings contain an unterminated multiline string".to_string());
            }
            continue;
        }
        if matches!(character, '\'' | '"') {
            let delimiter = character;
            let mut escaped = false;
            let mut terminated = false;
            cleaned.push(character);
            index += 1;
            while index < characters.len() {
                let quoted = characters[index];
                cleaned.push(quoted);
                index += 1;
                if escaped {
                    escaped = false;
                } else if quoted == '\\' {
                    escaped = true;
                } else if quoted == delimiter {
                    terminated = true;
                    break;
                }
            }
            if escaped {
                return Err("Gradle settings contain a dangling string escape".to_string());
            }
            if !terminated {
                return Err("Gradle settings contain an unterminated quoted string".to_string());
            }
            continue;
        }
        if character == '/' && characters.get(index + 1) == Some(&'/') {
            index += 2;
            while index < characters.len() {
                let comment_character = characters[index];
                index += 1;
                if comment_character == '\n' {
                    cleaned.push('\n');
                    break;
                }
            }
            continue;
        }
        if character == '/' && characters.get(index + 1) == Some(&'*') {
            index += 2;
            cleaned.push(' ');
            let mut depth = 1usize;
            while index < characters.len() && depth > 0 {
                if characters.get(index) == Some(&'/') && characters.get(index + 1) == Some(&'*') {
                    depth += 1;
                    cleaned.extend([' ', ' ']);
                    index += 2;
                } else if characters.get(index) == Some(&'*')
                    && characters.get(index + 1) == Some(&'/')
                {
                    depth -= 1;
                    cleaned.extend([' ', ' ']);
                    index += 2;
                } else {
                    cleaned.push(if characters[index] == '\n' { '\n' } else { ' ' });
                    index += 1;
                }
            }
            if depth != 0 {
                return Err("Gradle settings contain an unterminated block comment".to_string());
            }
            continue;
        }
        cleaned.push(character);
        index += 1;
    }

    Ok(cleaned)
}

fn reject_non_leading_gradle_includes(content: &str) -> Result<(), String> {
    let mut search_start = 0usize;
    while let Some(index) = find_unquoted_gradle_fragment(content, "include", search_start) {
        let before_is_boundary = content[..index]
            .chars()
            .next_back()
            .is_none_or(|character| !character.is_alphanumeric() && character != '_');
        if !before_is_boundary {
            search_start = index + "include".len();
            continue;
        }
        let token_end = content[index..]
            .char_indices()
            .take_while(|(_, character)| character.is_alphanumeric() || *character == '_')
            .last()
            .map_or(index, |(offset, character)| {
                index + offset + character.len_utf8()
            });
        let token = &content[index..token_end];
        if token == "include" {
            let line_start = content[..index].rfind('\n').map_or(0, |line| line + 1);
            if !content[line_start..index].trim().is_empty() {
                return Err("Unsupported qualified or conditional Gradle include".to_string());
            }
            if gradle_brace_depth(&content[..index])? != 0 {
                return Err("Unsupported block-scoped Gradle include".to_string());
            }
            if gradle_follows_detached_control_header(content, index)? {
                return Err("Unsupported conditional Gradle include".to_string());
            }
        } else if token.starts_with("include")
            && gradle_include_prefixed_token_is_executable(content, index, token_end)
        {
            return Err(format!("Unsupported Gradle workspace mutator {token}"));
        }
        search_start = token_end.max(index + "include".len());
    }
    Ok(())
}

fn gradle_include_prefixed_token_is_executable(
    content: &str,
    token_start: usize,
    token_end: usize,
) -> bool {
    let line_start = content[..token_start]
        .rfind('\n')
        .map_or(0, |line| line + 1);
    let leading = content[line_start..token_start].trim();
    let trailing = content[token_end..]
        .split_once('\n')
        .map_or(&content[token_end..], |(line, _)| line)
        .trim_start();
    if trailing.is_empty() {
        return false;
    }
    if trailing.starts_with("==")
        || trailing.starts_with("!=")
        || trailing.starts_with("<=")
        || trailing.starts_with(">=")
        || trailing.starts_with('.')
        || trailing.starts_with("?.")
    {
        return false;
    }
    leading.is_empty()
        || trailing.starts_with('(')
        || trailing.starts_with('\'')
        || trailing.starts_with('"')
        || trailing.starts_with('{')
        || trailing
            .chars()
            .next()
            .is_some_and(|character| character.is_alphanumeric() || character == '_')
}

fn gradle_follows_detached_control_header(
    content: &str,
    directive_start: usize,
) -> Result<bool, String> {
    let preceding = content[..directive_start].trim_end();
    if preceding.is_empty() {
        return Ok(false);
    }
    for keyword in ["else", "do", "try", "finally"] {
        if preceding.ends_with(keyword) {
            let keyword_start = preceding.len() - keyword.len();
            let boundary = preceding[..keyword_start]
                .chars()
                .next_back()
                .is_none_or(|character| !character.is_alphanumeric() && character != '_');
            if boundary {
                return Ok(true);
            }
        }
    }
    if !preceding.ends_with(')') {
        return Ok(false);
    }

    let mut quote = None;
    let mut escaped = false;
    let mut open_parens = Vec::new();
    let mut final_open = None;
    for (index, character) in preceding.char_indices() {
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == delimiter {
                quote = None;
            }
            continue;
        }
        if matches!(character, '\'' | '"') {
            quote = Some(character);
            continue;
        }
        if character == '(' {
            open_parens.push(index);
        } else if character == ')' {
            let Some(open) = open_parens.pop() else {
                return Err("Gradle settings contain unmatched parentheses".to_string());
            };
            if index + character.len_utf8() == preceding.len() {
                final_open = Some(open);
            }
        }
    }
    let Some(final_open) = final_open else {
        return Ok(false);
    };
    let before_open = preceding[..final_open].trim_end();
    let keyword_start = before_open
        .char_indices()
        .rev()
        .take_while(|(_, character)| character.is_alphanumeric() || *character == '_')
        .last()
        .map_or(before_open.len(), |(index, _)| index);
    let keyword = &before_open[keyword_start..];
    Ok(matches!(
        keyword,
        "if" | "when" | "for" | "while" | "switch" | "catch"
    ))
}

fn gradle_brace_depth(content: &str) -> Result<i32, String> {
    let mut depth = 0i32;
    let mut quote = None;
    let mut escaped = false;
    for character in content.chars() {
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == delimiter {
                quote = None;
            }
            continue;
        }
        if matches!(character, '\'' | '"') {
            quote = Some(character);
        } else if character == '{' {
            depth += 1;
        } else if character == '}' {
            depth -= 1;
            if depth < 0 {
                return Err("Gradle settings contain an unmatched closing brace".to_string());
            }
        }
    }
    Ok(depth)
}

fn reject_unsupported_gradle_project_dir_mutations(content: &str) -> Result<(), String> {
    reject_unrecognized_gradle_project_mutations(content)?;

    let mut search_start = 0usize;
    while let Some(index) = find_unquoted_gradle_fragment(content, "projectDir", search_start) {
        let before_is_boundary = content[..index]
            .chars()
            .next_back()
            .is_none_or(|character| !character.is_alphanumeric() && character != '_');
        let after_index = index + "projectDir".len();
        let after_is_boundary = content[after_index..]
            .chars()
            .next()
            .is_none_or(|character| !character.is_alphanumeric() && character != '_');
        let directly_dotted = content[..index].ends_with('.');
        if before_is_boundary && after_is_boundary && !directly_dotted {
            let after = content[after_index..].trim_start();
            if is_gradle_simple_assignment(after)?.then_some(()).is_some() {
                return Err("Unsupported indirect Gradle projectDir mutation".to_string());
            }
        }
        search_start = after_index;
    }

    search_start = 0;
    while let Some(index) = find_unquoted_gradle_fragment(content, "setProjectDir", search_start) {
        let before_is_boundary = content[..index]
            .chars()
            .next_back()
            .is_none_or(|character| !character.is_alphanumeric() && character != '_');
        let after_index = index + "setProjectDir".len();
        let after_is_boundary = content[after_index..]
            .chars()
            .next()
            .is_none_or(|character| !character.is_alphanumeric() && character != '_');
        let directly_dotted = content[..index].ends_with('.');
        if before_is_boundary
            && after_is_boundary
            && !directly_dotted
            && content[after_index..].trim_start().starts_with('(')
        {
            return Err("Unsupported indirect Gradle setProjectDir mutation".to_string());
        }
        search_start = after_index;
    }
    Ok(())
}

fn reject_unrecognized_gradle_project_mutations(content: &str) -> Result<(), String> {
    let mut search_start = 0usize;
    while let Some(project_index) = find_unquoted_gradle_fragment(content, "project", search_start)
    {
        let before_is_boundary = content[..project_index]
            .chars()
            .next_back()
            .is_none_or(|character| !character.is_alphanumeric() && character != '_');
        let after_keyword = &content[project_index + "project".len()..];
        let after_is_boundary = after_keyword
            .chars()
            .next()
            .is_none_or(|character| !character.is_alphanumeric() && character != '_');
        let project_call = after_keyword.trim_start();
        if !before_is_boundary || !after_is_boundary || !project_call.starts_with('(') {
            search_start = project_index + "project".len();
            continue;
        }

        let (_, remainder) = gradle_parenthesized(project_call)?;
        let call_end = content.len() - remainder.len();
        let suffix = gradle_project_chain_suffix(remainder);
        if suffix.is_empty() {
            search_start = call_end;
            continue;
        }
        let statement = gradle_project_chain_statement(suffix).trim();

        if suffix.starts_with(".setProjectDir") {
            search_start = call_end;
            continue;
        }
        if let Some(after_project_dir) = suffix.strip_prefix(".projectDir")
            && is_gradle_simple_assignment(after_project_dir.trim_start())?
        {
            search_start = call_end;
            continue;
        }
        if suffix.starts_with(".setProperty") {
            return Err("Unsupported dynamic Gradle project mutation".to_string());
        }

        if !gradle_project_statement_is_read_only(statement) {
            return Err("Unsupported executable Gradle project mutation".to_string());
        }
        search_start = call_end;
    }
    Ok(())
}

fn gradle_project_chain_suffix(remainder: &str) -> &str {
    let horizontal = remainder.trim_start_matches([' ', '\t', '\r']);
    if horizontal.starts_with('.') || horizontal.starts_with('[') {
        return horizontal;
    }
    if horizontal.starts_with('\n') {
        let continued = horizontal.trim_start();
        if continued.starts_with('.') || continued.starts_with('[') {
            return continued;
        }
    }
    ""
}

fn gradle_project_chain_statement(suffix: &str) -> &str {
    let mut quote = None;
    let mut escaped = false;
    let mut parentheses = 0i32;
    let mut brackets = 0i32;
    let mut braces = 0i32;
    for (index, character) in suffix.char_indices() {
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == delimiter {
                quote = None;
            }
            continue;
        }
        if matches!(character, '\'' | '"') {
            quote = Some(character);
            continue;
        }
        match character {
            '(' => parentheses += 1,
            ')' => parentheses -= 1,
            '[' => brackets += 1,
            ']' => brackets -= 1,
            '{' => braces += 1,
            '}' => braces -= 1,
            ';' if parentheses <= 0 && brackets <= 0 && braces <= 0 => {
                return &suffix[..index];
            }
            '\n' if parentheses <= 0 && brackets <= 0 && braces <= 0 => {
                let before = suffix[..index].trim_end();
                let after = suffix[index + character.len_utf8()..].trim_start();
                if !gradle_statement_continues(before, after) {
                    return &suffix[..index];
                }
            }
            _ => {}
        }
    }
    suffix
}

fn gradle_statement_continues(before: &str, after: &str) -> bool {
    before.chars().next_back().is_some_and(|character| {
        matches!(
            character,
            '.' | '?' | ':' | '=' | '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^'
        )
    }) || after.starts_with('.')
        || after.starts_with('[')
        || after.starts_with('=')
        || after.starts_with("+=")
        || after.starts_with("-=")
        || after.starts_with("*=")
        || after.starts_with("/=")
}

fn gradle_project_statement_is_read_only(statement: &str) -> bool {
    if statement.is_empty() || statement == ";" {
        return true;
    }
    if !statement.starts_with('.') && !statement.starts_with('[') {
        return false;
    }
    if gradle_contains_mutating_assignment(statement) || statement.contains('{') {
        return false;
    }

    let comparison = ["==", "!=", "<=", ">=", "=~", "!~"]
        .into_iter()
        .filter_map(|operator| statement.find(operator))
        .min();
    let invocation = find_unquoted_gradle_fragment(statement, "(", 0);
    match (comparison, invocation) {
        (Some(comparison), Some(invocation)) => comparison < invocation,
        (Some(_), None) | (None, None) => true,
        (None, Some(_)) => false,
    }
}

fn gradle_contains_mutating_assignment(statement: &str) -> bool {
    let mut quote = None;
    let mut escaped = false;
    for (index, character) in statement.char_indices() {
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == delimiter {
                quote = None;
            }
            continue;
        }
        if matches!(character, '\'' | '"') {
            quote = Some(character);
            continue;
        }
        if character != '=' {
            continue;
        }
        let before = statement[..index].chars().next_back();
        let after = statement[index + character.len_utf8()..].chars().next();
        if after.is_some_and(|next| matches!(next, '=' | '~'))
            || before.is_some_and(|previous| matches!(previous, '=' | '!' | '<' | '>'))
        {
            continue;
        }
        return true;
    }
    false
}

fn find_unquoted_gradle_fragment(
    content: &str,
    fragment: &str,
    search_start: usize,
) -> Option<usize> {
    let mut quote = None;
    let mut escaped = false;
    for (relative, character) in content[search_start..].char_indices() {
        let index = search_start + relative;
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == delimiter {
                quote = None;
            }
            continue;
        }
        if matches!(character, '\'' | '"') {
            quote = Some(character);
            continue;
        }
        if content[index..].starts_with(fragment) {
            return Some(index);
        }
    }
    None
}

fn is_gradle_include_start(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("include") else {
        return false;
    };
    rest.is_empty()
        || rest.chars().next().is_some_and(|character| {
            character.is_whitespace() || matches!(character, '(' | '\'' | '"')
        })
}

fn gradle_paren_balance(value: &str) -> i32 {
    let mut balance = 0i32;
    let mut quote = None;
    let mut escaped = false;
    for character in value.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' && quote.is_some() {
            escaped = true;
            continue;
        }
        if matches!(character, '\'' | '"') {
            if quote == Some(character) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(character);
            }
            continue;
        }
        if quote.is_none() {
            match character {
                '(' => balance += 1,
                ')' => balance -= 1,
                _ => {}
            }
        }
    }
    balance
}

fn gradle_string_arguments(mut value: &str) -> Result<Vec<String>, String> {
    let mut values = Vec::new();
    loop {
        value = value.trim_start();
        if value.is_empty() {
            return Ok(values);
        }

        let (parsed, remainder) = gradle_string_literal(value)?;
        values.push(parsed);
        value = remainder.trim_start();
        if value.is_empty() {
            return Ok(values);
        }
        let Some(remainder) = value.strip_prefix(',') else {
            return Err("Unsupported or dynamic Gradle expression".to_string());
        };
        value = remainder;
    }
}

fn gradle_string_literal(value: &str) -> Result<(String, &str), String> {
    let Some(delimiter) = value
        .chars()
        .next()
        .filter(|character| matches!(character, '\'' | '"'))
    else {
        return Err("Unsupported or dynamic Gradle expression".to_string());
    };
    let mut parsed = String::new();
    let body = &value[delimiter.len_utf8()..];
    let mut characters = body.char_indices().peekable();
    while let Some((index, character)) = characters.next() {
        if character == delimiter {
            let end = delimiter.len_utf8() + index + character.len_utf8();
            return Ok((parsed, &value[end..]));
        }
        if delimiter == '"' && character == '$' {
            return Err("Unsupported or dynamic Gradle expression".to_string());
        }
        if character != '\\' {
            parsed.push(character);
            continue;
        }

        let Some((_, escaped)) = characters.next() else {
            return Err("Gradle settings contain a dangling string escape".to_string());
        };
        match escaped {
            '\\' | '\'' | '"' | '$' => parsed.push(escaped),
            'n' => parsed.push('\n'),
            'r' => parsed.push('\r'),
            't' => parsed.push('\t'),
            'b' => parsed.push('\u{0008}'),
            'f' => parsed.push('\u{000c}'),
            'u' => {
                let mut value = 0u32;
                for _ in 0..4 {
                    let Some((_, digit)) = characters.next() else {
                        return Err(
                            "Gradle settings contain an incomplete Unicode escape".to_string()
                        );
                    };
                    let Some(digit) = digit.to_digit(16) else {
                        return Err("Gradle settings contain an invalid Unicode escape".to_string());
                    };
                    value = value * 16 + digit;
                }
                let Some(decoded) = char::from_u32(value) else {
                    return Err("Gradle settings contain an invalid Unicode escape".to_string());
                };
                if delimiter == '"' && decoded == '$' {
                    return Err("Unsupported or dynamic Gradle expression".to_string());
                }
                parsed.push(decoded);
            }
            '0'..='7' => {
                let mut value = escaped.to_digit(8).unwrap_or_default();
                for _ in 0..2 {
                    let Some((_, digit)) = characters.peek().copied() else {
                        break;
                    };
                    let Some(digit) = digit.to_digit(8) else {
                        break;
                    };
                    characters.next();
                    value = value * 8 + digit;
                }
                let Some(decoded) = char::from_u32(value) else {
                    return Err("Gradle settings contain an invalid octal escape".to_string());
                };
                if delimiter == '"' && decoded == '$' {
                    return Err("Unsupported or dynamic Gradle expression".to_string());
                }
                parsed.push(decoded);
            }
            _ => {
                return Err(format!(
                    "Gradle settings contain an unsupported string escape: \\{escaped}"
                ));
            }
        }
    }
    Err("Gradle settings contain an unterminated quoted string".to_string())
}

// ─── package.json (TypeScript/JavaScript) ────────────────────────────────

#[cfg(test)]
fn parse_package_json(root: &Path) -> Option<ManifestDiscovery> {
    let mut access = AmbientManifestAccess { root };
    parse_package_json_with_access(&mut access).ok().flatten()
}

fn parse_package_json_with_access(
    access: &mut dyn ManifestAccess,
) -> Result<Option<ManifestDiscovery>, String> {
    parse_package_json_with_access_and_hooks(access, |_| {}, |_, _| {})
}

#[cfg(test)]
fn parse_package_json_with_access_and_hook<AfterEnumeration>(
    access: &mut dyn ManifestAccess,
    after_enumeration: AfterEnumeration,
) -> Result<Option<ManifestDiscovery>, String>
where
    AfterEnumeration: FnMut(&Path),
{
    parse_package_json_with_access_and_hooks(access, after_enumeration, |_, _| {})
}

fn parse_package_json_with_access_and_hooks<AfterEnumeration, AfterChildRead>(
    access: &mut dyn ManifestAccess,
    mut after_enumeration: AfterEnumeration,
    mut after_child_read: AfterChildRead,
) -> Result<Option<ManifestDiscovery>, String>
where
    AfterEnumeration: FnMut(&Path),
    AfterChildRead: FnMut(&Path, &str),
{
    let Some(content) = access.read_text(Path::new("package.json"), "Node package manifest")?
    else {
        return Ok(None);
    };
    let json = parse_node_package_document(&content, Path::new("package.json"))?;
    let mut discovery = ManifestDiscovery::default();

    let name = json.get("name").and_then(|v| v.as_str()).unwrap_or("app");

    // Check for workspaces (monorepo)
    if let Some(workspace_patterns) =
        node_workspace_patterns(&json, Path::new("package.json"), access)?
    {
        let mut expanded_bases = HashSet::new();
        let mut seen_workspaces = HashSet::new();
        for pattern in workspace_patterns {
            // Simple glob: "packages/*" → look for subdirs
            let base = pattern.trim_end_matches("/*").trim_end_matches("/**");
            let base_dir =
                normalize_retained_manifest_path(Path::new(base), "Node workspace base")?;
            if !expanded_bases.insert(base_dir.clone()) {
                continue;
            }
            let workspace_names = access.child_directories(&base_dir, "Node workspace")?;
            after_enumeration(&base_dir);
            for ws_name in workspace_names {
                let workspace = base_dir.join(&ws_name);
                if !seen_workspaces.insert(workspace.clone()) {
                    continue;
                }
                let workspace_content = access.read_enumerated_child_text(
                    &base_dir,
                    &ws_name,
                    Path::new("package.json"),
                    "Node workspace package manifest",
                )?;
                after_child_read(&base_dir, &ws_name);
                access.verify_enumerated_child(&base_dir, &ws_name, "Node workspace")?;
                if let Some(workspace_content) = workspace_content {
                    let workspace_package = parse_node_package_document(
                        &workspace_content,
                        &workspace.join("package.json"),
                    )?;
                    node_workspace_patterns(
                        &workspace_package,
                        &workspace.join("package.json"),
                        access,
                    )?;
                    let src_dir = if access.enumerated_child_directory_exists(
                        &base_dir,
                        &ws_name,
                        Path::new("src"),
                        "Node workspace source",
                    )? {
                        manifest_path_text(&workspace.join("src"), "Node workspace source")?
                    } else {
                        manifest_path_text(&workspace, "Node workspace source")?
                    };
                    discovery.modules.insert(
                        ws_name.clone(),
                        ManifestModule {
                            name: ws_name.clone(),
                            source_paths: vec![src_dir.clone()],
                            dependencies: Vec::new(),
                        },
                    );
                    if !discovery.source_dirs.contains(&src_dir) {
                        discovery.source_dirs.push(src_dir);
                    }
                }
                access.verify_enumerated_child(&base_dir, &ws_name, "Node workspace")?;
            }
            access.verify_child_directories(&base_dir, "Node workspace")?;
            access.release_child_directories(&base_dir, "Node workspace")?;
        }
    }

    // Detect main source directory
    let main_field = json.get("main").and_then(|v| v.as_str()).unwrap_or("");
    let src_dir = if access.directory_exists(Path::new("src"), "Node source")? {
        "src"
    } else if access.directory_exists(Path::new("lib"), "Node source")? {
        "lib"
    } else if main_field.starts_with("./") {
        Path::new(main_field)
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or("src")
    } else {
        "src"
    };

    if discovery.modules.is_empty() {
        discovery.modules.insert(
            name.to_string(),
            ManifestModule {
                name: name.to_string(),
                source_paths: vec![src_dir.to_string()],
                dependencies: Vec::new(),
            },
        );
    }

    if !discovery.source_dirs.contains(&src_dir.to_string()) {
        discovery.source_dirs.push(src_dir.to_string());
    }

    Ok(Some(discovery))
}

fn parse_node_package_document(
    content: &str,
    manifest_path: &Path,
) -> Result<serde_json::Map<String, serde_json::Value>, String> {
    let value = serde_json::from_str::<serde_json::Value>(content).map_err(|error| {
        format!(
            "Cannot parse retained Node package manifest {} as JSON: {error}",
            display_manifest_path(manifest_path)
        )
    })?;
    value.as_object().cloned().ok_or_else(|| {
        format!(
            "Retained Node package manifest {} must contain a JSON object",
            display_manifest_path(manifest_path)
        )
    })
}

fn node_workspace_patterns(
    package: &serde_json::Map<String, serde_json::Value>,
    manifest_path: &Path,
    access: &mut dyn ManifestAccess,
) -> Result<Option<Vec<String>>, String> {
    let Some(workspaces) = package.get("workspaces") else {
        return Ok(None);
    };
    let entries = match workspaces {
        serde_json::Value::Array(entries) => entries,
        serde_json::Value::Object(object) => object
            .get("packages")
            .ok_or_else(|| {
                format!(
                    "Retained Node package manifest {} must define `workspaces.packages` as a string array",
                    display_manifest_path(manifest_path)
                )
            })?
            .as_array()
            .ok_or_else(|| {
                format!(
                    "Retained Node package manifest {} must define `workspaces.packages` as a string array",
                    display_manifest_path(manifest_path)
                )
            })?,
        _ => {
            return Err(format!(
                "Retained Node package manifest {} must define `workspaces` as a string array or an object with a `packages` string array",
                display_manifest_path(manifest_path)
            ));
        }
    };
    access.charge_entries(entries.len(), "Node workspace declaration")?;
    entries
        .iter()
        .map(|entry| {
            entry.as_str().map(str::to_string).ok_or_else(|| {
                format!(
                    "Retained Node package manifest {} has a non-string workspace entry",
                    display_manifest_path(manifest_path)
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

fn cargo_workspace_members(
    document: &toml::Table,
    manifest_path: &Path,
    access: &mut dyn ManifestAccess,
) -> Result<Option<Vec<String>>, String> {
    let Some(workspace) = document.get("workspace") else {
        return Ok(None);
    };
    let workspace = workspace.as_table().ok_or_else(|| {
        format!(
            "Retained Cargo manifest {} must define `workspace` as a TOML table",
            display_manifest_path(manifest_path)
        )
    })?;
    let Some(members) = workspace.get("members") else {
        return Ok(Some(Vec::new()));
    };
    let members = members.as_array().ok_or_else(|| {
        format!(
            "Retained Cargo manifest {} must define `workspace.members` as a string array",
            display_manifest_path(manifest_path)
        )
    })?;
    access.charge_entries(members.len(), "Cargo workspace declaration")?;
    members
        .iter()
        .map(|member| {
            member.as_str().map(str::to_string).ok_or_else(|| {
                format!(
                    "Retained Cargo manifest {} has a non-string workspace member",
                    display_manifest_path(manifest_path)
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

// ─── pubspec.yaml (Dart/Flutter) ─────────────────────────────────────────

#[cfg(test)]
#[allow(dead_code)]
fn parse_pubspec_yaml(root: &Path) -> Option<ManifestDiscovery> {
    let mut access = AmbientManifestAccess { root };
    parse_pubspec_yaml_with_access(&mut access).ok().flatten()
}

fn parse_pubspec_yaml_with_access(
    access: &mut dyn ManifestAccess,
) -> Result<Option<ManifestDiscovery>, String> {
    let Some(content) = access.read_text(Path::new("pubspec.yaml"), "Dart package manifest")?
    else {
        return Ok(None);
    };
    let mut discovery = ManifestDiscovery::default();

    // Extract name from `name: my_package`
    let name = content
        .lines()
        .find(|l| l.starts_with("name:"))
        .and_then(|l| l.strip_prefix("name:"))
        .map(|n| n.trim().to_string())
        .unwrap_or_else(|| "app".to_string());

    let src_dir = "lib";

    discovery.modules.insert(
        name.clone(),
        ManifestModule {
            name,
            source_paths: vec![src_dir.to_string()],
            dependencies: Vec::new(),
        },
    );
    discovery.source_dirs.push(src_dir.to_string());

    Ok(Some(discovery))
}

// ─── go.mod (Go) ─────────────────────────────────────────────────────────

#[cfg(test)]
fn parse_go_mod(root: &Path) -> Option<ManifestDiscovery> {
    let mut access = AmbientManifestAccess { root };
    parse_go_mod_with_access(&mut access).ok().flatten()
}

fn parse_go_mod_with_access(
    access: &mut dyn ManifestAccess,
) -> Result<Option<ManifestDiscovery>, String> {
    let Some(content) = access.read_text(Path::new("go.mod"), "Go module manifest")? else {
        return Ok(None);
    };
    let mut discovery = ManifestDiscovery::default();

    // Extract module name: `module github.com/user/repo`
    let module_name = content
        .lines()
        .find(|l| l.starts_with("module "))
        .and_then(|l| l.strip_prefix("module "))
        .map(|m| {
            // Use last segment as module name
            m.trim().rsplit('/').next().unwrap_or(m.trim()).to_string()
        })
        .unwrap_or_else(|| "app".to_string());

    // Go projects: scan for directories with .go files as packages
    // Common patterns: cmd/, internal/, pkg/
    let mut source_dirs = Vec::new();
    for dir_name in &["cmd", "internal", "pkg", "api"] {
        if access.directory_exists(Path::new(dir_name), "Go source")? {
            source_dirs.push(dir_name.to_string());
        }
    }

    // If none of the standard dirs exist, use "." (root)
    if source_dirs.is_empty() {
        source_dirs.push(".".to_string());
    }

    discovery.modules.insert(
        module_name.clone(),
        ManifestModule {
            name: module_name,
            source_paths: source_dirs.clone(),
            dependencies: Vec::new(),
        },
    );
    discovery.source_dirs = source_dirs;

    Ok(Some(discovery))
}

// ─── pyproject.toml (Python) ─────────────────────────────────────────────

#[cfg(test)]
#[allow(dead_code)]
fn parse_pyproject_toml(root: &Path) -> Option<ManifestDiscovery> {
    let mut access = AmbientManifestAccess { root };
    parse_pyproject_toml_with_access(&mut access).ok().flatten()
}

fn parse_pyproject_toml_with_access(
    access: &mut dyn ManifestAccess,
) -> Result<Option<ManifestDiscovery>, String> {
    let Some(content) = access.read_text(Path::new("pyproject.toml"), "Python project manifest")?
    else {
        return Ok(None);
    };
    let mut discovery = ManifestDiscovery::default();

    // Try [project] name first, then [tool.poetry] name
    let name = extract_toml_value(&content, "name", Some("[project]"))
        .or_else(|| extract_toml_value(&content, "name", Some("[tool.poetry]")))
        .unwrap_or_else(|| "app".to_string());

    // Check for packages in [tool.setuptools.packages.find]
    let src_dir = if access.directory_exists(Path::new("src"), "Python source")? {
        "src".to_string()
    } else if access.directory_exists(Path::new(&name), "Python source")? {
        name.clone()
    } else {
        ".".to_string()
    };

    discovery.modules.insert(
        name.clone(),
        ManifestModule {
            name,
            source_paths: vec![src_dir.to_string()],
            dependencies: Vec::new(),
        },
    );
    discovery.source_dirs.push(src_dir.to_string());

    Ok(Some(discovery))
}

// ─── TOML Helpers ────────────────────────────────────────────────────────

/// Extract a string value from a TOML key, optionally within a specific section.
fn extract_toml_value(content: &str, key: &str, section: Option<&str>) -> Option<String> {
    let search_content = if let Some(section_header) = section {
        extract_section(content, section_header)?
    } else {
        content.to_string()
    };

    for line in search_content.lines() {
        let line = line.trim();
        if let Some(eq_pos) = line.find('=') {
            let k = line[..eq_pos].trim();
            if k == key {
                let val = line[eq_pos + 1..].trim();
                // Strip quotes
                if val.starts_with('"') && val.ends_with('"') && val.len() >= 2 {
                    return Some(val[1..val.len() - 1].to_string());
                }
                return Some(val.to_string());
            }
        }
    }
    None
}

/// Extract the content of a TOML section (from header to next section or EOF).
fn extract_section(content: &str, header: &str) -> Option<String> {
    let start = content.find(header)?;
    let after = &content[start + header.len()..];
    // Find the next section header
    let end = after.find("\n[").map(|pos| pos + 1).unwrap_or(after.len());
    Some(after[..end].to_string())
}

/// Split TOML content into repeated array-of-table sections (e.g., [[bin]]).
fn split_toml_array_sections(content: &str, header: &str) -> Vec<String> {
    let mut sections = Vec::new();
    let mut search_from = 0;

    while let Some(start) = content[search_from..].find(header) {
        let abs_start = search_from + start + header.len();
        let rest = &content[abs_start..];

        // Find end: next [[...]] or [...]  section
        let end = rest
            .find("\n[[")
            .or_else(|| rest.find("\n["))
            .map(|pos| pos + 1)
            .unwrap_or(rest.len());

        sections.push(rest[..end].to_string());
        search_from = abs_start + end;
    }

    sections
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    struct CountingManifestAccess {
        texts: HashMap<PathBuf, String>,
        directories: HashSet<PathBuf>,
        children: HashMap<PathBuf, Vec<String>>,
        text_reads: HashMap<PathBuf, usize>,
        child_reads: HashMap<PathBuf, usize>,
        entries: usize,
        entry_limit: usize,
    }

    impl CountingManifestAccess {
        fn new(entry_limit: usize) -> Self {
            Self {
                texts: HashMap::new(),
                directories: HashSet::new(),
                children: HashMap::new(),
                text_reads: HashMap::new(),
                child_reads: HashMap::new(),
                entries: 0,
                entry_limit,
            }
        }
    }

    impl ManifestAccess for CountingManifestAccess {
        fn read_text(&mut self, relative: &Path, _label: &str) -> Result<Option<String>, String> {
            *self.text_reads.entry(relative.to_path_buf()).or_default() += 1;
            Ok(self.texts.get(relative).cloned())
        }

        fn directory_exists(&mut self, relative: &Path, _label: &str) -> Result<bool, String> {
            Ok(self.directories.contains(relative))
        }

        fn child_directories(
            &mut self,
            relative: &Path,
            _label: &str,
        ) -> Result<Vec<String>, String> {
            *self.child_reads.entry(relative.to_path_buf()).or_default() += 1;
            let children = self.children.get(relative).cloned().unwrap_or_default();
            self.charge_entries(children.len(), "test child directory")?;
            Ok(children)
        }

        fn charge_entries(&mut self, count: usize, _label: &str) -> Result<(), String> {
            if self.entries.saturating_add(count) > self.entry_limit {
                return Err(format!(
                    "Retained manifest discovery exceeds the {}-entry limit",
                    self.entry_limit
                ));
            }
            self.entries = self.entries.saturating_add(count);
            Ok(())
        }
    }

    #[test]
    fn test_parse_cargo_toml_basic() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("src")).unwrap();
        fs::write(tmp.path().join("src/lib.rs"), "").unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            r#"
[package]
name = "my-crate"
version = "0.1.0"

[dependencies]
serde = "1.0"
regex = "1.0"
"#,
        )
        .unwrap();

        let result = parse_cargo_toml(tmp.path()).unwrap();
        assert!(result.modules.contains_key("my-crate"));
        let module = &result.modules["my-crate"];
        assert_eq!(module.source_paths, vec!["src"]);
        assert!(module.dependencies.contains(&"serde".to_string()));
        assert!(module.dependencies.contains(&"regex".to_string()));
    }

    #[test]
    fn cargo_workspace_traversal_charges_declarations_and_memoizes_completed_members() {
        let mut access = CountingManifestAccess::new(MAX_RETAINED_MANIFEST_ENTRIES);
        access.texts.insert(
            PathBuf::from("Cargo.toml"),
            "[workspace]\nmembers = [\"level-one\", \"level-one\"]\n".to_string(),
        );
        access.texts.insert(
            PathBuf::from("level-one/Cargo.toml"),
            "[package]\nname = \"level-one\"\n[workspace]\nmembers = [\"level-two\", \"level-two\"]\n"
                .to_string(),
        );
        access.texts.insert(
            PathBuf::from("level-one/level-two/Cargo.toml"),
            "[package]\nname = \"level-two\"\n".to_string(),
        );

        let mut active = HashSet::new();
        let mut completed = HashSet::new();
        let discovery =
            parse_cargo_toml_with_access(&mut access, Path::new(""), &mut active, &mut completed)
                .unwrap()
                .unwrap();

        assert!(discovery.modules.contains_key("level-one"));
        assert!(discovery.modules.contains_key("level-two"));
        assert_eq!(access.entries, 4);
        assert_eq!(access.text_reads[Path::new("Cargo.toml")], 1);
        assert_eq!(access.text_reads[Path::new("level-one/Cargo.toml")], 1);
        assert_eq!(
            access.text_reads[Path::new("level-one/level-two/Cargo.toml")],
            1
        );
        assert_eq!(completed.len(), 3);
        assert!(active.is_empty());
    }

    #[test]
    fn cargo_workspace_operational_discovery_supports_multiline_members() {
        let mut access = CountingManifestAccess::new(MAX_RETAINED_MANIFEST_ENTRIES);
        access.texts.insert(
            PathBuf::from("Cargo.toml"),
            r#"
[workspace]
members = [
    "crates/core",
    # Inline comments and trailing commas are valid Cargo TOML.
    "crates/cli",
]
"#
            .to_string(),
        );
        access.texts.insert(
            PathBuf::from("crates/core/Cargo.toml"),
            "[package]\nname = \"core\"\nversion = \"0.1.0\"\n".to_string(),
        );
        access.texts.insert(
            PathBuf::from("crates/cli/Cargo.toml"),
            "[package]\nname = \"cli\"\nversion = \"0.1.0\"\n".to_string(),
        );

        let discovery = parse_cargo_toml_with_access(
            &mut access,
            Path::new(""),
            &mut HashSet::new(),
            &mut HashSet::new(),
        )
        .unwrap()
        .unwrap();

        assert!(discovery.modules.contains_key("core"));
        assert!(discovery.modules.contains_key("cli"));
        assert_eq!(access.entries, 2);
        assert_eq!(access.text_reads[Path::new("crates/core/Cargo.toml")], 1);
        assert_eq!(access.text_reads[Path::new("crates/cli/Cargo.toml")], 1);
    }

    #[test]
    fn cargo_workspace_operational_discovery_rejects_malformed_and_wrong_typed_toml() {
        for (content, expected) in [
            ("[workspace\nmembers = []\n", "as TOML"),
            ("workspace = []\n", "`workspace` as a TOML table"),
            (
                "[workspace]\nmembers = \"crates/core\"\n",
                "`workspace.members` as a string array",
            ),
        ] {
            let mut access = CountingManifestAccess::new(MAX_RETAINED_MANIFEST_ENTRIES);
            access
                .texts
                .insert(PathBuf::from("Cargo.toml"), content.to_string());

            let error = parse_cargo_toml_with_access(
                &mut access,
                Path::new(""),
                &mut HashSet::new(),
                &mut HashSet::new(),
            )
            .unwrap_err();

            assert!(error.contains(expected), "{error}");
        }
    }

    #[test]
    fn cargo_workspace_charges_every_member_before_rejecting_a_non_string() {
        let mut access = CountingManifestAccess::new(MAX_RETAINED_MANIFEST_ENTRIES);
        access.texts.insert(
            PathBuf::from("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/core\", 7]\n".to_string(),
        );

        let error = parse_cargo_toml_with_access(
            &mut access,
            Path::new(""),
            &mut HashSet::new(),
            &mut HashSet::new(),
        )
        .unwrap_err();

        assert!(error.contains("non-string workspace member"), "{error}");
        assert_eq!(access.entries, 2);
    }

    #[test]
    fn retained_entry_budget_accepts_the_limit_and_rejects_limit_plus_one() {
        let project = tempdir().unwrap();
        let retained = Dir::open_ambient_dir(project.path(), ambient_authority()).unwrap();
        let mut access = RetainedManifestAccess::new(&retained);

        access
            .charge_entries(MAX_RETAINED_MANIFEST_ENTRIES, "workspace declarations")
            .unwrap();
        assert_eq!(access.entries, MAX_RETAINED_MANIFEST_ENTRIES);

        let error = access
            .charge_entries(1, "workspace declaration")
            .unwrap_err();
        assert!(
            error.contains(&format!("{MAX_RETAINED_MANIFEST_ENTRIES}-entry limit")),
            "{error}"
        );
        assert_eq!(access.entries, MAX_RETAINED_MANIFEST_ENTRIES);
    }

    #[test]
    fn test_parse_package_swift_basic() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("Sources/MyLib")).unwrap();
        fs::write(
            tmp.path().join("Package.swift"),
            r#"
// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "MyPackage",
    targets: [
        .target(name: "MyLib", dependencies: ["Logging"]),
        .target(name: "MyApp", dependencies: [.target(name: "MyLib")], path: "Sources/App"),
        .testTarget(name: "MyLibTests", dependencies: ["MyLib"]),
    ]
)
"#,
        )
        .unwrap();

        let result = parse_package_swift(tmp.path()).unwrap();
        assert!(result.modules.contains_key("MyLib"));
        assert!(result.modules.contains_key("MyApp"));
        // testTarget should NOT be in modules
        assert!(!result.modules.contains_key("MyLibTests"));

        let mylib = &result.modules["MyLib"];
        assert_eq!(mylib.source_paths, vec!["Sources/MyLib"]);
        assert!(mylib.dependencies.contains(&"Logging".to_string()));

        let myapp = &result.modules["MyApp"];
        assert_eq!(myapp.source_paths, vec!["Sources/App"]);
    }

    #[test]
    fn test_parse_package_json_workspaces() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("packages/core/src")).unwrap();
        fs::create_dir_all(tmp.path().join("packages/web/src")).unwrap();
        fs::write(
            tmp.path().join("packages/core/package.json"),
            r#"{"name": "@app/core"}"#,
        )
        .unwrap();
        fs::write(
            tmp.path().join("packages/web/package.json"),
            r#"{"name": "@app/web"}"#,
        )
        .unwrap();
        fs::create_dir_all(tmp.path().join("src")).unwrap();
        fs::write(
            tmp.path().join("package.json"),
            r#"{"name": "my-app", "workspaces": ["packages/*"]}"#,
        )
        .unwrap();

        let result = parse_package_json(tmp.path()).unwrap();
        assert!(result.modules.contains_key("core"));
        assert!(result.modules.contains_key("web"));
        assert!(
            result
                .source_dirs
                .contains(&"packages/core/src".to_string())
        );
    }

    #[test]
    fn node_workspace_traversal_charges_patterns_and_deduplicates_cached_expansion() {
        let mut access = CountingManifestAccess::new(MAX_RETAINED_MANIFEST_ENTRIES);
        access.texts.insert(
            PathBuf::from("package.json"),
            r#"{"name":"root","workspaces":["packages/*","packages/**"]}"#.to_string(),
        );
        access
            .children
            .insert(PathBuf::from("packages"), vec!["member".to_string()]);
        access.texts.insert(
            PathBuf::from("packages/member/package.json"),
            r#"{"name":"member"}"#.to_string(),
        );
        access
            .directories
            .insert(PathBuf::from("packages/member/src"));

        let discovery = parse_package_json_with_access(&mut access)
            .unwrap()
            .unwrap();

        assert!(discovery.modules.contains_key("member"));
        assert_eq!(access.entries, 3);
        assert_eq!(access.child_reads[Path::new("packages")], 1);
        assert_eq!(
            access.text_reads[Path::new("packages/member/package.json")],
            1
        );
    }

    #[test]
    fn node_workspace_object_form_preserves_yarn_and_pnpm_package_metadata() {
        let mut access = CountingManifestAccess::new(MAX_RETAINED_MANIFEST_ENTRIES);
        access.texts.insert(
            PathBuf::from("package.json"),
            r#"{
                "name": "root",
                "packageManager": "pnpm@9.0.0",
                "workspaces": {
                    "packages": ["packages/*"],
                    "nohoist": ["**/native"]
                }
            }"#
            .to_string(),
        );
        access
            .children
            .insert(PathBuf::from("packages"), vec!["member".to_string()]);
        access.texts.insert(
            PathBuf::from("packages/member/package.json"),
            r#"{"name":"member"}"#.to_string(),
        );

        let discovery = parse_package_json_with_access(&mut access)
            .unwrap()
            .unwrap();

        assert!(discovery.modules.contains_key("member"));
        assert_eq!(access.entries, 2);
    }

    #[test]
    fn node_package_manifests_fail_closed_for_malformed_and_wrong_typed_inputs() {
        for (content, expected) in [
            ("{\"name\":", "as JSON"),
            ("[]", "must contain a JSON object"),
            (
                r#"{"workspaces":"packages/*"}"#,
                "`workspaces` as a string array",
            ),
            (
                r#"{"workspaces":{"packages":"packages/*"}}"#,
                "`workspaces.packages` as a string array",
            ),
            (
                r#"{"workspaces":{"nohoist":[]}}"#,
                "`workspaces.packages` as a string array",
            ),
        ] {
            let mut access = CountingManifestAccess::new(MAX_RETAINED_MANIFEST_ENTRIES);
            access
                .texts
                .insert(PathBuf::from("package.json"), content.to_string());

            let error = parse_package_json_with_access(&mut access).unwrap_err();

            assert!(error.contains(expected), "{error}");
        }
    }

    #[test]
    fn node_workspace_charges_every_entry_before_rejecting_a_non_string() {
        for content in [
            r#"{"workspaces":["packages/*",7]}"#,
            r#"{"workspaces":{"packages":["packages/*",7]}}"#,
        ] {
            let mut access = CountingManifestAccess::new(MAX_RETAINED_MANIFEST_ENTRIES);
            access
                .texts
                .insert(PathBuf::from("package.json"), content.to_string());

            let error = parse_package_json_with_access(&mut access).unwrap_err();

            assert!(error.contains("non-string workspace entry"), "{error}");
            assert_eq!(access.entries, 2);
        }
    }

    #[test]
    fn nested_node_package_manifest_is_strict_json_object_input() {
        let mut access = CountingManifestAccess::new(MAX_RETAINED_MANIFEST_ENTRIES);
        access.texts.insert(
            PathBuf::from("package.json"),
            r#"{"workspaces":["packages/*"]}"#.to_string(),
        );
        access
            .children
            .insert(PathBuf::from("packages"), vec!["member".to_string()]);
        access.texts.insert(
            PathBuf::from("packages/member/package.json"),
            "[]".to_string(),
        );

        let error = parse_package_json_with_access(&mut access).unwrap_err();

        assert!(error.contains("packages/member/package.json"), "{error}");
        assert!(error.contains("must contain a JSON object"), "{error}");
    }

    #[test]
    fn gradle_settings_support_groovy_kotlin_multiline_and_project_dir_overrides() {
        let modules = parse_gradle_settings(
            r#"
include ':groovy:member',
        ':second'
include(
    ":kotlin:member",
    ':vendor:member'
);
project(':vendor:member').projectDir = file('vendor/custom-member');
project(":second").projectDir = new File(rootDir, "modules/second")
"#,
        )
        .unwrap();

        assert_eq!(
            modules,
            vec![
                GradleSettingsModule {
                    name: "groovy/member".to_string(),
                    path: "groovy/member".to_string(),
                },
                GradleSettingsModule {
                    name: "kotlin/member".to_string(),
                    path: "kotlin/member".to_string(),
                },
                GradleSettingsModule {
                    name: "second".to_string(),
                    path: "modules/second".to_string(),
                },
                GradleSettingsModule {
                    name: "vendor/member".to_string(),
                    path: "vendor/custom-member".to_string(),
                },
            ]
        );
    }

    #[test]
    fn gradle_manifest_discovery_rejects_dynamic_include_without_partial_modules() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("src/main/kotlin")).unwrap();
        fs::write(tmp.path().join("build.gradle.kts"), "plugins {}\n").unwrap();
        fs::write(
            tmp.path().join("settings.gradle.kts"),
            r#"include(":safe", dynamicModule)"#,
        )
        .unwrap();

        let error = discover_from_manifests_checked(tmp.path()).unwrap_err();
        assert!(error.contains("Unsupported or dynamic Gradle expression"));
        let compatibility_result = discover_from_manifests(tmp.path());
        assert!(compatibility_result.modules.is_empty());
        assert!(compatibility_result.source_dirs.is_empty());
    }

    #[test]
    fn gradle_settings_reject_unsupported_include_prefixed_workspace_mutators() {
        for settings in [
            r#"includeFlat("../outside")"#,
            r#"includeBuild("../outside")"#,
            r#"includeWorkspace("../outside")"#,
            r#"settings.includeFlat "../outside""#,
        ] {
            let error = parse_gradle_settings(settings).unwrap_err();
            assert!(
                error.contains("Unsupported Gradle workspace mutator"),
                "unexpected include-prefixed mutator error: {error}"
            );
        }

        let modules = parse_gradle_settings(
            r#"
val includeFlat = "documentation"
println(includeFlat)
include(":member")
"#,
        )
        .unwrap();
        assert_eq!(
            modules,
            vec![GradleSettingsModule {
                name: "member".to_string(),
                path: "member".to_string(),
            }]
        );
    }

    #[test]
    fn gradle_settings_only_workspace_discovers_included_modules() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("member/src/main/kotlin")).unwrap();
        fs::write(
            tmp.path().join("settings.gradle.kts"),
            "include(\":member\")\n",
        )
        .unwrap();

        let result = discover_from_manifests_checked(tmp.path()).unwrap();

        assert_eq!(
            result.modules["member"].source_paths,
            vec!["member/src/main/kotlin"]
        );
    }

    #[test]
    fn gradle_settings_only_workspace_fails_closed_when_malformed() {
        let tmp = tempdir().unwrap();
        fs::write(tmp.path().join("settings.gradle"), "include(\":member\"\n").unwrap();

        let error = discover_from_manifests_checked(tmp.path()).unwrap_err();

        assert!(error.contains("Cannot parse Gradle settings manifest"));
        assert!(discover_from_manifests(tmp.path()).modules.is_empty());
    }

    #[test]
    fn gradle_manifest_discovery_rejects_non_regular_oversized_and_non_utf8_manifests() {
        for name in [
            "build.gradle.kts",
            "build.gradle",
            "settings.gradle.kts",
            "settings.gradle",
        ] {
            let directory_manifest = tempdir().unwrap();
            fs::create_dir(directory_manifest.path().join(name)).unwrap();
            let error = discover_from_manifests_checked(directory_manifest.path()).unwrap_err();
            assert!(
                error.contains(name) && error.contains("must be a regular file"),
                "unexpected {name} preflight error: {error}"
            );
        }

        let oversized_manifest = tempdir().unwrap();
        fs::write(
            oversized_manifest.path().join("settings.gradle.kts"),
            vec![b'a'; MAX_GRADLE_MANIFEST_BYTES as usize + 1],
        )
        .unwrap();
        let error = discover_from_manifests_checked(oversized_manifest.path()).unwrap_err();
        assert!(error.contains("exceeds the"));

        let non_utf8_manifest = tempdir().unwrap();
        fs::write(
            non_utf8_manifest.path().join("settings.gradle.kts"),
            [0xff, 0xfe],
        )
        .unwrap();
        let error = discover_from_manifests_checked(non_utf8_manifest.path()).unwrap_err();
        assert!(error.contains("not valid UTF-8"));
    }

    #[test]
    fn gradle_manifest_discovery_preflights_shadowed_groovy_variants() {
        for shadowed in ["build.gradle", "settings.gradle"] {
            let tmp = tempdir().unwrap();
            fs::write(tmp.path().join("build.gradle.kts"), "plugins {}\n").unwrap();
            fs::write(
                tmp.path().join("settings.gradle.kts"),
                "include(\":member\")\n",
            )
            .unwrap();
            fs::create_dir(tmp.path().join(shadowed)).unwrap();

            let error = discover_from_manifests_checked(tmp.path()).unwrap_err();

            assert!(
                error.contains(shadowed) && error.contains("must be a regular file"),
                "unexpected shadowed Gradle manifest error: {error}"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn gradle_manifest_reads_reject_regular_file_replacement_at_every_checkpoint() {
        for target in [
            GradleManifestReadCheckpoint::PreOpen,
            GradleManifestReadCheckpoint::Opened,
            GradleManifestReadCheckpoint::AfterOpen,
            GradleManifestReadCheckpoint::AfterRead,
        ] {
            let tmp = tempdir().unwrap();
            let manifest = tmp.path().join("settings.gradle.kts");
            let replacement = tmp.path().join("replacement.gradle.kts");
            fs::write(&manifest, "include(\":original\")\n").unwrap();
            fs::write(&replacement, "include(\":replacement\")\n").unwrap();
            let root = Dir::open_ambient_dir(tmp.path(), ambient_authority()).unwrap();
            let mut replaced = false;

            let error = gradle_confined_manifest_text_with_checkpoint(
                &root,
                "settings.gradle.kts",
                "settings",
                |checkpoint| {
                    if checkpoint == target && !replaced {
                        fs::rename(&replacement, &manifest).unwrap();
                        replaced = true;
                    }
                },
            )
            .unwrap_err();

            assert!(replaced);
            assert!(
                error.contains("changed during confined open")
                    || error.contains("changed during confined read"),
                "unexpected identity replacement error at {target:?}: {error}"
            );
        }
    }

    #[cfg(windows)]
    #[test]
    fn gradle_manifest_reads_reject_preopen_regular_file_replacement_on_windows() {
        let tmp = tempdir().unwrap();
        let manifest = tmp.path().join("settings.gradle.kts");
        let replacement = tmp.path().join("replacement.gradle.kts");
        fs::write(&manifest, "include(\":original\")\n").unwrap();
        fs::write(&replacement, "include(\":replacement\")\n").unwrap();
        let root = Dir::open_ambient_dir(tmp.path(), ambient_authority()).unwrap();

        let error = gradle_confined_manifest_text_with_checkpoint(
            &root,
            "settings.gradle.kts",
            "settings",
            |checkpoint| {
                if checkpoint == GradleManifestReadCheckpoint::PreOpen {
                    fs::remove_file(&manifest).unwrap();
                    fs::rename(&replacement, &manifest).unwrap();
                }
            },
        )
        .unwrap_err();

        assert!(error.contains("changed during confined open"));
    }

    #[test]
    fn retained_root_manifest_discovery_accepts_matching_capability_and_rejects_mismatch() {
        let project = tempdir().unwrap();
        fs::create_dir_all(project.path().join("member/src/main/kotlin")).unwrap();
        fs::write(
            project.path().join("settings.gradle.kts"),
            "include(\":member\")\n",
        )
        .unwrap();
        let retained = Dir::open_ambient_dir(project.path(), ambient_authority()).unwrap();

        let discovery =
            discover_from_manifests_checked_with_root(project.path(), &retained).unwrap();
        assert_eq!(
            discovery.modules["member"].source_paths,
            vec!["member/src/main/kotlin"]
        );

        let other = tempdir().unwrap();
        let error = discover_from_manifests_checked_with_root(other.path(), &retained).unwrap_err();
        assert!(error.contains("does not match the retained project root"));
    }

    #[cfg(unix)]
    #[test]
    fn retained_non_gradle_manifest_access_ignores_an_ambient_root_replacement() {
        use std::os::unix::fs::symlink;

        let tmp = tempdir().unwrap();
        let root = tmp.path().join("project");
        let original = tmp.path().join("original-project");
        let replacement = tmp.path().join("replacement-project");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(replacement.join("src")).unwrap();
        fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"retained-project\"\n",
        )
        .unwrap();
        fs::write(
            replacement.join("Cargo.toml"),
            "[package]\nname = \"replacement-project\"\n",
        )
        .unwrap();
        let retained = Dir::open_ambient_dir(&root, ambient_authority()).unwrap();
        fs::rename(&root, &original).unwrap();
        symlink(&replacement, &root).unwrap();

        let mut access = RetainedManifestAccess::new(&retained);
        let discovery = parse_cargo_toml_with_access(
            &mut access,
            Path::new(""),
            &mut HashSet::new(),
            &mut HashSet::new(),
        )
        .unwrap()
        .unwrap();

        assert!(discovery.modules.contains_key("retained-project"));
        assert!(!discovery.modules.contains_key("replacement-project"));
    }

    #[cfg(unix)]
    #[test]
    fn retained_non_gradle_manifest_access_rejects_a_symlink_without_disclosing_referent() {
        use std::os::unix::fs::symlink;

        let project = tempdir().unwrap();
        let outside = tempdir().unwrap();
        let sentinel = "RETAINED_MANIFEST_SENTINEL";
        fs::write(
            outside.path().join("Cargo.toml"),
            format!("[package]\nname = \"{sentinel}\"\n"),
        )
        .unwrap();
        symlink(
            outside.path().join("Cargo.toml"),
            project.path().join("Cargo.toml"),
        )
        .unwrap();
        let retained = Dir::open_ambient_dir(project.path(), ambient_authority()).unwrap();

        let error =
            discover_from_manifests_checked_with_root(project.path(), &retained).unwrap_err();

        assert!(error.contains("symlink or reparse point"), "{error}");
        assert!(!error.contains(sentinel), "{error}");
    }

    #[cfg(unix)]
    #[test]
    fn retained_duplicate_cargo_members_reject_a_linked_workspace_without_disclosure() {
        use std::os::unix::fs::symlink;

        let project = tempdir().unwrap();
        let outside = tempdir().unwrap();
        let sentinel = "RETAINED_WORKSPACE_SENTINEL";
        fs::write(
            project.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"member\", \"member\"]\n",
        )
        .unwrap();
        fs::write(
            outside.path().join("Cargo.toml"),
            format!("[package]\nname = \"{sentinel}\"\n"),
        )
        .unwrap();
        symlink(outside.path(), project.path().join("member")).unwrap();
        let retained = Dir::open_ambient_dir(project.path(), ambient_authority()).unwrap();

        let error =
            discover_from_manifests_checked_with_root(project.path(), &retained).unwrap_err();

        assert!(error.contains("symlink or reparse point"), "{error}");
        assert!(!error.contains(sentinel), "{error}");
    }

    #[test]
    fn retained_non_gradle_manifest_access_rejects_non_regular_and_oversized_inputs() {
        let non_regular = tempdir().unwrap();
        fs::create_dir(non_regular.path().join("Cargo.toml")).unwrap();
        let retained = Dir::open_ambient_dir(non_regular.path(), ambient_authority()).unwrap();
        let error =
            discover_from_manifests_checked_with_root(non_regular.path(), &retained).unwrap_err();
        assert!(error.contains("must be a regular file"), "{error}");

        let oversized = tempdir().unwrap();
        let manifest = fs::File::create(oversized.path().join("Cargo.toml")).unwrap();
        manifest
            .set_len(MAX_RETAINED_MANIFEST_BYTES.saturating_add(1))
            .unwrap();
        let retained = Dir::open_ambient_dir(oversized.path(), ambient_authority()).unwrap();
        let error =
            discover_from_manifests_checked_with_root(oversized.path(), &retained).unwrap_err();
        assert!(error.contains("exceeds the"), "{error}");
        assert!(error.contains("byte limit"), "{error}");
    }

    #[test]
    fn retained_nested_manifest_read_rejects_a_replaced_parent_directory() {
        let project = tempdir().unwrap();
        let workspace = project.path().join("workspace");
        let original = project.path().join("original-workspace");
        let replacement = project.path().join("replacement-workspace");
        fs::create_dir(&workspace).unwrap();
        fs::create_dir(&replacement).unwrap();
        fs::write(
            workspace.join("Cargo.toml"),
            "[package]\nname = \"retained\"\n",
        )
        .unwrap();
        let sentinel = "REPLACEMENT_MANIFEST_SENTINEL";
        fs::write(
            replacement.join("Cargo.toml"),
            format!("[package]\nname = \"{sentinel}\"\n"),
        )
        .unwrap();
        let retained = Dir::open_ambient_dir(project.path(), ambient_authority()).unwrap();

        let error = read_retained_manifest_text_with_hook(
            &retained,
            Path::new("workspace/Cargo.toml"),
            "Cargo manifest",
            MAX_RETAINED_MANIFEST_BYTES,
            MAX_RETAINED_MANIFEST_INPUT_BYTES,
            || {
                fs::rename(&workspace, &original).unwrap();
                fs::rename(&replacement, &workspace).unwrap();
            },
        )
        .unwrap_err();

        assert!(error.contains("directory workspace changed"), "{error}");
        assert!(!error.contains(sentinel), "{error}");
    }

    fn assert_retained_child_enumeration_rejects_nested_directory_replacement(
        relative: &Path,
        label: &str,
    ) {
        let project = tempdir().unwrap();
        let workspace = project.path().join(relative);
        let detached = project.path().join("detached-workspace");
        let replacement = project.path().join("replacement-workspace");
        fs::create_dir_all(workspace.join("original-member")).unwrap();
        fs::create_dir_all(replacement.join("replacement-member")).unwrap();
        let retained = Dir::open_ambient_dir(project.path(), ambient_authority()).unwrap();
        let mut access = RetainedManifestAccess::new(&retained);

        let error = access
            .child_directories_with_hook(relative, label, || {
                fs::rename(&workspace, &detached).unwrap();
                fs::rename(&replacement, &workspace).unwrap();
            })
            .unwrap_err();

        assert!(
            error.contains(&format!(
                "Retained {label} directory {} changed during confined read",
                relative.display()
            )),
            "{error}"
        );
        assert!(!access.children.contains_key(relative));
    }

    #[test]
    fn retained_cargo_child_enumeration_rejects_nested_regular_directory_replacement() {
        assert_retained_child_enumeration_rejects_nested_directory_replacement(
            Path::new("nested/crates"),
            "Cargo workspace",
        );
    }

    #[test]
    fn retained_node_child_enumeration_rejects_nested_regular_directory_replacement() {
        assert_retained_child_enumeration_rejects_nested_directory_replacement(
            Path::new("nested/packages"),
            "Node workspace",
        );
    }

    #[test]
    fn retained_node_workspace_rejects_replacement_after_enumeration_before_child_consumption() {
        let project = tempdir().unwrap();
        let workspace = project.path().join("packages");
        let detached = project.path().join("detached-packages");
        let replacement = project.path().join("replacement-packages");
        fs::create_dir_all(workspace.join("member/src")).unwrap();
        fs::create_dir_all(replacement.join("member/src")).unwrap();
        fs::write(
            project.path().join("package.json"),
            r#"{"name":"root","workspaces":["packages/*"]}"#,
        )
        .unwrap();
        fs::write(
            workspace.join("member/package.json"),
            r#"{"name":"original"}"#,
        )
        .unwrap();
        fs::write(
            replacement.join("member/package.json"),
            r#"{"name":"replacement"}"#,
        )
        .unwrap();
        let retained = Dir::open_ambient_dir(project.path(), ambient_authority()).unwrap();
        let mut access = RetainedManifestAccess::new(&retained);

        let error = parse_package_json_with_access_and_hook(&mut access, |relative| {
            assert_eq!(relative, Path::new("packages"));
            fs::rename(&workspace, &detached).unwrap();
            fs::rename(&replacement, &workspace).unwrap();
        })
        .unwrap_err();

        assert!(
            error.contains("Retained Node workspace directory packages"),
            "{error}"
        );
    }

    #[test]
    fn retained_node_workspace_swap_read_restore_uses_enumerated_child_capability() {
        let project = tempdir().unwrap();
        let workspace = project.path().join("packages");
        let detached = project.path().join("detached-packages");
        let replacement = project.path().join("replacement-packages");
        fs::create_dir_all(workspace.join("member/src")).unwrap();
        fs::create_dir_all(replacement.join("member")).unwrap();
        fs::write(
            project.path().join("package.json"),
            r#"{"name":"root","workspaces":["packages/*"]}"#,
        )
        .unwrap();
        fs::write(
            workspace.join("member/package.json"),
            r#"{"name":"original"}"#,
        )
        .unwrap();
        fs::write(
            replacement.join("member/package.json"),
            r#"{"name":"replacement","workspaces":"SWAP_READ_RESTORE_SENTINEL"}"#,
        )
        .unwrap();
        let retained = Dir::open_ambient_dir(project.path(), ambient_authority()).unwrap();
        let mut access = RetainedManifestAccess::new(&retained);

        let discovery = parse_package_json_with_access_and_hooks(
            &mut access,
            |relative| {
                assert_eq!(relative, Path::new("packages"));
                fs::rename(&workspace, &detached).unwrap();
                fs::rename(&replacement, &workspace).unwrap();
            },
            |relative, child| {
                assert_eq!(relative, Path::new("packages"));
                assert_eq!(child, "member");
                fs::rename(&workspace, &replacement).unwrap();
                fs::rename(&detached, &workspace).unwrap();
            },
        )
        .unwrap()
        .unwrap();

        assert!(discovery.modules.contains_key("member"));
        assert!(
            discovery
                .source_dirs
                .contains(&"packages/member/src".to_string())
        );
        assert_eq!(
            access
                .texts
                .get(Path::new("packages/member/package.json"))
                .and_then(Option::as_deref),
            Some(r#"{"name":"original"}"#)
        );
    }

    #[cfg(unix)]
    #[test]
    fn retained_node_workspace_enumeration_bounds_open_directory_handles() {
        const CHILD_ENV: &str = "SPECSYNC_MANIFEST_BOUNDED_HANDLES_CHILD";
        if std::env::var_os(CHILD_ENV).is_none() {
            let executable = std::env::current_exe().unwrap();
            let status = std::process::Command::new("sh")
                .args([
                    "-c",
                    "ulimit -n 64; exec \"$1\" retained_node_workspace_enumeration_bounds_open_directory_handles --nocapture",
                    "sh",
                ])
                .arg(executable)
                .env(CHILD_ENV, "1")
                .status()
                .unwrap();
            assert!(
                status.success(),
                "Node workspace discovery failed with a 64-descriptor soft limit"
            );
            return;
        }

        let project = tempdir().unwrap();
        fs::create_dir_all(project.path().join("packages")).unwrap();
        fs::write(
            project.path().join("package.json"),
            r#"{"name":"root","workspaces":["packages/*"]}"#,
        )
        .unwrap();
        for index in 0..200 {
            let workspace = project.path().join(format!("packages/member-{index:03}"));
            fs::create_dir_all(workspace.join("src")).unwrap();
            fs::write(
                workspace.join("package.json"),
                format!(r#"{{"name":"member-{index:03}"}}"#),
            )
            .unwrap();
        }
        let retained = Dir::open_ambient_dir(project.path(), ambient_authority()).unwrap();

        let discovery = discover_from_manifests_checked_with_root(project.path(), &retained)
            .expect("broad retained Node workspace discovery must stay descriptor-bounded");

        assert_eq!(discovery.modules.len(), 200);
    }

    #[cfg(unix)]
    #[test]
    fn retained_node_workspace_bases_bound_open_directory_handles() {
        const CHILD_ENV: &str = "SPECSYNC_MANIFEST_BOUNDED_BASE_HANDLES_CHILD";
        if std::env::var_os(CHILD_ENV).is_none() {
            let executable = std::env::current_exe().unwrap();
            let status = std::process::Command::new("sh")
                .args([
                    "-c",
                    "ulimit -n 64; exec \"$1\" retained_node_workspace_bases_bound_open_directory_handles --nocapture",
                    "sh",
                ])
                .arg(executable)
                .env(CHILD_ENV, "1")
                .status()
                .unwrap();
            assert!(
                status.success(),
                "Node workspace discovery retained handles across distinct bases"
            );
            return;
        }

        let project = tempdir().unwrap();
        let mut patterns = Vec::new();
        for index in 0..90 {
            let base = format!("base-{index:03}");
            let member = format!("member-{index:03}");
            patterns.push(format!(r#""{base}/*""#));
            let workspace = project.path().join(&base).join(&member);
            fs::create_dir_all(workspace.join("src")).unwrap();
            fs::write(
                workspace.join("package.json"),
                format!(r#"{{"name":"{member}"}}"#),
            )
            .unwrap();
        }
        fs::write(
            project.path().join("package.json"),
            format!(r#"{{"name":"root","workspaces":[{}]}}"#, patterns.join(",")),
        )
        .unwrap();
        let retained = Dir::open_ambient_dir(project.path(), ambient_authority()).unwrap();

        let discovery = discover_from_manifests_checked_with_root(project.path(), &retained)
            .expect("distinct retained Node workspace bases must stay descriptor-bounded");

        assert_eq!(discovery.modules.len(), 90);
    }

    #[test]
    fn gradle_settings_reject_unsupported_project_dir_bases_and_suffixes() {
        let outside_base = parse_gradle_settings(
            r#"
include(":outside")
project(":outside").projectDir = new File(rootDir.parentFile, "outside")
"#,
        )
        .unwrap_err();
        assert!(outside_base.contains("base must be exactly rootDir"));

        let project_root = parse_gradle_settings(
            "include(\":root\")\nproject(\":root\").projectDir = file(\".\")\n",
        )
        .unwrap();
        assert_eq!(project_root[0].path, ".");

        for assignment in [
            r#"file("modules/safe").parentFile"#,
            r#"new File(rootDir, "modules/safe") + "/outside""#,
        ] {
            let settings =
                format!("include(\":safe\")\nproject(\":safe\").projectDir = {assignment}\n");
            let error = parse_gradle_settings(&settings).unwrap_err();
            assert!(error.contains("Unsupported trailing Gradle projectDir assignment expression"));
        }
    }

    #[test]
    fn gradle_settings_reject_project_root_escapes() {
        for settings in [
            r#"
include(":outside")
project(":outside").projectDir = file("../outside")
"#,
            r#"
include(":outside")
project(":outside").setProjectDir(file("../outside"))
"#,
            r#"include(":..:outside")"#,
            r#"include("C:/outside")"#,
            r#"include("C:outside")"#,
            r#"include(":C:\\outside")"#,
            r#"
include(":outside")
project(":outside").projectDir = file("safe/../../outside")
"#,
            r#"
include(":outside")
project(":outside").projectDir = file("C:\\outside")
"#,
            r#"
include(":outside")
project(":outside").projectDir = file("\\\\server\\share")
"#,
            r#"
include(":safe")
project("C:/outside").projectDir = file("modules/safe")
"#,
            r#"
include(":safe")
project("C:outside").projectDir = file("modules/safe")
"#,
            r#"
include(":outside")
project(":outside").setProjectDir(file("\u002e\u002e/outside"))
"#,
            r#"
include(":outside")
project(":outside").setProjectDir(file("\056\056/outside"))
"#,
        ] {
            let error = parse_gradle_settings(settings).unwrap_err();
            assert!(
                error.contains("must remain beneath the project root"),
                "unexpected Gradle confinement error: {error}"
            );
        }
    }

    #[test]
    fn gradle_settings_preserve_rooted_nested_names_that_resemble_drive_relative_paths() {
        let modules = parse_gradle_settings(
            r#"
include(":C:member")
project(":C:member").projectDir = file("modules/member")
"#,
        )
        .unwrap();
        assert_eq!(
            modules,
            vec![GradleSettingsModule {
                name: "C/member".to_string(),
                path: "modules/member".to_string(),
            }]
        );
    }

    #[test]
    fn gradle_settings_support_literal_set_project_dir_forms() {
        let modules = parse_gradle_settings(
            r#"
include(":first", ":second")
project(":first").setProjectDir(file("modules/first"))
project(":second").setProjectDir(new File(rootDir, "modules/second"))
"#,
        )
        .unwrap();

        assert_eq!(
            modules,
            vec![
                GradleSettingsModule {
                    name: "first".to_string(),
                    path: "modules/first".to_string(),
                },
                GradleSettingsModule {
                    name: "second".to_string(),
                    path: "modules/second".to_string(),
                },
            ]
        );
    }

    #[test]
    fn gradle_settings_reject_dynamic_or_ambiguous_set_project_dir_forms() {
        for setter in [
            r#"project(":member").setProjectDir(projectDirProvider.get())"#,
            r#"project(":member").setProjectDir(file("modules/member"), file("other"))"#,
            r#"project(":member").setProjectDir(file("modules/member")).parentFile"#,
            r#"project(":member").setProjectDir(file("$outside"))"#,
            r#"project(":member").setProjectDir(file("${outside}"))"#,
            r#"project(":member").projectDir = file("$outside")"#,
            r#"project(":member").projectDir = file("\u0024outside")"#,
            r#"project(":member").projectDir = file("\044outside")"#,
            r#"project(":member").projectDir = newFile(rootDir, "../outside")"#,
            r#"project(":member").setProjectDir(newFile(rootDir, "../outside"))"#,
        ] {
            let settings = format!("include(\":member\")\n{setter}\n");
            let error = parse_gradle_settings(&settings).unwrap_err();
            assert!(
                error.contains("Unsupported Gradle projectDir assignment")
                    || error.contains("Unsupported or dynamic Gradle expression")
                    || error.contains("must contain one path")
                    || error.contains("Unsupported trailing Gradle projectDir assignment"),
                "unexpected setProjectDir parser error: {error}"
            );
        }
    }

    #[test]
    fn gradle_settings_reject_dynamic_and_unrecognized_project_mutations() {
        for mutation in [
            r#"project(":member").setProperty("projectDir", file("../outside"))"#,
            r#"val changed = project(":member").setProperty("projectDir", file("../outside"))"#,
            r#"project(":member")["projectDir"] = file("../outside")"#,
            r#"project(":member").properties["projectDir"] = file("../outside")"#,
            "project(\":member\")[\n    \"projectDir\"\n]\n    = file(\"../outside\")",
            r#"project(":member").configure { projectDir = file("../outside") }"#,
            r#"project(":member").customDirectory = file("../outside")"#,
            r#"if (enabled) project(":member").customDirectory = file("../outside")"#,
        ] {
            let settings = format!("include(\":member\")\n{mutation}\n");
            let error = parse_gradle_settings(&settings).unwrap_err();
            assert!(
                error.contains("Unsupported")
                    && (error.contains("project mutation")
                        || error.contains("projectDir mutation")),
                "unexpected executable project mutation error: {error}"
            );
        }

        let modules = parse_gradle_settings(
            r#"
include(":member")
project(":member").projectDir
project(":member")["projectDir"] == file("modules/member")
project(":member").properties["projectDir"]
val unchanged = project(":member").projectDir == file("modules/member")
"#,
        )
        .unwrap();
        assert_eq!(
            modules,
            vec![GradleSettingsModule {
                name: "member".to_string(),
                path: "member".to_string(),
            }]
        );
    }

    #[test]
    fn gradle_directive_control_flow_validation_is_local() {
        let modules = parse_gradle_settings(
            r#"
if (enabled) println("unrelated")
for (entry in entries) {
    println(entry)
}
include(":member")
project(":member").projectDir = file("modules/member")
"#,
        )
        .unwrap();
        assert_eq!(
            modules,
            vec![GradleSettingsModule {
                name: "member".to_string(),
                path: "modules/member".to_string(),
            }]
        );

        for settings in [
            r#"if (enabled) include(":member")"#,
            "if (\n    enabled\n)\ninclude(\":member\")",
            "if (enabled) {\ninclude(\":member\")\n}",
            "if (\n    enabled\n)\nproject(\":member\").projectDir = file(\"modules/member\")",
            "if (enabled) {\nproject(\":member\").setProjectDir(file(\"modules/member\"))\n}",
        ] {
            let error = parse_gradle_settings(settings).unwrap_err();
            assert!(
                error.contains("Unsupported")
                    && (error.contains("conditional") || error.contains("block-scoped")),
                "unexpected directive-local control-flow error: {error}"
            );
        }
    }

    #[test]
    fn gradle_settings_reject_indirect_or_conditional_mutations_but_allow_reads() {
        for settings in [
            r#"
include(":member")
val member = project(":member")
member.setProjectDir(file("../outside"))
"#,
            r#"
include(":member")
val member = project(":member")
member.projectDir = file("../outside")
"#,
            r#"if (enabled) include(":member")"#,
            "if (enabled)\ninclude(\":member\")",
            "if (enabled) {\ninclude(\":member\")\n}",
            r#"settings.include(":member")"#,
            r#"
include(":member")
if (enabled) project(":member").projectDir = file("modules/member")
"#,
            r#"
include(":member")
project(":member").projectDir += file("modules/member")
"#,
            r#"
include(":member")
project(":member") {
    projectDir = file("../outside")
}
"#,
            r#"
include(":member")
project(":member") {
    setProjectDir(file("../outside"))
}
"#,
            r#"
include(":member")
project(":member") . setProjectDir(file("../outside"))
"#,
            r#"
include(":member")
if (enabled) {
    project(":member").projectDir = file("modules/member")
}
"#,
            r#"include(""":member""")"#,
        ] {
            let error = parse_gradle_settings(settings).unwrap_err();
            assert!(
                error.contains("Unsupported") || error.contains("must contain a literal module"),
                "unexpected indirect Gradle mutation error: {error}"
            );
        }

        let modules = parse_gradle_settings(
            r#"
include(":member")
println(project(":member").projectDir)
val unchanged = project(":member").projectDir == file("modules/member")
"#,
        )
        .unwrap();
        assert_eq!(
            modules,
            vec![GradleSettingsModule {
                name: "member".to_string(),
                path: "member".to_string(),
            }]
        );
    }

    #[test]
    fn gradle_settings_ignore_multiline_literals_and_nested_comments() {
        let modules = parse_gradle_settings(
            r#"
val kotlinDocumentation = """
include(":kotlin-phantom")
project(":real").projectDir = file("../outside")
"""
def groovyDocumentation = '''
include(":groovy-phantom")
project(":real").setProjectDir(file("../outside"))
'''
/*
  include(":outer-comment-phantom")
  /* include(":nested-comment-phantom") */
  project(":real").projectDir = file("../outside")
*/
val regularDocumentation = ".projectDir include(\":quoted-phantom\")"
include(":real")
"#,
        )
        .unwrap();

        assert_eq!(
            modules,
            vec![GradleSettingsModule {
                name: "real".to_string(),
                path: "real".to_string(),
            }]
        );
    }

    #[test]
    fn gradle_settings_reject_interpolated_includes_without_partial_modules() {
        for include in [r#"include(":$module")"#, r#"include(":${module}")"#] {
            let error = parse_gradle_settings(include).unwrap_err();
            assert!(
                error.contains("Unsupported or dynamic Gradle expression"),
                "unexpected interpolated include error: {error}"
            );
        }
    }

    #[test]
    fn gradle_settings_preserve_literal_dollars() {
        let modules = parse_gradle_settings(
            r#"
include(':literal$module', ":escaped\$module")
project(':literal$module').setProjectDir(file('modules/$literal'))
project(":escaped\$module").setProjectDir(file("modules/escaped\$literal"))
"#,
        )
        .unwrap();

        assert_eq!(
            modules,
            vec![
                GradleSettingsModule {
                    name: "escaped$module".to_string(),
                    path: "modules/escaped$literal".to_string(),
                },
                GradleSettingsModule {
                    name: "literal$module".to_string(),
                    path: "modules/$literal".to_string(),
                },
            ]
        );
    }

    #[test]
    fn gradle_manifest_discovery_rejects_project_root_escape_without_partial_modules() {
        let tmp = tempdir().unwrap();
        let project = tmp.path().join("project");
        let outside = tmp.path().join("outside/src/main/kotlin");
        fs::create_dir_all(&project).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(
            project.join("settings.gradle.kts"),
            "include(\":outside\")\nproject(\":outside\").projectDir = file(\"../outside\")\n",
        )
        .unwrap();

        let error = discover_from_manifests_checked(&project).unwrap_err();
        assert!(error.contains("must remain beneath the project root"));
        let compatibility_result = discover_from_manifests(&project);
        assert!(compatibility_result.modules.is_empty());
        assert!(compatibility_result.source_dirs.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn gradle_manifest_discovery_rejects_symlinked_module_directories() {
        use std::os::unix::fs::symlink;

        let tmp = tempdir().unwrap();
        let project = tmp.path().join("project");
        let outside = tmp.path().join("outside");
        fs::create_dir_all(&project).unwrap();
        fs::create_dir_all(outside.join("src/main/kotlin")).unwrap();
        symlink(&outside, project.join("linked")).unwrap();
        fs::write(
            project.join("settings.gradle.kts"),
            "include(\":linked\")\n",
        )
        .unwrap();

        let error = discover_from_manifests_checked(&project).unwrap_err();
        assert!(
            error.contains("must not traverse a symlink or reparse point"),
            "unexpected Gradle symlink rejection: {error}"
        );
    }

    #[test]
    fn gradle_manifest_discovery_uses_the_effective_project_directory() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("vendor/custom/src/main/kotlin")).unwrap();
        fs::write(tmp.path().join("build.gradle"), "plugins {}\n").unwrap();
        fs::write(
            tmp.path().join("settings.gradle"),
            "include ':member'\nproject(':member').projectDir = file('vendor/custom')\n",
        )
        .unwrap();

        let result = parse_gradle(tmp.path()).unwrap();
        assert_eq!(
            result.modules["member"].source_paths,
            vec!["vendor/custom/src/main/kotlin"]
        );
        assert!(
            result
                .source_dirs
                .contains(&"vendor/custom/src/main/kotlin".to_string())
        );
    }

    #[test]
    fn gradle_settings_ignore_comments_and_decode_escaped_values() {
        let commented = parse_gradle_settings("include ':member' // \"comment\n").unwrap();
        assert_eq!(commented[0].name, "member");

        let modules = parse_gradle_settings(
            r#"
include(
    ":member", // ignored quote: "
    ':quoted\'member',
    ":slash//member" /* ignored quote: ' */
)
project(":member").projectDir = file("modules\\member")
"#,
        )
        .unwrap();

        assert_eq!(modules[0].name, "member");
        assert_eq!(modules[0].path, "modules/member");
        assert_eq!(modules[1].name, "quoted'member");
        assert_eq!(modules[2].name, "slash//member");
    }

    #[test]
    fn gradle_manifest_discovery_fails_closed_for_malformed_settings() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("src/main/kotlin")).unwrap();
        fs::write(tmp.path().join("build.gradle.kts"), "plugins {}\n").unwrap();
        fs::write(
            tmp.path().join("settings.gradle.kts"),
            "include(\":member\"\n",
        )
        .unwrap();

        let error = discover_from_manifests_checked(tmp.path()).unwrap_err();
        assert!(error.contains("Cannot parse Gradle settings manifest"));
        let result = discover_from_manifests(tmp.path());
        assert!(result.modules.is_empty());
        assert!(result.source_dirs.is_empty());
    }

    #[test]
    fn gradle_manifest_discovery_accepts_comments_and_escaped_paths() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("modules/member/src/main/kotlin")).unwrap();
        fs::write(tmp.path().join("build.gradle.kts"), "plugins {}\n").unwrap();
        fs::write(
            tmp.path().join("settings.gradle.kts"),
            r#"include(":member") // ignored unterminated quote: "
project(":member").projectDir = file("modules\\member") /* ignored: ' */
"#,
        )
        .unwrap();

        let result = discover_from_manifests(tmp.path());
        assert_eq!(
            result.modules["member"].source_paths,
            vec!["modules/member/src/main/kotlin"]
        );
    }

    #[test]
    fn test_parse_go_mod() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("cmd")).unwrap();
        fs::create_dir_all(tmp.path().join("internal")).unwrap();
        fs::write(
            tmp.path().join("go.mod"),
            "module github.com/user/myproject\n\ngo 1.21\n",
        )
        .unwrap();

        let result = parse_go_mod(tmp.path()).unwrap();
        assert!(result.modules.contains_key("myproject"));
        assert!(result.source_dirs.contains(&"cmd".to_string()));
        assert!(result.source_dirs.contains(&"internal".to_string()));
    }

    #[test]
    fn test_extract_balanced_parens() {
        assert_eq!(
            extract_balanced_parens("name: \"Foo\", path: \"bar\")"),
            Some("name: \"Foo\", path: \"bar\"".to_string())
        );
        assert_eq!(
            extract_balanced_parens("a(b), c)"),
            Some("a(b), c".to_string())
        );
        assert_eq!(extract_balanced_parens("no close paren"), None);
    }
}
