//! Canonical TOML output and migration loss checks.

use super::toml_syntax::{toml_array_line, toml_escape};
use crate::types::SpecSyncConfig;

/// Serialize a SpecSyncConfig to TOML format string.
pub fn config_to_toml(config: &SpecSyncConfig) -> String {
    let mut lines: Vec<String> = Vec::new();
    lines.push("# spec-sync configuration".to_string());
    lines.push("# Docs: https://github.com/CorvidLabs/spec-sync".to_string());
    lines.push(String::new());

    // Core settings
    lines.push(format!(
        "specs_dir = \"{}\"",
        toml_escape(&config.specs_dir)
    ));

    if !config.source_dirs.is_empty() {
        lines.push(format!(
            "source_dirs = [{}]",
            config
                .source_dirs
                .iter()
                .map(|s| format!("\"{}\"", toml_escape(s)))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    if let Some(ref schema_dir) = config.schema_dir {
        lines.push(format!("schema_dir = \"{}\"", toml_escape(schema_dir)));
    }
    if let Some(ref schema_pattern) = config.schema_pattern {
        lines.push(format!(
            "schema_pattern = \"{}\"",
            toml_escape(schema_pattern)
        ));
    }

    // These three fields default to a NON-empty value in the reader
    // (SpecSyncConfig::default), so an explicitly-empty value (a user opting out of
    // section enforcement / test-file exclusion) must be written as `= []` — omitting
    // it would silently restore the defaults on reload, flipping check/coverage
    // results on migrate. So they are always emitted, empty or not.
    lines.push(toml_array_line("exclude_dirs", &config.exclude_dirs));
    lines.push(toml_array_line(
        "exclude_patterns",
        &config.exclude_patterns,
    ));
    // source_extensions defaults to empty, so omitting it when empty round-trips.
    if !config.source_extensions.is_empty() {
        lines.push(toml_array_line(
            "source_extensions",
            &config.source_extensions,
        ));
    }
    if config.include_extensionless {
        lines.push("include_extensionless = true".to_string());
    }
    if config.require_draft_files {
        lines.push("require_draft_files = true".to_string());
    }

    lines.push(toml_array_line(
        "required_sections",
        &config.required_sections,
    ));

    // Export level
    match config.export_level {
        crate::types::ExportLevel::Type => lines.push("export_level = \"type\"".to_string()),
        crate::types::ExportLevel::Member => {} // default, omit
    }

    // Parse mode (AST is opt-in; regex is the default, so omit it)
    match config.parse_mode {
        crate::types::ParseMode::Ast => lines.push("parse_mode = \"ast\"".to_string()),
        crate::types::ParseMode::Regex => {} // default, omit
    }

    // Enforcement — written ALWAYS, never omitted as "the default".
    //
    // This used to skip `Warn` with the comment "default, omit". `Warn` has not
    // been the default since the default moved to `Strict`, so `migrate` dropped
    // an explicit `enforcement = "warn"` and the project silently became gating
    // on the next load. The adopter's CI went red with nothing in the diff to
    // explain it: the file did not GAIN a strict line, it LOST a warn line, and
    // the person debugging looks for something added.
    //
    // Omitting on equality with a default is only safe while the default never
    // moves, and an absent key is byte-identical to a key holding the default,
    // so nothing downstream can tell the two apart afterwards. Stating the value
    // costs one line and cannot rot.
    lines.push(format!(
        "enforcement = \"{}\"",
        match config.enforcement {
            crate::types::EnforcementMode::Warn => "warn",
            crate::types::EnforcementMode::EnforceNew => "enforce-new",
            crate::types::EnforcementMode::Strict => "strict",
        }
    ));

    if let Some(days) = config.task_archive_days {
        lines.push(format!("task_archive_days = {days}"));
    }

    // Rules section
    let rules = &config.rules;
    let has_rules = rules.max_changelog_entries.is_some()
        || rules.require_behavioral_examples.is_some()
        || rules.min_invariants.is_some()
        || rules.max_spec_size_kb.is_some()
        || rules.require_depends_on.is_some();

    if has_rules {
        lines.push(String::new());
        lines.push("[rules]".to_string());
        if let Some(n) = rules.max_changelog_entries {
            lines.push(format!("max_changelog_entries = {n}"));
        }
        if let Some(b) = rules.require_behavioral_examples {
            lines.push(format!("require_behavioral_examples = {b}"));
        }
        if let Some(n) = rules.min_invariants {
            lines.push(format!("min_invariants = {n}"));
        }
        if let Some(n) = rules.max_spec_size_kb {
            lines.push(format!("max_spec_size_kb = {n}"));
        }
        if let Some(b) = rules.require_depends_on {
            lines.push(format!("require_depends_on = {b}"));
        }
    }

    // GitHub section
    if let Some(ref gh) = config.github {
        lines.push(String::new());
        lines.push("[github]".to_string());
        if let Some(ref repo) = gh.repo {
            lines.push(format!("repo = \"{}\"", toml_escape(repo)));
        }
        if !gh.drift_labels.is_empty() {
            lines.push(format!(
                "drift_labels = [{}]",
                gh.drift_labels
                    .iter()
                    .map(|s| format!("\"{}\"", toml_escape(s)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !gh.verify_issues {
            lines.push(format!("verify_issues = {}", gh.verify_issues));
        }
    }

    // Lifecycle section
    let lc = &config.lifecycle;
    let has_lifecycle = !lc.guards.is_empty()
        || !lc.track_history
        || !lc.max_age.is_empty()
        || !lc.allowed_statuses.is_empty();

    if has_lifecycle {
        lines.push(String::new());
        lines.push("[lifecycle]".to_string());
        if !lc.track_history {
            lines.push(format!("track_history = {}", lc.track_history));
        }
        if !lc.allowed_statuses.is_empty() {
            lines.push(format!(
                "allowed_statuses = [{}]",
                lc.allowed_statuses
                    .iter()
                    .map(|s| format!("\"{}\"", toml_escape(s)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !lc.max_age.is_empty() {
            lines.push(String::new());
            lines.push("[lifecycle.max_age]".to_string());
            for (status, days) in &lc.max_age {
                lines.push(format!("{status} = {days}"));
            }
        }
        if !lc.guards.is_empty() {
            for (transition, guard) in &lc.guards {
                lines.push(String::new());
                lines.push(format!("[lifecycle.guards.\"{transition}\"]"));
                if let Some(score) = guard.min_score {
                    lines.push(format!("min_score = {score}"));
                }
                if !guard.require_sections.is_empty() {
                    lines.push(format!(
                        "require_sections = [{}]",
                        guard
                            .require_sections
                            .iter()
                            .map(|s| format!("\"{}\"", toml_escape(s)))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
                if let Some(no_stale) = guard.no_stale {
                    lines.push(format!("no_stale = {no_stale}"));
                }
                if let Some(threshold) = guard.stale_threshold {
                    lines.push(format!("stale_threshold = {threshold}"));
                }
                if let Some(ref msg) = guard.message {
                    lines.push(format!("message = \"{}\"", toml_escape(msg)));
                }
            }
        }
    }

    // Companions section
    if config.companions.design {
        lines.push(String::new());
        lines.push("[companions]".to_string());
        lines.push("design = true".to_string());
    }

    // Module definitions — one `[modules."name"]` table each. Sorted by name so the
    // output is deterministic (HashMap order is not) for clean diffs and stable tests.
    if !config.modules.is_empty() {
        let mut names: Vec<&String> = config.modules.keys().collect();
        names.sort();
        for name in names {
            let module = &config.modules[name];
            // A module with no files, no deps, and no owned paths carries no data and
            // would not round-trip (the reader materializes a module only from a key
            // line), so skip it to keep the writer and reader symmetric.
            if module.files.is_empty() && module.depends_on.is_empty() && module.owns.is_empty() {
                continue;
            }
            lines.push(String::new());
            lines.push(format!("[modules.\"{}\"]", toml_escape(name)));
            if !module.files.is_empty() {
                lines.push(format!(
                    "files = [{}]",
                    module
                        .files
                        .iter()
                        .map(|s| format!("\"{}\"", toml_escape(s)))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            if !module.depends_on.is_empty() {
                lines.push(format!(
                    "depends_on = [{}]",
                    module
                        .depends_on
                        .iter()
                        .map(|s| format!("\"{}\"", toml_escape(s)))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            if !module.owns.is_empty() {
                lines.push(format!(
                    "owns = [{}]",
                    module
                        .owns
                        .iter()
                        .map(|s| format!("\"{}\"", toml_escape(s)))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }
    }

    lines.push(String::new()); // trailing newline
    lines.join("\n")
}

/// Config fields that `config_to_toml` cannot faithfully serialize (and that the
/// hand-rolled TOML reader cannot parse back), so converting a config that uses
/// them would silently drop data. `migrate` calls this to refuse rather than lose
/// a user's config on JSON→TOML conversion.
///
/// Keep this in lockstep with `config_to_toml`: when a field gains real TOML
/// round-trip support above, remove it here.
pub fn config_to_toml_lossy_fields(config: &SpecSyncConfig) -> Vec<&'static str> {
    let mut fields = Vec::new();
    if !config.custom_rules.is_empty() {
        // `[[custom_rules]]` array-of-tables (with a nested `applies_to` filter) has
        // no writer above and no support in the hand-rolled reader.
        fields.push("customRules");
    }
    fields
}

#[cfg(test)]
mod tests;
