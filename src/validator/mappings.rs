use super::SourceSnapshot;
use crate::types::SpecSyncConfig;
use crate::util::levenshtein;
use std::collections::HashMap;
use std::path::Path;

/// Suggest a similar file path when a referenced file doesn't exist.
pub(super) fn suggest_similar_file(root: &Path, missing_file: &str) -> Option<String> {
    let missing_name = Path::new(missing_file).file_name()?.to_str()?;

    let parent = Path::new(missing_file).parent()?;
    let search_dir = root.join(parent);
    if !search_dir.exists() {
        return None;
    }

    let entries = std::fs::read_dir(&search_dir).ok()?;
    let mut best: Option<(String, usize)> = None;

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !entry.path().is_file() {
            continue;
        }
        let dist = levenshtein(missing_name, &name);
        if dist <= 3 && (best.is_none() || dist < best.as_ref().unwrap().1) {
            let suggestion = parent.join(&name).to_string_lossy().replace('\\', "/");
            best = Some((suggestion, dist));
        }
    }

    best.map(|(s, _)| s)
}

/// How many expanded source files a directory-mapping fix names before summarizing.
const DIRECTORY_MAPPING_FIX_LIMIT: usize = 5;

/// Build the actionable fix for a `files:` entry that resolved to a directory.
///
/// Names the source files the entry should have listed, reusing the same
/// directory expansion `generate`/`scaffold` apply to a `[modules."x"] files`
/// directory (`generator::find_module_source_files`), so the remedy matches what
/// generation would have written. Snapshot callers validate retained bytes only
/// and must not enumerate the ambient filesystem, so they get the shape guidance
/// without the file list.
pub(super) fn directory_mapping_fix(
    root: &Path,
    file: &str,
    config: &SpecSyncConfig,
    source_snapshots: Option<&HashMap<String, SourceSnapshot>>,
) -> String {
    let expanded = if source_snapshots.is_none() {
        expand_directory_mapping(root, file, config)
    } else {
        Vec::new()
    };
    if expanded.is_empty() {
        return format!(
            "Replace `{file}` in the files list with the source files beneath it (one entry per file)"
        );
    }
    let shown: Vec<String> = expanded
        .iter()
        .take(DIRECTORY_MAPPING_FIX_LIMIT)
        .cloned()
        .collect();
    let remaining = expanded.len().saturating_sub(shown.len());
    let listed = shown.join(", ");
    if remaining > 0 {
        format!(
            "Replace `{file}` in the files list with the {} source files beneath it: {listed}, and {remaining} more",
            expanded.len()
        )
    } else {
        format!("Replace `{file}` in the files list with: {listed}")
    }
}

/// Expand a project-relative directory into the root-relative source files beneath
/// it, excluding configured exclude directories. Ambient filesystem read: callers
/// must confine the entry first (`source_within_root`).
fn expand_directory_mapping(root: &Path, file: &str, config: &SpecSyncConfig) -> Vec<String> {
    let full_path = root.join(file);
    let mut expanded: Vec<String> =
        crate::generator::find_module_source_files(&full_path, config, root)
            .into_iter()
            .filter_map(|path| {
                let relative = Path::new(&path)
                    .strip_prefix(root)
                    .ok()
                    .map(|relative| relative.to_string_lossy().replace('\\', "/"))?;
                let excluded = Path::new(&relative).components().any(|component| {
                    component
                        .as_os_str()
                        .to_str()
                        .is_some_and(|name| config.exclude_dirs.iter().any(|dir| dir == name))
                });
                (!excluded).then_some(relative)
            })
            .collect();
    expanded.sort();
    expanded.dedup();
    expanded
}

/// Whether a spec `files:` entry resolves to a real path INSIDE the project root.
///
/// Source files a spec validates must live within the project. An entry that
/// escapes — an absolute path, `..` traversal, or a symlink pointing outside the
/// project — is rejected by the caller: reading it would count arbitrary host
/// files as covered and leak their exported identifiers into coverage output and
/// PR comments (a hostile-repo info-disclosure vector, the same threat model as a
/// committed executable configuration).
///
/// For a missing leaf, canonicalizes the nearest existing ancestor so a
/// symlinked parent cannot turn a planned in-project path into a future escape.
/// Returns `false` when the project root or every path ancestor is unreadable.
///
/// Shared: every site that reads a `files:` entry's CONTENT (export extraction in
/// `score`, `check --fix`, `diff`, `new`, …) must gate on this, or the out-of-root
/// identifier leak reopens through that command.
pub(super) fn source_within_root(root: &Path, file: &str) -> bool {
    let full = root.join(file);
    let Ok(canon_root) = root.canonicalize() else {
        return false;
    };
    for candidate in full.ancestors() {
        match candidate.symlink_metadata() {
            Ok(_) => {
                return candidate
                    .canonicalize()
                    .is_ok_and(|canonical| canonical.starts_with(&canon_root));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(_) => return false,
        }
    }
    false
}

/// Whether a not-yet-created source mapping is a portable project-relative path.
pub(super) fn planned_source_path_is_safe(file: &str) -> bool {
    !file.contains('\\') && normalize_source_mapping(file).is_some()
}

/// Normalize a safe project-relative source mapping for ownership and coverage.
pub(super) fn normalize_source_mapping(file: &str) -> Option<String> {
    let path = Path::new(file);
    if file.is_empty()
        || path.is_absolute()
        || file.as_bytes().get(1).is_some_and(|byte| *byte == b':')
    {
        return None;
    }

    let mut components = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::Normal(value) => {
                components.push(value.to_str()?.to_string());
            }
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir
            | std::path::Component::RootDir
            | std::path::Component::Prefix(_) => return None,
        }
    }
    (!components.is_empty()).then(|| components.join("/"))
}

#[cfg(test)]
mod tests;
