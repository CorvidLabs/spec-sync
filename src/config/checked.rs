//! Validation for repair paths and retained exact-byte configuration snapshots.

use super::json::parse_json_config_with_source_dirs;
use super::toml_config::{parse_toml_config, parse_toml_config_with_source_dirs};
use crate::types::SpecSyncConfig;
use std::fs;
use std::path::Path;

/// Validate that an existing configuration file is readable, syntactically
/// valid, and gives known path fields their documented shapes.
///
/// This intentionally does not reject unknown extension keys. It only prevents
/// `init --repair` from claiming success while a malformed or wrongly typed
/// canonical config remains unusable.
pub(crate) fn validate_config_file(path: &Path) -> Result<(), String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("failed to read config {}: {error}", path.display()))?;
    let is_json = path.extension().and_then(|extension| extension.to_str()) == Some("json");
    if is_json {
        let value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|error| format!("invalid JSON config {}: {error}", path.display()))?;
        let object = value.as_object().ok_or_else(|| {
            format!(
                "invalid JSON config {}: root must be an object",
                path.display()
            )
        })?;
        if let Some(specs_dir) = object.get("specsDir").or_else(|| object.get("specs_dir")) {
            required_nonempty_config_string(specs_dir.as_str(), path, "specsDir")?;
        }
        if let Some(source_dirs) = object
            .get("sourceDirs")
            .or_else(|| object.get("source_dirs"))
        {
            validate_config_string_array(source_dirs, path, "sourceDirs")?;
        }
        return Ok(());
    }

    let value: toml::Value = toml::from_str(&content)
        .map_err(|error| format!("invalid TOML config {}: {error}", path.display()))?;
    let table = value.as_table().ok_or_else(|| {
        format!(
            "invalid TOML config {}: root must be a table",
            path.display()
        )
    })?;
    if let Some(specs_dir) = table.get("specs_dir") {
        required_nonempty_config_string(specs_dir.as_str(), path, "specs_dir")?;
    }
    if let Some(source_dirs) = table.get("source_dirs") {
        validate_config_string_array(source_dirs, path, "source_dirs")?;
    }
    Ok(())
}

fn required_nonempty_config_string(
    value: Option<&str>,
    path: &Path,
    field: &str,
) -> Result<(), String> {
    match value {
        Some(value) if !value.trim().is_empty() => Ok(()),
        Some(_) => Err(format!(
            "invalid config {}: `{field}` must not be empty",
            path.display()
        )),
        None => Err(format!(
            "invalid config {}: `{field}` must be a string",
            path.display()
        )),
    }
}

fn validate_config_string_array(
    value: &impl ConfigArrayValue,
    path: &Path,
    field: &str,
) -> Result<(), String> {
    let Some(items) = value.config_array() else {
        return Err(format!(
            "invalid config {}: `{field}` must be an array of strings",
            path.display()
        ));
    };
    if items.iter().all(|item| item.config_string().is_some()) {
        Ok(())
    } else {
        Err(format!(
            "invalid config {}: `{field}` must contain only strings",
            path.display()
        ))
    }
}

trait ConfigArrayValue {
    type Item: ConfigStringValue;

    fn config_array(&self) -> Option<&Vec<Self::Item>>;
}

trait ConfigStringValue {
    fn config_string(&self) -> Option<&str>;
}

impl ConfigArrayValue for serde_json::Value {
    type Item = serde_json::Value;

    fn config_array(&self) -> Option<&Vec<Self::Item>> {
        self.as_array()
    }
}

impl ConfigStringValue for serde_json::Value {
    fn config_string(&self) -> Option<&str> {
        self.as_str()
    }
}

impl ConfigArrayValue for toml::Value {
    type Item = toml::Value;

    fn config_array(&self) -> Option<&Vec<Self::Item>> {
        self.as_array()
    }
}

impl ConfigStringValue for toml::Value {
    fn config_string(&self) -> Option<&str> {
        self.as_str()
    }
}

/// Parse a retained configuration snapshot without reopening its pathname.
///
/// The caller is responsible for obtaining the bytes from a trusted, bounded
/// filesystem handle. This function validates and parses those exact bytes while
/// preserving the normal JSON/TOML compatibility behavior.
pub(crate) fn parse_config_content_checked(
    config_path: &Path,
    content: &str,
    root: &Path,
) -> Result<SpecSyncConfig, String> {
    parse_config_content_checked_with_source_dirs(config_path, content, root, None)
}

/// Parse a retained configuration snapshot with caller-supplied source discovery.
///
/// Security-sensitive capability callers use this to avoid consulting an ambient
/// root pathname when the config omits its source directory list.
pub(crate) fn parse_config_content_checked_with_source_dirs(
    config_path: &Path,
    content: &str,
    root: &Path,
    detected_source_dirs: Option<Vec<String>>,
) -> Result<SpecSyncConfig, String> {
    let content = content.trim_start_matches('\u{feff}');
    if content.trim().is_empty() {
        return Err("config file is empty".to_string());
    }
    let is_toml = config_path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension == "toml");
    let mut config = if is_toml {
        let table = toml::from_str::<toml::Table>(content).map_err(|error| error.to_string())?;
        validate_toml_config_types(&table)?;
        if let Some(dirs) = detected_source_dirs.as_deref() {
            parse_toml_config_with_source_dirs(content, root, Some(dirs))
        } else {
            parse_toml_config(content, root)
        }
    } else {
        let value = serde_json::from_str::<serde_json::Value>(content)
            .map_err(|error| error.to_string())?;
        validate_checked_json_config_types(&value)?;
        parse_json_config_with_source_dirs(content, root, detected_source_dirs.as_deref())
            .map_err(|error| error.to_string())?
    };
    config.config_path = Some(config_path.to_path_buf());
    Ok(config)
}

fn validate_checked_json_config_types(value: &serde_json::Value) -> Result<(), String> {
    let root = value
        .as_object()
        .ok_or_else(|| "configuration root must be an object".to_string())?;
    let Some(github) = root.get("github") else {
        return Ok(());
    };
    let github = github
        .as_object()
        .ok_or_else(|| "`github` must be an object".to_string())?;
    if github
        .get("repo")
        .is_some_and(|repo| !repo.is_null() && !repo.is_string())
    {
        return Err("`github.repo` must be a string or null".to_string());
    }
    Ok(())
}

fn validate_toml_config_types(table: &toml::Table) -> Result<(), String> {
    for key in [
        "specs_dir",
        "schema_dir",
        "schema_pattern",
        "export_level",
        "parse_mode",
        "parseMode",
        "enforcement",
    ] {
        validate_toml_field(table, key, "a string", toml::Value::is_str)?;
    }
    if let Some(enforcement) = table.get("enforcement").and_then(toml::Value::as_str)
        && !matches!(
            enforcement,
            "strict" | "warn" | "enforce-new" | "enforce_new"
        )
    {
        return Err(format!(
            "unknown enforcement \"{enforcement}\" (expected \"warn\", \"enforce-new\", or \"strict\")"
        ));
    }
    for key in [
        "source_dirs",
        "required_sections",
        "exclude_dirs",
        "exclude_patterns",
        "source_extensions",
    ] {
        validate_toml_field(table, key, "an array of strings", is_toml_string_array)?;
    }
    for key in ["include_extensionless", "require_draft_files"] {
        validate_toml_field(table, key, "a boolean", toml::Value::is_bool)?;
    }
    validate_toml_field(
        table,
        "task_archive_days",
        "an integer",
        toml::Value::is_integer,
    )?;

    if let Some(github) = validate_toml_table(table, "github")? {
        validate_toml_field(github, "repo", "a string", toml::Value::is_str)?;
        validate_toml_field(
            github,
            "drift_labels",
            "an array of strings",
            is_toml_string_array,
        )?;
        validate_toml_field(github, "verify_issues", "a boolean", toml::Value::is_bool)?;
    }
    if let Some(rules) = validate_toml_table(table, "rules")? {
        for key in [
            "max_changelog_entries",
            "min_invariants",
            "max_spec_size_kb",
        ] {
            validate_toml_field(rules, key, "an integer", toml::Value::is_integer)?;
        }
        for key in ["require_behavioral_examples", "require_depends_on"] {
            validate_toml_field(rules, key, "a boolean", toml::Value::is_bool)?;
        }
    }
    if let Some(companions) = validate_toml_table(table, "companions")? {
        validate_toml_field(companions, "design", "a boolean", toml::Value::is_bool)?;
    }
    if let Some(lifecycle) = validate_toml_table(table, "lifecycle")? {
        for key in ["track_history", "trackHistory"] {
            validate_toml_field(lifecycle, key, "a boolean", toml::Value::is_bool)?;
        }
        for key in ["allowed_statuses", "allowedStatuses"] {
            validate_toml_field(lifecycle, key, "an array of strings", is_toml_string_array)?;
        }
        for key in ["max_age", "maxAge"] {
            if let Some(max_age) = validate_toml_table(lifecycle, key)? {
                for (status, value) in max_age {
                    if !value.is_integer() {
                        return Err(format!("`lifecycle.{key}.{status}` must be an integer"));
                    }
                }
            }
        }
        if let Some(guards) = validate_toml_table(lifecycle, "guards")? {
            for (transition, value) in guards {
                let guard = value
                    .as_table()
                    .ok_or_else(|| format!("`lifecycle.guards.{transition}` must be a table"))?;
                for key in ["min_score", "minScore", "stale_threshold", "staleThreshold"] {
                    validate_toml_field(guard, key, "an integer", toml::Value::is_integer)?;
                }
                for key in ["no_stale", "noStale"] {
                    validate_toml_field(guard, key, "a boolean", toml::Value::is_bool)?;
                }
                for key in ["require_sections", "requireSections"] {
                    validate_toml_field(guard, key, "an array of strings", is_toml_string_array)?;
                }
                validate_toml_field(guard, "message", "a string", toml::Value::is_str)?;
            }
        }
    }
    if let Some(modules) = validate_toml_table(table, "modules")? {
        for (name, value) in modules {
            let module = value
                .as_table()
                .ok_or_else(|| format!("`modules.{name}` must be a table"))?;
            for key in ["files", "depends_on", "dependsOn", "owns"] {
                validate_toml_field(module, key, "an array of strings", is_toml_string_array)?;
            }
        }
    }

    Ok(())
}

fn validate_toml_table<'a>(
    table: &'a toml::Table,
    key: &str,
) -> Result<Option<&'a toml::Table>, String> {
    table
        .get(key)
        .map(|value| {
            value
                .as_table()
                .ok_or_else(|| format!("`{key}` must be a table"))
        })
        .transpose()
}

fn validate_toml_field(
    table: &toml::Table,
    key: &str,
    expected: &str,
    predicate: fn(&toml::Value) -> bool,
) -> Result<(), String> {
    if table.get(key).is_some_and(|value| !predicate(value)) {
        return Err(format!("`{key}` must be {expected}"));
    }
    Ok(())
}

fn is_toml_string_array(value: &toml::Value) -> bool {
    value
        .as_array()
        .is_some_and(|items| items.iter().all(toml::Value::is_str))
}

#[cfg(test)]
mod tests;
