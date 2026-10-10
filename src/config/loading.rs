//! Filesystem loading, precedence, refusal, and retired local-config diagnostics.

use super::checked::parse_config_content_checked;
use super::discovery::detect_source_dirs;
use super::json::parse_json_config;
use crate::types::SpecSyncConfig;
use std::fs;
use std::path::Path;

pub(crate) const CONFIG_PATH_CANDIDATES: &[&str] = &[
    ".specsync/config.toml",
    ".specsync/config.json",
    ".specsync.toml",
    "specsync.json",
];

/// Load config from TOML or JSON, falling back to defaults.
/// When no config file exists, auto-detects source directories.
///
/// Config file search order (v4 first, then legacy):
/// Read a config file into a String, dropping a leading UTF-8 BOM (U+FEFF). Some
/// editors prepend a BOM; left in place it attaches to the first TOML key (which then
/// becomes an unknown key and is silently ignored) or breaks JSON parsing entirely.
/// Stripping a leading BOM is lossless. Returns None when the file can't be read.
/// Avoids allocating when there is no BOM (the common case).
pub fn read_config_file(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let trimmed = content.trim_start_matches('\u{feff}');
    Some(if trimmed.len() == content.len() {
        content
    } else {
        trimmed.to_string()
    })
}

/// Load config from a specific file path (JSON or TOML based on extension).
/// Used by migration to convert a known source file rather than relying on precedence.
pub fn load_config_from_path(config_path: &Path, root: &Path) -> SpecSyncConfig {
    let ext = config_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match ext {
        "toml" => load_toml_config(config_path, root),
        _ => load_json_config(config_path, root),
    }
}

/// 1. `.specsync/config.toml` (v4 TOML — canonical)
/// 2. `.specsync/config.json` (v4 JSON — pre-TOML migration)
/// 3. `.specsync.toml` (legacy root TOML)
/// 4. `specsync.json` (legacy root JSON)
///
/// After loading the shared config, `.specsync/config.local.toml` is read for
/// migration diagnostics. Retired embedded-AI keys are ignored safely.
/// Load the project configuration, refusing to continue when a config file
/// exists but could not be used.
///
/// This is the default loader precisely because it fails closed. #570 installed
/// the refusal at `commands::load_and_discover`, described in the source as
/// "a single choke point" — which it was not. `load_config` is a second door,
/// and `rules`, `compact` and `rehash` all came through it. `rehash` is the one
/// that mattered: it does not merely report, it WRITES `.specsync/hashes.json`
/// from specs interpreted under default configuration, and that cache is what
/// later `check` runs consult to decide which specs are unchanged and may be
/// skipped. A broken config therefore did not produce one wrong answer; it
/// wrote a stale-skip cache that silently shortened every subsequent run (#583).
///
/// Callers whose job is to REPAIR a broken config need the raw loader and must
/// ask for it by name. That inverts the default: a caller added later gets the
/// guard unless it deliberately opts out, instead of getting the hole unless it
/// remembers to opt in.
pub fn load_config(root: &Path) -> SpecSyncConfig {
    let config = load_config_allowing_unloadable(root);
    crate::commands::refuse_unloadable_config(&config);
    config
}

/// Load the project configuration WITHOUT refusing an unloadable file.
///
/// Only for callers that must keep working when the config is broken: the
/// repair paths, and tests that exercise the fallback itself. Everything else
/// wants [`load_config`].
pub fn load_config_allowing_unloadable(root: &Path) -> SpecSyncConfig {
    let v4_toml = root.join(".specsync/config.toml");
    let v4_json = root.join(".specsync/config.json");
    let legacy_toml = root.join(".specsync.toml");
    let legacy_json = root.join("specsync.json");

    let (mut config, config_path) = if v4_toml.exists() {
        (load_toml_config(&v4_toml, root), Some(v4_toml))
    } else if v4_json.exists() {
        (load_json_config(&v4_json, root), Some(v4_json))
    } else if legacy_toml.exists() {
        (load_toml_config(&legacy_toml, root), Some(legacy_toml))
    } else if legacy_json.exists() {
        (load_json_config(&legacy_json, root), Some(legacy_json))
    } else {
        (
            SpecSyncConfig {
                source_dirs: detect_source_dirs(root),
                ..Default::default()
            },
            None,
        )
    };

    config.config_path = config_path;

    // Read the old local override file only to surface migration guidance.
    let local_toml = root.join(".specsync/config.local.toml");
    if local_toml.exists() {
        inspect_legacy_local_config(&local_toml);
    }

    config
}

/// Inspect a legacy local config file and ignore all retired embedded-AI keys.
///
/// NOTE: this is a second, deliberately minimal line-by-line reader. It handles
/// only SCALAR keys (ai_* and a few strings) — it does NOT support multi-line
/// arrays the way `load_toml_config` does. That is safe today because no array
/// key is read here and `config.local.toml` is never rewritten. If an array key
/// is ever added to local config, route it through the shared array handling
/// (`find_toml_array_close` accumulation) to avoid re-introducing `["["]`
/// corruption.
fn inspect_legacy_local_config(local_path: &Path) {
    let content = match read_config_file(local_path) {
        Some(c) => c,
        None => {
            // Local config is optional, but a file that exists yet cannot be read
            // (not valid UTF-8) must not be silently ignored — its per-developer
            // retired settings would be dropped with no signal.
            if local_path.exists() {
                eprintln!(
                    "Warning: local config {} exists but could not be read (not valid UTF-8 or unreadable); \
                     retired embedded-AI settings are NOT inspected",
                    local_path.display()
                );
            }
            return;
        }
    };

    let mut current_section: Option<String> = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            current_section = Some(line[1..line.len() - 1].trim().to_string());
            continue;
        }

        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            // Only legacy top-level AI fields were supported in local config.
            if current_section.is_some() {
                continue;
            }

            match key {
                key if LEGACY_AI_TOML_KEYS.contains(&key) => warn_retired_ai_key(key),
                _ => {
                    eprintln!("Warning: unknown key \"{key}\" in config.local.toml (ignored)");
                }
            }
        }
    }
}

/// Detect whether this project is using a legacy 3.x layout.
/// Returns true if root-level config files exist without a .specsync/version stamp.
pub fn is_legacy_layout(root: &Path) -> bool {
    let has_root_json = root.join("specsync.json").exists();
    let has_root_toml = root.join(".specsync.toml").exists();
    let has_root_registry = root.join("specsync-registry.toml").exists();
    let has_v4_version = root.join(".specsync/version").exists();

    (has_root_json || has_root_toml || has_root_registry) && !has_v4_version
}

pub(super) const LEGACY_AI_TOML_KEYS: &[&str] = &[
    "ai_provider",
    "ai_model",
    "ai_command",
    "ai_api_key",
    "ai_base_url",
    "ai_timeout",
];

pub(super) fn warn_retired_ai_key(key: &str) {
    eprintln!(
        "Warning: retired embedded-AI config key `{key}` is ignored. spec-sync 5.0 generates deterministic scaffolds; use your coding agent to enrich them."
    );
}

fn load_json_config(config_path: &Path, root: &Path) -> SpecSyncConfig {
    let content = match read_config_file(config_path) {
        Some(c) => c,
        None => {
            // See load_toml_config: a present-but-unreadable config must fail loud
            // rather than silently downgrade enforcement to defaults.
            if config_path.exists() {
                eprintln!(
                    "Warning: config file {} exists but could not be read (not valid UTF-8 or unreadable); \
                     using built-in defaults — its settings (enforcement, required sections, etc.) are NOT applied",
                    config_path.display()
                );
            }
            return SpecSyncConfig {
                load_error: Some(format!(
                    "config file {} exists but could not be loaded; built-in defaults are in use",
                    config_path.display()
                )),
                ..SpecSyncConfig::default()
            };
        }
    };

    match parse_json_config(&content, root) {
        Ok(config) => config,
        Err(error) => {
            eprintln!(
                "Warning: failed to parse {}: {error}",
                config_path.display()
            );
            SpecSyncConfig {
                load_error: Some(format!(
                    "config file {} exists but could not be loaded; built-in defaults are in use",
                    config_path.display()
                )),
                ..SpecSyncConfig::default()
            }
        }
    }
}

/// Parse a TOML config file using zero-dependency parsing.
/// Supports the same fields as specsync.json but with TOML syntax:
///
/// ```toml
/// specs_dir = "specs"
/// source_dirs = ["src", "lib"]
/// schema_dir = "db/migrations"
/// exclude_dirs = ["__tests__"]
/// exclude_patterns = ["**/*.test.ts"]
/// required_sections = ["Purpose", "Public API"]
/// ```
fn load_toml_config(config_path: &Path, root: &Path) -> SpecSyncConfig {
    let content = match read_config_file(config_path) {
        Some(c) => c,
        None => {
            // read_config_file returns None for both "absent" and "present but
            // unreadable". This function is reached only after an `.exists()` check
            // (load_config) or with a known migration source path, so a None on a
            // file that still exists means it could not be read (e.g. not valid
            // UTF-8) — fail loud instead of silently reverting to defaults, which
            // would downgrade enforcement (strict→warn, exit 1→0) with no signal.
            if config_path.exists() {
                eprintln!(
                    "Warning: config file {} exists but could not be read (not valid UTF-8 or unreadable); \
                     using built-in defaults — its settings (enforcement, required sections, etc.) are NOT applied",
                    config_path.display()
                );
            }
            return SpecSyncConfig {
                load_error: Some(format!(
                    "config file {} exists but could not be loaded; built-in defaults are in use",
                    config_path.display()
                )),
                ..SpecSyncConfig::default()
            };
        }
    };

    match parse_config_content_checked(config_path, &content, root) {
        Ok(config) => config,
        Err(error) => {
            eprintln!(
                "Warning: failed to parse {}: {error}",
                config_path.display()
            );
            SpecSyncConfig {
                load_error: Some(format!(
                    "config file {} exists but could not be loaded; built-in defaults are in use",
                    config_path.display()
                )),
                ..SpecSyncConfig::default()
            }
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod legacy_tests;
