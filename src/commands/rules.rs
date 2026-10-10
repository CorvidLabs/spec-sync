use colored::Colorize;
use std::path::Path;

use crate::config::load_config;
use crate::types::{CustomRuleType, RuleSeverity};

pub fn cmd_rules(root: &Path) {
    let config = load_config(root);

    println!("{}", "Built-in rules:".bold());
    println!();
    print_builtin(
        "max_changelog_entries",
        "Warn if Change Log exceeds N entries",
        config.rules.max_changelog_entries.map(|n| n.to_string()),
    );
    print_builtin(
        "require_behavioral_examples",
        "Require at least one ### Scenario",
        config
            .rules
            .require_behavioral_examples
            .map(|b| b.to_string()),
    );
    print_builtin(
        "min_invariants",
        "Require at least N numbered invariants",
        config.rules.min_invariants.map(|n| n.to_string()),
    );
    print_builtin(
        "max_spec_size_kb",
        "Warn if spec file exceeds N KB",
        config.rules.max_spec_size_kb.map(|n| n.to_string()),
    );
    print_builtin(
        "require_depends_on",
        "Require non-empty depends_on",
        config.rules.require_depends_on.map(|b| b.to_string()),
    );
    println!();
    println!("{}", principles_rules_summary(load_principles_file(root)));
    println!();

    if config.custom_rules.is_empty() {
        println!("{}", "No custom rules defined.".dimmed());
        println!(
            "{}",
            "Add \"customRules\" to specsync.json to define declarative rules.".dimmed()
        );
        return;
    }

    println!(
        "{} ({} rule{}):",
        "Custom rules".bold(),
        config.custom_rules.len(),
        if config.custom_rules.len() == 1 {
            ""
        } else {
            "s"
        }
    );
    println!();

    for rule in &config.custom_rules {
        let severity_str = match rule.severity {
            RuleSeverity::Error => "error".red().to_string(),
            RuleSeverity::Warning => "warning".yellow().to_string(),
            RuleSeverity::Info => "info".blue().to_string(),
        };

        let type_str = match rule.rule_type {
            CustomRuleType::RequireSection => "require_section",
            CustomRuleType::MinWordCount => "min_word_count",
            CustomRuleType::RequirePattern => "require_pattern",
            CustomRuleType::ForbidPattern => "forbid_pattern",
        };

        println!("  {} [{}] ({})", rule.name.bold(), severity_str, type_str);

        if let Some(ref section) = rule.section {
            println!("    section: {section}");
        }
        if let Some(ref pattern) = rule.pattern {
            println!("    pattern: {pattern}");
        }
        if let Some(min) = rule.min_words {
            println!("    min_words: {min}");
        }
        if let Some(ref filter) = rule.applies_to {
            let mut parts = Vec::new();
            if let Some(ref s) = filter.status {
                parts.push(format!("status={s}"));
            }
            if let Some(ref m) = filter.module {
                parts.push(format!("module=/{m}/"));
            }
            if !parts.is_empty() {
                println!("    applies_to: {}", parts.join(", "));
            }
        }
        if let Some(ref msg) = rule.message {
            println!("    message: {msg}");
        }
        println!();
    }
}

fn load_principles_file(root: &Path) -> Option<String> {
    crate::change::load_policy(root).and_then(|policy| policy.principles_file)
}

/// What `specsync rules` says about standing principles. Spec-shape rules stay a separate list.
fn principles_rules_summary(principles_file: Option<String>) -> String {
    match principles_file {
        Some(path) => format!(
            "Standing principles:\n  {path}\n  A lesson becomes a standing rule only when a person writes it into this file. SpecSync does not copy lessons here."
        ),
        None => "Standing principles:\n  none configured\n  Set principles_file in .specsync/sdd.json to name a standing principles file.".to_string(),
    }
}

fn print_builtin(name: &str, description: &str, value: Option<String>) {
    let status = match &value {
        Some(v) => format!("{} = {v}", "active".green()),
        None => "off".dimmed().to_string(),
    };
    println!("  {name:.<40} {status}");
    println!("  {}", description.dimmed());
}

#[cfg(test)]
mod tests {
    use super::principles_rules_summary;

    #[test]
    fn rules_names_the_principles_file_and_does_not_invent_one() {
        let text = principles_rules_summary(Some("PRINCIPLES.md".into()));
        assert!(text.contains("PRINCIPLES.md"), "{text}");
        assert!(text.contains("only when a person writes it"), "{text}");
        assert!(text.contains("does not copy lessons"), "{text}");
    }

    #[test]
    fn rules_says_when_no_principles_file_is_configured() {
        let text = principles_rules_summary(None);
        assert!(text.contains("none configured"), "{text}");
        assert!(text.contains("Standing principles:"), "{text}");
    }
}
