//! Compatibility TOML section dispatch after checked syntax validation.

use super::discovery::detect_source_dirs;
use super::loading::{LEGACY_AI_TOML_KEYS, warn_retired_ai_key};
use super::toml_syntax::{
    find_toml_array_close, parse_toml_bool, parse_toml_string, parse_toml_string_array,
    strip_inline_comment, toml_unescape,
};
use crate::types::SpecSyncConfig;
use std::path::Path;

pub(super) fn parse_toml_config(content: &str, root: &Path) -> SpecSyncConfig {
    parse_toml_config_with_source_dirs(content, root, None)
}

pub(super) fn parse_toml_config_with_source_dirs(
    content: &str,
    root: &Path,
    detected_source_dirs: Option<&[String]>,
) -> SpecSyncConfig {
    let mut config = SpecSyncConfig::default();
    let mut has_source_dirs = false;
    let mut current_section: Option<String> = None;
    let mut malformed: Vec<String> = Vec::new();

    // Peekable so a value that opens a multi-line array can consume its
    // continuation lines while leaving a following key/section un-consumed if the
    // array turns out to be unterminated.
    let mut lines = content.lines().peekable();
    #[allow(clippy::while_let_on_iterator)]
    while let Some(raw_line) = lines.next() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Track TOML section headers like [rules]
        if line.starts_with('[') && line.ends_with(']') {
            current_section = Some(line[1..line.len() - 1].trim().to_string());
            continue;
        }

        // A line that opens a header and never closes it is malformed TOML, and
        // this reader is the ONLY place that can say so on this path: it is a
        // hand-rolled line scanner, so it has no parser to raise an error and
        // silently skipped anything it did not recognise. That silence is why a
        // typo in `[rules]` disabled every rule while `check` reported success
        // (#583). The test is deliberately narrow — an unterminated header is
        // unambiguous — rather than "anything this scanner cannot parse", which
        // would reject valid TOML the scanner simply does not implement.
        // Array continuation lines never reach here; they are consumed by the
        // multi-line array branch below.
        if line.starts_with('[') {
            malformed.push(line.to_string());
            continue;
        }

        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            // A value may open a multi-line array (`key = [` with items and the
            // closing `]` on following lines). Accumulate continuation lines until
            // the array's bracket is balanced — otherwise the value is just `[`,
            // which parses to `["["]`, silently dropping the real entries AND
            // (because `migrate` reads then rewrites the config) overwriting the
            // user's file with the corrupted result.
            let raw_value = line[eq_pos + 1..].trim();
            let is_array = raw_value.starts_with('[');
            // Strip an inline `#` comment (quote/escape-aware) from the value. This
            // applies to SCALARS too: `specs_dir = "specs" # note` must parse as
            // `specs`, not the literal `"specs" # note` — otherwise the specs dir
            // mis-resolves and every spec becomes invisible, silently turning
            // `check` into an exit-0 pass. A `#` inside a quoted string is preserved.
            let mut acc = strip_inline_comment(raw_value);
            // Close detection is quote/escape/bracket-depth aware (see
            // `find_toml_array_close`), so a `]` INSIDE a quoted item — e.g. a glob
            // char-class `"**/[abc]/**"` — does not falsely end the array.
            if is_array && find_toml_array_close(&acc).is_none() {
                while let Some(&peeked) = lines.peek() {
                    let cont = strip_inline_comment(peeked.trim());
                    if cont.is_empty() {
                        lines.next(); // consume blank / whole-line comment in array
                        continue;
                    }
                    // Array items are quoted strings — basic `"..."` or literal
                    // `'...'` — and the close is `]`. A continuation line that is none
                    // of these means the `]` was omitted — stop WITHOUT consuming it so
                    // the following key/section still parses, bounding a malformed
                    // array's damage to its own key rather than swallowing the rest of
                    // the file. (These configs never use nested arrays, so a leading
                    // `[` is a following section header, not array content — it must
                    // NOT be absorbed.)
                    if !cont.starts_with('"') && !cont.starts_with('\'') && !cont.starts_with(']') {
                        break;
                    }
                    lines.next();
                    acc.push(' ');
                    acc.push_str(&cont);
                    if find_toml_array_close(&acc).is_some() {
                        break;
                    }
                }
            }
            let value = acc.as_str();

            // Route to section-specific parsing
            if let Some(ref section) = current_section {
                match section.as_str() {
                    "rules" => {
                        parse_toml_rules_key(key, value, &mut config.rules);
                        continue;
                    }
                    "github" => {
                        parse_toml_github_key(key, value, &mut config);
                        continue;
                    }
                    "lifecycle" => {
                        parse_toml_lifecycle_key(key, value, &mut config.lifecycle);
                        continue;
                    }
                    "companions" => {
                        parse_toml_companions_key(key, value, &mut config.companions);
                        continue;
                    }
                    s if s.starts_with("lifecycle.") => {
                        parse_toml_lifecycle_nested(s, key, value, &mut config.lifecycle);
                        continue;
                    }
                    s if s.starts_with("modules.") => {
                        parse_toml_modules_nested(s, key, value, &mut config.modules);
                        continue;
                    }
                    _ => {
                        // Unknown section — skip silently
                        continue;
                    }
                }
            }

            match key {
                "specs_dir" => config.specs_dir = parse_toml_string(value),
                "source_dirs" => {
                    config.source_dirs = parse_toml_string_array(value);
                    has_source_dirs = true;
                }
                "schema_dir" => config.schema_dir = Some(parse_toml_string(value)),
                "schema_pattern" => config.schema_pattern = Some(parse_toml_string(value)),
                "exclude_dirs" => config.exclude_dirs = parse_toml_string_array(value),
                "exclude_patterns" => config.exclude_patterns = parse_toml_string_array(value),
                "source_extensions" => config.source_extensions = parse_toml_string_array(value),
                "include_extensionless" => config.include_extensionless = parse_toml_bool(value),
                "require_draft_files" => config.require_draft_files = parse_toml_bool(value),
                key if LEGACY_AI_TOML_KEYS.contains(&key) => warn_retired_ai_key(key),
                "export_level" => {
                    let s = parse_toml_string(value);
                    match s.as_str() {
                        "type" => {
                            config.export_level = crate::types::ExportLevel::Type;
                        }
                        "member" => {
                            config.export_level = crate::types::ExportLevel::Member;
                        }
                        _ => eprintln!(
                            "Warning: unknown export_level \"{s}\" (expected \"type\" or \"member\")"
                        ),
                    }
                }
                "parse_mode" | "parseMode" => {
                    let s = parse_toml_string(value);
                    match s.as_str() {
                        "ast" => config.parse_mode = crate::types::ParseMode::Ast,
                        "regex" => config.parse_mode = crate::types::ParseMode::Regex,
                        _ => eprintln!(
                            "Warning: unknown parse_mode \"{s}\" (expected \"regex\" or \"ast\")"
                        ),
                    }
                }
                "required_sections" => {
                    config.required_sections = parse_toml_string_array(value);
                }
                "task_archive_days" => {
                    if let Ok(n) = value.trim().parse::<u32>() {
                        config.task_archive_days = Some(n);
                    }
                }
                "enforcement" => {
                    let s = parse_toml_string(value);
                    // Mark enforcement as explicitly configured: gate commands
                    // honor an explicit `warn` (exit 0), while an UNSET
                    // enforcement gates on errors (see default_enforcement).
                    config.enforcement_set = true;
                    match s.as_str() {
                        "strict" => {
                            config.enforcement = crate::types::EnforcementMode::Strict;
                        }
                        "enforce-new" | "enforce_new" => {
                            config.enforcement = crate::types::EnforcementMode::EnforceNew;
                        }
                        "warn" => {
                            config.enforcement = crate::types::EnforcementMode::Warn;
                        }
                        _ => eprintln!(
                            "Warning: unknown enforcement \"{s}\" (expected \"warn\", \"enforce-new\", or \"strict\")"
                        ),
                    }
                }
                _ => {
                    eprintln!("Warning: unknown key \"{key}\" in config.toml (ignored)");
                }
            }
        }
    }

    // What the file actually stated, recorded before any fallback overwrites
    // the list: a configured `["src"]` is indistinguishable from the `["src"]`
    // default afterwards, and coverage has to tell them apart (#723).
    config.source_dirs_set = has_source_dirs;
    if !has_source_dirs {
        config.source_dirs = detected_source_dirs
            .map(<[String]>::to_vec)
            .unwrap_or_else(|| detect_source_dirs(root));
    }

    if !malformed.is_empty() {
        // Record it rather than printing and continuing. `load_error` is what
        // `refuse_unloadable_config` reads, so recording is what turns a
        // silently-ignored typo into a refusal — the settings this file was
        // written to apply are NOT in effect, and the defaults standing in for
        // them are not the project's choice.
        // Same wording as the unreadable-file refusal above, deliberately. Two
        // shapes of one failure — a file that cannot be read and a file that
        // cannot be parsed — should read the same way to whoever hits them, and
        // a consumer matching on the refusal should not have to know which door
        // it came through. The specific cause follows the shared sentence.
        config.load_error = Some(format!(
            "config file exists but could not be loaded; built-in defaults are in use \
             ({} malformed line(s), first is {:?})",
            malformed.len(),
            malformed[0],
        ));
    }

    config
}

/// Parse a key=value pair inside a `[rules]` TOML section.
fn parse_toml_rules_key(key: &str, value: &str, rules: &mut crate::types::ValidationRules) {
    match key {
        "max_changelog_entries" => {
            if let Ok(n) = value.trim().parse::<usize>() {
                rules.max_changelog_entries = Some(n);
            }
        }
        "require_behavioral_examples" => {
            rules.require_behavioral_examples = Some(parse_toml_bool(value));
        }
        "min_invariants" => {
            if let Ok(n) = value.trim().parse::<usize>() {
                rules.min_invariants = Some(n);
            }
        }
        "max_spec_size_kb" => {
            if let Ok(n) = value.trim().parse::<usize>() {
                rules.max_spec_size_kb = Some(n);
            }
        }
        "require_depends_on" => {
            rules.require_depends_on = Some(parse_toml_bool(value));
        }
        _ => {
            eprintln!("Warning: unknown rule \"{key}\" in [rules] section (ignored)");
        }
    }
}

/// Parse a key=value pair inside a `[github]` TOML section.
fn parse_toml_github_key(key: &str, value: &str, config: &mut SpecSyncConfig) {
    let gh = config
        .github
        .get_or_insert_with(|| crate::types::GitHubConfig {
            repo: None,
            drift_labels: vec!["spec-drift".to_string()],
            verify_issues: true,
        });

    match key {
        "repo" => gh.repo = Some(parse_toml_string(value)),
        "drift_labels" => gh.drift_labels = parse_toml_string_array(value),
        "verify_issues" => gh.verify_issues = parse_toml_bool(value),
        _ => {
            eprintln!("Warning: unknown key \"{key}\" in [github] section (ignored)");
        }
    }
}

/// Parse a key=value pair inside a `[companions]` TOML section.
fn parse_toml_companions_key(
    key: &str,
    value: &str,
    companions: &mut crate::types::CompanionConfig,
) {
    match key {
        "design" => companions.design = parse_toml_bool(value),
        _ => {
            eprintln!("Warning: unknown key \"{key}\" in [companions] section (ignored)");
        }
    }
}

/// Parse a key=value pair inside a `[modules."name"]` TOML section (the form
/// `config_to_toml` writes). The module name is the quoted segment after
/// `modules.`; a name containing `.` is preserved (the whole remainder is the key).
fn parse_toml_modules_nested(
    section: &str,
    key: &str,
    value: &str,
    modules: &mut std::collections::HashMap<String, crate::types::ModuleDefinition>,
) {
    let Some(raw_name) = section.strip_prefix("modules.") else {
        return;
    };
    // The name is a quoted basic string in the header (`[modules."name"]`); strip
    // the quotes and decode escapes so a name with a quote/backslash round-trips.
    let trimmed = raw_name.trim();
    let inner = trimmed.strip_prefix('"').unwrap_or(trimmed);
    let inner = inner.strip_suffix('"').unwrap_or(inner);
    let name = toml_unescape(inner);
    if name.is_empty() {
        return;
    }
    let module = modules.entry(name).or_default();
    match key {
        "files" => module.files = parse_toml_string_array(value),
        "depends_on" | "dependsOn" => module.depends_on = parse_toml_string_array(value),
        "owns" => module.owns = parse_toml_string_array(value),
        _ => {
            eprintln!("Warning: unknown key \"{key}\" in [modules] section (ignored)");
        }
    }
}

/// Parse a key=value pair inside a `[lifecycle]` TOML section.
///
/// Accepts both snake_case (what `init`/`config_to_toml` write) and the camelCase
/// form documented in `--help` and the README's JSON example, so a config written
/// either way is honored rather than silently ignored.
fn parse_toml_lifecycle_key(key: &str, value: &str, lc: &mut crate::types::LifecycleConfig) {
    match key {
        "track_history" | "trackHistory" => lc.track_history = parse_toml_bool(value),
        "allowed_statuses" | "allowedStatuses" => {
            lc.allowed_statuses = parse_toml_string_array(value)
        }
        _ => {
            eprintln!("Warning: unknown key \"{key}\" in [lifecycle] section (ignored)");
        }
    }
}

/// Parse a key=value pair inside nested lifecycle sections like `[lifecycle.max_age]`
/// or `[lifecycle.guards."review→active"]`. Accepts the documented camelCase section
/// and key names as aliases for their snake_case equivalents.
fn parse_toml_lifecycle_nested(
    section: &str,
    key: &str,
    value: &str,
    lc: &mut crate::types::LifecycleConfig,
) {
    if section == "lifecycle.max_age" || section == "lifecycle.maxAge" {
        // Keys here are status names (e.g. "draft"), so they are NOT normalized.
        if let Ok(days) = value.trim().parse::<u64>() {
            lc.max_age.insert(key.to_string(), days);
        }
    } else if let Some(guard_name) = section.strip_prefix("lifecycle.guards.") {
        // Strip surrounding quotes from guard name if present
        let name = guard_name.trim_matches('"').to_string();
        let guard = lc.guards.entry(name).or_default();
        match key {
            "min_score" | "minScore" => {
                if let Ok(n) = value.trim().parse::<u32>() {
                    guard.min_score = Some(n);
                }
            }
            "require_sections" | "requireSections" => {
                guard.require_sections = parse_toml_string_array(value)
            }
            "no_stale" | "noStale" => guard.no_stale = Some(parse_toml_bool(value)),
            "stale_threshold" | "staleThreshold" => {
                if let Ok(n) = value.trim().parse::<usize>() {
                    guard.stale_threshold = Some(n);
                }
            }
            "message" => guard.message = Some(parse_toml_string(value)),
            _ => {
                eprintln!("Warning: unknown key \"{key}\" in [lifecycle.guards] section (ignored)");
            }
        }
    } else {
        // An unknown lifecycle subsection (e.g. a typo'd `[lifecycle.maxAgee]`) would
        // otherwise be silently dropped — warn so a mis-scoped gate isn't a no-op.
        eprintln!("Warning: unknown lifecycle section \"[{section}]\" (ignored)");
    }
}

#[cfg(test)]
mod tests;
