//! Manifest-aware and scan-based source discovery.

use crate::exports::has_extension;
use crate::manifest;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Directories that should never be treated as source directories.
const IGNORED_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    ".hg",
    ".svn",
    "dist",
    "build",
    "out",
    "target",
    "vendor",
    ".next",
    ".nuxt",
    ".output",
    ".cache",
    ".turbo",
    "coverage",
    "__pycache__",
    ".mypy_cache",
    ".pytest_cache",
    ".tox",
    ".venv",
    "venv",
    "env",
    ".env",
    ".idea",
    ".vscode",
    ".DS_Store",
    "specs",
    "docs",
    "doc",
    ".github",
    ".gitlab",
    "migrations",
    "Pods",
    ".dart_tool",
    ".gradle",
    "bin",
    "obj",
];
const DEFAULT_STATIC_SOURCE_EXTENSIONS: &[&str] = &["html", "htm", "css"];

pub(crate) fn source_detection_ignores_directory(name: &str) -> bool {
    name.starts_with('.') || IGNORED_DIRS.contains(&name)
}

/// Auto-detect source directories by first checking manifest files
/// (Cargo.toml, Package.swift, build.gradle.kts, package.json, etc.),
/// then falling back to scanning the project root for files with supported
/// language extensions. Returns directories relative to root.
pub fn detect_source_dirs(root: &Path) -> Vec<String> {
    detect_source_dirs_checked(root).unwrap_or_else(|_| detect_source_dirs_by_scan(root))
}

/// Auto-detect source directories while surfacing malformed manifest inputs.
pub fn detect_source_dirs_checked(root: &Path) -> Result<Vec<String>, String> {
    // Try manifest-aware detection first
    let manifest_discovery = manifest::discover_from_manifests_checked(root)?;
    if !manifest_discovery.source_dirs.is_empty() {
        let mut dirs = manifest_discovery.source_dirs;
        dirs.sort();
        dirs.dedup();
        return Ok(dirs);
    }

    // Fall back to directory scanning
    Ok(detect_source_dirs_by_scan(root))
}

/// Discover modules from manifest files (Package.swift, Cargo.toml, etc.).
/// Returns the manifest discovery result for use in module detection.
#[allow(dead_code)]
pub fn discover_manifest_modules(root: &Path) -> manifest::ManifestDiscovery {
    manifest::discover_from_manifests(root)
}

/// Discover manifest modules while surfacing malformed manifest inputs.
#[allow(dead_code)]
pub fn discover_manifest_modules_checked(
    root: &Path,
) -> Result<manifest::ManifestDiscovery, String> {
    manifest::discover_from_manifests_checked(root)
}

/// Scan-based source directory detection (fallback when no manifests found).
fn detect_source_dirs_by_scan(root: &Path) -> Vec<String> {
    let detected = scan_source_dirs(root);
    if detected.is_empty() {
        // Fallback to "src" if nothing detected
        return vec!["src".to_string()];
    }
    detected
}

/// Like [`detect_source_dirs`], but also reports whether any source
/// directories were actually detected. `false` means the `["src"]` fallback
/// was used — callers (e.g. `init`) must not present it as "detected".
pub(crate) fn detect_source_dirs_with_confidence(root: &Path) -> (Vec<String>, bool) {
    // Manifest-aware detection first
    let manifest_discovery = manifest::discover_from_manifests(root);
    if !manifest_discovery.source_dirs.is_empty() {
        let mut dirs = manifest_discovery.source_dirs;
        dirs.sort();
        dirs.dedup();
        return (dirs, true);
    }

    let scanned = scan_source_dirs(root);
    if scanned.is_empty() {
        (vec!["src".to_string()], false)
    } else {
        (scanned, true)
    }
}

/// Raw directory scan with no fallback — returns an empty vector when no
/// source directories or root-level source files are found.
fn scan_source_dirs(root: &Path) -> Vec<String> {
    let ignored: HashSet<&str> = IGNORED_DIRS.iter().copied().collect();
    let mut source_dirs: Vec<String> = Vec::new();
    let mut has_root_source_files = false;

    // Check immediate children of root
    let entries = match fs::read_dir(root) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden dirs and ignored dirs
        if source_detection_ignores_directory(&name) {
            continue;
        }

        let path = entry.path();

        if path.is_dir() {
            // Check if this directory contains any source files (scan up to 3 levels deep)
            if dir_contains_source_files(&path, &ignored, 3) {
                source_dirs.push(name);
            }
        } else if path.is_file() && is_detectable_source_file(&path) {
            // Source file directly in root
            has_root_source_files = true;
        }
    }

    // If source files exist directly in root, add "." as a source dir
    if has_root_source_files && source_dirs.is_empty() {
        return vec![".".to_string()];
    }

    source_dirs.sort();
    source_dirs
}
/// Check if a directory contains source files, scanning up to `max_depth` levels.
fn dir_contains_source_files(dir: &Path, ignored: &HashSet<&str>, max_depth: usize) -> bool {
    for entry in WalkDir::new(dir)
        .max_depth(max_depth)
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_str().unwrap_or("");
                !name.starts_with('.') && !ignored.contains(name)
            } else {
                true
            }
        })
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && is_detectable_source_file(path) {
            return true;
        }
    }
    false
}

pub(crate) fn is_detectable_source_file(path: &Path) -> bool {
    has_extension(path, &[])
        || path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| DEFAULT_STATIC_SOURCE_EXTENSIONS.contains(&extension))
}

#[cfg(test)]
mod tests;
