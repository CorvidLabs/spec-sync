use crate::parser::{body_has_section, find_section_offset};
use crate::types::{CustomRuleType, Frontmatter, RuleSeverity, SpecSyncConfig, ValidationResult};
use crate::util::safe_regex;
use regex::Regex;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

static NUMBERED_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^\d+\.\s+\S").unwrap());

pub(super) fn validate_companion_scaffold_markers(
    spec_path: &Path,
    root: &Path,
    result: &mut ValidationResult,
) {
    const MARKERS: &[(&str, &str, &str)] = &[
        (
            "context.md",
            "<!-- Describe the context and motivation for this module. -->",
            "replace the generated context marker with concrete motivation",
        ),
        (
            "context.md",
            "- Record architectural or design decisions relevant to this spec.",
            "record concrete architectural decisions",
        ),
        (
            "context.md",
            "- List the most important files an agent or new developer should read.",
            "list concrete files to read",
        ),
        (
            "context.md",
            "- Summarize implemented behavior, active work, and known blockers.",
            "summarize the current implementation status",
        ),
        (
            "requirements.md",
            "- As a developer, I want to <!-- describe the goal -->",
            "replace the generated user-story marker",
        ),
        (
            "requirements.md",
            "- <!-- List measurable acceptance criteria. -->",
            "list measurable acceptance criteria",
        ),
        (
            "requirements.md",
            "- Define acceptance criteria from the module's source behavior and user-facing responsibilities.",
            "define concrete acceptance criteria",
        ),
        (
            "testing.md",
            "- <!-- List unit test scenarios. -->",
            "list concrete automated test scenarios",
        ),
        (
            "testing.md",
            "List the automated tests and fixtures that protect this module.",
            "list concrete automated tests and fixtures",
        ),
        (
            "testing.md",
            "List manual QA flows, platform checks, and review notes for this module.",
            "record concrete manual QA flows",
        ),
        (
            "tasks.md",
            "- [ ] Add implementation, validation, or release tasks that belong to this spec.",
            "replace the generated task with concrete work or remove it",
        ),
        (
            "design.md",
            "- Document layout structure, responsive breakpoints, and positioning rules.",
            "document concrete layout behavior",
        ),
        (
            "design.md",
            "- Document component tree, inputs, outputs, and slots.",
            "document the concrete component hierarchy",
        ),
        (
            "design.md",
            "- Document color, spacing, typography, and state token overrides.",
            "document concrete design tokens",
        ),
        (
            "design.md",
            "- List icons, images, illustrations, and asset ownership.",
            "list concrete assets and ownership",
        ),
    ];
    let Some(directory) = spec_path.parent() else {
        return;
    };
    for (file_name, marker, correction) in MARKERS {
        let path = directory.join(file_name);
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        let mut fence: Option<char> = None;
        for (index, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            let fence_character = if trimmed.starts_with("```") {
                Some('`')
            } else if trimmed.starts_with("~~~") {
                Some('~')
            } else {
                None
            };
            if let Some(character) = fence_character {
                if fence == Some(character) {
                    fence = None;
                } else if fence.is_none() {
                    fence = Some(character);
                }
                continue;
            }
            if fence.is_none() && trimmed == *marker {
                let relative = path
                    .strip_prefix(root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/");
                result.warnings.push(format!(
                    "Unfilled companion scaffold marker at {relative}:{} — {correction}",
                    index + 1
                ));
            }
        }
    }
}

/// Apply project-specific custom validation rules from config.
pub(super) fn apply_custom_rules(
    _spec_path: &Path,
    body: &str,
    fm: &Frontmatter,
    config: &SpecSyncConfig,
    config_hint: &str,
    result: &mut ValidationResult,
) {
    let rules = &config.rules;

    // max_spec_size_kb check is handled by the shared validation core to avoid duplicate warnings

    // max_changelog_entries: warn if Change Log has too many rows
    if let Some(max_entries) = rules.max_changelog_entries {
        let count = count_changelog_entries(body);
        if count > max_entries {
            result.warnings.push(format!(
                "Change Log has {count} entries — exceeds limit of {max_entries} (run `specsync compact`)"
            ));
        }
    }

    // require_behavioral_examples: require at least one ### Scenario
    if rules.require_behavioral_examples == Some(true) {
        let scenario_count = body.matches("### Scenario").count();
        if scenario_count == 0 {
            result.errors.push(
                "No behavioral examples found (rule: require_behavioral_examples)".to_string(),
            );
            result.fixes.push(
                "Add at least one `### Scenario:` under `## Behavioral Examples`".to_string(),
            );
        }
    }

    // min_invariants: require a minimum number of numbered invariants
    if let Some(min) = rules.min_invariants {
        let count = count_invariants(body);
        if count < min {
            result.warnings.push(format!(
                "Only {count} invariant(s) found — minimum is {min}"
            ));
        }
    }

    // require_depends_on: require non-empty depends_on in frontmatter
    if rules.require_depends_on == Some(true) && fm.depends_on.is_empty() {
        result
            .warnings
            .push("No consumed dependencies documented (rule: require_depends_on)".to_string());
    }

    // ─── Declarative Custom Rules ────────────────────────────────────
    for rule in &config.custom_rules {
        if !custom_rule_applies(rule, fm) {
            continue;
        }
        if let Some(msg) = evaluate_custom_rule(rule, body) {
            let tagged = format!("{msg} (rule: {}){config_hint}", rule.name);
            match rule.severity {
                RuleSeverity::Error => result.errors.push(tagged),
                RuleSeverity::Warning => result.warnings.push(tagged),
                RuleSeverity::Info => result.warnings.push(format!("[info] {tagged}")),
            }
        }
    }
}

/// Check whether a custom rule applies to the given spec based on its filter.
fn custom_rule_applies(rule: &crate::types::CustomRule, fm: &Frontmatter) -> bool {
    let Some(ref filter) = rule.applies_to else {
        return true;
    };

    if let Some(ref status) = filter.status {
        let spec_status = fm.status.as_deref().unwrap_or("");
        if !spec_status.eq_ignore_ascii_case(status) {
            return false;
        }
    }

    if let Some(ref module_pattern) = filter.module {
        let spec_module = fm.module.as_deref().unwrap_or("");
        if let Some(re) = safe_regex(module_pattern)
            && !re.is_match(spec_module)
        {
            return false;
        }
    }

    true
}

/// Evaluate a single custom rule against the spec body.
/// Returns `Some(message)` if the rule is violated, `None` if it passes.
fn evaluate_custom_rule(rule: &crate::types::CustomRule, body: &str) -> Option<String> {
    match rule.rule_type {
        CustomRuleType::RequireSection => {
            let section = rule.section.as_deref()?;
            if !body_has_section(body, section) {
                let msg = rule
                    .message
                    .clone()
                    .unwrap_or_else(|| format!("Missing required section: ## {section}"));
                return Some(msg);
            }
            None
        }
        CustomRuleType::MinWordCount => {
            let section = rule.section.as_deref()?;
            let min = rule.min_words.unwrap_or(1);
            let header = format!("## {section}");
            let section_start = find_section_offset(body, section)?;
            let after_header = &body[section_start + header.len()..];
            // Bound section to next ## heading
            let section_end = after_header.find("\n## ").unwrap_or(after_header.len());
            let section_body = &after_header[..section_end];
            let word_count = section_body.split_whitespace().count();
            if word_count < min {
                let msg = rule.message.clone().unwrap_or_else(|| {
                    format!("Section ## {section} has {word_count} words — minimum is {min}")
                });
                return Some(msg);
            }
            None
        }
        CustomRuleType::RequirePattern => {
            let pattern = rule.pattern.as_deref()?;
            let re = safe_regex(pattern)?;
            if !re.is_match(body) {
                let msg = rule
                    .message
                    .clone()
                    .unwrap_or_else(|| format!("Required pattern not found: {pattern}"));
                return Some(msg);
            }
            None
        }
        CustomRuleType::ForbidPattern => {
            let pattern = rule.pattern.as_deref()?;
            let re = safe_regex(pattern)?;
            if re.is_match(body) {
                let msg = rule
                    .message
                    .clone()
                    .unwrap_or_else(|| format!("Forbidden pattern found: {pattern}"));
                return Some(msg);
            }
            None
        }
    }
}

/// Count data rows in the Change Log table (excluding header and separator).
fn count_changelog_entries(body: &str) -> usize {
    let changelog_start = match find_section_offset(body, "Change Log") {
        Some(pos) => pos,
        None => return 0,
    };
    let section = &body[changelog_start..];
    // Find next ## heading to bound the section
    let section_end = section[1..]
        .find("\n## ")
        .map(|p| p + 1)
        .unwrap_or(section.len());
    let section = &section[..section_end];

    // Count data rows: skip the first two table lines (header + separator)
    let mut table_line_count = 0usize;
    section
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            if !trimmed.starts_with('|') {
                return false;
            }
            table_line_count += 1;
            table_line_count > 2
        })
        .count()
}

/// Count numbered invariants in the Invariants section.
fn count_invariants(body: &str) -> usize {
    let inv_start = match body.find("## Invariants") {
        Some(pos) => pos,
        None => return 0,
    };
    let section = &body[inv_start..];
    let section_end = section[1..]
        .find("\n## ")
        .map(|p| p + 1)
        .unwrap_or(section.len());
    let section = &section[..section_end];

    NUMBERED_RE.find_iter(section).count()
}

#[cfg(test)]
#[path = "tests/policy.rs"]
mod tests;
