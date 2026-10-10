use super::super::mappings::{normalize_source_mapping, planned_source_path_is_safe};
use super::access::{
    coverage_metadata_identity, coverage_metadata_is_link, display_coverage_path,
    open_coverage_child_directory, open_retained_coverage_directory,
    read_coverage_file_with_expected_identity_and_hook, reopen_discovered_coverage_directory,
    verify_coverage_child_directory, verify_retained_coverage_directory,
};
use super::sources::{
    DiscoveredCoverageDirectory, ensure_coverage_depth, read_coverage_entry_names,
};
use super::{CoverageSpecFile, CoverageTraversalBudget};
use crate::parser::parse_frontmatter;
use cap_std::fs::Dir;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(super) fn discover_coverage_spec_files(
    project: &Dir,
    configured: &str,
    budget: &mut CoverageTraversalBudget,
) -> Result<Vec<CoverageSpecFile>, String> {
    discover_coverage_spec_files_with_hook(project, configured, budget, |_| {})
}

fn discover_coverage_spec_files_with_hook<AfterEnumeration>(
    project: &Dir,
    configured: &str,
    budget: &mut CoverageTraversalBudget,
    mut after_enumeration: AfterEnumeration,
) -> Result<Vec<CoverageSpecFile>, String>
where
    AfterEnumeration: FnMut(&Path),
{
    let relative = if configured == "." {
        PathBuf::new()
    } else {
        PathBuf::from(normalize_source_mapping(configured).ok_or_else(|| {
            format!("Coverage specs directory must remain beneath the project root: {configured}")
        })?)
    };
    ensure_coverage_depth(&relative, budget.limits.max_depth)?;
    let Some(retained_directory) = open_retained_coverage_directory(project, &relative)? else {
        return Ok(Vec::new());
    };
    let mut specs = Vec::new();
    discover_coverage_spec_directory(
        &retained_directory.directory,
        &relative,
        budget,
        &mut after_enumeration,
        &mut specs,
    )?;
    verify_retained_coverage_directory(project, &retained_directory)?;
    specs.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    specs.dedup_by(|left, right| left.relative_path == right.relative_path);
    Ok(specs)
}

fn discover_coverage_spec_directory<AfterEnumeration>(
    directory: &Dir,
    relative: &Path,
    budget: &mut CoverageTraversalBudget,
    after_enumeration: &mut AfterEnumeration,
    specs: &mut Vec<CoverageSpecFile>,
) -> Result<(), String>
where
    AfterEnumeration: FnMut(&Path),
{
    let names = read_coverage_entry_names(directory, relative, budget)?;
    let mut children = Vec::new();
    for name in names {
        let name_text = name.to_str().ok_or_else(|| {
            format!(
                "Coverage spec path beneath {} is not valid UTF-8",
                display_coverage_path(relative)
            )
        })?;
        let child = relative.join(&name);
        ensure_coverage_depth(&child, budget.limits.max_depth)?;
        let metadata = directory.symlink_metadata(&name).map_err(|error| {
            format!(
                "Cannot inspect retained coverage spec path {}: {error}",
                child.display()
            )
        })?;
        if coverage_metadata_is_link(&metadata) {
            return Err(format!(
                "Coverage spec path {} must not traverse a symlink or reparse point",
                child.display()
            ));
        }
        if metadata.is_dir() {
            let identity = coverage_metadata_identity(&metadata).map_err(|error| {
                format!(
                    "Cannot identify retained coverage spec directory {}: {error}",
                    child.display()
                )
            })?;
            children.push(DiscoveredCoverageDirectory {
                name,
                relative: child,
                identity,
            });
            continue;
        }
        if !metadata.is_file() {
            return Err(format!(
                "Coverage spec path {} must be a regular file or directory",
                child.display()
            ));
        }
        if name_text.ends_with(".spec.md") && !name_text.starts_with('_') {
            let identity = coverage_metadata_identity(&metadata).map_err(|error| {
                format!(
                    "Cannot identify retained coverage spec file {}: {error}",
                    child.display()
                )
            })?;
            specs.push(CoverageSpecFile {
                relative_path: child,
                identity,
            });
        }
    }
    after_enumeration(relative);
    for child in children {
        let child_directory = reopen_discovered_coverage_directory(directory, &child)?;
        discover_coverage_spec_directory(
            &child_directory,
            &child.relative,
            budget,
            after_enumeration,
            specs,
        )?;
        verify_coverage_child_directory(directory, &child.name, &child.relative, &child_directory)?;
    }
    Ok(())
}

// ─── Coverage ────────────────────────────────────────────────────────────

pub(super) fn collect_specced_files(
    project: &Dir,
    spec_files: &[CoverageSpecFile],
    budget: &mut CoverageTraversalBudget,
) -> Result<HashSet<String>, String> {
    collect_specced_files_with_hook(project, spec_files, budget, |_| {})
}

fn collect_specced_files_with_hook<BeforeRead>(
    project: &Dir,
    spec_files: &[CoverageSpecFile],
    budget: &mut CoverageTraversalBudget,
    mut before_read: BeforeRead,
) -> Result<HashSet<String>, String>
where
    BeforeRead: FnMut(&Path),
{
    let mut specced = HashSet::new();
    let mut observed_specs = HashSet::new();
    for spec_file in spec_files {
        let relative = &spec_file.relative_path;
        if !observed_specs.insert(relative.clone()) {
            continue;
        }
        budget.charge_entries(1)?;
        ensure_coverage_depth(relative, budget.limits.max_depth)?;
        let parent = relative.parent().unwrap_or_else(|| Path::new(""));
        let name = relative.file_name().ok_or_else(|| {
            format!(
                "Coverage spec path {} has no terminal filename",
                relative.display()
            )
        })?;
        let Some(retained_directory) = open_retained_coverage_directory(project, parent)? else {
            return Err(format!(
                "Coverage spec file {} is missing from the retained project",
                relative.display()
            ));
        };
        let directory = &retained_directory.directory;
        verify_retained_coverage_directory(project, &retained_directory)?;
        before_read(relative);
        let bytes = read_coverage_file_with_expected_identity_and_hook(
            directory,
            name,
            relative,
            budget.limits.max_file_bytes,
            budget.remaining_bytes(),
            budget.limits.max_input_bytes,
            Some(spec_file.identity),
            || {},
        )?;
        verify_retained_coverage_directory(project, &retained_directory)?;
        budget.charge_input_file(relative, bytes.len() as u64)?;
        let content = std::str::from_utf8(&bytes).map_err(|_| {
            format!(
                "Coverage spec file {} is not valid UTF-8",
                relative.display()
            )
        })?;
        let normalized = content.replace("\r\n", "\n");
        if let Some(parsed) = parse_frontmatter(&normalized) {
            if parsed.frontmatter.parsed_status() == Some(crate::types::SpecStatus::Archived) {
                continue;
            }
            for file in &parsed.frontmatter.files {
                if planned_source_path_is_safe(file)
                    && let Some(normalized) = normalize_source_mapping(file)
                {
                    specced.insert(normalized);
                }
            }
        }
    }
    Ok(specced)
}

pub(super) fn select_coverage_spec_files(
    root: &Path,
    spec_files: &[PathBuf],
    retained_inventory: &[CoverageSpecFile],
) -> Result<Vec<CoverageSpecFile>, String> {
    let selected_spec_files: HashSet<PathBuf> = spec_files
        .iter()
        .map(|spec_file| coverage_relative_spec_path(root, spec_file))
        .collect::<Result<_, _>>()?;
    let retained_spec_files: Vec<CoverageSpecFile> = retained_inventory
        .iter()
        .filter(|spec| selected_spec_files.contains(&spec.relative_path))
        .cloned()
        .collect();
    if retained_spec_files.len() != selected_spec_files.len() {
        let retained_spec_set: HashSet<&PathBuf> = retained_spec_files
            .iter()
            .map(|spec| &spec.relative_path)
            .collect();
        let missing = selected_spec_files
            .iter()
            .find(|selected| !retained_spec_set.contains(selected))
            .expect("selected and retained spec counts differ");
        return Err(format!(
            "Coverage selected spec {} is missing from the retained spec inventory",
            missing.display()
        ));
    }
    Ok(retained_spec_files)
}

fn coverage_relative_spec_path(root: &Path, spec_file: &Path) -> Result<PathBuf, String> {
    let candidate = match spec_file.strip_prefix(root) {
        Ok(relative) => relative,
        Err(_) if spec_file.is_relative() => spec_file,
        Err(_) => {
            return Err(format!(
                "Coverage spec path {} must remain beneath the project root",
                spec_file.display()
            ));
        }
    };
    let mut relative = PathBuf::new();
    for component in candidate.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::Normal(name) => relative.push(name),
            std::path::Component::ParentDir
            | std::path::Component::RootDir
            | std::path::Component::Prefix(_) => {
                return Err(format!(
                    "Coverage spec path {} must remain beneath the project root",
                    spec_file.display()
                ));
            }
        }
    }
    if relative.as_os_str().is_empty() {
        return Err("Coverage spec path cannot be the project root".to_string());
    }
    Ok(relative)
}

/// Get spec module directories that actually contain a .spec.md file.
/// Empty directories (e.g. from a failed prior generation) are ignored.
pub(super) fn get_spec_module_dirs(
    project: &Dir,
    configured: &str,
    budget: &mut CoverageTraversalBudget,
) -> Result<Vec<String>, String> {
    let relative = if configured == "." {
        PathBuf::new()
    } else {
        PathBuf::from(normalize_source_mapping(configured).ok_or_else(|| {
            format!("Coverage specs directory must remain beneath the project root: {configured}")
        })?)
    };
    ensure_coverage_depth(&relative, budget.limits.max_depth)?;
    let mut modules = Vec::new();
    let Some(retained_directory) = open_retained_coverage_directory(project, &relative)? else {
        return Ok(modules);
    };
    let directory = &retained_directory.directory;
    let names = read_coverage_entry_names(directory, &relative, budget)?;
    for name in names {
        let name_text = name.to_str().ok_or_else(|| {
            format!(
                "Coverage spec-module path beneath {} is not valid UTF-8",
                display_coverage_path(&relative)
            )
        })?;
        let child = relative.join(&name);
        ensure_coverage_depth(&child, budget.limits.max_depth)?;
        let metadata = directory.symlink_metadata(&name).map_err(|error| {
            format!(
                "Cannot inspect retained coverage spec-module path {}: {error}",
                child.display()
            )
        })?;
        if coverage_metadata_is_link(&metadata) {
            return Err(format!(
                "Coverage spec-module path {} must not traverse a symlink or reparse point",
                child.display()
            ));
        }
        if !metadata.is_dir() {
            continue;
        }
        let child_directory = open_coverage_child_directory(directory, &name, &child)?;
        let spec_names = read_coverage_entry_names(&child_directory, &child, budget)?;
        let mut has_spec = false;
        for spec_name in spec_names {
            let spec_name_text = spec_name.to_str().ok_or_else(|| {
                format!(
                    "Coverage spec path beneath {} is not valid UTF-8",
                    child.display()
                )
            })?;
            if !spec_name_text.ends_with(".spec.md") {
                continue;
            }
            let spec_path = child.join(&spec_name);
            ensure_coverage_depth(&spec_path, budget.limits.max_depth)?;
            let spec_metadata = child_directory
                .symlink_metadata(&spec_name)
                .map_err(|error| {
                    format!(
                        "Cannot inspect retained coverage spec path {}: {error}",
                        spec_path.display()
                    )
                })?;
            if coverage_metadata_is_link(&spec_metadata) {
                return Err(format!(
                    "Coverage spec path {} must not traverse a symlink or reparse point",
                    spec_path.display()
                ));
            }
            if spec_metadata.is_file() {
                has_spec = true;
                continue;
            }
            if !spec_metadata.is_dir() {
                return Err(format!(
                    "Coverage spec path {} must be a regular file",
                    spec_path.display()
                ));
            }
        }
        verify_coverage_child_directory(directory, &name, &child, &child_directory)?;
        if has_spec {
            modules.push(name_text.to_string());
        }
    }
    verify_retained_coverage_directory(project, &retained_directory)?;
    modules.sort();
    Ok(modules)
}

#[cfg(test)]
#[path = "tests/specs.rs"]
mod tests;
