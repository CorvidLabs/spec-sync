use super::super::mappings::normalize_source_mapping;
use super::access::{
    CoverageEntryIdentity, coverage_directory_identity, coverage_metadata_identity,
    coverage_metadata_is_link, display_coverage_path, open_coverage_child_directory,
    open_retained_coverage_directory, read_coverage_file, reopen_discovered_coverage_directory,
    verify_coverage_child_directory, verify_coverage_directory_edge,
    verify_retained_coverage_directory,
};
use super::{
    CoverageSourceFile, CoverageSourceSnapshot, CoverageTraversalBudget, has_coverage_extension,
};
use crate::config::{is_detectable_source_file, source_detection_ignores_directory};
use crate::exports::is_test_file;
use crate::types::SpecSyncConfig;
use cap_std::fs::Dir;
use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

pub(super) fn retained_source_dirs(
    project: &Dir,
    root: &Path,
    budget: &mut CoverageTraversalBudget,
) -> Result<Vec<String>, String> {
    match crate::manifest::discover_from_manifests_checked_with_root(root, project) {
        Ok(mut discovery) if !discovery.source_dirs.is_empty() => {
            discovery.source_dirs.sort();
            discovery.source_dirs.dedup();
            Ok(discovery.source_dirs)
        }
        Ok(_) => retained_source_dirs_by_scan(project, root, budget),
        // Coverage itself repeats retained manifest discovery and owns the
        // command-specific inconclusive JSON. Preserve legacy omitted-source
        // compatibility for shared CLI discovery by using the bounded retained
        // scan here; the checked coverage pass still reports the manifest error.
        Err(_) => retained_source_dirs_by_scan(project, root, budget),
    }
}

fn retained_source_dirs_by_scan(
    project: &Dir,
    root: &Path,
    budget: &mut CoverageTraversalBudget,
) -> Result<Vec<String>, String> {
    let mut source_dirs = Vec::new();
    let mut has_root_source_files = false;
    let names = read_coverage_entry_names(project, Path::new(""), budget)?;
    for name in names {
        let name_text = name.to_str().ok_or_else(|| {
            "Coverage source-detection path beneath . is not valid UTF-8".to_string()
        })?;
        if source_detection_ignores_directory(name_text) {
            continue;
        }
        let relative = PathBuf::from(&name);
        let metadata = project.symlink_metadata(&name).map_err(|error| {
            format!(
                "Cannot inspect retained source-detection path {}: {error}",
                relative.display()
            )
        })?;
        if coverage_metadata_is_link(&metadata) {
            budget.record_skipped_link(&relative);
            continue;
        }
        if metadata.is_file() {
            if is_detectable_source_file(&root.join(&relative)) {
                has_root_source_files = true;
            }
            continue;
        }
        if !metadata.is_dir() {
            continue;
        }
        let directory = open_coverage_child_directory(project, &name, &relative)?;
        if retained_directory_contains_source(project, root, directory, &relative, 0, budget)? {
            source_dirs.push(name_text.to_string());
        }
    }
    verify_coverage_directory_edge(project, Path::new(""), project)?;
    if has_root_source_files && source_dirs.is_empty() {
        return Ok(vec![".".to_string()]);
    }
    if source_dirs.is_empty() {
        return Ok(vec!["src".to_string()]);
    }
    source_dirs.sort();
    source_dirs.dedup();
    Ok(source_dirs)
}

fn retained_directory_contains_source(
    project: &Dir,
    root: &Path,
    directory: Dir,
    relative: &Path,
    depth: usize,
    budget: &mut CoverageTraversalBudget,
) -> Result<bool, String> {
    let retained = directory.try_clone().map_err(|error| {
        format!(
            "Cannot retain source-detection directory {}: {error}",
            relative.display()
        )
    })?;
    let names = read_coverage_entry_names(&directory, relative, budget)?;
    let mut found = false;
    for name in names {
        let name_text = name.to_str().ok_or_else(|| {
            format!(
                "Coverage source-detection path beneath {} is not valid UTF-8",
                relative.display()
            )
        })?;
        if source_detection_ignores_directory(name_text) {
            continue;
        }
        let child = relative.join(&name);
        ensure_coverage_depth(&child, budget.limits.max_depth)?;
        let metadata = directory.symlink_metadata(&name).map_err(|error| {
            format!(
                "Cannot inspect retained source-detection path {}: {error}",
                child.display()
            )
        })?;
        if coverage_metadata_is_link(&metadata) {
            budget.record_skipped_link(&child);
            continue;
        }
        if metadata.is_file() && is_detectable_source_file(&root.join(&child)) {
            found = true;
            break;
        }
        if metadata.is_dir() && depth < 2 {
            let child_directory = open_coverage_child_directory(&directory, &name, &child)?;
            if retained_directory_contains_source(
                project,
                root,
                child_directory,
                &child,
                depth + 1,
                budget,
            )? {
                found = true;
                break;
            }
        }
    }
    verify_coverage_directory_edge(project, relative, &retained)?;
    Ok(found)
}

pub(super) struct DiscoveredCoverageDirectory {
    pub(super) name: OsString,
    pub(super) relative: PathBuf,
    pub(super) identity: CoverageEntryIdentity,
}

pub(super) struct SelectedCoverageSourceDirectory {
    configured: String,
    normalized: PathBuf,
    identity: CoverageEntryIdentity,
}

#[cfg(test)]
fn snapshot_coverage_sources(
    project: &Dir,
    root: &Path,
    config: &SpecSyncConfig,
    exclude_dirs: &HashSet<String>,
    budget: &mut CoverageTraversalBudget,
) -> Result<CoverageSourceSnapshot, String> {
    snapshot_coverage_sources_with_hook(project, root, config, exclude_dirs, budget, |_| {})
}

#[cfg(test)]
fn snapshot_coverage_sources_with_hook<AfterEnumeration>(
    project: &Dir,
    root: &Path,
    config: &SpecSyncConfig,
    exclude_dirs: &HashSet<String>,
    budget: &mut CoverageTraversalBudget,
    mut after_enumeration: AfterEnumeration,
) -> Result<CoverageSourceSnapshot, String>
where
    AfterEnumeration: FnMut(&Path),
{
    let selected_sources =
        select_coverage_source_directories(project, config, budget.limits.max_depth)?;
    snapshot_selected_coverage_sources_with_hook(
        project,
        root,
        config,
        exclude_dirs,
        budget,
        selected_sources,
        &mut after_enumeration,
    )
}

pub(super) fn select_coverage_source_directories(
    project: &Dir,
    config: &SpecSyncConfig,
    max_depth: usize,
) -> Result<Vec<SelectedCoverageSourceDirectory>, String> {
    let mut selected_sources = Vec::new();
    for configured in &config.source_dirs {
        let normalized = if configured == "." {
            PathBuf::new()
        } else {
            PathBuf::from(normalize_source_mapping(configured).ok_or_else(|| {
                format!(
                    "Coverage source directory must remain beneath the project root: {configured}"
                )
            })?)
        };
        ensure_coverage_depth(&normalized, max_depth)?;
        let Some(retained) = open_retained_coverage_directory(project, &normalized)? else {
            continue;
        };
        let identity = coverage_directory_identity(&retained.directory).map_err(|error| {
            format!(
                "Cannot identify selected coverage source directory {}: {error}",
                display_coverage_path(&normalized)
            )
        })?;
        verify_retained_coverage_directory(project, &retained)?;
        selected_sources.push(SelectedCoverageSourceDirectory {
            configured: configured.clone(),
            normalized,
            identity,
        });
    }
    Ok(selected_sources)
}

pub(super) fn snapshot_selected_coverage_sources_with_hook<AfterEnumeration>(
    project: &Dir,
    root: &Path,
    config: &SpecSyncConfig,
    exclude_dirs: &HashSet<String>,
    budget: &mut CoverageTraversalBudget,
    selected_sources: Vec<SelectedCoverageSourceDirectory>,
    after_enumeration: &mut AfterEnumeration,
) -> Result<CoverageSourceSnapshot, String>
where
    AfterEnumeration: FnMut(&Path),
{
    let mut snapshot = CoverageSourceSnapshot::default();
    for source in selected_sources {
        let Some(retained) = open_retained_coverage_directory(project, &source.normalized)? else {
            return Err(format!(
                "Selected coverage source directory {} changed during retained traversal",
                display_coverage_path(&source.normalized)
            ));
        };
        let observed_identity =
            coverage_directory_identity(&retained.directory).map_err(|error| {
                format!(
                    "Cannot identify reopened coverage source directory {}: {error}",
                    display_coverage_path(&source.normalized)
                )
            })?;
        if observed_identity != source.identity {
            return Err(format!(
                "Selected coverage source directory {} changed during retained traversal",
                display_coverage_path(&source.normalized)
            ));
        }
        let mut immediate_modules = Vec::new();
        let mut accumulator = CoverageSourceAccumulator {
            exclude_dirs,
            config,
            root,
            immediate_modules: &mut immediate_modules,
            files: &mut snapshot.files,
            budget,
        };
        snapshot_coverage_directory_with_hook(
            &retained.directory,
            &source.normalized,
            &mut accumulator,
            after_enumeration,
        )?;
        verify_retained_coverage_directory(project, &retained)?;
        immediate_modules.sort();
        immediate_modules.dedup();
        snapshot
            .immediate_modules
            .insert(source.configured, immediate_modules);
    }
    snapshot
        .files
        .sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    snapshot
        .files
        .dedup_by(|left, right| left.relative_path == right.relative_path);
    Ok(snapshot)
}

struct CoverageSourceAccumulator<'a> {
    exclude_dirs: &'a HashSet<String>,
    config: &'a SpecSyncConfig,
    root: &'a Path,
    immediate_modules: &'a mut Vec<OsString>,
    files: &'a mut Vec<CoverageSourceFile>,
    budget: &'a mut CoverageTraversalBudget,
}

fn snapshot_coverage_directory_with_hook<AfterEnumeration>(
    directory: &Dir,
    relative: &Path,
    accumulator: &mut CoverageSourceAccumulator<'_>,
    after_enumeration: &mut AfterEnumeration,
) -> Result<(), String>
where
    AfterEnumeration: FnMut(&Path),
{
    snapshot_coverage_directory(directory, relative, true, accumulator, after_enumeration)
}

fn snapshot_coverage_directory<AfterEnumeration>(
    directory: &Dir,
    relative: &Path,
    source_root: bool,
    accumulator: &mut CoverageSourceAccumulator<'_>,
    after_enumeration: &mut AfterEnumeration,
) -> Result<(), String>
where
    AfterEnumeration: FnMut(&Path),
{
    let names = read_coverage_entry_names(directory, relative, accumulator.budget)?;
    let mut children = Vec::new();
    for name in names {
        if coverage_name_is_excluded(&name, accumulator.exclude_dirs) {
            continue;
        }
        name.to_str().ok_or_else(|| {
            format!(
                "Coverage source path beneath {} is not valid UTF-8",
                display_coverage_path(relative)
            )
        })?;
        let child = relative.join(&name);
        ensure_coverage_depth(&child, accumulator.budget.limits.max_depth)?;
        let metadata = directory.symlink_metadata(&name).map_err(|error| {
            format!(
                "Cannot inspect retained coverage source {}: {error}",
                child.display()
            )
        })?;
        if coverage_metadata_is_link(&metadata) {
            accumulator.budget.record_skipped_link(&child);
            continue;
        }
        if metadata.is_dir() {
            if source_root {
                accumulator.immediate_modules.push(name.clone());
            }
            let identity = coverage_metadata_identity(&metadata).map_err(|error| {
                format!(
                    "Cannot identify retained coverage source directory {}: {error}",
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
                "Coverage source {} must be a regular file or directory",
                child.display()
            ));
        }
        let logical_path = accumulator.root.join(&child);
        if !has_coverage_extension(&logical_path, accumulator.config)
            || is_test_file(&logical_path, accumulator.root)
        {
            continue;
        }
        if accumulator.budget.source_files.contains(&child) {
            continue;
        }
        if metadata.len() > accumulator.budget.limits.max_file_bytes {
            return Err(format!(
                "Coverage source file {} exceeds the {} per-file limit",
                child.display(),
                display_coverage_byte_limit(accumulator.budget.limits.max_file_bytes)
            ));
        }
        if metadata.len() > accumulator.budget.remaining_bytes() {
            return Err(format!(
                "Coverage source inputs exceed the {} cumulative limit",
                display_coverage_byte_limit(accumulator.budget.limits.max_input_bytes)
            ));
        }
        let bytes = read_coverage_file(
            directory,
            &name,
            &child,
            accumulator.budget.limits.max_file_bytes,
            accumulator.budget.remaining_bytes(),
            accumulator.budget.limits.max_input_bytes,
        )?;
        let content = std::str::from_utf8(&bytes).map_err(|_| {
            format!(
                "Coverage source file {} is not valid UTF-8",
                child.display()
            )
        })?;
        if accumulator
            .budget
            .charge_source_file(&child, bytes.len() as u64)?
        {
            accumulator.files.push(CoverageSourceFile {
                relative_path: child,
                loc: content.lines().count(),
            });
        }
    }
    after_enumeration(relative);
    for child in children {
        let child_directory = reopen_discovered_coverage_directory(directory, &child)?;
        snapshot_coverage_directory(
            &child_directory,
            &child.relative,
            false,
            accumulator,
            after_enumeration,
        )?;
        verify_coverage_child_directory(directory, &child.name, &child.relative, &child_directory)?;
    }
    Ok(())
}

pub(super) fn read_coverage_entry_names(
    directory: &Dir,
    relative: &Path,
    budget: &mut CoverageTraversalBudget,
) -> Result<Vec<OsString>, String> {
    let entries = directory.read_dir(".").map_err(|error| {
        format!(
            "Cannot read retained coverage source directory {}: {error}",
            display_coverage_path(relative)
        )
    })?;
    let mut names = Vec::new();
    let remaining = budget.remaining_entries();
    for entry in entries {
        let entry = entry.map_err(|error| {
            format!(
                "Cannot inspect retained coverage source beneath {}: {error}",
                display_coverage_path(relative)
            )
        })?;
        names.push(entry.file_name());
        if names.len() > remaining {
            return Err(format!(
                "Coverage traversal exceeds the {}-entry limit",
                budget.limits.max_entries
            ));
        }
    }
    names.sort();
    budget.charge_entries(names.len())?;
    Ok(names)
}

fn coverage_name_is_excluded(name: &OsStr, exclude_dirs: &HashSet<String>) -> bool {
    exclude_dirs
        .iter()
        .any(|excluded| name == OsStr::new(excluded))
}

pub(super) fn ensure_coverage_depth(path: &Path, max_depth: usize) -> Result<(), String> {
    let depth = path
        .components()
        .filter(|component| matches!(component, std::path::Component::Normal(_)))
        .count();
    if depth > max_depth {
        return Err(format!(
            "Coverage source path {} exceeds the {max_depth}-component depth limit",
            path.display()
        ));
    }
    Ok(())
}

pub(super) fn display_coverage_byte_limit(bytes: u64) -> String {
    if bytes.is_multiple_of(1024 * 1024) {
        format!("{} MiB", bytes / (1024 * 1024))
    } else if bytes == 1 {
        "1 byte".to_string()
    } else {
        format!("{bytes} bytes")
    }
}

#[cfg(test)]
#[path = "tests/sources.rs"]
mod tests;
