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

// Scoring dimension weights (each out of 20, total = 100)
const DIMENSION_MAX: u32 = 20;

// Frontmatter field weights (sum = DIMENSION_MAX)
const FM_MODULE_POINTS: u32 = 5;
const FM_VERSION_POINTS: u32 = 5;
const FM_STATUS_POINTS: u32 = 4;
const FM_FILES_POINTS: u32 = 6;

// Depth sub-weights (sum = DIMENSION_MAX)
const DEPTH_CONTENT_POINTS: u32 = 14;
const DEPTH_PLACEHOLDER_POINTS: u32 = 6;

// Freshness sub-weights
const FRESH_FILES_MAX: u32 = 15;
const FRESH_GIT_MAX: u32 = 5;
const FRESH_FILE_PENALTY_PER: u32 = 5;
const FRESH_DEP_PENALTY_PER: u32 = 3;

// Grade thresholds
const GRADE_A_MIN: u32 = 90;
const GRADE_B_MIN: u32 = 80;
const GRADE_C_MIN: u32 = 70;
const GRADE_D_MIN: u32 = 60;

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
    /// Content depth — sections have real content, not just unfinished markers (0-20).
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
    // Presence alone is not enough (#441): `version: "notanumber"` and
    // `status: bogus` used to score full marks. Validate the VALUES.
    let version_valid = fm.version.as_deref().is_some_and(|v| {
        let v = v.trim().trim_matches('"').trim_matches('\'');
        !v.is_empty()
            && v.split('.')
                .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
    });
    let status_valid = fm.parsed_status().is_some();
    let mut fm_points = 0u32;
    let mut fm_missing: Vec<String> = Vec::new();
    if fm.module.is_some() {
        fm_points += FM_MODULE_POINTS;
    } else {
        fm_missing.push("module (-5pts)".to_string());
    }
    if version_valid {
        fm_points += FM_VERSION_POINTS;
    } else if fm.version.is_some() {
        fm_missing.push("valid version (-5pts)".to_string());
    } else {
        fm_missing.push("version (-5pts)".to_string());
    }
    if status_valid {
        fm_points += FM_STATUS_POINTS;
    } else if fm.status.is_some() {
        fm_missing.push("valid status (-4pts)".to_string());
    } else {
        fm_missing.push("status (-4pts)".to_string());
    }
    if !fm.files.is_empty() {
        fm_points += FM_FILES_POINTS;
    } else {
        fm_missing.push("files (-6pts)".to_string());
    }
    score.frontmatter_score = fm_points;
    if !fm_missing.is_empty() {
        let lost = DIMENSION_MAX - fm_points;
        score.suggestions.push(format!(
            "Frontmatter (-{lost}pts): missing {}",
            fm_missing.join(", ")
        ));
    }
    score.explain.push(ExplainDetail {
        dimension: "Frontmatter".to_string(),
        score: fm_points,
        max_score: DIMENSION_MAX,
        criteria: vec![
            CriterionResult {
                name: "has_module".to_string(),
                passed: fm.module.is_some(),
                points: if fm.module.is_some() {
                    FM_MODULE_POINTS
                } else {
                    0
                },
                max_points: FM_MODULE_POINTS,
                detail: if fm.module.is_none() {
                    Some("add `module:` field".to_string())
                } else {
                    None
                },
            },
            CriterionResult {
                name: "has_version".to_string(),
                passed: version_valid,
                points: if version_valid { FM_VERSION_POINTS } else { 0 },
                max_points: FM_VERSION_POINTS,
                detail: if fm.version.is_none() {
                    Some("add `version:` field".to_string())
                } else if !version_valid {
                    Some("`version` must be numeric (e.g. `1` or `1.2`)".to_string())
                } else {
                    None
                },
            },
            CriterionResult {
                name: "has_status".to_string(),
                passed: status_valid,
                points: if status_valid { FM_STATUS_POINTS } else { 0 },
                max_points: FM_STATUS_POINTS,
                detail: if fm.status.is_none() {
                    Some("add `status:` field".to_string())
                } else if !status_valid {
                    Some("`status` is not a recognized lifecycle status".to_string())
                } else {
                    None
                },
            },
            CriterionResult {
                name: "has_files".to_string(),
                passed: !fm.files.is_empty(),
                points: if !fm.files.is_empty() {
                    FM_FILES_POINTS
                } else {
                    0
                },
                max_points: FM_FILES_POINTS,
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
    let total_sections = config.required_sections.len();
    {
        let missing_set: HashSet<&str> = missing.iter().map(|s| s.as_str()).collect();
        // Distribute DIMENSION_MAX points across the sections EXACTLY (first
        // `remainder` sections get one extra point). The previous round() gave
        // every section ceil(20/n) points, so the criteria summed to e.g. 21
        // while the dimension reported /20 — the --explain output contradicted
        // itself (#441). The dimension score is now literally the criteria sum.
        let (base, remainder) = if total_sections > 0 {
            (
                DIMENSION_MAX / total_sections as u32,
                DIMENSION_MAX % total_sections as u32,
            )
        } else {
            (0, 0)
        };
        let section_criteria: Vec<CriterionResult> = config
            .required_sections
            .iter()
            .enumerate()
            .map(|(i, sec)| {
                let present = !missing_set.contains(sec.as_str());
                let max = base + u32::from((i as u32) < remainder);
                CriterionResult {
                    name: sec.clone(),
                    passed: present,
                    points: if present { max } else { 0 },
                    max_points: max,
                    detail: if !present {
                        Some(format!("add ## {sec} section"))
                    } else {
                        None
                    },
                }
            })
            .collect();
        score.sections_score = if total_sections == 0 {
            DIMENSION_MAX
        } else {
            section_criteria.iter().map(|c| c.points).sum()
        };
        score.explain.push(ExplainDetail {
            dimension: "Sections".to_string(),
            score: score.sections_score,
            max_score: DIMENSION_MAX,
            criteria: section_criteria,
        });
    }
    if !missing.is_empty() {
        let lost = DIMENSION_MAX - score.sections_score;
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

    // ─── API Coverage (0-20) ─────────────────────────────────────────
    if !fm.files.is_empty() {
        let mut all_exports: Vec<String> = Vec::new();
        let mut unreadable_files = 0usize;
        for file in &fm.files {
            // Never read a `files:` entry that escapes the project root — it would
            // leak arbitrary host-file identifiers into score suggestions (and the
            // MCP score tool). validate/check reports such entries as errors.
            if !crate::validator::source_within_root(root, file) {
                continue;
            }
            let full_path = root.join(file);
            match crate::exports::scan_exported_symbols(&full_path) {
                crate::exports::ExportScan::Parsed(syms) => all_exports.extend(syms),
                // A recognized-language source file that can't be read (missing /
                // non-UTF-8) leaves its API unknown — it must NOT be scored as if it
                // had nothing to document (that awarded a perfect API dimension and
                // inflated the gating total, e.g. past a lifecycle min_score guard).
                crate::exports::ExportScan::Unreadable => unreadable_files += 1,
                // A non-source file (e.g. a `.md`/`.sql`) legitimately has no
                // extractable exports — not a failure, and not counted against.
                crate::exports::ExportScan::UnknownLanguage => {}
            }
        }
        let mut seen = HashSet::new();
        all_exports.retain(|s| seen.insert(s.clone()));

        let spec_symbols = get_spec_symbols(body);
        let export_set: HashSet<&str> = all_exports.iter().map(|s| s.as_str()).collect();

        let documented = spec_symbols
            .iter()
            .filter(|s| export_set.contains(s.as_str()))
            .count();

        if all_exports.is_empty() && unreadable_files == 0 {
            // Genuinely nothing to document: every listed file parsed cleanly (or was
            // a non-source file) and produced no exports. Full marks.
            score.api_score = DIMENSION_MAX;
            score.explain.push(ExplainDetail {
                dimension: "API".to_string(),
                score: DIMENSION_MAX,
                max_score: DIMENSION_MAX,
                criteria: vec![CriterionResult {
                    name: "documented_exports".to_string(),
                    passed: true,
                    points: DIMENSION_MAX,
                    max_points: DIMENSION_MAX,
                    detail: Some("no exports to document".to_string()),
                }],
            });
        } else if all_exports.is_empty() {
            // Empty ONLY because listed source file(s) could not be read — API
            // coverage is unverifiable, so withhold the credit rather than award a
            // perfect (and gating-relevant) score for code we could not analyze.
            score.api_score = 0;
            score.explain.push(ExplainDetail {
                dimension: "API".to_string(),
                score: 0,
                max_score: DIMENSION_MAX,
                criteria: vec![CriterionResult {
                    name: "documented_exports".to_string(),
                    passed: false,
                    points: 0,
                    max_points: DIMENSION_MAX,
                    detail: Some(format!(
                        "could not analyze exports for {unreadable_files} file(s) (missing or not UTF-8)"
                    )),
                }],
            });
        } else {
            score.api_score = ((documented as f64 / all_exports.len() as f64)
                * DIMENSION_MAX as f64)
                .round() as u32;
            let undocumented = all_exports.len() - documented;
            if undocumented > 0 {
                let lost = DIMENSION_MAX - score.api_score;
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
                max_score: DIMENSION_MAX,
                criteria: vec![CriterionResult {
                    name: "documented_exports".to_string(),
                    passed: undocumented == 0,
                    points: score.api_score,
                    max_points: DIMENSION_MAX,
                    detail: api_detail,
                }],
            });
        }
    } else {
        score.api_score = 0;
        score.explain.push(ExplainDetail {
            dimension: "API".to_string(),
            score: 0,
            max_score: DIMENSION_MAX,
            criteria: vec![CriterionResult {
                name: "documented_exports".to_string(),
                passed: false,
                points: 0,
                max_points: DIMENSION_MAX,
                detail: Some("no files listed in frontmatter".to_string()),
            }],
        });
    }

    // ─── Content Depth (0-20) ────────────────────────────────────────
    let mut depth_points = 0u32;
    let todo_count = count_placeholder_todos(body);
    // Untouched `specsync new`/`generate` scaffolds contain no TODOs or
    // `<!-- -->` markers, yet are pure boilerplate — count the scaffold's
    // stock sentences as placeholders too, or an empty scaffold passes the
    // "placeholder_free" check and clears the ≥80 bar (#441).
    let placeholder_count = count_placeholder_comments(body) + count_boilerplate_lines(body);

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
    depth_points += (content_ratio * DEPTH_CONTENT_POINTS as f64).round() as u32;

    // Penalize unfinished draft markers.
    if todo_count == 0 && placeholder_count == 0 {
        depth_points += DEPTH_PLACEHOLDER_POINTS;
    } else if todo_count <= 2 {
        depth_points += DEPTH_PLACEHOLDER_POINTS / 2;
    } else {
        score.suggestions.push(format!(
            "Content depth: replace {todo_count} unfinished draft marker(s) with real content"
        ));
    }
    depth_points = depth_points.saturating_sub(stub_penalty);
    score.depth_score = depth_points.min(DIMENSION_MAX);
    if score.depth_score < DIMENSION_MAX {
        let lost = DIMENSION_MAX - score.depth_score;
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
            "Draft-only sections: ## {names}{suffix} — replace unfinished text with real content"
        ));
        if stub_penalty > 0 {
            score.suggestions.push(
                "Draft-only section ratio is high — complete those sections to improve depth score.".to_string(),
            );
        }
    }
    let content_points = (content_ratio * DEPTH_CONTENT_POINTS as f64).round() as u32;
    let todo_points = if todo_count == 0 && placeholder_count == 0 {
        DEPTH_PLACEHOLDER_POINTS
    } else if todo_count <= 2 {
        DEPTH_PLACEHOLDER_POINTS / 2
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
        Some(format!("{todo_count} unfinished draft marker(s)"))
    } else {
        None
    };
    score.explain.push(ExplainDetail {
        dimension: "Depth".to_string(),
        score: score.depth_score,
        max_score: DIMENSION_MAX,
        criteria: vec![
            CriterionResult {
                name: "sections_with_content".to_string(),
                passed: content_points >= DEPTH_CONTENT_POINTS,
                points: content_points,
                max_points: DEPTH_CONTENT_POINTS,
                detail: stub_detail,
            },
            CriterionResult {
                name: "placeholder_free".to_string(),
                passed: todo_points == DEPTH_PLACEHOLDER_POINTS,
                points: todo_points,
                max_points: DEPTH_PLACEHOLDER_POINTS,
                detail: todo_detail,
            },
        ],
    });

    // ─── Freshness (0-20) ────────────────────────────────────────────
    let mut fresh_points = DIMENSION_MAX;
    let mut stale_files = 0u32;
    for file in &fm.files {
        if !root.join(file).exists() {
            stale_files += 1;
        }
    }
    let file_penalty = if fm.files.is_empty() {
        // `files_exist` was vacuously 15/15 when `files:` was missing entirely
        // — nothing to check passed as everything checked out (#441). An empty
        // files list makes freshness unverifiable, not perfect.
        fresh_points = fresh_points.saturating_sub(FRESH_FILES_MAX);
        score.suggestions.push(format!(
            "Freshness (-{FRESH_FILES_MAX}pts): no files listed in frontmatter — freshness is unverifiable"
        ));
        FRESH_FILES_MAX
    } else if stale_files > 0 {
        let penalty = (stale_files * FRESH_FILE_PENALTY_PER).min(FRESH_FILES_MAX);
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
        let penalty = stale_deps * FRESH_DEP_PENALTY_PER;
        fresh_points = fresh_points.saturating_sub(penalty);
        score.suggestions.push(format!(
            "Freshness (-{penalty}pts): {stale_deps} depends_on path(s) don't exist"
        ));
        penalty
    } else {
        0
    };

    // Git-based staleness: penalize if source files have commits since spec was last updated.
    // For an untracked/new spec, fall back to modification times so freshness does
    // not vacuously score 20/20 when a source changed after its spec.
    let mut git_penalty = 0u32;
    let mut git_behind: usize = 0;
    let mut git_baseline_available = false;
    let mut source_newer_than_spec = false;
    if !fm.files.is_empty() && git_utils::is_git_repo(root) {
        let rel_path = spec_path
            .strip_prefix(root)
            .unwrap_or(spec_path)
            .to_string_lossy()
            .to_string();
        if let Some(spec_commit) = git_utils::git_last_commit_hash(root, &rel_path) {
            git_baseline_available = true;
            let mut max_behind: usize = 0;
            for file in &fm.files {
                if root.join(file).exists() {
                    let behind = git_utils::git_commits_since(root, &spec_commit, file);
                    max_behind = max_behind.max(behind);
                }
            }
            git_behind = max_behind;
            if max_behind >= 10 {
                git_penalty = FRESH_GIT_MAX;
                fresh_points = fresh_points.saturating_sub(git_penalty);
                score.suggestions.push(format!(
                    "Freshness (-{git_penalty}pts): spec is {max_behind} commits behind source files"
                ));
            } else if max_behind >= 5 {
                git_penalty = FRESH_GIT_MAX - 2;
                fresh_points = fresh_points.saturating_sub(git_penalty);
                score.suggestions.push(format!(
                    "Freshness (-{git_penalty}pts): spec is {max_behind} commits behind source files"
                ));
            }
        }
    }
    if !fm.files.is_empty()
        && !git_baseline_available
        && let Ok(spec_modified) = fs::metadata(spec_path).and_then(|meta| meta.modified())
    {
        source_newer_than_spec = fm.files.iter().any(|file| {
            crate::validator::source_within_root(root, file)
                && fs::metadata(root.join(file))
                    .and_then(|meta| meta.modified())
                    .is_ok_and(|modified| modified > spec_modified)
        });
        if source_newer_than_spec {
            git_penalty = FRESH_GIT_MAX;
            fresh_points = fresh_points.saturating_sub(git_penalty);
            score.suggestions.push(format!(
                "Freshness (-{git_penalty}pts): source files were modified after the spec"
            ));
        }
    }

    score.freshness_score = fresh_points;
    // Budget the dimension so the --explain criteria sum EXACTLY to the
    // reported score (previously criteria summed to 15+5+variable ≠ 20 — the
    // raw max was silently rescaled, contradicting the dimension line, #441).
    // fresh_points = 20 - file_penalty - dep_penalty - git_penalty, so:
    //   files_exist: max 15-dep_budget, points max-file_penalty
    //   deps_exist:  max dep_budget,     points budget-dep_penalty
    //   git:         max 5,              points 5-git_penalty
    let dep_budget = if fm.depends_on.is_empty() {
        0
    } else {
        dep_penalty.max(FRESH_DEP_PENALTY_PER)
    };
    let files_budget = FRESH_FILES_MAX - dep_budget;
    score.explain.push(ExplainDetail {
        dimension: "Freshness".to_string(),
        score: fresh_points,
        max_score: DIMENSION_MAX,
        criteria: vec![
            CriterionResult {
                name: "files_exist".to_string(),
                passed: !fm.files.is_empty() && stale_files == 0,
                points: files_budget.saturating_sub(file_penalty),
                max_points: files_budget,
                detail: if fm.files.is_empty() {
                    Some("no files listed in frontmatter".to_string())
                } else if stale_files > 0 {
                    Some(format!("{stale_files} file(s) missing"))
                } else {
                    None
                },
            },
            CriterionResult {
                name: "deps_exist".to_string(),
                passed: stale_deps == 0,
                points: dep_budget.saturating_sub(dep_penalty),
                max_points: dep_budget,
                detail: if stale_deps > 0 {
                    Some(format!("{stale_deps} depends_on path(s) missing"))
                } else {
                    None
                },
            },
            CriterionResult {
                name: "git_freshness".to_string(),
                passed: git_penalty == 0,
                points: FRESH_GIT_MAX.saturating_sub(git_penalty),
                max_points: FRESH_GIT_MAX,
                detail: if source_newer_than_spec {
                    Some("source files were modified after the spec".to_string())
                } else if git_behind >= 5 {
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

    score.grade = letter_grade(score.total);

    // A generated/all-TODO scaffold must not clear the documented 80-point bar.
    // If at least half of required sections are stubs and unfinished markers are
    // still present, cap below B until the draft content is replaced.
    let total_req = config.required_sections.len();
    if total_req > 0
        && stub_sections.len() * 2 >= total_req
        && todo_count + placeholder_count > 0
        && score.total >= GRADE_B_MIN
    {
        score.total = GRADE_B_MIN - 1;
        score.grade = letter_grade(score.total);
        score.suggestions.push(format!(
            "Score capped below 80: {}/{} required sections contain only unfinished draft content — replace it with real documentation",
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
    use std::sync::LazyLock;

    // Compiled once and reused across every spec scored in a run, rather than
    // recompiling both patterns on each call.
    static CODE_BLOCK_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?s)```[^\n]*\n.*?```").expect("valid code-block regex"));
    static TODO_LINE_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?i)^TODO\s*(:.*)?$").expect("valid TODO-line regex"));

    // Strip fenced code blocks
    let stripped = CODE_BLOCK_RE.replace_all(body, "");

    let mut count = 0;
    for line in stripped.lines() {
        let trimmed = line
            .trim()
            .trim_start_matches("- ")
            .trim_start_matches("* ");
        if TODO_LINE_RE.is_match(trimmed) {
            count += 1;
        }
    }
    count
}

/// Count unfilled HTML-comment placeholders (`<!-- ... -->`) in prose.
///
/// Fenced code blocks and inline code spans are stripped first, so a spec that
/// *documents* an HTML-comment directive (e.g. ``a `<!-- specsync-ignore -->`
/// directive``) isn't penalized for showing real syntax. Mirrors the
/// code-stripping that [`count_placeholder_todos`] already does for TODOs.
fn count_placeholder_comments(body: &str) -> usize {
    use regex::Regex;
    use std::sync::LazyLock;

    static CODE_BLOCK_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?s)```[^\n]*\n.*?```").expect("valid code-block regex"));
    static INLINE_CODE_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"`[^`]*`").expect("valid inline-code regex"));

    let no_fenced = CODE_BLOCK_RE.replace_all(body, "");
    let stripped = INLINE_CODE_RE.replace_all(&no_fenced, "");
    stripped.matches("<!-- ").count()
}

/// Count lines that are verbatim `specsync new`/`generate` scaffold
/// boilerplate ("Document this module's responsibility…", stock Given/When/
/// Then, etc.) — an untouched scaffold must not count as placeholder-free.
fn count_boilerplate_lines(body: &str) -> usize {
    body.lines()
        .filter(|l| crate::parser::is_boilerplate_line(l))
        .count()
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

fn letter_grade(score: u32) -> &'static str {
    match score {
        s if s >= GRADE_A_MIN => "A",
        s if s >= GRADE_B_MIN => "B",
        s if s >= GRADE_C_MIN => "C",
        s if s >= GRADE_D_MIN => "D",
        _ => "F",
    }
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

    let grade = letter_grade(average_score.round() as u32);

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
    fn test_placeholder_comments_count_prose_only() {
        // A real un-filled HTML comment in prose counts.
        assert_eq!(
            count_placeholder_comments("Body with <!-- fill me --> here"),
            1
        );
        // The same syntax shown as documentation in inline code or a fenced
        // block does NOT count — it's real content, not a placeholder.
        assert_eq!(
            count_placeholder_comments("Use a `<!-- specsync-ignore: x -->` directive"),
            0
        );
        assert_eq!(
            count_placeholder_comments("```\n<!-- specsync-ignore: x -->\n```\n"),
            0
        );
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
    fn test_score_spec_out_of_root_file_does_not_leak_identifiers() {
        // Security regression: a spec whose `files:` resolves outside the project
        // root (here an absolute path) must not have that file's exported
        // identifiers read and surfaced in `score` suggestions (incl. MCP score).
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(
            outside.path().join("secret.ts"),
            "export const AWS_SECRET_ACCESS_KEY = \"leak\";\n",
        )
        .unwrap();

        let spec_dir = root.path().join("specs").join("s");
        std::fs::create_dir_all(&spec_dir).unwrap();
        let spec_content = format!(
            "---\nmodule: s\nversion: 1\nstatus: active\nfiles:\n  - {}\ndb_tables: []\ndepends_on: []\n---\n\n# S\n\n## Purpose\nx\n\n## Public API\n| Export | Description |\n|--------|-------------|\n\n## Invariants\n1. x\n\n## Behavioral Examples\n### Scenario: a\n- **Given** a **When** b **Then** c\n\n## Error Cases\n| Condition | Behavior |\n|-----------|----------|\n\n## Dependencies\nNone.\n\n## Change Log\n| Date | Change |\n|------|--------|\n| 2024-01-01 | Initial |\n",
            outside.path().join("secret.ts").display()
        );
        let spec_file = spec_dir.join("s.spec.md");
        std::fs::write(&spec_file, spec_content).unwrap();

        let config = SpecSyncConfig::default();
        let score = score_spec(&spec_file, root.path(), &config);

        assert!(
            !score
                .suggestions
                .iter()
                .any(|s| s.contains("AWS_SECRET_ACCESS_KEY")),
            "out-of-root identifier leaked into score suggestions: {:?}",
            score.suggestions
        );
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

        // Depth score should be penalized because most sections are draft-only (>=50% -> -10pts ceiling)
        assert!(
            score.depth_score <= 10,
            "Expected low depth score for draft-only sections, got {}",
            score.depth_score
        );
        // Should have a suggestion about draft-only sections.
        assert!(
            score
                .suggestions
                .iter()
                .any(|s| s.contains("Draft-only sections")),
            "Expected draft-only section suggestion, got: {:?}",
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

    #[test]
    fn all_generator_placeholder_spec_does_not_score_a() {
        // #421: an untouched generated scaffold must stay below the documented
        // CI quality bar, not merely below an A.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/thing.ts"), "export function thing() {}\n").unwrap();
        let spec_dir = root.join("specs").join("thing");
        std::fs::create_dir_all(&spec_dir).unwrap();
        let spec_path = spec_dir.join("thing.spec.md");
        std::fs::write(
            &spec_path,
            "---\nmodule: thing\nversion: 1\nstatus: draft\nfiles:\n  - src/thing.ts\ndb_tables: []\ndepends_on: []\n---\n\n# Thing\n\n## Purpose\n\nDocument this module's responsibility, inputs, outputs, and ownership boundaries.\n\n## Public API\n\n| Export | Description |\n|--------|-------------|\n| `thing` | Document the export's responsibility and caller-visible behavior. |\n\n## Invariants\n\n1. Define an invariant that must remain true for supported inputs.\n\n## Behavioral Examples\n\n### Scenario: Core behavior\n\n- **Given** precondition\n- **When** action\n- **Then** result\n\n## Error Cases\n\n| Condition | Behavior |\n|-----------|----------|\n\n## Dependencies\n\nList runtime dependencies and the specific symbols, services, or data they provide.\n\n## Change Log\n\n| Change | Date | Version |\n|--------|------|---------|\n",
        )
        .unwrap();
        let config = crate::types::SpecSyncConfig::default();
        let score = score_spec(&spec_path, root, &config);
        assert!(
            score.total < 80,
            "all-placeholder spec must stay below 80, got {} ({})",
            score.total,
            score.grade
        );
        assert_ne!(score.grade, "A");
        assert_ne!(score.grade, "B");
    }

    #[test]
    fn all_todo_spec_stays_below_passing_bar() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/empty.ts"), "const internal = true;\n").unwrap();
        let spec_dir = root.join("specs").join("empty");
        std::fs::create_dir_all(&spec_dir).unwrap();
        let spec_path = spec_dir.join("empty.spec.md");
        std::fs::write(
            &spec_path,
            "---\nmodule: empty\nversion: 1\nstatus: draft\nfiles:\n  - src/empty.ts\ndb_tables: []\ndepends_on: []\n---\n\n# Empty\n\n## Purpose\nTODO\n\n## Public API\nTODO\n\n## Invariants\nTODO\n\n## Behavioral Examples\nTODO\n\n## Error Cases\nTODO\n\n## Dependencies\nTODO\n\n## Change Log\nTODO\n",
        )
        .unwrap();

        let score = score_spec(&spec_path, root, &SpecSyncConfig::default());
        assert!(score.total < 80, "all-TODO score was {}", score.total);
    }

    #[test]
    fn freshness_detects_source_newer_than_untracked_spec() {
        use std::fs::{FileTimes, OpenOptions};
        use std::time::{Duration, UNIX_EPOCH};

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("src")).unwrap();
        let source_path = root.join("src/auth.ts");
        std::fs::write(&source_path, "export function login() {}\n").unwrap();
        let spec_dir = root.join("specs").join("auth");
        std::fs::create_dir_all(&spec_dir).unwrap();
        let spec_path = spec_dir.join("auth.spec.md");
        std::fs::write(
            &spec_path,
            "---\nmodule: auth\nversion: 1\nstatus: active\nfiles:\n  - src/auth.ts\ndb_tables: []\ndepends_on: []\n---\n\n# Auth\n\n## Purpose\nAuthenticates users.\n\n## Public API\n\n| Export | Description |\n|--------|-------------|\n| `login` | Authenticates a user. |\n\n## Invariants\nCredentials are validated.\n\n## Behavioral Examples\nA valid login returns a session.\n\n## Error Cases\nInvalid credentials are rejected.\n\n## Dependencies\nNo runtime dependencies.\n\n## Change Log\n2020-01-01: initial.\n",
        )
        .unwrap();

        let spec_time = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        let source_time = spec_time + Duration::from_secs(60);
        OpenOptions::new()
            .write(true)
            .open(&spec_path)
            .unwrap()
            .set_times(FileTimes::new().set_modified(spec_time))
            .unwrap();
        OpenOptions::new()
            .write(true)
            .open(&source_path)
            .unwrap()
            .set_times(FileTimes::new().set_modified(source_time))
            .unwrap();

        let score = score_spec(&spec_path, root, &SpecSyncConfig::default());
        assert_eq!(score.freshness_score, 15, "{:?}", score.suggestions);
        assert!(
            score
                .suggestions
                .iter()
                .any(|suggestion| suggestion.contains("modified after the spec"))
        );
    }
}
