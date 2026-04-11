pub mod archive_tasks;
pub mod changelog;
pub mod check;
pub mod comment;
pub mod compact;
pub mod coverage;
pub mod deps;
pub mod diff;
pub mod generate;
pub mod hooks;
pub mod import;
pub mod init;
pub mod init_registry;
pub mod issues;
pub mod lifecycle;
pub mod merge;
pub mod new;
pub mod report;
pub mod resolve;
pub mod rules;
pub mod scaffold;
pub mod score;
pub mod stale;
pub mod view;
pub mod wizard;

use colored::Colorize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process;

use crate::config::load_config;
use crate::ignore::IgnoreRules;
use crate::parser;
use crate::schema;
use crate::scoring;
use crate::types;
use crate::types::SpecStatus;
use crate::validator::{find_spec_files, validate_spec};

pub fn load_and_discover(root: &Path, allow_empty: bool) -> (types::SpecSyncConfig, Vec<PathBuf>) {
    let config = load_config(root);
    let specs_dir = root.join(&config.specs_dir);
    let spec_files: Vec<PathBuf> = find_spec_files(&specs_dir)
        .into_iter()
        .filter(|f| {
            f.file_name()
                .and_then(|n| n.to_str())
                .map(|n| !n.starts_with('_'))
                .unwrap_or(true)
        })
        .collect();

    if spec_files.is_empty() && !allow_empty {
        let abs_specs = root.join(&config.specs_dir);
        println!(
            "No spec files found in {}/. Run `specsync generate` to scaffold specs.",
            abs_specs.display()
        );
        process::exit(0);
    }

    (config, spec_files)
}

/// Filter spec files by user-provided spec names/paths.
/// Matches against: exact file path, relative path, module name (from filename stem).
/// Returns the full list if `filters` is empty.
pub fn filter_specs(root: &Path, spec_files: &[PathBuf], filters: &[String]) -> Vec<PathBuf> {
    if filters.is_empty() {
        return spec_files.to_vec();
    }

    let mut matched: Vec<PathBuf> = Vec::new();
    let mut unmatched: Vec<&String> = Vec::new();

    for filter in filters {
        let mut found = false;
        for spec_file in spec_files {
            let rel = spec_file
                .strip_prefix(root)
                .unwrap_or(spec_file)
                .to_string_lossy()
                .to_string();

            // Match by: exact path, relative path, filename, or module name
            let stem = spec_file.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let module = stem.strip_suffix(".spec").unwrap_or(stem);

            if rel == *filter
                || spec_file.to_string_lossy() == *filter
                || stem == *filter
                || module == *filter
                || filter.ends_with(".spec.md") && rel.ends_with(filter.as_str())
            {
                if !matched.contains(spec_file) {
                    matched.push(spec_file.clone());
                }
                found = true;
            }
        }
        if !found {
            unmatched.push(filter);
        }
    }

    if !unmatched.is_empty() {
        eprintln!(
            "{} No specs matched: {}",
            "Warning:".yellow(),
            unmatched
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    matched
}

/// Filter spec files by lifecycle status.
/// `exclude` removes specs with any of the listed statuses.
/// `only` keeps only specs with one of the listed statuses.
/// If both are empty, returns the full list unchanged.
pub fn filter_by_status(
    spec_files: &[PathBuf],
    exclude: &[String],
    only: &[String],
) -> Vec<PathBuf> {
    if exclude.is_empty() && only.is_empty() {
        return spec_files.to_vec();
    }

    let exclude_set: HashSet<SpecStatus> = exclude
        .iter()
        .filter_map(|s| SpecStatus::from_str_loose(s))
        .collect();
    let only_set: HashSet<SpecStatus> = only
        .iter()
        .filter_map(|s| SpecStatus::from_str_loose(s))
        .collect();

    spec_files
        .iter()
        .filter(|path| {
            let status = std::fs::read_to_string(path)
                .ok()
                .and_then(|content| parser::parse_frontmatter(&content.replace("\r\n", "\n")))
                .and_then(|parsed| parsed.frontmatter.parsed_status());

            // If we can't parse status: include when excluding (let validation catch the error),
            // but exclude when --only-status is active (no status ≠ matching status).
            let status = match status {
                Some(s) => s,
                None => return only_set.is_empty(),
            };

            if !exclude_set.is_empty() && exclude_set.contains(&status) {
                return false;
            }
            if !only_set.is_empty() && !only_set.contains(&status) {
                return false;
            }
            true
        })
        .cloned()
        .collect()
}

/// Build column-level schema from migration files (if schema_dir is configured).
pub fn build_schema_columns(
    root: &Path,
    config: &types::SpecSyncConfig,
) -> std::collections::HashMap<String, schema::SchemaTable> {
    match &config.schema_dir {
        Some(dir) => schema::build_schema(&root.join(dir)),
        None => std::collections::HashMap::new(),
    }
}

/// Run validation, returning counts and collected error/warning strings.
/// When `collect` is true, errors/warnings are collected into vectors instead of printing inline.
/// When `explain` is true (text mode), shows per-category score breakdown for each spec.
#[allow(clippy::too_many_arguments)]
pub fn run_validation(
    root: &Path,
    spec_files: &[PathBuf],
    schema_tables: &std::collections::HashSet<String>,
    schema_columns: &std::collections::HashMap<String, schema::SchemaTable>,
    config: &types::SpecSyncConfig,
    collect: bool,
    explain: bool,
    ignore_rules: &IgnoreRules,
) -> (usize, usize, usize, usize, Vec<String>, Vec<String>) {
    let mut total_errors = 0;
    let mut total_warnings = 0;
    let mut passed = 0;
    let mut all_errors: Vec<String> = Vec::new();
    let mut all_warnings: Vec<String> = Vec::new();

    for spec_file in spec_files {
        let result = validate_spec(spec_file, root, schema_tables, schema_columns, config);

        // Parse inline ignore directives from the spec file
        let inline_ignores = std::fs::read_to_string(spec_file)
            .map(|content| IgnoreRules::parse_inline(&content))
            .unwrap_or_default();

        // Filter out suppressed warnings
        let filtered_warnings: Vec<&String> = result
            .warnings
            .iter()
            .filter(|w| !ignore_rules.is_suppressed(w, &result.spec_path, &inline_ignores))
            .collect();

        if collect {
            let prefix = &result.spec_path;
            all_errors.extend(result.errors.iter().map(|e| format!("{prefix}: {e}")));
            all_warnings.extend(filtered_warnings.iter().map(|w| format!("{prefix}: {w}")));
            total_errors += result.errors.len();
            total_warnings += filtered_warnings.len();
            if result.errors.is_empty() {
                passed += 1;
            }
            continue;
        }

        // Use filtered warnings for text output
        let warnings: Vec<&str> = filtered_warnings.iter().map(|w| w.as_str()).collect();

        println!("\n{}", result.spec_path.bold());

        // Frontmatter check
        let has_fm_errors = result
            .errors
            .iter()
            .any(|e| e.starts_with("Frontmatter") || e.starts_with("Missing or malformed"));
        if has_fm_errors {
            println!("  {} Frontmatter valid", "✗".red());
        } else {
            println!("  {} Frontmatter valid", "✓".green());
        }

        // File existence
        let file_errors: Vec<&str> = result
            .errors
            .iter()
            .filter(|e| e.starts_with("Source file"))
            .map(|s| s.as_str())
            .collect();
        let has_files_field = !result.errors.iter().any(|e| e.contains("files (must be"));

        if file_errors.is_empty() && has_files_field {
            println!("  {} All source files exist", "✓".green());
        } else {
            for e in &file_errors {
                println!("  {} {e}", "✗".red());
            }
        }

        // DB table check
        let table_errors: Vec<&str> = result
            .errors
            .iter()
            .filter(|e| e.starts_with("DB table"))
            .map(|s| s.as_str())
            .collect();
        if !table_errors.is_empty() {
            for e in &table_errors {
                println!("  {} {e}", "✗".red());
            }
        } else if !schema_tables.is_empty() {
            println!("  {} All DB tables exist in schema", "✓".green());
        }

        // Schema column check
        let col_errors: Vec<&str> = result
            .errors
            .iter()
            .filter(|e| e.starts_with("Schema column"))
            .map(|s| s.as_str())
            .collect();
        let col_warnings: Vec<&str> = warnings
            .iter()
            .filter(|w| w.starts_with("Schema column"))
            .copied()
            .collect();
        for e in &col_errors {
            println!("  {} {e}", "✗".red());
        }
        for w in &col_warnings {
            println!("  {} {w}", "⚠".yellow());
        }

        // Section check
        let section_errors: Vec<&str> = result
            .errors
            .iter()
            .filter(|e| e.starts_with("Missing required section"))
            .map(|s| s.as_str())
            .collect();
        if section_errors.is_empty() {
            println!("  {} All required sections present", "✓".green());
        } else {
            for e in &section_errors {
                println!("  {} {e}", "✗".red());
            }
        }

        // API surface
        let api_line = warnings.iter().find(|w| {
            w.contains("exports documented")
                && w.chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
        });
        if let Some(line) = api_line {
            println!("  {} {line}", "✓".green());
        } else if let Some(ref summary) = result.export_summary {
            println!("  {} {summary}", "✓".green());
        }

        let spec_nonexistent: Vec<&str> = result
            .errors
            .iter()
            .filter(|e| e.starts_with("Spec documents"))
            .map(|s| s.as_str())
            .collect();
        for e in &spec_nonexistent {
            println!("  {} {e}", "✗".red());
        }

        let undocumented: Vec<&str> = warnings
            .iter()
            .filter(|w| w.starts_with("Export '") || w.starts_with("Undocumented export '"))
            .copied()
            .collect();
        for w in &undocumented {
            println!("  {} {w}", "⚠".yellow());
        }

        // Dependency check
        let dep_errors: Vec<&str> = result
            .errors
            .iter()
            .filter(|e| e.starts_with("Dependency spec"))
            .map(|s| s.as_str())
            .collect();
        if dep_errors.is_empty() {
            println!("  {} All dependency specs exist", "✓".green());
        } else {
            for e in &dep_errors {
                println!("  {} {e}", "✗".red());
            }
        }

        // Consumed-by warnings
        for w in warnings.iter().filter(|w| w.starts_with("Consumed By")) {
            println!("  {} {w}", "⚠".yellow());
        }

        // Stub section warnings
        for w in warnings
            .iter()
            .filter(|w| w.starts_with("Section ##") && w.contains("stub"))
        {
            println!("  {} {w}", "⚠".yellow());
        }

        // Requirements companion file warnings
        for w in warnings.iter().filter(|w| w.contains("requirements")) {
            println!("  {} {w}", "⚠".yellow());
        }

        // Custom rule violations and any other uncategorized warnings/errors
        let categorized_error_prefixes = [
            "Frontmatter",
            "Missing or malformed",
            "Source file",
            "DB table",
            "Schema column",
            "Missing required section",
            "Spec documents",
            "Dependency spec",
        ];
        let categorized_warning_prefixes = [
            "exports documented",
            "Export '",
            "Undocumented export '",
            "Consumed By",
            "Schema column",
        ];
        for e in result
            .errors
            .iter()
            .filter(|e| !categorized_error_prefixes.iter().any(|p| e.starts_with(p)))
        {
            println!("  {} {e}", "✗".red());
        }
        for w in warnings.iter().filter(|w| {
            !(categorized_warning_prefixes
                .iter()
                .any(|p| w.starts_with(p) || w.contains(p))
                || (w.starts_with("Section ##") && w.contains("stub"))
                || w.contains("requirements"))
        }) {
            println!("  {} {w}", "⚠".yellow());
        }

        // Show fix suggestions when there are errors or warnings with fixes
        if !result.fixes.is_empty() && (!result.errors.is_empty() || !warnings.is_empty()) {
            println!("  {}", "Suggested fixes:".cyan());
            for fix in &result.fixes {
                println!("    {} {fix}", "->".cyan());
            }
        }

        // --explain: show per-category score breakdown
        if explain {
            let score = scoring::score_spec(spec_file, root, config);
            let grade_colored = match score.grade {
                "A" => score.grade.green().bold().to_string(),
                "B" => score.grade.green().to_string(),
                "C" => score.grade.yellow().to_string(),
                "D" => score.grade.yellow().bold().to_string(),
                _ => score.grade.red().bold().to_string(),
            };
            println!(
                "  {} [{}] {}/100 — {} {}/20  {} {}/20  {} {}/20  {} {}/20  {} {}/20",
                "Score:".dimmed(),
                grade_colored,
                score.total,
                "FM:".dimmed(),
                colorize_subscore(score.frontmatter_score),
                "Sec:".dimmed(),
                colorize_subscore(score.sections_score),
                "API:".dimmed(),
                colorize_subscore(score.api_score),
                "Depth:".dimmed(),
                colorize_subscore(score.depth_score),
                "Fresh:".dimmed(),
                colorize_subscore(score.freshness_score),
            );
            for suggestion in &score.suggestions {
                println!("    {} {suggestion}", "->".cyan());
            }
        }

        total_errors += result.errors.len();
        total_warnings += warnings.len();
        if result.errors.is_empty() {
            passed += 1;
        }
    }

    (
        total_errors,
        total_warnings,
        passed,
        spec_files.len(),
        all_errors,
        all_warnings,
    )
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

/// Compute exit code without printing or exiting.
pub fn compute_exit_code(
    total_errors: usize,
    total_warnings: usize,
    strict: bool,
    enforcement: types::EnforcementMode,
    coverage: &types::CoverageReport,
    require_coverage: Option<usize>,
) -> i32 {
    use types::EnforcementMode::*;
    match enforcement {
        Warn => {
            // Non-blocking: always exit 0 regardless of errors or warnings.
        }
        EnforceNew => {
            // Block only if files without specs exist (not yet in the registry).
            if !coverage.unspecced_files.is_empty() {
                return 1;
            }
        }
        Strict => {
            // Block on any validation error; also block on warnings when --strict.
            if total_errors > 0 {
                return 1;
            }
            if strict && total_warnings > 0 {
                return 1;
            }
        }
    }
    if let Some(req) = require_coverage
        && coverage.coverage_percent < req
    {
        return 1;
    }
    0
}

pub fn exit_with_status(
    total_errors: usize,
    total_warnings: usize,
    strict: bool,
    enforcement: types::EnforcementMode,
    coverage: &types::CoverageReport,
    require_coverage: Option<usize>,
) {
    use types::EnforcementMode::*;
    match enforcement {
        Warn => {
            // Non-blocking: never exit non-zero from errors/warnings.
        }
        EnforceNew => {
            if !coverage.unspecced_files.is_empty() {
                println!(
                    "\n{}: {} file(s) not yet in the spec registry",
                    "--enforcement enforce-new".red(),
                    coverage.unspecced_files.len()
                );
                process::exit(1);
            }
        }
        Strict => {
            if total_errors > 0 {
                process::exit(1);
            }
            if strict && total_warnings > 0 {
                println!(
                    "\n{}: {total_warnings} warning(s) treated as errors",
                    "--strict mode".red()
                );
                process::exit(1);
            }
        }
    }

    if let Some(req) = require_coverage
        && coverage.coverage_percent < req
    {
        println!(
            "\n{} {req}%: actual coverage is {}% ({} file(s) missing specs)",
            "--require-coverage".red(),
            coverage.coverage_percent,
            coverage.unspecced_files.len()
        );
        for f in &coverage.unspecced_files {
            println!("  {} {f}", "✗".red());
        }
        process::exit(1);
    }
}

/// Create GitHub issues for specs with validation errors.
/// `all_errors` contains strings in the format `"spec/path: error message"`.
pub fn create_drift_issues(
    root: &Path,
    config: &types::SpecSyncConfig,
    all_errors: &[String],
    format: types::OutputFormat,
) {
    let repo_config = config.github.as_ref().and_then(|g| g.repo.as_deref());
    let repo = match crate::github::resolve_repo(repo_config, root) {
        Ok(r) => r,
        Err(e) => {
            if matches!(format, types::OutputFormat::Text) {
                eprintln!("{} Cannot create issues: {e}", "error:".red().bold());
            }
            return;
        }
    };

    let labels = config
        .github
        .as_ref()
        .map(|g| g.drift_labels.clone())
        .unwrap_or_else(|| vec!["spec-drift".to_string()]);

    // Group errors by spec path (format: "spec/path: error message")
    let mut errors_by_spec: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for entry in all_errors {
        if let Some((spec, error)) = entry.split_once(": ") {
            errors_by_spec
                .entry(spec.to_string())
                .or_default()
                .push(error.to_string());
        }
    }

    if matches!(format, types::OutputFormat::Text) {
        println!(
            "\n{} Creating GitHub issues for {} spec(s) with errors...",
            "⟳".cyan(),
            errors_by_spec.len()
        );
    }

    for (spec_path, errors) in &errors_by_spec {
        match crate::github::create_drift_issue(&repo, spec_path, errors, &labels) {
            Ok(issue) => {
                if matches!(format, types::OutputFormat::Text) {
                    println!(
                        "  {} Created issue #{} for {spec_path}: {}",
                        "✓".green(),
                        issue.number,
                        issue.url
                    );
                }
            }
            Err(e) => {
                if matches!(format, types::OutputFormat::Text) {
                    eprintln!(
                        "  {} Failed to create issue for {spec_path}: {e}",
                        "✗".red()
                    );
                }
            }
        }
    }
}
