//! Legacy JSON compatibility, including explicit invalid-repository preservation.

use super::discovery::detect_source_dirs;
use super::loading::warn_retired_ai_key;
use crate::types::SpecSyncConfig;
use std::path::Path;

/// Known config keys in specsync.json (camelCase).
const KNOWN_JSON_KEYS: &[&str] = &[
    "specsDir",
    "sourceDirs",
    "schemaDir",
    "schemaPattern",
    "requiredSections",
    "excludeDirs",
    "excludePatterns",
    "sourceExtensions",
    "includeExtensionless",
    "requireDraftFiles",
    "exportLevel",
    "parseMode",
    "modules",
    "rules",
    "customRules",
    "taskArchiveDays",
    "github",
    "enforcement",
    "lifecycle",
    "companions",
];

const LEGACY_AI_JSON_KEYS: &[&str] = &[
    "aiProvider",
    "aiModel",
    "aiCommand",
    "aiApiKey",
    "aiBaseUrl",
    "aiTimeout",
];

const INVALID_JSON_GITHUB_REPO: &str = "invalid-github-repo-config";

fn fail_closed_invalid_json_github_repo(raw: &mut serde_json::Value) {
    let Some(github) = raw
        .as_object_mut()
        .and_then(|config| config.get_mut("github"))
    else {
        return;
    };
    let Some(repo) = github
        .as_object_mut()
        .and_then(|github| github.get_mut("repo"))
    else {
        return;
    };
    if repo.is_null() || repo.is_string() {
        return;
    }

    eprintln!(
        "Warning: failed to parse specsync.json: `github.repo` must be a string or null; \
         repository resolution will fail closed"
    );
    *repo = serde_json::Value::String(INVALID_JSON_GITHUB_REPO.to_string());
}

pub(super) fn parse_json_config(
    content: &str,
    root: &Path,
) -> Result<SpecSyncConfig, serde_json::Error> {
    parse_json_config_with_source_dirs(content, root, None)
}

pub(super) fn parse_json_config_with_source_dirs(
    content: &str,
    root: &Path,
    detected_source_dirs: Option<&[String]>,
) -> Result<SpecSyncConfig, serde_json::Error> {
    let mut raw = serde_json::from_str::<serde_json::Value>(content)?;
    let source_dirs_configured = raw
        .as_object()
        .is_some_and(|config| config.contains_key("sourceDirs"));

    // Warn about unknown keys.
    if let Some(obj) = raw.as_object() {
        for key in obj.keys() {
            if LEGACY_AI_JSON_KEYS.contains(&key.as_str()) {
                warn_retired_ai_key(key);
            } else if !KNOWN_JSON_KEYS.contains(&key.as_str()) {
                eprintln!("Warning: unknown key \"{key}\" in specsync.json (ignored)");
            }
        }
    }
    fail_closed_invalid_json_github_repo(&mut raw);

    let mut config = serde_json::from_value::<SpecSyncConfig>(raw)?;
    // `source_dirs_set` is `#[serde(skip)]`, so it is false after the value
    // deserialization above whatever the file said. Record what the file
    // actually stated: once loading is done, a configured `["src"]` and the
    // `["src"]` default are the same list (#723).
    config.source_dirs_set = source_dirs_configured;
    if !source_dirs_configured {
        config.source_dirs = detected_source_dirs
            .map(<[String]>::to_vec)
            .unwrap_or_else(|| detect_source_dirs(root));
    }
    Ok(config)
}

#[cfg(test)]
mod tests;
