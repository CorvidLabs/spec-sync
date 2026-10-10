//! Checked MCP issue verification and bounded, content-free diagnostics.

use super::{load_confined_config, read_file_bounded, validate_spec_file_mappings};
use crate::types::SpecSyncConfig;
use serde_json::{Value, json};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const MAX_ISSUE_DIAGNOSTIC_PATH_CHARS: usize = 240;

pub(super) fn tool_issues(root: &Path) -> Result<Value, String> {
    use crate::github;

    let (config, references) = discover_issue_references(root)?;

    let mut results: Vec<Value> = Vec::new();
    let mut total_valid = 0usize;
    let mut total_closed = 0usize;
    let mut total_not_found = 0usize;

    if references.is_empty() {
        return Ok(json!({
            "repo": null,
            "total_valid": 0,
            "total_closed": 0,
            "total_not_found": 0,
            "specs": results,
        }));
    }

    let repo = config
        .github
        .as_ref()
        .and_then(|github| github.repo.as_deref())
        .filter(|repo| !repo.trim().is_empty())
        .ok_or_else(|| {
            "MCP specsync_issues requires an explicit `github.repo`; Git metadata auto-detection is disabled by the server security boundary".to_string()
        })?
        .to_string();

    for verification in github::verify_issue_batch(&repo, &references) {
        let rel_path = verification.spec_path.clone();
        ensure_issue_verification_complete(&rel_path, &verification)?;

        total_valid += verification.valid.len();
        total_closed += verification.closed.len();
        total_not_found += verification.not_found.len();

        results.push(json!({
            "spec": rel_path,
            "valid": verification.valid.iter().map(|i| json!({
                "number": i.number,
                "title": i.title,
                "state": i.state,
            })).collect::<Vec<_>>(),
            "closed": verification.closed.iter().map(|i| json!({
                "number": i.number,
                "title": i.title,
            })).collect::<Vec<_>>(),
            "not_found": verification.not_found,
        }));
    }

    Ok(json!({
        "repo": repo,
        "total_valid": total_valid,
        "total_closed": total_closed,
        "total_not_found": total_not_found,
        "specs": results,
    }))
}

type IssueReferences = (String, Vec<u64>, Vec<u64>);

fn is_issue_spec_file_name(name: &OsStr) -> Result<bool, ()> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;

        let bytes = name.as_bytes();
        if !bytes.ends_with(b".spec.md") {
            return Ok(false);
        }
        if name.to_str().is_none() {
            return Err(());
        }
        Ok(!bytes.starts_with(b"_"))
    }

    #[cfg(not(unix))]
    {
        Ok(name
            .to_str()
            .is_some_and(|name| name.ends_with(".spec.md") && !name.starts_with('_')))
    }
}

fn find_issue_spec_files_checked(root: &Path, specs_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut spec_files = Vec::new();
    for result in WalkDir::new(specs_dir)
        .follow_links(false)
        .sort_by_file_name()
    {
        let entry = result.map_err(|error| {
            let rel_path = issue_relative_spec_path(root, error.path().unwrap_or(specs_dir));
            format!(
                "MCP specsync_issues issue discovery is inconclusive for {rel_path}: spec directory entry could not be inspected"
            )
        })?;
        let is_spec = is_issue_spec_file_name(entry.file_name()).map_err(|()| {
            let rel_path = issue_relative_spec_path(root, entry.path());
            format!(
                "MCP specsync_issues issue discovery is inconclusive for {rel_path}: spec filename is not valid UTF-8"
            )
        })?;
        if !is_spec {
            continue;
        }
        if !entry.file_type().is_file() {
            let rel_path = issue_relative_spec_path(root, entry.path());
            return Err(format!(
                "MCP specsync_issues issue discovery is inconclusive for {rel_path}: spec path is not a regular file"
            ));
        }
        spec_files.push(entry.into_path());
    }
    spec_files.sort();
    Ok(spec_files)
}

fn discover_issue_references(
    root: &Path,
) -> Result<(SpecSyncConfig, Vec<IssueReferences>), String> {
    let config = load_confined_config(root).map_err(|error| {
        format!(
            "MCP specsync_issues issue discovery is inconclusive: {}",
            issue_project_error_reason(&error)
        )
    })?;
    let specs_dir = root.join(&config.specs_dir);
    let spec_files = find_issue_spec_files_checked(root, &specs_dir)?;

    if spec_files.is_empty() {
        return Err(
            "MCP specsync_issues issue discovery is inconclusive: no spec files were found"
                .to_string(),
        );
    }

    let mut references = Vec::new();
    for spec_path in &spec_files {
        let rel_path = issue_relative_spec_path(root, spec_path);
        let content = read_file_bounded(root, spec_path, "spec file").map_err(|error| {
            format!(
                "MCP specsync_issues issue discovery is inconclusive for {rel_path}: {}",
                issue_spec_read_error_reason(&error)
            )
        })?;
        let (implements, tracks) = crate::parser::parse_checked_issue_references(&content)
            .map_err(|reason| {
                format!(
                    "MCP specsync_issues issue discovery is inconclusive for {rel_path}: {reason}"
                )
            })?;

        if !implements.is_empty() || !tracks.is_empty() {
            references.push((rel_path, implements, tracks));
        }
    }

    validate_spec_file_mappings(root, &spec_files, &config.exclude_dirs).map_err(|error| {
        format!(
            "MCP specsync_issues issue discovery is inconclusive: {}",
            issue_project_error_reason(&error)
        )
    })?;

    Ok((config, references))
}

fn is_unsafe_issue_diagnostic_character(character: char) -> bool {
    character.is_control()
        || matches!(
            character,
            '\u{061c}'
                | '\u{200e}'
                | '\u{200f}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202a}'..='\u{202e}'
                | '\u{2066}'..='\u{2069}'
        )
}

fn issue_relative_spec_path(root: &Path, spec_path: &Path) -> String {
    let Some(relative) = spec_path.strip_prefix(root).ok() else {
        return "<spec>".to_string();
    };
    let relative = relative.to_string_lossy();
    let mut sanitized = String::with_capacity(relative.len());
    for character in relative.chars() {
        if is_unsafe_issue_diagnostic_character(character) {
            sanitized.push_str(&format!("\\u{{{:04X}}}", character as u32));
        } else if cfg!(windows) && character == '\\' {
            sanitized.push('/');
        } else {
            sanitized.push(character);
        }
    }
    if sanitized.chars().count() <= MAX_ISSUE_DIAGNOSTIC_PATH_CHARS {
        return sanitized;
    }

    let mut bounded: String = sanitized
        .chars()
        .take(MAX_ISSUE_DIAGNOSTIC_PATH_CHARS.saturating_sub(3))
        .collect();
    bounded.push_str("...");
    bounded
}

fn issue_spec_read_error_reason(error: &str) -> &'static str {
    if error.contains("not valid UTF-8") {
        "spec file is not valid UTF-8"
    } else if error.contains("exceeds") {
        "spec file exceeds the configured size limit"
    } else if error.contains("not a file") {
        "spec path is not a regular file"
    } else if error.contains("Cannot inspect") {
        "spec file metadata could not be inspected"
    } else if error.contains("Cannot read") {
        "spec file could not be read"
    } else {
        "spec path failed safety validation"
    }
}

fn issue_project_error_reason(error: &str) -> &'static str {
    if error.contains("exceeds") || error.contains("limit") {
        "project input exceeds a configured safety limit"
    } else if error.contains("Cannot read") || error.contains("not valid UTF-8") {
        "project metadata could not be read"
    } else if error.contains("Cannot inspect") {
        "project metadata could not be inspected"
    } else if error.contains("escape")
        || error.contains("outside")
        || error.contains("symlink")
        || error.contains("junction")
        || error.contains("relative path")
    {
        "project input failed confinement validation"
    } else {
        "project discovery failed safety validation"
    }
}

fn ensure_issue_verification_complete(
    spec_path: &str,
    verification: &crate::github::IssueVerification,
) -> Result<(), String> {
    if verification.errors.is_empty() {
        return Ok(());
    }
    Err(format!(
        "GitHub issue verification was inconclusive for {spec_path}: {}",
        verification.errors.join("; ")
    ))
}

#[cfg(test)]
mod tests;
