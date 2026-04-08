use colored::Colorize;
use std::path::Path;

use crate::scoring;
use crate::types;

use super::{filter_specs, load_and_discover};

pub fn cmd_score(root: &Path, format: types::OutputFormat, explain: bool, spec_filters: &[String]) {
    let json = matches!(format, types::OutputFormat::Json);
    let (config, all_spec_files) = load_and_discover(root, false);
    let spec_files = filter_specs(root, &all_spec_files, spec_filters);
    let scores: Vec<scoring::SpecScore> = spec_files
        .iter()
        .map(|f| scoring::score_spec(f, root, &config))
        .collect();
    let project = scoring::compute_project_score(scores);

    if json {
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
        return;
    }

    println!(
        "\n--- {} ------------------------------------------------",
        "Spec Quality Scores".bold()
    );

    for s in &project.spec_scores {
        let grade_colored = match s.grade {
            "A" => s.grade.green().bold().to_string(),
            "B" => s.grade.green().to_string(),
            "C" => s.grade.yellow().to_string(),
            "D" => s.grade.yellow().bold().to_string(),
            _ => s.grade.red().bold().to_string(),
        };

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

    let avg_str = format!("{:.1}", project.average_score);
    let grade_colored = match project.grade {
        "A" => project.grade.green().bold().to_string(),
        "B" => project.grade.green().to_string(),
        "C" => project.grade.yellow().to_string(),
        "D" => project.grade.yellow().bold().to_string(),
        _ => project.grade.red().bold().to_string(),
    };

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

/// Colorize a subscore (out of 20) — green for 20, yellow for 10-19, red for <10.
fn colorize_subscore(score: u32) -> String {
    let s = score.to_string();
    match score {
        20 => s.green().to_string(),
        10..=19 => s.yellow().to_string(),
        _ => s.red().to_string(),
    }
}
