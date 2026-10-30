//! Manifest and autodetection confinement checked before a snapshot is used.

use super::confinement::{
    MAX_CONFINEMENT_ENTRIES, read_file_bounded, validate_existing_path, validate_path_or_ancestor,
};
use super::snapshot_manifest::{
    normalize_snapshot_input, package_workspace_declarations, parse_cargo_snapshot_manifest,
};
use crate::manifest::parse_gradle_settings;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use walkdir::WalkDir;

pub(super) const MAX_MANIFEST_PREFLIGHTS: usize = 1_000;

pub(super) const AUTODETECT_IGNORED_DIRS: &[&str] = &[
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

pub(super) fn validate_manifest_inputs(root: &Path) -> Result<(), String> {
    let mut visiting = HashSet::new();
    let mut validated = HashSet::new();
    let mut manifests_seen = 0usize;
    let mut traversal_entries_seen = 0usize;
    validate_cargo_workspace_manifest(
        root,
        root,
        &mut visiting,
        &mut validated,
        &mut manifests_seen,
        &mut traversal_entries_seen,
    )?;
    validate_package_workspaces(root, &mut traversal_entries_seen)?;
    validate_gradle_modules(root)?;
    validate_python_package_path(root)?;
    Ok(())
}

pub(super) fn validate_autodetection_tree(root: &Path) -> Result<(), String> {
    let canonical_root = root.canonicalize().map_err(|error| {
        format!(
            "Cannot resolve MCP server root {} during source autodetection preflight: {error}",
            root.display()
        )
    })?;
    let ignored: HashSet<&str> = AUTODETECT_IGNORED_DIRS.iter().copied().collect();
    let mut entries_seen = 0usize;
    let entries = WalkDir::new(&canonical_root)
        .max_depth(4)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            if entry.depth() == 0 {
                return true;
            }
            let name = entry.file_name().to_str().unwrap_or("");
            if entry.depth() == 1 && (name.starts_with('.') || ignored.contains(name)) {
                return false;
            }
            !entry.file_type().is_dir() || (!name.starts_with('.') && !ignored.contains(name))
        });

    for entry in entries {
        entries_seen += 1;
        if entries_seen > MAX_CONFINEMENT_ENTRIES {
            return Err(format!(
                "MCP source autodetection preflight exceeds {MAX_CONFINEMENT_ENTRIES} entries"
            ));
        }
        let entry = entry
            .map_err(|error| format!("Cannot inspect MCP source autodetection input: {error}"))?;
        if !entry.file_type().is_symlink() {
            continue;
        }
        let canonical = entry.path().canonicalize().map_err(|error| {
            format!(
                "Cannot resolve symlink used by MCP source autodetection {}: {error}",
                entry.path().display()
            )
        })?;
        if !canonical.starts_with(&canonical_root) {
            return Err(format!(
                "Symlink used by MCP source autodetection escapes the configured server root: {}",
                entry.path().display()
            ));
        }
    }

    Ok(())
}

pub(super) fn validate_cargo_workspace_manifest(
    server_root: &Path,
    manifest_dir: &Path,
    visiting: &mut HashSet<PathBuf>,
    validated: &mut HashSet<PathBuf>,
    manifests_seen: &mut usize,
    traversal_entries_seen: &mut usize,
) -> Result<(), String> {
    let manifest_path = manifest_dir.join("Cargo.toml");
    match fs::symlink_metadata(&manifest_path) {
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(format!(
                "Cannot inspect MCP Cargo workspace manifest {}: {error}",
                manifest_path.display()
            ));
        }
    }
    validate_existing_path(
        server_root,
        &manifest_path,
        "Cargo workspace manifest",
        None,
    )?;
    let canonical_manifest = manifest_path.canonicalize().map_err(|error| {
        format!(
            "Cannot resolve MCP Cargo workspace manifest {}: {error}",
            manifest_path.display()
        )
    })?;
    if validated.contains(&canonical_manifest) {
        return Ok(());
    }
    if !visiting.insert(canonical_manifest.clone()) {
        return Err(format!(
            "MCP Cargo workspace manifest cycle detected at {}",
            manifest_path.display()
        ));
    }
    *manifests_seen += 1;
    if *manifests_seen > MAX_MANIFEST_PREFLIGHTS {
        return Err(format!(
            "MCP Cargo workspace preflight exceeds {MAX_MANIFEST_PREFLIGHTS} manifests"
        ));
    }

    let content = read_file_bounded(server_root, &manifest_path, "Cargo workspace manifest")?;
    let cargo = parse_cargo_snapshot_manifest(&content, &manifest_path)?;
    let mut member_roots = HashSet::new();
    for member in cargo.workspace_members {
        charge_manifest_traversal_entry(traversal_entries_seen, "MCP Cargo workspace preflight")?;
        let member_root = validate_manifest_relative_candidate(
            server_root,
            manifest_dir,
            &member,
            "Cargo workspace member",
        )?;
        let member_key = member_root.canonicalize().unwrap_or_else(|_| {
            member_root
                .strip_prefix(server_root)
                .map(normalize_snapshot_input)
                .unwrap_or_else(|_| member_root.clone())
        });
        if !member_roots.insert(member_key) {
            continue;
        }
        let nested_manifest = member_root.join("Cargo.toml");
        match fs::symlink_metadata(&nested_manifest) {
            Ok(_) => validate_cargo_workspace_manifest(
                server_root,
                &member_root,
                visiting,
                validated,
                manifests_seen,
                traversal_entries_seen,
            )?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "Cannot inspect MCP Cargo workspace member manifest {}: {error}",
                    nested_manifest.display()
                ));
            }
        }
    }

    visiting.remove(&canonical_manifest);
    validated.insert(canonical_manifest);
    Ok(())
}

pub(super) fn validate_package_workspaces(
    root: &Path,
    traversal_entries_seen: &mut usize,
) -> Result<(), String> {
    let package_path = root.join("package.json");
    let content = match read_file_bounded(root, &package_path, "package workspace manifest") {
        Ok(content) => content,
        Err(_) if !package_path.exists() => return Ok(()),
        Err(error) => return Err(error),
    };
    let json: Value = serde_json::from_str(&content)
        .map_err(|error| format!("Cannot parse MCP package.json as JSON: {error}"))?;
    let declarations = package_workspace_declarations(&json, "MCP package.json")?;
    for _ in declarations {
        charge_manifest_traversal_entry(traversal_entries_seen, "MCP package workspace preflight")?;
    }
    let workspace_patterns = declarations
        .iter()
        .map(|declaration| {
            declaration
                .as_str()
                .ok_or_else(|| "MCP package.json workspace entries must be strings".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut seen_patterns = HashSet::new();
    let mut seen_bases = HashSet::new();
    let mut seen_workspaces = HashSet::new();
    for pattern in workspace_patterns {
        if !seen_patterns.insert(pattern.to_string()) {
            continue;
        }
        let base = pattern.trim_end_matches("/*").trim_end_matches("/**");
        let base_dir = if base.is_empty() {
            root.to_path_buf()
        } else {
            validate_manifest_relative_candidate(root, root, base, "package workspace base")?
        };
        match fs::symlink_metadata(&base_dir) {
            Ok(_) => validate_existing_path(root, &base_dir, "package workspace base", None)?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(format!(
                    "Cannot inspect MCP package workspace base {}: {error}",
                    base_dir.display()
                ));
            }
        }
        let Ok(canonical_base) = base_dir.canonicalize() else {
            continue;
        };
        if !canonical_base.is_dir() || !seen_bases.insert(canonical_base) {
            continue;
        }
        let entries = match fs::read_dir(&base_dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries {
            charge_manifest_traversal_entry(
                traversal_entries_seen,
                "MCP package workspace preflight",
            )?;
            let entry = entry.map_err(|error| {
                format!(
                    "Cannot inspect entry beneath MCP package workspace base {}: {error}",
                    base_dir.display()
                )
            })?;
            validate_existing_path(root, &entry.path(), "package workspace entry", None)?;
            let canonical_entry = entry.path().canonicalize().map_err(|error| {
                format!(
                    "Cannot resolve MCP package workspace entry {}: {error}",
                    entry.path().display()
                )
            })?;
            if !canonical_entry.is_dir() {
                continue;
            }
            if !seen_workspaces.insert(canonical_entry) {
                continue;
            }
            let nested_package = entry.path().join("package.json");
            match fs::symlink_metadata(&nested_package) {
                Ok(_) => {
                    validate_existing_path(
                        root,
                        &nested_package,
                        "package workspace manifest",
                        None,
                    )?;
                    let nested_content =
                        read_file_bounded(root, &nested_package, "package workspace manifest")?;
                    let nested_json: Value =
                        serde_json::from_str(&nested_content).map_err(|error| {
                            format!(
                                "Cannot parse MCP package workspace manifest {} as JSON: {error}",
                                nested_package.display()
                            )
                        })?;
                    package_workspace_declarations(
                        &nested_json,
                        &format!(
                            "MCP package workspace manifest {}",
                            nested_package.display()
                        ),
                    )?;
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => {
                    return Err(format!(
                        "Cannot inspect MCP package workspace manifest {}: {error}",
                        nested_package.display()
                    ));
                }
            }
        }
    }
    Ok(())
}

fn charge_manifest_traversal_entry(entries_seen: &mut usize, label: &str) -> Result<(), String> {
    *entries_seen = entries_seen.saturating_add(1);
    if *entries_seen > MAX_CONFINEMENT_ENTRIES {
        return Err(format!("{label} exceeds {MAX_CONFINEMENT_ENTRIES} entries"));
    }
    Ok(())
}

fn validate_gradle_modules(root: &Path) -> Result<(), String> {
    let settings_path = ["settings.gradle.kts", "settings.gradle"]
        .iter()
        .map(|name| root.join(name))
        .find(|path| path.exists());
    let Some(settings_path) = settings_path else {
        return Ok(());
    };
    let settings = read_file_bounded(root, &settings_path, "Gradle settings manifest")?;
    let modules = parse_gradle_settings(&settings)
        .map_err(|error| format!("MCP Gradle discovery is inconclusive: {error}"))?;
    for module in modules {
        validate_manifest_relative_candidate(root, root, &module.path, "Gradle module path")?;
    }
    Ok(())
}

fn validate_python_package_path(root: &Path) -> Result<(), String> {
    let src = root.join("src");
    match fs::symlink_metadata(&src) {
        Ok(_) => {
            validate_existing_path(root, &src, "Python source directory", None)?;
            return Ok(());
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(format!(
                "Cannot inspect MCP Python source directory {}: {error}",
                src.display()
            ));
        }
    }

    let pyproject_path = root.join("pyproject.toml");
    let content = match read_file_bounded(root, &pyproject_path, "Python workspace manifest") {
        Ok(content) => content,
        Err(_) if !pyproject_path.exists() => return Ok(()),
        Err(error) => return Err(error),
    };
    let name = extract_manifest_toml_value(&content, "name", "[project]")
        .or_else(|| extract_manifest_toml_value(&content, "name", "[tool.poetry]"));
    if let Some(name) = name {
        validate_manifest_relative_candidate(root, root, &name, "Python package path")?;
    }
    Ok(())
}

fn validate_manifest_relative_candidate(
    server_root: &Path,
    base: &Path,
    configured: &str,
    label: &str,
) -> Result<PathBuf, String> {
    let relative = Path::new(configured);
    if configured.is_empty()
        || relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(format!(
            "MCP {label} must be a non-empty project-relative path without traversal: {configured}"
        ));
    }
    let candidate = base.join(relative);
    validate_path_or_ancestor(server_root, &candidate, label, None)?;
    Ok(candidate)
}

fn manifest_toml_section<'a>(content: &'a str, header: &str) -> Option<&'a str> {
    let start = content.find(header)?;
    let after = &content[start + header.len()..];
    let end = after
        .find("\n[")
        .map(|position| position + 1)
        .unwrap_or(after.len());
    Some(&after[..end])
}

pub(super) fn extract_manifest_toml_value(
    content: &str,
    key: &str,
    section: &str,
) -> Option<String> {
    let section = manifest_toml_section(content, section)?;
    for line in section.lines().map(str::trim) {
        let Some((candidate_key, value)) = line.split_once('=') else {
            continue;
        };
        if candidate_key.trim() != key {
            continue;
        }
        let value = value.trim();
        if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
            return Some(value[1..value.len() - 1].to_string());
        }
        return Some(value.to_string());
    }
    None
}

#[cfg(test)]
mod tests;
