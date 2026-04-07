use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process;

use crate::config::load_config;
use crate::generator;
use crate::github;
use crate::importer;

pub fn cmd_import(root: &Path, source: &str, id: &str, repo_override: Option<&str>) {
    let config = load_config(root);
    let specs_dir = root.join(&config.specs_dir);

    let result = match source.to_lowercase().as_str() {
        "github" | "gh" => {
            let repo = repo_override
                .map(|r| r.to_string())
                .or_else(|| {
                    config
                        .github
                        .as_ref()
                        .and_then(|g| g.repo.clone())
                })
                .or_else(|| github::detect_repo(root))
                .unwrap_or_else(|| {
                    eprintln!(
                        "{} Cannot determine GitHub repo. Use --repo or set github.repo in specsync.json.",
                        "Error:".red()
                    );
                    process::exit(1);
                });

            let number: u64 = id.parse().unwrap_or_else(|_| {
                eprintln!("{} Invalid issue number: {id}", "Error:".red());
                process::exit(1);
            });

            println!(
                "  {} Fetching GitHub issue #{number} from {repo}...",
                "→".blue()
            );
            importer::import_github_issue(&repo, number)
        }
        "jira" => {
            println!("  {} Fetching Jira issue {id}...", "→".blue());
            importer::import_jira_issue(id)
        }
        "confluence" | "wiki" => {
            println!("  {} Fetching Confluence page {id}...", "→".blue());
            importer::import_confluence_page(id)
        }
        _ => {
            eprintln!(
                "{} Unknown source '{}'. Supported: github, jira, confluence",
                "Error:".red(),
                source
            );
            process::exit(1);
        }
    };

    let item = match result {
        Ok(item) => item,
        Err(e) => {
            eprintln!("{} {e}", "Error:".red());
            process::exit(1);
        }
    };

    println!("  {} Imported: {}", "✓".green(), item.purpose);
    if !item.requirements.is_empty() {
        println!(
            "  {} Extracted {} requirement(s)",
            "i".blue(),
            item.requirements.len()
        );
    }

    let spec_dir = specs_dir.join(&item.module_name);
    let spec_file = spec_dir.join(format!("{}.spec.md", item.module_name));

    if spec_file.exists() {
        eprintln!(
            "{} Spec already exists: {}",
            "!".yellow(),
            spec_file.strip_prefix(root).unwrap_or(&spec_file).display()
        );
        process::exit(1);
    }

    let spec_content = importer::render_spec(&item);

    if let Err(e) = fs::create_dir_all(&spec_dir) {
        eprintln!("Failed to create {}: {e}", spec_dir.display());
        process::exit(1);
    }

    match fs::write(&spec_file, &spec_content) {
        Ok(_) => {
            let rel = spec_file.strip_prefix(root).unwrap_or(&spec_file).display();
            println!("  {} Created {rel}", "✓".green());
            generator::generate_companion_files_for_spec(&spec_dir, &item.module_name);
            println!(
                "\n{} Run {} to validate and fill in the details.",
                "Tip:".cyan().bold(),
                "specsync check".bold()
            );
        }
        Err(e) => {
            eprintln!("Failed to write {}: {e}", spec_file.display());
            process::exit(1);
        }
    }
}
