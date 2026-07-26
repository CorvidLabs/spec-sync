use colored::Colorize;
use std::path::Path;

use crate::scoring;
use crate::types;
use crate::validator::compute_coverage;

use super::{
    compute_exit_code, exit_with_status, filter_by_status, filter_specs, load_and_discover,
};

#[allow(clippy::too_many_arguments)]
pub fn cmd_score(
    root: &Path,
    strict: bool,
    enforcement: Option<types::EnforcementMode>,
    require_coverage: Option<usize>,
    min_score: Option<u32>,
    format: types::OutputFormat,
    explain: bool,
    all: bool,
    spec_filters: &[String],
    exclude_status: &[String],
    only_status: &[String],
) {
    let json = matches!(format, types::OutputFormat::Json);
    let machine_output = json || matches!(format, types::OutputFormat::Csv);
    let minimum_score = match (min_score, strict) {
        (Some(value), true) => Some(value.max(80)),
        (Some(value), false) => Some(value),
        (None, true) => Some(80),
        (None, false) => None,
    };
    // The global `--strict` / `--enforcement` / `--require-coverage` flags are
    // gates, and so is a project's configured `enforcement`. `score` must honor
    // all of them: when a gate could fire, don't take load_and_discover's no-spec
    // early-exit — otherwise a spec-less project bypasses the gate (exit 0) exactly
    // like the `check` no-spec bug did. Resolve the effective enforcement up-front
    // (peeking the config only when no CLI flag decides it) so a CONFIG-only
    // enforce-new/strict gate is covered too, not just the CLI flags.
    let enforcement = enforcement.unwrap_or_else(|| {
        if strict {
            types::EnforcementMode::Strict
        } else {
            crate::config::load_config(root).enforcement
        }
    });
    let gate_requested = require_coverage.is_some()
        || minimum_score.is_some()
        || !matches!(enforcement, types::EnforcementMode::Warn);
    let (config, all_spec_files) = load_and_discover(root, gate_requested || machine_output);
    let spec_files = filter_specs(root, &all_spec_files, spec_filters);
    let spec_files = filter_by_status(&spec_files, exclude_status, only_status);
    let scores: Vec<scoring::SpecScore> = spec_files
        .iter()
        .map(|f| scoring::score_spec(f, root, &config))
        .collect();
    let project = scoring::compute_project_score(scores);
    let failing_scores: Vec<&scoring::SpecScore> = minimum_score
        .map(|minimum| {
            project
                .spec_scores
                .iter()
                .filter(|score| score.total < minimum)
                .collect()
        })
        .unwrap_or_default();
    let score_gate_passed =
        minimum_score.is_none() || (project.total_specs > 0 && failing_scores.is_empty());

    // Show progress header for batch/--all mode
    let batch_mode = all || spec_filters.is_empty();
    if batch_mode && !json && !matches!(format, types::OutputFormat::Csv) {
        println!(
            "  {} Scoring {} spec(s)...",
            "→".blue(),
            project.total_specs
        );
    }

    match format {
        types::OutputFormat::Json => {
            let specs_json: Vec<serde_json::Value> = project
                .spec_scores
                .iter()
                .map(|s| {
                    let mut entry = serde_json::json!({
                        "spec": s.spec_path,
                        "total": s.total,
                        "grade": s.grade,
                        "frontmatter": s.frontmatter_score,
                        "sections": s.sections_score,
                        "api": s.api_score,
                        "depth": s.depth_score,
                        "freshness": s.freshness_score,
                        "suggestions": s.suggestions,
                    });
                    if explain {
                        entry["explain"] = serde_json::to_value(&s.explain).unwrap_or_default();
                    }
                    entry
                })
                .collect();
            let output = serde_json::json!({
                "average_score": (project.average_score * 10.0).round() / 10.0,
                "grade": project.grade,
                "total_specs": project.total_specs,
                "minimum_score": minimum_score,
                "gate_passed": score_gate_passed,
                "distribution": {
                    "A": project.grade_distribution[0],
                    "B": project.grade_distribution[1],
                    "C": project.grade_distribution[2],
                    "D": project.grade_distribution[3],
                    "F": project.grade_distribution[4],
                },
                "specs": specs_json,
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        types::OutputFormat::Table => {
            print_table_output(&project, explain);
        }
        types::OutputFormat::Csv => {
            print_csv_output(&project);
        }
        _ => {
            print_text_output(&project, explain);
        }
    }

    if !score_gate_passed && !machine_output {
        match minimum_score {
            Some(minimum) if project.total_specs == 0 => {
                eprintln!(
                    "{}: no specs were scored, so the minimum score of {minimum} cannot be verified",
                    "score gate failed".red()
                );
            }
            Some(minimum) => {
                eprintln!(
                    "{}: {} spec(s) scored below {minimum}",
                    "score gate failed".red(),
                    failing_scores.len()
                );
                for score in &failing_scores {
                    eprintln!("  {}: {}/100", score.spec_path, score.total);
                }
            }
            None => {}
        }
    }

    // Honor the global gate flags. `score` is advisory by default (Warn config →
    // exit 0), but `--require-coverage`, `--enforcement enforce-new`, and a strict
    // config now gate the exit code — previously they were silent no-ops here.
    // score does no validation, so errors/warnings are 0; coverage and
    // unspecced-file gating still apply.
    let coverage = compute_coverage(root, &spec_files, &config);
    // JSON and CSV are machine formats: their body is already printed, so gate
    // SILENTLY via the exit code — appending exit_with_status's human message
    // would corrupt the parseable output. Other formats get the explanation.
    if machine_output {
        let validation_exit =
            compute_exit_code(0, 0, strict, enforcement, &coverage, require_coverage);
        std::process::exit(if score_gate_passed {
            validation_exit
        } else {
            1
        });
    }
    if !score_gate_passed {
        std::process::exit(1);
    }
    exit_with_status(0, 0, strict, enforcement, &coverage, require_coverage);
}

fn print_text_output(project: &scoring::ProjectScore, explain: bool) {
    println!(
        "\n--- {} ------------------------------------------------",
        "Spec Quality Scores".bold()
    );

    for s in &project.spec_scores {
        let grade_colored = colorize_grade(s.grade);

        println!(
            "\n  {} [{}] {}/100",
            s.spec_path.bold(),
            grade_colored,
            s.total
        );

        if explain {
            // Per-dimension breakdown with per-criterion lines
            for dim in &s.explain {
                let dim_score_str = colorize_subscore(dim.score);
                println!(
                    "    {:<14} {}/{} ",
                    dim.dimension.bold(),
                    dim_score_str,
                    dim.max_score
                );
                for crit in &dim.criteria {
                    let check = if crit.passed {
                        "✓".green().to_string()
                    } else {
                        "✗".red().to_string()
                    };
                    let pts = format!("{}/{}", crit.points, crit.max_points);
                    let pts_colored = if crit.passed {
                        pts.green().to_string()
                    } else {
                        pts.red().to_string()
                    };
                    let detail_str = crit
                        .detail
                        .as_deref()
                        .map(|d| format!("  — {d}"))
                        .unwrap_or_default();
                    println!(
                        "      {check} {:<26} ({pts_colored}){detail_str}",
                        crit.name,
                    );
                }
            }
        } else {
            println!(
                "    Frontmatter: {}/20  Sections: {}/20  API: {}/20  Depth: {}/20  Fresh: {}/20",
                s.frontmatter_score,
                s.sections_score,
                s.api_score,
                s.depth_score,
                s.freshness_score
            );
        }

        if !s.suggestions.is_empty() {
            for suggestion in &s.suggestions {
                println!("    {} {suggestion}", "->".cyan());
            }
        }
    }

    print_text_summary(project);
}

fn print_text_summary(project: &scoring::ProjectScore) {
    let avg_str = format!("{:.1}", project.average_score);
    let grade_colored = colorize_grade(project.grade);

    println!(
        "\n{} specs scored: average {avg_str}/100 [{}]",
        project.total_specs, grade_colored
    );
    println!(
        "  A: {}  B: {}  C: {}  D: {}  F: {}",
        project.grade_distribution[0],
        project.grade_distribution[1],
        project.grade_distribution[2],
        project.grade_distribution[3],
        project.grade_distribution[4]
    );
}

/// Render scores as an ASCII table.
fn print_table_output(project: &scoring::ProjectScore, explain: bool) {
    // Compute column widths
    let spec_width = project
        .spec_scores
        .iter()
        .map(|s| s.spec_path.len())
        .max()
        .unwrap_or(4)
        .max(4); // min width = "Spec"

    let header_sep = "-".repeat(spec_width + 2);

    if explain {
        println!(
            "\n{:<spec_width$}  {:>5}  {:>5}  {:>4}  {:>4}  {:>4}  {:>5}  {:>5}",
            "Spec",
            "Score",
            "Grade",
            "FM",
            "Sec",
            "API",
            "Depth",
            "Fresh",
            spec_width = spec_width
        );
        println!("{header_sep}  -----  -----  ----  ----  ----  -----  -----");
        for s in &project.spec_scores {
            println!(
                "{:<spec_width$}  {:>5}  {:>5}  {:>4}  {:>4}  {:>4}  {:>5}  {:>5}",
                s.spec_path,
                s.total,
                s.grade,
                s.frontmatter_score,
                s.sections_score,
                s.api_score,
                s.depth_score,
                s.freshness_score,
                spec_width = spec_width
            );
        }
        println!("{header_sep}  -----  -----  ----  ----  ----  -----  -----");
    } else {
        println!(
            "\n{:<spec_width$}  {:>5}  {:>5}",
            "Spec",
            "Score",
            "Grade",
            spec_width = spec_width
        );
        println!("{header_sep}  -----  -----");
        for s in &project.spec_scores {
            println!(
                "{:<spec_width$}  {:>5}  {:>5}",
                s.spec_path,
                s.total,
                s.grade,
                spec_width = spec_width
            );
        }
        println!("{header_sep}  -----  -----");
    }

    let avg_str = format!("{:.1}", project.average_score);
    println!(
        "\n{} specs  avg {}/100 [{}]  A:{} B:{} C:{} D:{} F:{}",
        project.total_specs,
        avg_str,
        project.grade,
        project.grade_distribution[0],
        project.grade_distribution[1],
        project.grade_distribution[2],
        project.grade_distribution[3],
        project.grade_distribution[4]
    );
}

/// Render scores as CSV for machine consumption / dashboards.
fn print_csv_output(project: &scoring::ProjectScore) {
    println!("spec,score,grade,frontmatter,sections,api,depth,freshness");
    for s in &project.spec_scores {
        println!(
            "{},{},{},{},{},{},{},{}",
            s.spec_path,
            s.total,
            s.grade,
            s.frontmatter_score,
            s.sections_score,
            s.api_score,
            s.depth_score,
            s.freshness_score
        );
    }
    // Summary row
    println!(
        "SUMMARY,{:.1},{},{},{},{},{},{},{}",
        project.average_score,
        project.grade,
        project.grade_distribution[0],
        project.grade_distribution[1],
        project.grade_distribution[2],
        project.grade_distribution[3],
        project.grade_distribution[4],
        project.total_specs
    );
}

/// Colorize a grade letter.
fn colorize_grade(grade: &str) -> String {
    match grade {
        "A" => grade.green().bold().to_string(),
        "B" => grade.green().to_string(),
        "C" => grade.yellow().to_string(),
        "D" => grade.yellow().bold().to_string(),
        _ => grade.red().bold().to_string(),
    }
}

/// Colorize a subscore (out of 20) — green for 20, yellow for 10-19, red for <10.
fn colorize_subscore(score: u32) -> String {
    let s = score.to_string();
    match score {
        20 => s.green().to_string(),
        10..=19 => s.yellow().to_string(),
        _ => s.red().to_string(),
    }
}
