//! Manifest-aware module detection.
//!
//! Parses language-specific manifest files (Package.swift, Cargo.toml,
//! build.gradle.kts, package.json, etc.) to discover targets, source paths,
//! and module names instead of relying on directory scanning alone.

use cap_std::ambient_authority;
use cap_std::fs::Dir;
use std::collections::{HashMap, HashSet};
use std::path::Path;

mod access;
mod cargo;
mod gradle;
mod identity;
mod node;
mod simple_manifests;
mod swift;

use access::RetainedManifestAccess;
use cargo::parse_cargo_toml_with_access;
use gradle::parse_gradle_checked_with_root;
use identity::verify_retained_project_root;
use node::parse_package_json_with_access;
use simple_manifests::{
    parse_go_mod_with_access, parse_pubspec_yaml_with_access, parse_pyproject_toml_with_access,
};
use swift::parse_package_swift_with_access;

pub(crate) const MAX_GRADLE_MANIFEST_BYTES: u64 = 4 * 1024 * 1024;

/// A module discovered from a manifest file.
#[derive(Debug, Clone)]
pub struct ManifestModule {
    /// Module/target name.
    pub name: String,
    /// Source paths relative to project root. Coverage reads these to decide
    /// whether a manifest module's files are already mapped by some spec,
    /// whatever that spec is named.
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
    if let Some(d) = parse_gradle_checked_with_root(root, project_root)? {
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

/// Parse Groovy or Kotlin Gradle settings into effective module directories.
///
/// Supports parenthesized and bare `include` forms, single or double quotes,
/// multiline declarations, nested `:module:name` paths, and the common
/// `project(...).projectDir = ...` and `project(...).setProjectDir(...)`
/// overrides with `file(...)` / `new File(rootDir, ...)` path expressions.
pub(crate) fn parse_gradle_settings(content: &str) -> Result<Vec<GradleSettingsModule>, String> {
    gradle::parse_gradle_settings(content)
}

/// True when `source_dir` is a JVM language root (`src/main/kotlin`, …).
/// Immediate children of that root are package segments, not modules.
pub(crate) fn is_jvm_package_source_root(source_dir: &str) -> bool {
    gradle::is_jvm_package_source_root(source_dir)
}

#[cfg(test)]
mod test_support;
