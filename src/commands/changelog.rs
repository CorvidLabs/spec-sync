use colored::Colorize;
use std::path::Path;
use std::process;

use crate::changelog;
use crate::config::load_config;
use crate::types;

pub fn cmd_changelog(root: &Path, range: &str, format: types::OutputFormat) {
    let (from_ref, to_ref) = match changelog::parse_range(range) {
        Some(r) => r,
        None => {
            eprintln!(
                "{} Invalid range format. Expected FROM..TO (e.g., v0.1..v0.2 or HEAD~5..HEAD)",
                "Error:".red().bold()
            );
            process::exit(1);
        }
    };

    let config = load_config(root);
    let report = changelog::generate_changelog(root, &config.specs_dir, &from_ref, &to_ref);

    match format {
        types::OutputFormat::Json => {
            println!("{}", changelog::format_json(&report));
        }
        types::OutputFormat::Markdown => {
            print!("{}", changelog::format_markdown(&report));
        }
        types::OutputFormat::Text
        | types::OutputFormat::Github
        | types::OutputFormat::Table
        | types::OutputFormat::Csv => {
            print!("{}", changelog::format_text(&report));
        }
    }
}
