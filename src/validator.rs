use crate::schema::SchemaTable;
use crate::types::{SpecSyncConfig, ValidationResult};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use validation::validate_spec_content_internal;
use walkdir::WalkDir;

mod coverage;
mod mappings;
mod schema_checks;
mod validation;

/// Check if a dependency reference is a cross-project reference.
/// Cross-project refs use the format `owner/repo@module` (e.g. `corvid-labs/algochat@auth`).
pub fn is_cross_project_ref(dep: &str) -> bool {
    dep.contains('/') && dep.contains('@')
}

/// Parse a cross-project reference into (owner/repo, module).
/// Returns None if not a valid cross-project ref.
pub fn parse_cross_project_ref(dep: &str) -> Option<(&str, &str)> {
    if !is_cross_project_ref(dep) {
        return None;
    }
    let at_pos = dep.find('@')?;
    let repo = &dep[..at_pos];
    let module = &dep[at_pos + 1..];
    if repo.is_empty() || module.is_empty() {
        return None;
    }
    Some((repo, module))
}

// ─── File Discovery ──────────────────────────────────────────────────────

/// Find all *.spec.md files in a directory recursively.
pub fn find_spec_files(dir: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();
    if !dir.exists() {
        return results;
    }

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file()
            && let Some(name) = path.file_name().and_then(|n| n.to_str())
            && name.ends_with(".spec.md")
        {
            results.push(path.to_path_buf());
        }
    }

    results.sort();
    results
}

// ─── Single Spec Validation ──────────────────────────────────────────────

/// A capability-confined source-file observation supplied by a snapshot caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SourceSnapshot {
    Present(Vec<u8>),
    Missing,
    Rejected,
    Unreadable,
    /// The mapping resolved to a directory inside the project. Kept distinct from
    /// `Rejected` so validation reports the real cause instead of a security escape.
    Directory,
}

/// Validate a single spec file against source code.
pub fn validate_spec(
    spec_path: &Path,
    root: &Path,
    schema_tables: &HashSet<String>,
    schema_columns: &HashMap<String, SchemaTable>,
    config: &SpecSyncConfig,
) -> ValidationResult {
    let content = match fs::read_to_string(spec_path) {
        Ok(content) => content,
        Err(error) => {
            let rel_path = spec_path
                .strip_prefix(root)
                .unwrap_or(spec_path)
                .to_string_lossy()
                .to_string();
            let mut result = ValidationResult::new(rel_path);
            result.errors.push(format!("Cannot read spec: {error}"));
            return result;
        }
    };

    validate_spec_content_internal(
        spec_path,
        &content,
        root,
        schema_tables,
        schema_columns,
        config,
        true,
        None,
    )
}

/// Validate already-read spec content against source code.
///
/// `spec_path` is used only as the logical location for diagnostics. Neither the
/// spec nor adjacent companion files are opened, allowing callers with a confined
/// snapshot to validate without crossing back into ambient path resolution.
#[allow(dead_code)]
pub fn validate_spec_content(
    spec_path: &Path,
    content: &str,
    root: &Path,
    schema_tables: &HashSet<String>,
    schema_columns: &HashMap<String, SchemaTable>,
    config: &SpecSyncConfig,
) -> ValidationResult {
    validate_spec_content_internal(
        spec_path,
        content,
        root,
        schema_tables,
        schema_columns,
        config,
        false,
        None,
    )
}

/// Validate supplied spec bytes and supplied source observations without reopening
/// either through ambient project paths.
pub(crate) fn validate_spec_content_with_sources(
    spec_path: &Path,
    content: &str,
    root: &Path,
    schema_tables: &HashSet<String>,
    schema_columns: &HashMap<String, SchemaTable>,
    config: &SpecSyncConfig,
    sources: &HashMap<String, SourceSnapshot>,
) -> ValidationResult {
    validate_spec_content_internal(
        spec_path,
        content,
        root,
        schema_tables,
        schema_columns,
        config,
        false,
        Some(sources),
    )
}

pub(crate) fn validate_local_dependency(
    dep: &str,
    root: &Path,
    specs_dir: &str,
) -> Result<PathBuf, String> {
    if is_cross_project_ref(dep) {
        // Cross-project refs (e.g. "owner/repo@module") are validated
        // by `specsync resolve`, not during local checks.
        return Ok(root.join(specs_dir));
    }
    let trimmed = dep.trim();
    if trimmed.is_empty() {
        return Err("Dependency spec entry is empty".to_string());
    }
    let as_path = Path::new(trimmed);
    if as_path.is_absolute()
        || as_path.components().any(|c| {
            matches!(
                c,
                std::path::Component::ParentDir | std::path::Component::Prefix(_)
            )
        })
        || trimmed.contains('\\')
    {
        return Err(format!(
            "Dependency spec path escapes the project root: {trimmed} (absolute paths and `..` traversal are not allowed)"
        ));
    }
    // Bare module names (no path separators or extension) resolve
    // against the specs directory, not the project root.
    let full_path = if !trimmed.contains('/') && !trimmed.contains('.') {
        root.join(specs_dir).join(trimmed)
    } else {
        root.join(trimmed)
    };
    if !source_within_root(root, trimmed) {
        return Err(format!(
            "Dependency spec path escapes the project root: {trimmed} (symlink resolution leaves the project)"
        ));
    }
    if !full_path.exists() {
        return Err(format!("Dependency spec not found: {trimmed}"));
    }
    Ok(full_path)
}

/// Extract canonical table names from the ordered schema replay.
pub fn get_schema_table_names(root: &Path, config: &SpecSyncConfig) -> HashSet<String> {
    schema_checks::get_schema_table_names(root, config)
}

pub(crate) fn schema_table_names_from_snapshot(
    snapshot: &crate::schema::SchemaSnapshot,
    config: &SpecSyncConfig,
) -> Result<HashSet<String>, String> {
    schema_checks::schema_table_names_from_snapshot(snapshot, config)
}

pub(crate) fn schema_config_problems_for_snapshot(
    config: &SpecSyncConfig,
    snapshot: Option<&crate::schema::SchemaSnapshot>,
) -> Vec<String> {
    schema_checks::schema_config_problems_for_snapshot(config, snapshot)
}

/// Load CLI discovery inputs through one retained project-root capability.
pub(crate) fn load_config_and_discover_retained(
    root: &Path,
) -> Result<(SpecSyncConfig, Vec<PathBuf>), String> {
    coverage::load_config_and_discover_retained(root)
}

/// Compute file and module coverage.
#[allow(dead_code)]
pub fn compute_coverage(
    root: &Path,
    spec_files: &[PathBuf],
    config: &SpecSyncConfig,
) -> crate::types::CoverageReport {
    coverage::compute_coverage(root, spec_files, config)
}

/// Compute file and module coverage while surfacing malformed manifest inputs.
pub fn compute_coverage_checked(
    root: &Path,
    spec_files: &[PathBuf],
    config: &SpecSyncConfig,
) -> Result<crate::types::CoverageReport, String> {
    coverage::compute_coverage_checked(root, spec_files, config)
}

/// Return whether a source mapping remains beneath the project root.
pub fn source_within_root(root: &Path, file: &str) -> bool {
    mappings::source_within_root(root, file)
}

/// Normalize a safe portable project-relative source mapping.
pub(crate) fn normalize_source_mapping(file: &str) -> Option<String> {
    mappings::normalize_source_mapping(file)
}

#[cfg(test)]
mod tests;
