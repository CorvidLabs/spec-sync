use colored::Colorize;
use std::path::Path;
use std::process;

use crate::changelog;
use crate::config::load_config;
use crate::types;

pub fn cmd_changelog(root: &Path, range: &str, format: types::OutputFormat) {
    let (from_ref, to_ref, three_dot) = match changelog::parse_range_full(range) {
        Some(r) => r,
        None => {
            eprintln!(
                "{} Invalid range format. Expected FROM..TO (e.g., v0.1..v0.2 or HEAD~5..HEAD)",
                "Error:".red().bold()
            );
            process::exit(1);
        }
    };

    // Validate BOTH endpoints before comparing: an unresolvable ref used to be
    // silently treated as an empty tree, fabricating a plausible changelog
    // with exit 0 — exactly the wrong output for CI (#418).
    let to_ref = match changelog::resolve_ref(root, &to_ref) {
        Ok(_) => to_ref,
        Err(e) => {
            eprintln!("{} {e}", "Error:".red().bold());
            process::exit(1);
        }
    };
    let from_ref = match changelog::resolve_ref(root, &from_ref) {
        Ok(_) => from_ref,
        Err(e) => {
            eprintln!("{} {e}", "Error:".red().bold());
            process::exit(1);
        }
    };

    // Three-dot range (A...B): compare from the real merge-base, not from A.
    let from_ref = if three_dot {
        match changelog::merge_base(root, &from_ref, &to_ref) {
            Ok(base) => base,
            Err(e) => {
                eprintln!("{} {e}", "Error:".red().bold());
                process::exit(1);
            }
        }
    } else {
        from_ref
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
