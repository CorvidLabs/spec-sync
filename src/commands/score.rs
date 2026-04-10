use colored::Colorize;
use std::path::Path;

use crate::scoring;
use crate::types;

use super::{filter_specs, load_and_discover};

pub fn cmd_score(
    root: &Path,
    format: types::OutputFormat,
    explain: bool,
    all: bool,
    spec_filters: &[String],
) {
    let json = matches!(format, types::OutputFormat::Json);
    let (config, all_spec_files) = load_and_discover(root, false);
    let spec_files = filter_specs(root, &all_spec_files, spec_filters);
    let scores: Vec<scoring::SpecScore> = spec_files
        .iter()
        .map(|f| scoring::score_spec(f, root, &config))
        .collect();
    let project = scoring::compute_project_score(scores);

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
                    serde_json::json!({
                        "spec": s.spec_path,
                        "total": s.total,
                        "grade": s.grade,
                        "frontmatter": s.frontmatter_score,
                        "sections": s.sections_score,
                        "api": s.api_score,
                        "depth": s.depth_score,
                        "freshness": s.freshness_score,
                        "suggestions": s.suggestions,
                    })
                })
                .collect();
            let output = serde_json::json!({
                "average_score": (project.average_score * 10.0).round() / 10.0,
                "grade": project.grade,
                "total_specs": project.total_specs,
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
            // Show color-coded per-category bars
            println!(
                "    {} {}/20  {} {}/20  {} {}/20  {} {}/20  {} {}/20",
                "Frontmatter:".dimmed(),
                colorize_subscore(s.frontmatter_score),
                "Sections:".dimmed(),
                colorize_subscore(s.sections_score),
                "API:".dimmed(),
                colorize_subscore(s.api_score),
                "Depth:".dimmed(),
                colorize_subscore(s.depth_score),
                "Fresh:".dimmed(),
                colorize_subscore(s.freshness_score),
            );
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
