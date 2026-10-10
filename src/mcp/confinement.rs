mod manifests;

use super::MAX_CONFIGURED_PATHS;
use super::MAX_CONFINEMENT_ENTRIES;
use super::MAX_PROJECT_FILE_BYTES;
use super::MAX_PROJECT_INPUT_BYTES;
use crate::config::load_config;
use crate::types::SpecSyncConfig;
use crate::validator::find_spec_files;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::io;
use std::io::Read;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use walkdir::WalkDir;

const AUTODETECT_IGNORED_DIRS: &[&str] = &[
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

pub(super) fn load_and_discover(
    root: &Path,
    allow_empty: bool,
) -> Result<(SpecSyncConfig, Vec<PathBuf>), String> {
    let config = load_confined_config(root)?;
    let specs_dir = root.join(&config.specs_dir);
    let spec_files: Vec<PathBuf> = find_spec_files(&specs_dir)
        .into_iter()
        .filter(|f| {
            f.file_name()
                .and_then(|n| n.to_str())
                .map(|n| !n.starts_with('_'))
                .unwrap_or(true)
        })
        .collect();

    validate_spec_file_mappings(root, &spec_files, &config.exclude_dirs)?;

    if spec_files.is_empty() && !allow_empty {
        return Err(format!(
            "No spec files found in {}/. Run specsync generate to scaffold specs.",
            config.specs_dir
        ));
    }

    Ok((config, spec_files))
}

pub(super) fn load_confined_config(root: &Path) -> Result<SpecSyncConfig, String> {
    validate_known_config_files(root)?;
    validate_manifest_inputs(root)?;
    if config_requires_source_autodetection(root) {
        validate_autodetection_tree(root)?;
    }
    let config = load_config(root);

    let configured_path_count = 1usize
        + config.source_dirs.len()
        + usize::from(config.schema_dir.is_some())
        + config
            .modules
            .values()
            .map(|module| module.files.len())
            .sum::<usize>();
    if configured_path_count > MAX_CONFIGURED_PATHS {
        return Err(format!(
            "MCP configuration exceeds {MAX_CONFIGURED_PATHS} path entries"
        ));
    }

    let no_exclusions = HashSet::new();
    let source_exclusions: HashSet<String> = config.exclude_dirs.iter().cloned().collect();
    let mut confinement_entries_seen = 0usize;

    validate_configured_path_with_budget(
        root,
        &config.specs_dir,
        "specs_dir",
        Some(&no_exclusions),
        &mut confinement_entries_seen,
    )?;
    for source_dir in &config.source_dirs {
        validate_configured_path_with_budget(
            root,
            source_dir,
            "source_dirs entry",
            Some(&source_exclusions),
            &mut confinement_entries_seen,
        )?;
    }
    if let Some(schema_dir) = config.schema_dir.as_deref()
        && !schema_dir.is_empty()
    {
        validate_configured_path_with_budget(
            root,
            schema_dir,
            "schema_dir",
            Some(&no_exclusions),
            &mut confinement_entries_seen,
        )?;
    }
    for (module_name, module) in &config.modules {
        validate_module_name(module_name)?;
        for file in &module.files {
            validate_configured_path_with_budget(
                root,
                file,
                "configured module file",
                Some(&source_exclusions),
                &mut confinement_entries_seen,
            )?;
        }
    }

    validate_project_input_budget(root, &config)?;

    Ok(config)
}

pub(super) fn validate_known_config_files(root: &Path) -> Result<(), String> {
    for relative in [
        ".specsync/config.toml",
        ".specsync/config.json",
        ".specsync/config.local.toml",
        ".specsync.toml",
        "specsync.json",
        ".specsync/hashes.json",
        ".git",
        "Cargo.toml",
        "Package.swift",
        "build.gradle.kts",
        "build.gradle",
        "settings.gradle.kts",
        "settings.gradle",
        "package.json",
        "pubspec.yaml",
        "go.mod",
        "pyproject.toml",
    ] {
        let candidate = root.join(relative);
        match fs::symlink_metadata(&candidate) {
            Ok(_) => validate_existing_path(root, &candidate, "project metadata file", None)?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "Cannot inspect MCP configuration path {}: {error}",
                    candidate.display()
                ));
            }
        }
    }
    Ok(())
}

fn selected_config_path(root: &Path) -> Option<PathBuf> {
    [
        ".specsync/config.toml",
        ".specsync/config.json",
        ".specsync.toml",
        "specsync.json",
    ]
    .iter()
    .map(|relative| root.join(relative))
    .find(|path| path.exists())
}

fn config_requires_source_autodetection(root: &Path) -> bool {
    let Some(config_path) = selected_config_path(root) else {
        return true;
    };
    let Some(content) = crate::config::read_config_file(&config_path) else {
        return false;
    };
    if config_path
        .extension()
        .and_then(|extension| extension.to_str())
        == Some("json")
    {
        return serde_json::from_str::<Value>(&content).is_ok()
            && !content.contains("\"sourceDirs\"");
    }

    let mut top_level = true;
    for line in content.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            top_level = false;
            continue;
        }
        if top_level
            && line
                .split_once('=')
                .is_some_and(|(key, _)| key.trim() == "source_dirs")
        {
            return false;
        }
    }
    true
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

fn validate_configured_path_with_budget(
    root: &Path,
    configured: &str,
    label: &str,
    tree_exclusions: Option<&HashSet<String>>,
    confinement_entries_seen: &mut usize,
) -> Result<(), String> {
    let path = Path::new(configured);
    if configured.is_empty() || path.is_absolute() {
        return Err(format!(
            "MCP {label} must be a non-empty project-relative path: {configured}"
        ));
    }
    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(format!(
            "MCP {label} must not contain traversal or an absolute prefix: {configured}"
        ));
    }

    validate_path_or_ancestor_with_budget(
        root,
        &root.join(path),
        label,
        tree_exclusions,
        confinement_entries_seen,
    )
}

pub(super) fn validate_path_or_ancestor(
    root: &Path,
    candidate: &Path,
    label: &str,
    tree_exclusions: Option<&HashSet<String>>,
) -> Result<(), String> {
    let mut confinement_entries_seen = 0usize;
    validate_path_or_ancestor_with_budget(
        root,
        candidate,
        label,
        tree_exclusions,
        &mut confinement_entries_seen,
    )
}

fn validate_path_or_ancestor_with_budget(
    root: &Path,
    candidate: &Path,
    label: &str,
    tree_exclusions: Option<&HashSet<String>>,
    confinement_entries_seen: &mut usize,
) -> Result<(), String> {
    let mut current = candidate;
    loop {
        match fs::symlink_metadata(current) {
            Ok(_) => {
                let exclusions = (current == candidate).then_some(tree_exclusions).flatten();
                validate_existing_path_with_budget(
                    root,
                    current,
                    label,
                    exclusions,
                    confinement_entries_seen,
                )?;
                return Ok(());
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                current = current.parent().ok_or_else(|| {
                    format!(
                        "MCP {label} has no resolvable ancestor: {}",
                        candidate.display()
                    )
                })?;
            }
            Err(error) => {
                return Err(format!(
                    "Cannot inspect MCP {label} {}: {error}",
                    current.display()
                ));
            }
        }
    }
}

pub(super) fn validate_existing_path(
    root: &Path,
    candidate: &Path,
    label: &str,
    tree_exclusions: Option<&HashSet<String>>,
) -> Result<(), String> {
    let mut confinement_entries_seen = 0usize;
    validate_existing_path_with_budget(
        root,
        candidate,
        label,
        tree_exclusions,
        &mut confinement_entries_seen,
    )
}

fn validate_existing_path_with_budget(
    root: &Path,
    candidate: &Path,
    label: &str,
    tree_exclusions: Option<&HashSet<String>>,
    confinement_entries_seen: &mut usize,
) -> Result<(), String> {
    let canonical_root = root.canonicalize().map_err(|error| {
        format!(
            "Cannot resolve MCP server root {} while validating {label}: {error}",
            root.display()
        )
    })?;
    let canonical = candidate.canonicalize().map_err(|error| {
        format!(
            "Cannot resolve MCP {label} {} (including symlink targets): {error}",
            candidate.display()
        )
    })?;
    if !canonical.starts_with(&canonical_root) {
        return Err(format!(
            "MCP {label} escapes the configured server root: {}",
            candidate.display()
        ));
    }
    let metadata = fs::metadata(&canonical).map_err(|error| {
        format!(
            "Cannot inspect MCP {label} {} after canonicalization: {error}",
            candidate.display()
        )
    })?;
    if metadata.is_file() && metadata.len() > MAX_PROJECT_FILE_BYTES {
        return Err(format!(
            "MCP {label} exceeds the {} MiB project-file limit: {}",
            MAX_PROJECT_FILE_BYTES / (1024 * 1024),
            candidate.display()
        ));
    }
    if let Some(exclusions) = tree_exclusions
        && canonical.is_dir()
    {
        validate_tree_confinement(
            &canonical_root,
            &canonical,
            label,
            exclusions,
            confinement_entries_seen,
        )?;
    }
    Ok(())
}

pub(super) fn read_file_bounded(root: &Path, path: &Path, label: &str) -> Result<String, String> {
    validate_existing_path(root, path, label, None)?;
    let metadata = fs::metadata(path)
        .map_err(|error| format!("Cannot inspect MCP {label} {}: {error}", path.display()))?;
    if !metadata.is_file() {
        return Err(format!("MCP {label} is not a file: {}", path.display()));
    }
    if metadata.len() > MAX_PROJECT_FILE_BYTES {
        return Err(format!(
            "MCP {label} exceeds the {} MiB project-file limit: {}",
            MAX_PROJECT_FILE_BYTES / (1024 * 1024),
            path.display()
        ));
    }

    let file = fs::File::open(path)
        .map_err(|error| format!("Cannot read MCP {label} {}: {error}", path.display()))?;
    let mut bytes = Vec::new();
    file.take(MAX_PROJECT_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("Cannot read MCP {label} {}: {error}", path.display()))?;
    if bytes.len() as u64 > MAX_PROJECT_FILE_BYTES {
        return Err(format!(
            "MCP {label} exceeds the {} MiB project-file limit: {}",
            MAX_PROJECT_FILE_BYTES / (1024 * 1024),
            path.display()
        ));
    }
    String::from_utf8(bytes)
        .map_err(|_| format!("MCP {label} is not valid UTF-8: {}", path.display()))
}

fn validate_project_input_budget(root: &Path, config: &SpecSyncConfig) -> Result<(), String> {
    let mut candidates = vec![root.join(&config.specs_dir)];
    candidates.extend(config.source_dirs.iter().map(|path| root.join(path)));
    if let Some(schema_dir) = config.schema_dir.as_deref()
        && !schema_dir.is_empty()
    {
        candidates.push(root.join(schema_dir));
    }
    for module in config.modules.values() {
        candidates.extend(module.files.iter().map(|path| root.join(path)));
    }

    let mut exclusions: HashSet<String> = AUTODETECT_IGNORED_DIRS
        .iter()
        .map(|name| (*name).to_string())
        .collect();
    exclusions.extend(config.exclude_dirs.iter().cloned());
    let canonical_root = root
        .canonicalize()
        .map_err(|error| format!("Cannot resolve MCP server root {}: {error}", root.display()))?;
    let mut seen_files = HashSet::new();
    let mut total_bytes = 0u64;
    let mut entries_seen = 0usize;

    for candidate in candidates {
        if !candidate.exists() {
            continue;
        }
        let entries = WalkDir::new(&candidate)
            .follow_links(false)
            .into_iter()
            .filter_entry(|entry| {
                entry.depth() == 0
                    || !entry.file_type().is_dir()
                    || !entry
                        .file_name()
                        .to_str()
                        .is_some_and(|name| exclusions.contains(name))
            });
        for entry in entries {
            entries_seen += 1;
            if entries_seen > MAX_CONFINEMENT_ENTRIES {
                return Err(format!(
                    "MCP project input scan exceeds {MAX_CONFINEMENT_ENTRIES} entries"
                ));
            }
            let entry =
                entry.map_err(|error| format!("Cannot inspect MCP project input: {error}"))?;
            if !entry.file_type().is_file() {
                continue;
            }
            let canonical = entry.path().canonicalize().map_err(|error| {
                format!(
                    "Cannot resolve MCP project input {}: {error}",
                    entry.path().display()
                )
            })?;
            if !canonical.starts_with(&canonical_root) {
                return Err(format!(
                    "MCP project input escapes the configured server root: {}",
                    entry.path().display()
                ));
            }
            if !seen_files.insert(canonical) {
                continue;
            }
            let bytes = entry
                .metadata()
                .map_err(|error| {
                    format!(
                        "Cannot inspect MCP project input {}: {error}",
                        entry.path().display()
                    )
                })?
                .len();
            if bytes > MAX_PROJECT_FILE_BYTES {
                return Err(format!(
                    "MCP project input exceeds the {} MiB per-file limit: {}",
                    MAX_PROJECT_FILE_BYTES / (1024 * 1024),
                    entry.path().display()
                ));
            }
            total_bytes = total_bytes.saturating_add(bytes);
            if total_bytes > MAX_PROJECT_INPUT_BYTES {
                return Err(format!(
                    "MCP project inputs exceed the {} MiB cumulative limit",
                    MAX_PROJECT_INPUT_BYTES / (1024 * 1024)
                ));
            }
        }
    }

    Ok(())
}

fn validate_tree_confinement(
    root: &Path,
    start: &Path,
    label: &str,
    exclusions: &HashSet<String>,
    entries_seen: &mut usize,
) -> Result<(), String> {
    let entries = WalkDir::new(start)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            entry.depth() == 0
                || !entry.file_type().is_dir()
                || !entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| exclusions.contains(name))
        });

    for entry in entries {
        *entries_seen += 1;
        if *entries_seen > MAX_CONFINEMENT_ENTRIES {
            return Err(format!(
                "MCP {label} confinement scan exceeds {MAX_CONFINEMENT_ENTRIES} entries"
            ));
        }
        let entry =
            entry.map_err(|error| format!("Cannot inspect entry beneath MCP {label}: {error}"))?;
        if !entry.file_type().is_symlink() {
            continue;
        }
        let canonical = entry.path().canonicalize().map_err(|error| {
            format!(
                "Cannot resolve symlink beneath MCP {label} {}: {error}",
                entry.path().display()
            )
        })?;
        if !canonical.starts_with(root) {
            return Err(format!(
                "Symlink beneath MCP {label} escapes the configured server root: {}",
                entry.path().display()
            ));
        }
    }

    Ok(())
}

pub(super) fn validate_spec_file_mappings(
    root: &Path,
    spec_files: &[PathBuf],
    exclude_dirs: &[String],
) -> Result<(), String> {
    let mut confinement_entries_seen = 0usize;
    let exclusions: HashSet<String> = exclude_dirs.iter().cloned().collect();
    for spec_file in spec_files {
        validate_existing_path(root, spec_file, "spec file", None)?;
        let content = read_file_bounded(root, spec_file, "spec file")?;
        let Some(parsed) = crate::parser::parse_frontmatter(&content) else {
            continue;
        };
        for file in &parsed.frontmatter.files {
            validate_configured_path_with_budget(
                root,
                file,
                "spec file mapping",
                Some(&exclusions),
                &mut confinement_entries_seen,
            )?;
        }
        for dependency in &parsed.frontmatter.depends_on {
            if crate::validator::is_cross_project_ref(dependency) {
                continue;
            }
            if !dependency.contains('/') && !dependency.contains('.') {
                validate_module_name(dependency)?;
            } else {
                validate_configured_path_with_budget(
                    root,
                    dependency,
                    "dependency reference",
                    None,
                    &mut confinement_entries_seen,
                )?;
            }
        }
        validate_consumed_by_paths(root, &parsed.body, &mut confinement_entries_seen)?;
    }
    Ok(())
}

fn validate_consumed_by_paths(
    root: &Path,
    body: &str,
    confinement_entries_seen: &mut usize,
) -> Result<(), String> {
    let mut in_section = false;
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed == "### Consumed By" {
            in_section = true;
            continue;
        }
        if in_section && (trimmed.starts_with("## ") || trimmed.starts_with("### ")) {
            break;
        }
        if !in_section || !trimmed.starts_with('|') {
            continue;
        }
        for (index, value) in line.split('`').enumerate() {
            if index % 2 == 1 && value.rsplit_once('.').is_some() {
                validate_configured_path_with_budget(
                    root,
                    value,
                    "Consumed By file reference",
                    None,
                    confinement_entries_seen,
                )?;
            }
        }
    }
    Ok(())
}

fn validate_module_name(module_name: &str) -> Result<(), String> {
    crate::commands::validate_module_name(module_name)
        .map_err(|error| format!("MCP module name is unsafe: {error}"))
}

pub(super) fn validate_generated_module_names(module_names: &[String]) -> Result<(), String> {
    for module_name in module_names {
        validate_module_name(module_name)?;
    }
    Ok(())
}

// Keep manifest preflight behind the confinement module boundary.
pub(super) fn validate_manifest_inputs(root: &Path) -> Result<(), String> {
    manifests::validate_manifest_inputs(root)
}

#[cfg(test)]
mod tests;
