use crate::exports::get_exported_symbols;
use crate::git_utils;
use crate::parser::{
    find_stub_sections, get_missing_sections, get_spec_symbols, parse_frontmatter,
    section_has_content,
};
use crate::types::SpecSyncConfig;
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Pass/fail result for a single scoring criterion within a dimension.
#[derive(Debug, Clone, Serialize)]
pub struct CriterionResult {
    pub name: String,
    pub passed: bool,
    pub points: u32,
    pub max_points: u32,
    pub detail: Option<String>,
}

/// Per-dimension breakdown used by `--explain`.
#[derive(Debug, Clone, Serialize)]
pub struct ExplainDetail {
    pub dimension: String,
    pub score: u32,
    pub max_score: u32,
    pub criteria: Vec<CriterionResult>,
}

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
    /// Per-criterion breakdown populated during scoring (used by --explain).
    pub explain: Vec<ExplainDetail>,
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
        explain: Vec::new(),
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
    score.explain.push(ExplainDetail {
        dimension: "Frontmatter".to_string(),
        score: fm_points,
        max_score: 20,
        criteria: vec![
            CriterionResult {
                name: "has_module".to_string(),
                passed: fm.module.is_some(),
                points: if fm.module.is_some() { 5 } else { 0 },
                max_points: 5,
                detail: if fm.module.is_none() {
                    Some("add `module:` field".to_string())
                } else {
                    None
                },
            },
            CriterionResult {
                name: "has_version".to_string(),
                passed: fm.version.is_some(),
                points: if fm.version.is_some() { 5 } else { 0 },
                max_points: 5,
                detail: if fm.version.is_none() {
                    Some("add `version:` field".to_string())
                } else {
                    None
                },
            },
            CriterionResult {
                name: "has_status".to_string(),
                passed: fm.status.is_some(),
                points: if fm.status.is_some() { 4 } else { 0 },
                max_points: 4,
                detail: if fm.status.is_none() {
                    Some("add `status:` field".to_string())
                } else {
                    None
                },
            },
            CriterionResult {
                name: "has_files".to_string(),
                passed: !fm.files.is_empty(),
                points: if !fm.files.is_empty() { 6 } else { 0 },
                max_points: 6,
                detail: if fm.files.is_empty() {
                    Some("add `files:` list".to_string())
                } else {
                    None
                },
            },
        ],
    });

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
        score
            .suggestions
            .push(format!("Sections (-{lost}pts): missing ## {names}{suffix}"));
    }
    {
        let missing_set: HashSet<&str> = missing.iter().map(|s| s.as_str()).collect();
        let per_section_max = if total_sections > 0 {
            ((20.0 / total_sections as f64).round() as u32).max(1)
        } else {
            0
        };
        let section_criteria: Vec<CriterionResult> = config
            .required_sections
            .iter()
            .map(|sec| {
                let present = !missing_set.contains(sec.as_str());
                CriterionResult {
                    name: sec.clone(),
                    passed: present,
                    points: if present { per_section_max } else { 0 },
                    max_points: per_section_max,
                    detail: if !present {
                        Some(format!("add ## {sec} section"))
                    } else {
                        None
                    },
                }
            })
            .collect();
        score.explain.push(ExplainDetail {
            dimension: "Sections".to_string(),
            score: score.sections_score,
            max_score: 20,
            criteria: section_criteria,
        });
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
            score.explain.push(ExplainDetail {
                dimension: "API".to_string(),
                score: 20,
                max_score: 20,
                criteria: vec![CriterionResult {
                    name: "documented_exports".to_string(),
                    passed: true,
                    points: 20,
                    max_points: 20,
                    detail: Some("no exports to document".to_string()),
                }],
            });
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
            let api_detail = if undocumented > 0 {
                Some(format!(
                    "{documented}/{} exports documented",
                    all_exports.len()
                ))
            } else {
                None
            };
            score.explain.push(ExplainDetail {
                dimension: "API".to_string(),
                score: score.api_score,
                max_score: 20,
                criteria: vec![CriterionResult {
                    name: "documented_exports".to_string(),
                    passed: undocumented == 0,
                    points: score.api_score,
                    max_points: 20,
                    detail: api_detail,
                }],
            });
        }
    } else {
        score.api_score = 0;
        score.explain.push(ExplainDetail {
            dimension: "API".to_string(),
            score: 0,
            max_score: 20,
            criteria: vec![CriterionResult {
                name: "documented_exports".to_string(),
                passed: false,
                points: 0,
                max_points: 20,
                detail: Some("no files listed in frontmatter".to_string()),
            }],
        });
    }

    // ─── Content Depth (0-20) ────────────────────────────────────────
    let mut depth_points = 0u32;
    let todo_count = count_placeholder_todos(body);
    let placeholder_count = body.matches("<!-- ").count();

    // Check each required section has meaningful content (stubs don't count)
    let sections_with_content = count_sections_with_content(body, &config.required_sections);
    let stub_sections = find_stub_sections(body, &config.required_sections);
    let stub_ratio = if !config.required_sections.is_empty() {
        stub_sections.len() as f64 / config.required_sections.len() as f64
    } else {
        0.0
    };
    let stub_penalty = if stub_ratio >= 0.5 {
        10
    } else if stub_ratio >= 0.33 {
        5
    } else {
        0
    };
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
    depth_points = depth_points.saturating_sub(stub_penalty);
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

    // Report stub sections specifically so users know which sections need real content
    if !stub_sections.is_empty() {
        let names = stub_sections
            .iter()
            .take(4)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        let suffix = if stub_sections.len() > 4 {
            format!(" (+{} more)", stub_sections.len() - 4)
        } else {
            String::new()
        };
        score.suggestions.push(format!(
            "Stub sections: ## {names}{suffix} — replace placeholder text (TBD, N/A, TODO, etc.) with real content"
        ));
        if stub_penalty > 0 {
            score.suggestions.push(
                "Stub ratio is high — fill in TBD sections to improve depth score.".to_string(),
            );
        }
    }
    let content_points = (content_ratio * 14.0).round() as u32;
    let todo_points = if todo_count == 0 && placeholder_count == 0 {
        6u32
    } else if todo_count <= 2 {
        3u32
    } else {
        0u32
    };
    let stub_detail = if !stub_sections.is_empty() {
        Some(format!(
            "{} stub section(s): {}",
            stub_sections.len(),
            stub_sections
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        ))
    } else {
        None
    };
    let todo_detail = if todo_count > 0 {
        Some(format!("{todo_count} TODO placeholder(s)"))
    } else {
        None
    };
    score.explain.push(ExplainDetail {
        dimension: "Depth".to_string(),
        score: score.depth_score,
        max_score: 20,
        criteria: vec![
            CriterionResult {
                name: "sections_with_content".to_string(),
                passed: content_points >= 14,
                points: content_points,
                max_points: 14,
                detail: stub_detail,
            },
            CriterionResult {
                name: "placeholder_free".to_string(),
                passed: todo_points == 6,
                points: todo_points,
                max_points: 6,
                detail: todo_detail,
            },
        ],
    });

    // ─── Freshness (0-20) ────────────────────────────────────────────
    let mut fresh_points = 20u32;
    let mut stale_files = 0u32;
    for file in &fm.files {
        if !root.join(file).exists() {
            stale_files += 1;
        }
    }
    let file_penalty = if stale_files > 0 {
        let penalty = (stale_files * 5).min(15);
        fresh_points = fresh_points.saturating_sub(penalty);
        score.suggestions.push(format!(
            "Freshness (-{penalty}pts): {stale_files} file(s) in frontmatter don't exist"
        ));
        penalty
    } else {
        0
    };

    // Check depends_on references
    let mut stale_deps = 0u32;
    for dep in &fm.depends_on {
        if !root.join(dep).exists() {
            stale_deps += 1;
        }
    }
    let dep_penalty = if stale_deps > 0 {
        let penalty = stale_deps * 3;
        fresh_points = fresh_points.saturating_sub(penalty);
        score.suggestions.push(format!(
            "Freshness (-{penalty}pts): {stale_deps} depends_on path(s) don't exist"
        ));
        penalty
    } else {
        0
    };

    // Git-based staleness: penalize if source files have commits since spec was last updated
    let mut git_penalty = 0u32;
    let mut git_behind: usize = 0;
    if !fm.files.is_empty() && git_utils::is_git_repo(root) {
        let rel_path = spec_path
            .strip_prefix(root)
            .unwrap_or(spec_path)
            .to_string_lossy()
            .to_string();
        if git_utils::git_last_commit_hash(root, &rel_path).is_some() {
            let mut max_behind: usize = 0;
            for file in &fm.files {
                if root.join(file).exists() {
                    let behind = git_utils::git_commits_between(root, &rel_path, file);
                    max_behind = max_behind.max(behind);
                }
            }
            git_behind = max_behind;
            if max_behind >= 10 {
                let penalty = 5u32;
                git_penalty = penalty;
                fresh_points = fresh_points.saturating_sub(penalty);
                score.suggestions.push(format!(
                    "Freshness (-{penalty}pts): spec is {max_behind} commits behind source files"
                ));
            } else if max_behind >= 5 {
                let penalty = 3u32;
                git_penalty = penalty;
                fresh_points = fresh_points.saturating_sub(penalty);
                score.suggestions.push(format!(
                    "Freshness (-{penalty}pts): spec is {max_behind} commits behind source files"
                ));
            }
        }
    }

    score.freshness_score = fresh_points;
    score.explain.push(ExplainDetail {
        dimension: "Freshness".to_string(),
        score: fresh_points,
        max_score: 20,
        criteria: vec![
            CriterionResult {
                name: "files_exist".to_string(),
                passed: stale_files == 0,
                points: 15u32.saturating_sub(file_penalty),
                max_points: 15,
                detail: if stale_files > 0 {
                    Some(format!("{stale_files} file(s) missing"))
                } else {
                    None
                },
            },
            CriterionResult {
                name: "deps_exist".to_string(),
                passed: stale_deps == 0,
                points: (stale_deps * 3).saturating_sub(dep_penalty).min(
                    if fm.depends_on.is_empty() {
                        0
                    } else {
                        stale_deps * 3
                    },
                ),
                max_points: (fm.depends_on.len() as u32 * 3).min(6),
                detail: if stale_deps > 0 {
                    Some(format!("{stale_deps} depends_on path(s) missing"))
                } else {
                    None
                },
            },
            CriterionResult {
                name: "git_freshness".to_string(),
                passed: git_penalty == 0,
                points: 5u32.saturating_sub(git_penalty),
                max_points: 5,
                detail: if git_behind >= 5 {
                    Some(format!("{git_behind} commits behind source files"))
                } else {
                    None
                },
            },
        ],
    });

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

    // A-grade requires real content — specs with ≥50% stub sections are capped at B.
    // This prevents fully-stubbed specs with clean metadata from reaching an A.
    let total_req = config.required_sections.len();
    if score.grade == "A" && total_req > 0 && stub_sections.len() * 2 >= total_req {
        score.grade = "B";
        score.total = score.total.min(89);
        score.suggestions.push(format!(
            "Grade capped at B: {}/{} required sections contain only stub/placeholder content — replace TBD/N/A/TODO with real documentation",
            stub_sections.len(),
            total_req
        ));
    }

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
        if section_has_content(body, section) {
            count += 1;
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
                explain: vec![],
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
                explain: vec![],
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

    #[test]
    fn test_count_sections_with_content_stubs_not_counted() {
        let body = "## Purpose\nTBD\n\n## Public API\nN/A\n\n## Invariants\nReal invariant here\n";
        let sections = vec![
            "Purpose".to_string(),
            "Public API".to_string(),
            "Invariants".to_string(),
        ];
        // Only Invariants has real content; Purpose and Public API are stubs
        assert_eq!(count_sections_with_content(body, &sections), 1);
    }

    #[test]
    fn test_score_spec_stub_sections_penalized() {
        let tmp = tempfile::tempdir().unwrap();
        let src_dir = tmp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(src_dir.join("stub.ts"), "export function doStuff() {}\n").unwrap();

        let spec_dir = tmp.path().join("specs").join("stub");
        std::fs::create_dir_all(&spec_dir).unwrap();
        let spec_content = r#"---
module: stub
version: 1
status: active
files:
  - src/stub.ts
db_tables: []
depends_on: []
---

# Stub

## Purpose

TBD

## Public API

| Export | Description |
|--------|-------------|
| `doStuff` | Does stuff |

## Invariants

N/A

## Behavioral Examples

Coming soon

## Error Cases

TBD

## Dependencies

None.

## Change Log

| Date | Change |
|------|--------|
| 2024-01-01 | Initial |
"#;
        let spec_file = spec_dir.join("stub.spec.md");
        std::fs::write(&spec_file, spec_content).unwrap();

        let config = SpecSyncConfig::default();
        let score = score_spec(&spec_file, tmp.path(), &config);

        // Depth score should be penalized because most sections are stubs (>=50% → -10pts ceiling)
        assert!(
            score.depth_score <= 10,
            "Expected low depth score for stub sections, got {}",
            score.depth_score
        );
        // Should have a suggestion about stub sections
        assert!(
            score
                .suggestions
                .iter()
                .any(|s| s.contains("Stub sections")),
            "Expected stub section suggestion, got: {:?}",
            score.suggestions
        );
    }

    #[test]
    fn test_explain_frontmatter_criteria_complete() {
        let tmp = tempfile::tempdir().unwrap();
        let src_dir = tmp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(src_dir.join("x.ts"), "export function foo() {}\n").unwrap();

        let spec_dir = tmp.path().join("specs");
        std::fs::create_dir_all(&spec_dir).unwrap();
        let spec_content = "---\nmodule: x\nversion: 1\nstatus: active\nfiles:\n  - src/x.ts\ndb_tables: []\ndepends_on: []\n---\n\n## Purpose\nContent.\n";
        let spec_file = spec_dir.join("x.spec.md");
        std::fs::write(&spec_file, spec_content).unwrap();

        let config = SpecSyncConfig::default();
        let score = score_spec(&spec_file, tmp.path(), &config);

        let fm = score
            .explain
            .iter()
            .find(|d| d.dimension == "Frontmatter")
            .unwrap();
        assert_eq!(fm.score, 20);
        assert_eq!(fm.max_score, 20);
        assert!(fm.criteria.iter().all(|c| c.passed));
        let module_crit = fm.criteria.iter().find(|c| c.name == "has_module").unwrap();
        assert_eq!(module_crit.points, 5);
        assert_eq!(module_crit.max_points, 5);
    }

    #[test]
    fn test_explain_frontmatter_criteria_missing_fields() {
        let tmp = tempfile::tempdir().unwrap();
        let spec_dir = tmp.path().join("specs");
        std::fs::create_dir_all(&spec_dir).unwrap();
        // Missing version and status
        let spec_content = "---\nmodule: x\nfiles: []\ndb_tables: []\ndepends_on: []\n---\n\n## Purpose\nContent.\n";
        let spec_file = spec_dir.join("x.spec.md");
        std::fs::write(&spec_file, spec_content).unwrap();

        let config = SpecSyncConfig::default();
        let score = score_spec(&spec_file, tmp.path(), &config);

        let fm = score
            .explain
            .iter()
            .find(|d| d.dimension == "Frontmatter")
            .unwrap();
        assert!(fm.score < 20);
        let version_crit = fm
            .criteria
            .iter()
            .find(|c| c.name == "has_version")
            .unwrap();
        assert!(!version_crit.passed);
        assert_eq!(version_crit.points, 0);
        let status_crit = fm.criteria.iter().find(|c| c.name == "has_status").unwrap();
        assert!(!status_crit.passed);
        assert!(status_crit.detail.is_some());
    }

    #[test]
    fn test_explain_depth_criteria() {
        let tmp = tempfile::tempdir().unwrap();
        let spec_dir = tmp.path().join("specs");
        std::fs::create_dir_all(&spec_dir).unwrap();
        let spec_content = "---\nmodule: x\nversion: 1\nstatus: active\nfiles: []\ndb_tables: []\ndepends_on: []\n---\n\n## Purpose\nReal content here.\n\n## Invariants\nTBD\n";
        let spec_file = spec_dir.join("x.spec.md");
        std::fs::write(&spec_file, spec_content).unwrap();

        let config = SpecSyncConfig::default();
        let score = score_spec(&spec_file, tmp.path(), &config);

        let depth = score
            .explain
            .iter()
            .find(|d| d.dimension == "Depth")
            .unwrap();
        assert_eq!(depth.max_score, 20);
        let content_crit = depth
            .criteria
            .iter()
            .find(|c| c.name == "sections_with_content")
            .unwrap();
        assert_eq!(content_crit.max_points, 14);
        let todo_crit = depth
            .criteria
            .iter()
            .find(|c| c.name == "placeholder_free")
            .unwrap();
        assert_eq!(todo_crit.max_points, 6);
    }

    #[test]
    fn test_explain_has_all_dimensions() {
        let tmp = tempfile::tempdir().unwrap();
        let spec_dir = tmp.path().join("specs");
        std::fs::create_dir_all(&spec_dir).unwrap();
        let spec_content = "---\nmodule: x\nversion: 1\nstatus: active\nfiles: []\ndb_tables: []\ndepends_on: []\n---\n\n## Purpose\nContent.\n";
        let spec_file = spec_dir.join("x.spec.md");
        std::fs::write(&spec_file, spec_content).unwrap();

        let config = SpecSyncConfig::default();
        let score = score_spec(&spec_file, tmp.path(), &config);

        let dimensions: Vec<&str> = score.explain.iter().map(|d| d.dimension.as_str()).collect();
        assert!(dimensions.contains(&"Frontmatter"), "missing Frontmatter");
        assert!(dimensions.contains(&"Sections"), "missing Sections");
        assert!(dimensions.contains(&"API"), "missing API");
        assert!(dimensions.contains(&"Depth"), "missing Depth");
        assert!(dimensions.contains(&"Freshness"), "missing Freshness");
    }
}
