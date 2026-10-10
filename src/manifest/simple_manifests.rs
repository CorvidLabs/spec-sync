//! Dart, Go, and Python discovery with shared TOML metadata helpers.

use std::path::Path;

#[cfg(test)]
use super::access::AmbientManifestAccess;
use super::access::ManifestAccess;
use super::{ManifestDiscovery, ManifestModule};

#[cfg(test)]
#[allow(dead_code)]
fn parse_pubspec_yaml(root: &Path) -> Option<ManifestDiscovery> {
    let mut access = AmbientManifestAccess { root };
    parse_pubspec_yaml_with_access(&mut access).ok().flatten()
}

pub(super) fn parse_pubspec_yaml_with_access(
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

pub(super) fn parse_go_mod_with_access(
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

pub(super) fn parse_pyproject_toml_with_access(
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
pub(super) fn extract_toml_value(
    content: &str,
    key: &str,
    section: Option<&str>,
) -> Option<String> {
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
pub(super) fn extract_section(content: &str, header: &str) -> Option<String> {
    let start = content.find(header)?;
    let after = &content[start + header.len()..];
    // Find the next section header
    let end = after.find("\n[").map(|pos| pos + 1).unwrap_or(after.len());
    Some(after[..end].to_string())
}

/// Split TOML content into repeated array-of-table sections (e.g., [[bin]]).
pub(super) fn split_toml_array_sections(content: &str, header: &str) -> Vec<String> {
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
mod tests;
