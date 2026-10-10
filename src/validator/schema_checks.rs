use crate::config::default_schema_pattern;
use crate::schema;
use crate::types::{SpecSyncConfig, ValidationResult};
use crate::util::safe_regex;
use std::collections::{HashMap, HashSet};
use std::path::Path;

// ─── Schema Table Discovery ──────────────────────────────────────────────

/// Extract canonical table names from the ordered schema replay. An explicit
/// schema pattern may supplement replayed names, but cannot restore a table
/// identity retired by DROP TABLE or ALTER TABLE RENAME.
pub(super) fn get_schema_table_names(root: &Path, config: &SpecSyncConfig) -> HashSet<String> {
    // Replay is authoritative; retain the public default-pattern contract as a
    // compile-checked compatibility value without re-scanning CREATE statements.
    debug_assert!(safe_regex(default_schema_pattern()).is_some());

    let schema_dir = match &config.schema_dir {
        Some(d) => root.join(d),
        None => return HashSet::new(),
    };

    let snapshot = match schema::build_schema_snapshot(&schema_dir) {
        Ok(snapshot) => snapshot,
        // The schema could not be replayed. Returning an empty set here would be read by
        // `add_missing_db_table_error` as "these tables do not exist", which is a different
        // claim entirely — see `schema_table_names_available`.
        Err(_) => return HashSet::new(),
    };
    schema_table_names_from_snapshot(&snapshot, config).unwrap_or_default()
}

/// Whether the declared table set is KNOWN, as opposed to merely empty.
///
/// `get_schema_table_names` collapses three outcomes into one empty `HashSet`: a schema that
/// genuinely declares no tables, a schema that failed to replay, and a `schema_pattern` that
/// failed to compile. Only the first justifies telling a user their table is missing.
///
/// Conflating them is how one unparseable migration reports EVERY declared table as absent,
/// including tables created correctly in an unrelated file. A parse failure must degrade to
/// "unknown", never to "not there".
pub(super) fn schema_table_names_available(root: &Path, config: &SpecSyncConfig) -> bool {
    let Some(schema_dir) = config.schema_dir.as_ref().map(|dir| root.join(dir)) else {
        return false;
    };
    let Ok(snapshot) = schema::build_schema_snapshot(&schema_dir) else {
        return false;
    };
    schema_table_names_from_snapshot(&snapshot, config).is_ok()
}

pub(super) fn schema_table_names_from_snapshot(
    snapshot: &schema::SchemaSnapshot,
    config: &SpecSyncConfig,
) -> Result<HashSet<String>, String> {
    let mut tables: HashSet<String> = snapshot.tables.keys().cloned().collect();
    if let Some(pattern) = config.schema_pattern.as_deref() {
        let regex = safe_regex(pattern).ok_or_else(|| {
            "Invalid `schema_pattern`: the regex could not be compiled within safety limits"
                .to_string()
        })?;
        if regex.captures_len() < 2 {
            return Err(
                "Invalid `schema_pattern`: it must contain a capture group for the table name"
                    .to_string(),
            );
        }
        tables.extend(
            snapshot
                .pattern_table_names(&regex)
                .map_err(|error| format!("Invalid `schema_pattern` result: {error}"))?,
        );
    }
    Ok(tables)
}

pub(super) fn schema_config_problems_for_snapshot(
    config: &SpecSyncConfig,
    snapshot: Option<&schema::SchemaSnapshot>,
) -> Vec<String> {
    let Some(pattern) = config.schema_pattern.as_deref() else {
        return Vec::new();
    };

    let mut problems = Vec::new();
    let regex = match safe_regex(pattern) {
        Some(regex) if regex.captures_len() >= 2 => Some(regex),
        Some(_) => {
            problems.push(
                "Invalid `schema_pattern`: it must contain a capture group for the table name"
                    .to_string(),
            );
            None
        }
        None => {
            problems.push(
                "Invalid `schema_pattern`: the regex could not be compiled within safety limits"
                    .to_string(),
            );
            None
        }
    };

    let has_schema_dir = match &config.schema_dir {
        Some(_) => true,
        None => {
            problems.push(
                "`schema_pattern` is configured but `schema_dir` is not configured; DB schema validation cannot scan for tables"
                    .to_string(),
            );
            false
        }
    };

    if has_schema_dir
        && let (Some(regex), Some(snapshot)) = (regex, snapshot)
        && let Err(error) = snapshot.pattern_table_names(&regex)
    {
        problems.push(format!("Invalid `schema_pattern` result: {error}"));
    }

    problems
}

pub(super) fn schema_table_exists(
    declaration: &str,
    schema_tables: &HashSet<String>,
) -> Result<bool, String> {
    for discovered in schema_tables {
        if schema::table_reference_matches(declaration, discovered)? {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(super) fn resolve_schema_value<'a, Value>(
    declaration: &str,
    values: &'a HashMap<String, Value>,
) -> Result<Option<(&'a str, &'a Value)>, String> {
    let canonical_declaration = schema::canonicalize_table_name(declaration)?;
    let mut exact = Vec::new();
    let mut compatible = Vec::new();

    for (name, value) in values {
        let canonical_name = schema::canonicalize_table_name(name)?;
        if canonical_name == canonical_declaration {
            exact.push((name.as_str(), value));
        } else if schema::table_reference_matches(&canonical_declaration, &canonical_name)? {
            compatible.push((name.as_str(), value));
        }
    }

    let candidates = if exact.is_empty() { compatible } else { exact };
    if candidates.len() > 1 {
        let mut names: Vec<&str> = candidates.iter().map(|(name, _)| *name).collect();
        names.sort_unstable();
        return Err(format!(
            "DB table reference `{declaration}` is ambiguous across schema tables: {}",
            names.join(", ")
        ));
    }

    Ok(candidates.into_iter().next())
}

pub(super) fn validate_db_table_identifier(
    table: &str,
    result: &mut ValidationResult,
) -> Result<(), String> {
    match schema::canonicalize_table_name(table) {
        Ok(_) => Ok(()),
        Err(error) => {
            result
                .errors
                .push(format!("Invalid DB table identifier `{table}`: {error}"));
            result.fixes.push(format!(
                "Use a bare, quoted, or schema-qualified SQL identifier instead of `{table}`"
            ));
            Err(error)
        }
    }
}

pub(super) fn add_missing_db_table_error(table: &str, result: &mut ValidationResult) {
    result
        .errors
        .push(format!("DB table not found in schema: {table}"));
    result.fixes.push(format!(
        "Remove `{table}` from db_tables or add a CREATE TABLE migration"
    ));
}

pub(super) fn add_schema_resolution_error(error: String, result: &mut ValidationResult) {
    if !result.errors.contains(&error) {
        result.errors.push(error);
    }
}

#[cfg(test)]
mod tests;
