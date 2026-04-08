use crate::exports::get_exported_symbols;
use crate::parser::{get_missing_sections, get_spec_symbols, parse_frontmatter};
use crate::types::SpecSyncConfig;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Quality score for a single spec file.
#[derive(Debug)]
pub struct SpecScore {
    pub spec_path: String,
    /// Frontmatter completeness (0-20).
    pub frontmatter_score: u32,
    /// Required sections present (0-20).
    pub sections_score: u32,
    /// API documentation coverage (0-20).
    pub api_score: u32,
    /// Content depth — sections have real content, not just TODOs (0-20).
    pub depth_score: u32,
    /// Freshness — files exist, no stale references (0-20).
    pub freshness_score: u32,
    /// Overall score (0-100).
    pub total: u32,
    /// Letter grade.
    pub grade: &'static str,
    /// Actionable suggestions for improvement.
    pub suggestions: Vec<String>,
}

/// Score a single spec file.
pub fn score_spec(spec_path: &Path, root: &Path, config: &SpecSyncConfig) -> SpecScore {
    let rel_path = spec_path
        .strip_prefix(root)
        .unwrap_or(spec_path)
        .to_string_lossy()
        .to_string();

    let mut score = SpecScore {
        spec_path: rel_path,
        frontmatter_score: 0,
        sections_score: 0,
        api_score: 0,
        depth_score: 0,
        freshness_score: 0,
        total: 0,
        grade: "F",
        suggestions: Vec::new(),
    };

    let content = match fs::read_to_string(spec_path) {
        Ok(c) => c.replace("\r\n", "\n"),
        Err(_) => {
            score.suggestions.push("Cannot read spec file".to_string());
            return score;
        }
    };

    let parsed = match parse_frontmatter(&content) {
        Some(p) => p,
        None => {
            score
                .suggestions
                .push("Add YAML frontmatter with --- delimiters".to_string());
            return score;
        }
    };

    let fm = &parsed.frontmatter;
    let body = &parsed.body;

    // ─── Frontmatter (0-20) ──────────────────────────────────────────
    let mut fm_points = 0u32;
    let mut fm_missing: Vec<&str> = Vec::new();
    if fm.module.is_some() {
        fm_points += 5;
    } else {
        fm_missing.push("module (-5pts)");
    }
    if fm.version.is_some() {
        fm_points += 5;
    } else {
        fm_missing.push("version (-5pts)");
    }
    if fm.status.is_some() {
        fm_points += 4;
    } else {
        fm_missing.push("status (-4pts)");
    }
    if !fm.files.is_empty() {
        fm_points += 6;
    } else {
        fm_missing.push("files (-6pts)");
    }
    score.frontmatter_score = fm_points;
    if !fm_missing.is_empty() {
        let lost = 20 - fm_points;
        score.suggestions.push(format!(
            "Frontmatter (-{lost}pts): missing {}",
            fm_missing.join(", ")
        ));
    }

    // ─── Sections (0-20) ─────────────────────────────────────────────
    let missing = get_missing_sections(body, &config.required_sections);
    let present = config.required_sections.len() - missing.len();
    let total_sections = config.required_sections.len();
    score.sections_score = if total_sections == 0 {
        20
    } else {
        ((present as f64 / total_sections as f64) * 20.0).round() as u32
    };
    if !missing.is_empty() {
        let lost = 20 - score.sections_score;
        let names = missing
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        let suffix = if missing.len() > 3 {
            format!(" (+{} more)", missing.len() - 3)
        } else {
            String::new()
        };
        score.suggestions.push(format!(
            "Sections (-{lost}pts): missing ## {names}{suffix}"
        ));
    }

    // ─── API Coverage (0-20) ─────────────────────────────────────────
    if !fm.files.is_empty() {
        let mut all_exports: Vec<String> = Vec::new();
        for file in &fm.files {
            let full_path = root.join(file);
            all_exports.extend(get_exported_symbols(&full_path));
        }
        let mut seen = HashSet::new();
        all_exports.retain(|s| seen.insert(s.clone()));

        let spec_symbols = get_spec_symbols(body);
        let export_set: HashSet<&str> = all_exports.iter().map(|s| s.as_str()).collect();

        let documented = spec_symbols
            .iter()
            .filter(|s| export_set.contains(s.as_str()))
            .count();

        if all_exports.is_empty() {
            score.api_score = 20; // No exports to document
        } else {
            score.api_score =
                ((documented as f64 / all_exports.len() as f64) * 20.0).round() as u32;
            let undocumented = all_exports.len() - documented;
            if undocumented > 0 {
                let lost = 20 - score.api_score;
                let undoc_names: Vec<&str> = all_exports
                    .iter()
                    .filter(|s| !spec_symbols.iter().any(|ss| ss == *s))
                    .take(5)
                    .map(|s| s.as_str())
                    .collect();
                let names_str = undoc_names.join("`, `");
                let suffix = if undocumented > 5 {
                    format!(" (+{} more)", undocumented - 5)
                } else {
                    String::new()
                };
                score.suggestions.push(format!(
                    "API coverage (-{lost}pts): {undocumented} undocumented export(s) — `{names_str}`{suffix}"
                ));
            }
        }
    } else {
        score.api_score = 0;
    }

    // ─── Content Depth (0-20) ────────────────────────────────────────
    let mut depth_points = 0u32;
    let todo_count = count_placeholder_todos(body);
    let placeholder_count = body.matches("<!-- ").count();

    // Check each required section has meaningful content
    let sections_with_content = count_sections_with_content(body, &config.required_sections);
    let content_ratio = if config.required_sections.is_empty() {
        1.0
    } else {
        sections_with_content as f64 / config.required_sections.len() as f64
    };
    depth_points += (content_ratio * 14.0).round() as u32;

    // Penalize TODOs
    if todo_count == 0 && placeholder_count == 0 {
        depth_points += 6;
    } else if todo_count <= 2 {
        depth_points += 3;
    } else {
        score.suggestions.push(format!(
            "Content depth: fill in {todo_count} TODO placeholder(s) with real content"
        ));
    }
    score.depth_score = depth_points.min(20);
    if score.depth_score < 20 {
        let lost = 20 - score.depth_score;
        let filled = sections_with_content;
        let total_req = config.required_sections.len();
        if filled < total_req {
            score.suggestions.push(format!(
                "Content depth (-{lost}pts): only {filled}/{total_req} sections have meaningful content"
            ));
        }
    }

    // ─── Freshness (0-20) ────────────────────────────────────────────
    let mut fresh_points = 20u32;
    let mut stale_files = 0u32;
    for file in &fm.files {
        if !root.join(file).exists() {
            stale_files += 1;
        }
    }
    if stale_files > 0 {
        let penalty = (stale_files * 5).min(15);
        fresh_points = fresh_points.saturating_sub(penalty);
        score.suggestions.push(format!(
            "Freshness (-{penalty}pts): {stale_files} file(s) in frontmatter don't exist"
        ));
    }

    // Check depends_on references
    let mut stale_deps = 0u32;
    for dep in &fm.depends_on {
        if !root.join(dep).exists() {
            stale_deps += 1;
        }
    }
    if stale_deps > 0 {
        let dep_penalty = stale_deps * 3;
        fresh_points = fresh_points.saturating_sub(dep_penalty);
        score.suggestions.push(format!(
            "Freshness (-{dep_penalty}pts): {stale_deps} depends_on path(s) don't exist"
        ));
    }
    score.freshness_score = fresh_points;

    // ─── Total & Grade ───────────────────────────────────────────────
    score.total = score.frontmatter_score
        + score.sections_score
        + score.api_score
        + score.depth_score
        + score.freshness_score;

    score.grade = match score.total {
        90..=100 => "A",
        80..=89 => "B",
        70..=79 => "C",
        60..=69 => "D",
        _ => "F",
    };

    score
}

/// Count TODO/todo occurrences that are actual placeholders, ignoring:
/// - Occurrences inside fenced code blocks (``` ... ```)
/// - Compound terms like "TODO-marker", "TODO_detection", "TODOs"
/// - Descriptive prose where TODO is used as a concept (e.g., "TODO comments", "detect TODO")
fn count_placeholder_todos(body: &str) -> usize {
    use regex::Regex;

    // Strip fenced code blocks
    let code_block_re = Regex::new(r"(?s)```[^\n]*\n.*?```").unwrap();
    let stripped = code_block_re.replace_all(body, "");

    // Placeholder pattern: line is just "TODO"/"todo", or starts with "TODO:"
    let todo_line_re = Regex::new(r"(?i)^TODO\s*(:.*)?$").unwrap();

    let mut count = 0;
    for line in stripped.lines() {
        let trimmed = line
            .trim()
            .trim_start_matches("- ")
            .trim_start_matches("* ");
        if todo_line_re.is_match(trimmed) {
            count += 1;
        }
    }
    count
}

/// Count how many required sections have meaningful content (more than just a heading).
fn count_sections_with_content(body: &str, required_sections: &[String]) -> usize {
    let mut count = 0;
    for section in required_sections {
        let header = format!("## {section}");
        if let Some(start) = body.find(&header) {
            let after = start + header.len();
            // Find end of this section (next ## or end of body)
            let rest = &body[after..];
            let end = rest.find("\n## ").unwrap_or(rest.len());
            let section_body = rest[..end].trim();

            // Has content beyond placeholders?
            let meaningful = section_body
                .lines()
                .filter(|l| {
                    let t = l.trim();
                    !t.is_empty()
                        && !t.starts_with("<!--")
                        && !t.starts_with("|--")
                        && !t.starts_with("| -")
                        && t != "TODO"
                        && !t.contains("<!-- TODO")
                })
                .count();

            if meaningful >= 1 {
                count += 1;
            }
        }
    }
    count
}

/// Aggregate scores for a project.
pub struct ProjectScore {
    pub spec_scores: Vec<SpecScore>,
    pub average_score: f64,
    pub grade: &'static str,
    pub total_specs: usize,
    pub grade_distribution: [usize; 5], // A, B, C, D, F
}

pub fn compute_project_score(spec_scores: Vec<SpecScore>) -> ProjectScore {
    let total_specs = spec_scores.len();
    let average_score = if total_specs == 0 {
        0.0
    } else {
        spec_scores.iter().map(|s| s.total as f64).sum::<f64>() / total_specs as f64
    };

    let mut distribution = [0usize; 5];
    for s in &spec_scores {
        match s.grade {
            "A" => distribution[0] += 1,
            "B" => distribution[1] += 1,
            "C" => distribution[2] += 1,
            "D" => distribution[3] += 1,
            _ => distribution[4] += 1,
        }
    }

    let grade = match average_score.round() as u32 {
        90..=100 => "A",
        80..=89 => "B",
        70..=79 => "C",
        60..=69 => "D",
        _ => "F",
    };

    ProjectScore {
        spec_scores,
        average_score,
        grade,
        total_specs,
        grade_distribution: distribution,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_placeholder_todos() {
        let body = "## Purpose\nSomething useful\n\n## Invariants\n- TODO: fill this in\n- TODO\n";
        assert_eq!(count_placeholder_todos(body), 2);
    }

    #[test]
    fn test_count_placeholder_todos_in_code_blocks() {
        let body = "## Purpose\n```\nTODO: this is in a code block\n```\n\nTODO: this is real\n";
        assert_eq!(count_placeholder_todos(body), 1);
    }

    #[test]
    fn test_count_placeholder_todos_zero() {
        let body = "## Purpose\nAll sections filled in with real content.\n";
        assert_eq!(count_placeholder_todos(body), 0);
    }

    #[test]
    fn test_count_sections_with_content() {
        let body =
            "## Purpose\nReal content here\n\n## Public API\n\n## Invariants\n1. Must be valid\n";
        let sections = vec![
            "Purpose".to_string(),
            "Public API".to_string(),
            "Invariants".to_string(),
        ];
        assert_eq!(count_sections_with_content(body, &sections), 2); // Purpose + Invariants
    }

    #[test]
    fn test_count_sections_with_content_empty() {
        let body = "## Purpose\n\n## Public API\n\n";
        let sections = vec!["Purpose".to_string(), "Public API".to_string()];
        assert_eq!(count_sections_with_content(body, &sections), 0);
    }

    #[test]
    fn test_compute_project_score_empty() {
        let project = compute_project_score(vec![]);
        assert_eq!(project.total_specs, 0);
        assert_eq!(project.average_score, 0.0);
        assert_eq!(project.grade, "F");
    }

    #[test]
    fn test_compute_project_score_distribution() {
        let scores = vec![
            SpecScore {
                spec_path: "a".to_string(),
                frontmatter_score: 20,
                sections_score: 20,
                api_score: 20,
                depth_score: 20,
                freshness_score: 15,
                total: 95,
                grade: "A",
                suggestions: vec![],
            },
            SpecScore {
                spec_path: "b".to_string(),
                frontmatter_score: 10,
                sections_score: 10,
                api_score: 10,
                depth_score: 10,
                freshness_score: 10,
                total: 50,
                grade: "F",
                suggestions: vec![],
            },
        ];
        let project = compute_project_score(scores);
        assert_eq!(project.total_specs, 2);
        assert_eq!(project.grade_distribution[0], 1); // 1 A
        assert_eq!(project.grade_distribution[4], 1); // 1 F
        assert!((project.average_score - 72.5).abs() < 0.1);
    }

    #[test]
    fn test_score_spec_complete() {
        let tmp = tempfile::tempdir().unwrap();
        let src_dir = tmp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(
            src_dir.join("auth.ts"),
            "export function createAuth() {}\nexport class AuthService {}\n",
        )
        .unwrap();

        let spec_dir = tmp.path().join("specs").join("auth");
        std::fs::create_dir_all(&spec_dir).unwrap();
        let spec_content = r#"---
module: auth
version: 1
status: active
files:
  - src/auth.ts
db_tables: []
depends_on: []
---

# Auth

## Purpose

The auth module handles authentication.

## Public API

| Export | Description |
|--------|-------------|
| `createAuth` | Creates auth instance |
| `AuthService` | Main auth service class |

## Invariants

1. Tokens must be validated before use

## Behavioral Examples

### Scenario: Valid login

- **Given** valid credentials
- **When** login is called
- **Then** a token is returned

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Invalid token | Returns 401 |

## Dependencies

None.

## Change Log

| Date | Change |
|------|--------|
| 2024-01-01 | Initial |
"#;
        let spec_file = spec_dir.join("auth.spec.md");
        std::fs::write(&spec_file, spec_content).unwrap();

        let config = SpecSyncConfig::default();
        let score = score_spec(&spec_file, tmp.path(), &config);

        assert_eq!(score.frontmatter_score, 20);
        assert!(
            score.total >= 80,
            "Expected high score, got {}",
            score.total
        );
        assert!(score.grade == "A" || score.grade == "B");
    }
}
