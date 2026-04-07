pub mod check;
pub mod compact;
pub mod coverage;
pub mod diff;
pub mod generate;
pub mod hooks;
pub mod init;
pub mod issues;
pub mod merge;
pub mod resolve;
pub mod score;
pub mod view;

use colored::Colorize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process;

use crate::config::load_config;
use crate::schema;
use crate::types;
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
        println!(
            "No spec files found in {}/. Run `specsync generate` to scaffold specs.",
            config.specs_dir
        );
        process::exit(0);
    }

    (config, spec_files)
}

/// Build column-level schema from migration files (if schema_dir is configured).
pub fn build_schema_columns(
    root: &Path,
    config: &types::SpecSyncConfig,
) -> HashMap<String, schema::SchemaTable> {
    match &config.schema_dir {
        Some(dir) => schema::build_schema(&root.join(dir)),
        None => HashMap::new(),
    }
}

/// Run validation, returning counts and collected error/warning strings.
/// When `collect` is true, errors/warnings are collected into vectors instead of printing inline.
pub fn run_validation(
    root: &Path,
    spec_files: &[PathBuf],
    schema_tables: &std::collections::HashSet<String>,
    schema_columns: &HashMap<String, schema::SchemaTable>,
    config: &types::SpecSyncConfig,
    collect: bool,
) -> (usize, usize, usize, usize, Vec<String>, Vec<String>) {
    let mut total_errors = 0;
    let mut total_warnings = 0;
    let mut passed = 0;
    let mut all_errors: Vec<String> = Vec::new();
    let mut all_warnings: Vec<String> = Vec::new();

    for spec_file in spec_files {
        let result = validate_spec(spec_file, root, schema_tables, schema_columns, config);

        if collect {
            let prefix = &result.spec_path;
            all_errors.extend(result.errors.iter().map(|e| format!("{prefix}: {e}")));
            all_warnings.extend(result.warnings.iter().map(|w| format!("{prefix}: {w}")));
            total_errors += result.errors.len();
            total_warnings += result.warnings.len();
            if result.errors.is_empty() {
                passed += 1;
            }
            continue;
        }

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
        let col_warnings: Vec<&str> = result
            .warnings
            .iter()
            .filter(|w| w.starts_with("Schema column"))
            .map(|s| s.as_str())
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
        let api_line = result.warnings.iter().find(|w| {
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

        let undocumented: Vec<&str> = result
            .warnings
            .iter()
            .filter(|w| w.starts_with("Export '"))
            .map(|s| s.as_str())
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
        for w in result
            .warnings
            .iter()
            .filter(|w| w.starts_with("Consumed By"))
        {
            println!("  {} {w}", "⚠".yellow());
        }

        // Show fix suggestions when there are errors
        if !result.fixes.is_empty() && !result.errors.is_empty() {
            println!("  {}", "Suggested fixes:".cyan());
            for fix in &result.fixes {
                println!("    {} {fix}", "->".cyan());
            }
        }

        total_errors += result.errors.len();
        total_warnings += result.warnings.len();
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

/// Compute exit code without printing or exiting.
pub fn compute_exit_code(
    total_errors: usize,
    total_warnings: usize,
    strict: bool,
    coverage: &types::CoverageReport,
    require_coverage: Option<usize>,
) -> i32 {
    if total_errors > 0 {
        return 1;
    }
    if strict && total_warnings > 0 {
        return 1;
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
    coverage: &types::CoverageReport,
    require_coverage: Option<usize>,
) {
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

