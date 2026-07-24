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
                        "{} Cannot determine GitHub repo. Use --repo, or set `repo` under `[github]` in .specsync/config.toml.",
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
            generator::generate_companion_files_for_spec(
                &spec_dir,
                &item.module_name,
                config.companions.design,
            );
            println!(
                "\n{} Run {} to validate and complete the imported details.",
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
                "{} Cannot determine GitHub repo. Use --repo, or set `repo` under `[github]` in .specsync/config.toml.",
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
                generator::generate_companion_files_for_spec(
                    &spec_dir,
                    &item.module_name,
                    config.companions.design,
                );
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
        let raw_stem = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        // `foo.spec.md` imports as module `foo`, not `foo-spec`.
        let stem = raw_stem.strip_suffix(".spec").unwrap_or(raw_stem);
        let file_display = file_path
            .strip_prefix(root)
            .unwrap_or(file_path)
            .to_string_lossy()
            .replace('\\', "/");
        print!("  {} {} ", progress.dimmed(), stem);

        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                println!("{} Failed to read: {e}", "✗".red());
                stats.errors += 1;
                continue;
            }
        };

        let import = match build_imported_spec(root, &config, stem, &file_display, &content) {
            Ok(import) => import,
            Err(reason) => {
                // Unparseable source files fail loudly — never a silent ✓ over
                // discarded content.
                println!("{} not imported: {reason}", "✗".red());
                stats.errors += 1;
                continue;
            }
        };

        let spec_dir = specs_dir.join(&import.module_name);
        let spec_file = spec_dir.join(format!("{}.spec.md", import.module_name));

        if spec_file.exists() {
            println!("{} skipped — spec already exists", "~".yellow());
            stats.skipped += 1;
            continue;
        }

        if let Err(e) = fs::create_dir_all(&spec_dir) {
            println!("{} Failed to create dir: {e}", "✗".red());
            stats.errors += 1;
            continue;
        }

        match fs::write(&spec_file, &import.spec_content) {
            Ok(_) => {
                let rel = spec_file.strip_prefix(root).unwrap_or(&spec_file).display();
                println!("{} → {}", "✓".green(), rel);
                for warning in &import.warnings {
                    println!("    {} {warning}", "⚠".yellow());
                }
                generator::generate_companion_files_for_spec(
                    &spec_dir,
                    &import.module_name,
                    config.companions.design,
                );
                stats.imported += 1;
            }
            Err(e) => {
                println!("{} Failed to write spec: {e}", "✗".red());
                stats.errors += 1;
            }
        }
    }

    print_batch_summary("import", &stats);
    if stats.errors > 0 {
        process::exit(1);
    }
}

/// The result of converting one markdown file into spec content.
struct FromDirImport {
    module_name: String,
    spec_content: String,
    warnings: Vec<String>,
}

/// Convert a markdown file into spec content, preserving everything parseable:
/// existing frontmatter fields, body sections, API tables, and changelogs are
/// kept verbatim; only missing pieces are scaffolded.
///
/// Returns `Err` for content that cannot be parsed at all (empty file, or a
/// frontmatter block that doesn't parse) — the caller must fail loudly rather
/// than overwrite the source with a skeleton.
fn build_imported_spec(
    root: &Path,
    config: &crate::types::SpecSyncConfig,
    stem: &str,
    file_display: &str,
    content: &str,
) -> Result<FromDirImport, String> {
    let module_name = importer::slugify(stem);
    if module_name.is_empty() {
        return Err(format!("could not derive a module name from '{stem}'"));
    }
    if content.trim().is_empty() {
        return Err("file is empty — nothing to import".to_string());
    }

    let normalized = content.trim_start_matches('\u{feff}').replace("\r\n", "\n");

    // Auto-detect source files so imported specs don't fail the tool's own
    // non-empty `files:` gate when the module's sources are discoverable.
    let mut detected: Vec<String> = generator::find_files_for_module(root, &module_name, config)
        .into_iter()
        .map(|f| {
            Path::new(&f)
                .strip_prefix(root)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or(f)
        })
        .collect();
    if detected.is_empty()
        && let Some(single) = generator::find_single_source_fallback(root, config)
    {
        detected.push(single);
    }
    detected.sort();

    let mut warnings = Vec::new();
    let files_yaml = if detected.is_empty() {
        warnings.push(format!(
            "no source files detected for '{module_name}' — `files:` is empty; \
             fill it in before `specsync check` will pass"
        ));
        "files: []".to_string()
    } else {
        let items: String = detected.iter().map(|f| format!("  - {f}\n")).collect();
        format!("files:\n{items}")
    };

    let import_note = format!("Imported from {file_display}");

    let (mut spec_text, body) = if normalized.starts_with("---\n") {
        // Existing spec-style file: preserve it, patch only what's missing.
        let Some(parsed) = crate::parser::parse_frontmatter(&normalized) else {
            return Err(
                "frontmatter block is present but could not be parsed — fix the YAML \
                 frontmatter (module/version/status/files) or remove it"
                    .to_string(),
            );
        };
        let patched = patch_frontmatter(
            &normalized,
            &module_name,
            &parsed.frontmatter,
            &files_yaml,
        );
        (patched, parsed.body)
    } else {
        // Plain markdown document: wrap the content in spec frontmatter and
        // keep the body verbatim below it.
        let body = ensure_h1_title(&normalized, stem);
        let fm = format!(
            "---\nmodule: {module_name}\nversion: 1\nstatus: draft\n{files_yaml}\ndb_tables: []\ndepends_on: []\n---\n"
        );
        (format!("{fm}\n{body}"), body)
    };

    // Scaffold only the required sections the source doesn't already have.
    let missing = crate::parser::get_missing_sections(&body, &config.required_sections);
    for section in &missing {
        spec_text.push_str(&stub_for_section(section, &import_note));
    }

    Ok(FromDirImport {
        module_name,
        spec_content: spec_text,
        warnings,
    })
}

/// Patch a spec's raw frontmatter text in place: fill in a missing `module`,
/// `version`, `status`, and a missing/empty `files:` list. Everything else —
/// including unknown/custom fields — is preserved verbatim.
fn patch_frontmatter(
    content: &str,
    module_name: &str,
    fm: &crate::types::Frontmatter,
    files_yaml: &str,
) -> String {
    let Some(fm_end) = content.find("\n---\n") else {
        return content.to_string();
    };
    let (fm_block, rest) = content.split_at(fm_end);
    let mut fm_block = fm_block.to_string();

    let mut insertions: Vec<String> = Vec::new();
    if fm.module.is_none() {
        insertions.push(format!("module: {module_name}"));
    }
    if fm.version.is_none() {
        insertions.push("version: 1".to_string());
    }
    if fm.status.is_none() {
        insertions.push("status: draft".to_string());
    }

    // Replace an empty files list (or add one when absent) with detected files.
    let files_block_re = regex::Regex::new(
        r"(?m)^files:\s*\[\]\s*$|^files:[ \t]*\n(?:[ \t]+-[ \t]+.+[ \t]*\n?)*",
    )
    .unwrap();
    if fm.files.is_empty() {
        if files_block_re.is_match(&fm_block) {
            if !files_yaml.starts_with("files: []") {
                fm_block = files_block_re
                    .replace(&fm_block, files_yaml.trim_end())
                    .to_string();
            }
        } else {
            // No `files:` key at all — add one explicitly (empty or detected).
            insertions.push(files_yaml.to_string());
        }
    }

    if !insertions.is_empty() {
        if !fm_block.ends_with('\n') {
            fm_block.push('\n');
        }
        fm_block.push_str(&insertions.join("\n"));
        fm_block.push('\n');
        // trim_end keeps the block newline-normalized before the closing `---`
        fm_block = format!("{}\n", fm_block.trim_end());
    }

    format!("{fm_block}{rest}")
}

/// Ensure the document body has an H1 title; derive one from the file stem if not.
fn ensure_h1_title(body: &str, stem: &str) -> String {
    if body.lines().any(|l| l.starts_with("# ")) {
        return body.to_string();
    }
    let title = stem
        .split(['-', '_'])
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    format!("# {title}\n\n{body}")
}

/// Scaffold content for one missing required section.
fn stub_for_section(section: &str, import_note: &str) -> String {
    match section {
        "Purpose" => "\n## Purpose\n\nDocument this module's responsibility, inputs, outputs, and ownership boundaries.\n".to_string(),
        "Public API" => "\n## Public API\n\n| Export | Description |\n|--------|-------------|\n".to_string(),
        "Invariants" => "\n## Invariants\n\n1. Define an invariant that must remain true for supported inputs.\n".to_string(),
        "Behavioral Examples" => "\n## Behavioral Examples\n\n### Scenario: Core behavior\n\n- **Given** precondition\n- **When** action\n- **Then** result\n".to_string(),
        "Error Cases" => "\n## Error Cases\n\n| Condition | Behavior |\n|-----------|----------|\n".to_string(),
        "Dependencies" => "\n## Dependencies\n\nList runtime dependencies and the specific symbols, services, or data they provide.\n".to_string(),
        "Change Log" => format!(
            "\n## Change Log\n\n| Date | Change |\n|------|--------|\n| {} | {import_note} |\n",
            import_today()
        ),
        other => format!("\n## {other}\n\nTODO: document {other}.\n"),
    }
}

/// Today's date as YYYY-MM-DD (no chrono dependency).
fn import_today() -> String {
    let output = std::process::Command::new("date")
        .args(["+%Y-%m-%d"])
        .output();
    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => "YYYY-MM-DD".to_string(),
    }
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


#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> crate::types::SpecSyncConfig {
        crate::types::SpecSyncConfig::default()
    }

    #[test]
    fn from_dir_preserves_existing_spec_content() {
        // #416: import must not discard frontmatter/sections/API tables.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/auth.rs"), "pub fn login() {}\n").unwrap();

        let source = "---\nmodule: auth\nversion: 3\nstatus: active\nfiles:\n  - src/auth.rs\ndb_tables: []\ndepends_on: []\ncustom_field: keepme\n---\n\n# Auth\n\n## Purpose\n\nHandles login sessions and token refresh.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2024-01-01 | Real history |\n";
        let import = build_imported_spec(root, &test_config(), "auth", "docs/auth.md", source)
            .expect("parseable spec imports");

        // Original content preserved verbatim.
        assert!(import.spec_content.contains("version: 3"));
        assert!(import.spec_content.contains("status: active"));
        assert!(import.spec_content.contains("custom_field: keepme"));
        assert!(import.spec_content.contains("Handles login sessions and token refresh."));
        assert!(import.spec_content.contains("2024-01-01 | Real history"));
        // No wrong attribution.
        assert!(!import.spec_content.contains("Confluence"));
        // Missing required sections scaffolded.
        assert!(import.spec_content.contains("## Invariants"));
        assert!(import.spec_content.contains("## Error Cases"));
    }

    #[test]
    fn from_dir_wraps_plain_markdown_without_losing_content() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let doc = "# Billing\n\nBilling handles invoices and payments.\n";
        let import = build_imported_spec(root, &test_config(), "billing", "docs/billing.md", doc)
            .expect("plain markdown imports");
        assert!(import.spec_content.contains("module: billing"));
        assert!(import.spec_content.contains("Billing handles invoices and payments."));
        assert!(import.spec_content.contains("## Purpose"));
        assert!(import.spec_content.contains("Imported from docs/billing.md"));
        assert!(!import.spec_content.contains("Confluence"));
    }

    #[test]
    fn from_dir_detects_source_files() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("src/billing")).unwrap();
        std::fs::write(root.join("src/billing/mod.rs"), "pub fn invoice() {}\n").unwrap();
        let import = build_imported_spec(root, &test_config(), "billing", "docs/billing.md", "# Billing\n")
            .unwrap();
        assert!(import.spec_content.contains("- src/billing/mod.rs"), "{}", import.spec_content);
        assert!(import.warnings.is_empty());
    }

    #[test]
    fn from_dir_fails_loudly_on_unparseable_input() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // Empty file
        assert!(build_imported_spec(root, &test_config(), "x", "x.md", "  \n").is_err());
        // Broken frontmatter (opening --- with no closing)
        assert!(
            build_imported_spec(root, &test_config(), "x", "x.md", "---\nmodule: x\nno closing\n")
                .is_err()
        );
    }

    #[test]
    fn from_dir_strips_spec_suffix_from_stem() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let import = build_imported_spec(root, &test_config(), "auth", "docs/auth.spec.md", "# Auth\n")
            .unwrap();
        assert_eq!(import.module_name, "auth");
    }
}
