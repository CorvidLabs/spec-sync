use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use crate::config::load_config;
use crate::generator;
use crate::github;
use crate::importer;

/// Result of a single import attempt (used for batch summary).
#[derive(Default)]
struct BatchStats {
    imported: usize,
    skipped: usize,
    errors: usize,
}

pub fn cmd_import(
    root: &Path,
    source: Option<&str>,
    id: Option<&str>,
    repo_override: Option<&str>,
    all_issues: bool,
    label: Option<&str>,
    from_dir: Option<&Path>,
) {
    // Route to batch or single import
    if all_issues {
        cmd_import_all_issues(root, repo_override, label);
        return;
    }
    if let Some(dir) = from_dir {
        cmd_import_from_dir(root, dir);
        return;
    }

    // Single import — source and id are required
    let source = source.unwrap_or_else(|| {
        eprintln!(
            "{} SOURCE is required. Use: specsync import <source> <id>",
            "Error:".red()
        );
        eprintln!(
            "  Or use {} or {} for batch import.",
            "--all-issues".bold(),
            "--from-dir".bold()
        );
        process::exit(1);
    });
    let id = id.unwrap_or_else(|| {
        eprintln!(
            "{} ID is required. Use: specsync import <source> <id>",
            "Error:".red()
        );
        process::exit(1);
    });

    cmd_import_single(root, source, id, repo_override);
}

fn cmd_import_single(root: &Path, source: &str, id: &str, repo_override: Option<&str>) {
    let config = load_config(root);
    let specs_dir = root.join(&config.specs_dir);

    let result = match source.to_lowercase().as_str() {
        "github" | "gh" => {
            let repo = repo_override
                .map(|r| r.to_string())
                .or_else(|| config.github.as_ref().and_then(|g| g.repo.clone()))
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

/// Batch import all open GitHub issues as spec drafts.
fn cmd_import_all_issues(root: &Path, repo_override: Option<&str>, label: Option<&str>) {
    let config = load_config(root);
    let specs_dir = root.join(&config.specs_dir);

    let repo = repo_override
        .map(|r| r.to_string())
        .or_else(|| config.github.as_ref().and_then(|g| g.repo.clone()))
        .or_else(|| github::detect_repo(root))
        .unwrap_or_else(|| {
            eprintln!(
                "{} Cannot determine GitHub repo. Use --repo or set github.repo in specsync.json.",
                "Error:".red()
            );
            process::exit(1);
        });

    let label_display = label.map(|l| format!(" (label: {l})")).unwrap_or_default();
    println!(
        "\n--- {} -----------------------------------------------",
        "Batch Import: GitHub Issues".bold()
    );
    println!(
        "  {} Fetching open issues from {repo}{label_display}...",
        "→".blue()
    );

    let issues = match github::list_issues(&repo, label) {
        Ok(issues) => issues,
        Err(e) => {
            eprintln!("{} {e}", "Error:".red());
            process::exit(1);
        }
    };

    if issues.is_empty() {
        println!("  {} No open issues found.", "i".blue());
        return;
    }

    println!(
        "  {} Found {} issue(s) to import\n",
        "i".blue(),
        issues.len()
    );

    let mut stats = BatchStats::default();
    let total = issues.len();

    for (idx, issue) in issues.iter().enumerate() {
        let progress = format!("[{}/{}]", idx + 1, total);
        print!("  {} ", progress.dimmed());

        let result = importer::import_github_issue(&repo, issue.number);
        let item = match result {
            Ok(item) => item,
            Err(e) => {
                println!("{} #{}: {}", "✗".red(), issue.number, e);
                stats.errors += 1;
                continue;
            }
        };

        let spec_dir = specs_dir.join(&item.module_name);
        let spec_file = spec_dir.join(format!("{}.spec.md", item.module_name));

        if spec_file.exists() {
            println!(
                "{} #{} skipped — spec already exists: {}",
                "~".yellow(),
                issue.number,
                item.module_name
            );
            stats.skipped += 1;
            continue;
        }

        let spec_content = importer::render_spec(&item);

        if let Err(e) = fs::create_dir_all(&spec_dir) {
            println!("{} #{}: Failed to create dir: {e}", "✗".red(), issue.number);
            stats.errors += 1;
            continue;
        }

        match fs::write(&spec_file, &spec_content) {
            Ok(_) => {
                let rel = spec_file.strip_prefix(root).unwrap_or(&spec_file).display();
                println!("{} #{} → {}", "✓".green(), issue.number, rel);
                generator::generate_companion_files_for_spec(&spec_dir, &item.module_name);
                stats.imported += 1;
            }
            Err(e) => {
                println!("{} #{}: Failed to write spec: {e}", "✗".red(), issue.number);
                stats.errors += 1;
            }
        }
    }

    print_batch_summary("import", &stats);
}

/// Batch import all markdown files from a directory as spec drafts.
fn cmd_import_from_dir(root: &Path, dir: &Path) {
    let config = load_config(root);
    let specs_dir = root.join(&config.specs_dir);

    let dir = if dir.is_absolute() {
        dir.to_path_buf()
    } else {
        root.join(dir)
    };

    if !dir.exists() {
        eprintln!("{} Directory not found: {}", "Error:".red(), dir.display());
        process::exit(1);
    }

    println!(
        "\n--- {} -----------------------------------------------",
        "Batch Import: Directory".bold()
    );
    println!(
        "  {} Scanning {} for markdown files...",
        "→".blue(),
        dir.display()
    );

    // Collect all .md files in the directory (non-recursive by default)
    let md_files = collect_markdown_files(&dir);

    if md_files.is_empty() {
        println!(
            "  {} No markdown files found in {}",
            "i".blue(),
            dir.display()
        );
        return;
    }

    println!(
        "  {} Found {} file(s) to import\n",
        "i".blue(),
        md_files.len()
    );

    let mut stats = BatchStats::default();
    let total = md_files.len();

    for (idx, file_path) in md_files.iter().enumerate() {
        let progress = format!("[{}/{}]", idx + 1, total);
        let filename = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        print!("  {} {} ", progress.dimmed(), filename);

        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                println!("{} Failed to read: {e}", "✗".red());
                stats.errors += 1;
                continue;
            }
        };

        let item = parse_markdown_as_import_item(filename, &content);

        let spec_dir = specs_dir.join(&item.module_name);
        let spec_file = spec_dir.join(format!("{}.spec.md", item.module_name));

        if spec_file.exists() {
            println!("{} skipped — spec already exists", "~".yellow());
            stats.skipped += 1;
            continue;
        }

        let spec_content = importer::render_spec(&item);

        if let Err(e) = fs::create_dir_all(&spec_dir) {
            println!("{} Failed to create dir: {e}", "✗".red());
            stats.errors += 1;
            continue;
        }

        match fs::write(&spec_file, &spec_content) {
            Ok(_) => {
                let rel = spec_file.strip_prefix(root).unwrap_or(&spec_file).display();
                println!("{} → {}", "✓".green(), rel);
                generator::generate_companion_files_for_spec(&spec_dir, &item.module_name);
                stats.imported += 1;
            }
            Err(e) => {
                println!("{} Failed to write spec: {e}", "✗".red());
                stats.errors += 1;
            }
        }
    }

    print_batch_summary("import", &stats);
}

/// Collect all .md files in a directory (one level deep).
fn collect_markdown_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return files,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file()
            && path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e == "md")
                .unwrap_or(false)
        {
            files.push(path);
        }
    }
    files.sort();
    files
}

/// Parse a markdown file into an ImportedItem for spec generation.
fn parse_markdown_as_import_item(filename: &str, content: &str) -> importer::ImportedItem {
    // Extract title from first H1 heading, fall back to filename
    let title = content
        .lines()
        .find(|l| l.starts_with("# "))
        .map(|l| l.trim_start_matches("# ").trim().to_string())
        .unwrap_or_else(|| filename.to_string());

    // Purpose: first non-empty paragraph after the title (or the title itself)
    let purpose = content
        .lines()
        .skip_while(|l| l.starts_with("# ") || l.trim().is_empty())
        .find(|l| !l.trim().is_empty())
        .unwrap_or(&title)
        .trim()
        .to_string();

    let requirements = importer::extract_requirements_pub(content);
    let module_name = importer::slugify(filename);

    importer::ImportedItem {
        module_name,
        purpose,
        requirements,
        labels: Vec::new(),
        source_url: String::new(),
        issue_number: None,
        source_type: importer::ImportSource::Confluence, // closest semantic match for "doc"
    }
}

fn print_batch_summary(operation: &str, stats: &BatchStats) {
    let total = stats.imported + stats.skipped + stats.errors;
    println!(
        "\n{} Batch {operation} complete: {} imported, {} skipped, {} error(s) ({} total)",
        "→".blue(),
        stats.imported.to_string().green(),
        stats.skipped.to_string().yellow(),
        if stats.errors > 0 {
            stats.errors.to_string().red().to_string()
        } else {
            stats.errors.to_string()
        },
        total
    );
    if stats.imported > 0 {
        println!(
            "\n{} Run {} to validate imported specs.",
            "Tip:".cyan().bold(),
            "specsync check".bold()
        );
    }
}
