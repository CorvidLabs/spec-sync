mod ai;
mod config;
mod exports;
mod generator;
mod parser;
mod types;
mod validator;
mod watch;

use clap::{Parser, Subcommand};
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use config::load_config;
use generator::{generate_specs_for_unspecced_modules, generate_specs_for_unspecced_modules_paths};
use validator::{compute_coverage, find_spec_files, get_schema_table_names, validate_spec};

#[derive(Parser)]
#[command(
    name = "specsync",
    about = "Bidirectional spec-to-code validation — language-agnostic, blazing fast",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Treat warnings as errors
    #[arg(long, global = true)]
    strict: bool,

    /// Fail if file coverage percent is below this threshold
    #[arg(long, value_name = "N", global = true)]
    require_coverage: Option<usize>,

    /// Project root directory (default: cwd)
    #[arg(long, global = true)]
    root: Option<PathBuf>,

    /// Output results as JSON instead of colored text
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Validate all specs against source code (default)
    Check,
    /// Show file and module coverage report
    Coverage,
    /// Scaffold spec files for unspecced modules
    Generate {
        /// Use Claude AI to generate meaningful spec content instead of templates
        #[arg(long)]
        ai: bool,
    },
    /// Create a specsync.json config file
    Init,
    /// Watch spec and source files, re-running check on changes
    Watch,
}

fn main() {
    let cli = Cli::parse();
    let root = cli
        .root
        .unwrap_or_else(|| std::env::current_dir().expect("Cannot determine cwd"));
    let root = root.canonicalize().unwrap_or(root);

    let command = cli.command.unwrap_or(Command::Check);

    match command {
        Command::Init => cmd_init(&root),
        Command::Check => cmd_check(&root, cli.strict, cli.require_coverage, cli.json),
        Command::Coverage => cmd_coverage(&root, cli.strict, cli.require_coverage, cli.json),
        Command::Generate { ai } => cmd_generate(&root, cli.strict, cli.require_coverage, cli.json, ai),
        Command::Watch => watch::run_watch(&root, cli.strict, cli.require_coverage),
    }
}

fn cmd_init(root: &Path) {
    let config_path = root.join("specsync.json");
    if config_path.exists() {
        println!("specsync.json already exists");
        return;
    }

    let default = serde_json::json!({
        "specsDir": "specs",
        "sourceDirs": ["src"],
        "requiredSections": [
            "Purpose",
            "Public API",
            "Invariants",
            "Behavioral Examples",
            "Error Cases",
            "Dependencies",
            "Change Log"
        ],
        "excludeDirs": ["__tests__"],
        "excludePatterns": ["**/__tests__/**", "**/*.test.ts", "**/*.spec.ts"]
    });

    let content = serde_json::to_string_pretty(&default).unwrap() + "\n";
    fs::write(&config_path, content).expect("Failed to write specsync.json");
    println!("{} Created specsync.json", "✓".green());
}

fn cmd_check(root: &Path, strict: bool, require_coverage: Option<usize>, json: bool) {
    let (config, spec_files) = load_and_discover(root, false);
    let schema_tables = get_schema_table_names(root, &config);
    let (total_errors, total_warnings, passed, total, all_errors, all_warnings) =
        run_validation(root, &spec_files, &schema_tables, &config, json);
    let coverage = compute_coverage(root, &spec_files, &config);

    if json {
        let exit_code = compute_exit_code(
            total_errors,
            total_warnings,
            strict,
            &coverage,
            require_coverage,
        );
        let output = serde_json::json!({
            "passed": exit_code == 0,
            "errors": all_errors,
            "warnings": all_warnings,
            "specs_checked": total,
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
        process::exit(exit_code);
    }

    print_summary(total, passed, total_warnings, total_errors);
    print_coverage_line(&coverage);
    exit_with_status(
        total_errors,
        total_warnings,
        strict,
        &coverage,
        require_coverage,
    );
}

fn cmd_coverage(root: &Path, strict: bool, require_coverage: Option<usize>, json: bool) {
    let (config, spec_files) = load_and_discover(root, false);
    let schema_tables = get_schema_table_names(root, &config);
    let (total_errors, total_warnings, passed, total, _all_errors, _all_warnings) =
        run_validation(root, &spec_files, &schema_tables, &config, json);
    let coverage = compute_coverage(root, &spec_files, &config);

    if json {
        let file_coverage = if coverage.total_source_files == 0 {
            100.0
        } else {
            (coverage.specced_file_count as f64 / coverage.total_source_files as f64) * 100.0
        };

        let loc_coverage = if coverage.total_loc == 0 {
            100.0
        } else {
            (coverage.specced_loc as f64 / coverage.total_loc as f64) * 100.0
        };

        let modules: Vec<serde_json::Value> = coverage
            .unspecced_modules
            .iter()
            .map(|m| serde_json::json!({ "name": m, "has_spec": false }))
            .collect();

        let uncovered_files: Vec<serde_json::Value> = coverage
            .unspecced_file_loc
            .iter()
            .map(|(f, loc)| serde_json::json!({ "file": f, "loc": loc }))
            .collect();

        let output = serde_json::json!({
            "file_coverage": (file_coverage * 100.0).round() / 100.0,
            "files_covered": coverage.specced_file_count,
            "files_total": coverage.total_source_files,
            "loc_coverage": (loc_coverage * 100.0).round() / 100.0,
            "loc_covered": coverage.specced_loc,
            "loc_total": coverage.total_loc,
            "modules": modules,
            "uncovered_files": uncovered_files,
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
        process::exit(0);
    }

    print_coverage_report(&coverage);
    print_summary(total, passed, total_warnings, total_errors);
    print_coverage_line(&coverage);
    exit_with_status(
        total_errors,
        total_warnings,
        strict,
        &coverage,
        require_coverage,
    );
}

fn cmd_generate(root: &Path, strict: bool, require_coverage: Option<usize>, json: bool, ai: bool) {
    let (config, spec_files) = load_and_discover(root, true);
    let schema_tables = get_schema_table_names(root, &config);

    let (mut total_errors, mut total_warnings, mut passed, mut total) = if spec_files.is_empty() {
        println!("No existing specs found. Scanning for source modules...");
        (0, 0, 0, 0)
    } else {
        let (te, tw, p, t, _, _) =
            run_validation(root, &spec_files, &schema_tables, &config, json);
        (te, tw, p, t)
    };

    let mut coverage = compute_coverage(root, &spec_files, &config);

    let ai_command = if ai {
        match ai::resolve_ai_command(&config) {
            Ok(cmd) => Some(cmd),
            Err(e) => {
                eprintln!("{e}");
                process::exit(1);
            }
        }
    } else {
        None
    };

    if json {
        let generated_paths =
            generate_specs_for_unspecced_modules_paths(root, &coverage, &config, ai_command.as_deref());
        let output = serde_json::json!({
            "generated": generated_paths,
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
        process::exit(0);
    }

    print_coverage_report(&coverage);

    println!(
        "\n--- {} -----------------------------------------------",
        if ai { "Generating Specs (AI)" } else { "Generating Specs" }.bold()
    );
    let generated =
        generate_specs_for_unspecced_modules(root, &coverage, &config, ai_command.as_deref());
    if generated == 0 && coverage.unspecced_modules.is_empty() {
        println!(
            "  {} No specs to generate — full module coverage",
            "✓".green()
        );
    } else if generated > 0 {
        println!(
            "\n  Generated {} spec file(s) — edit them to fill in details",
            generated
        );

        // Recompute coverage and validation now that new specs exist
        let (config, spec_files) = load_and_discover(root, true);
        let schema_tables = get_schema_table_names(root, &config);
        coverage = compute_coverage(root, &spec_files, &config);
        if !spec_files.is_empty() {
            let (te, tw, p, t, _, _) =
                run_validation(root, &spec_files, &schema_tables, &config, json);
            total_errors = te;
            total_warnings = tw;
            passed = p;
            total = t;
        }
    }

    print_summary(total, passed, total_warnings, total_errors);
    print_coverage_line(&coverage);
    exit_with_status(
        total_errors,
        total_warnings,
        strict,
        &coverage,
        require_coverage,
    );
}

// ─── Helpers ─────────────────────────────────────────────────────────────

fn load_and_discover(root: &Path, allow_empty: bool) -> (types::SpecSyncConfig, Vec<PathBuf>) {
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

/// Run validation, returning counts and collected error/warning strings.
fn run_validation(
    root: &Path,
    spec_files: &[PathBuf],
    schema_tables: &std::collections::HashSet<String>,
    config: &types::SpecSyncConfig,
    json: bool,
) -> (usize, usize, usize, usize, Vec<String>, Vec<String>) {
    let mut total_errors = 0;
    let mut total_warnings = 0;
    let mut passed = 0;
    let mut all_errors: Vec<String> = Vec::new();
    let mut all_warnings: Vec<String> = Vec::new();

    for spec_file in spec_files {
        let result = validate_spec(spec_file, root, schema_tables, config);

        if json {
            all_errors.extend(result.errors.iter().cloned());
            all_warnings.extend(result.warnings.iter().cloned());
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
fn compute_exit_code(
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

fn print_summary(total: usize, passed: usize, warnings: usize, _errors: usize) {
    let failed = total - passed;
    println!(
        "\n{total} specs checked: {} passed, {} warning(s), {} failed",
        passed.to_string().green(),
        warnings.to_string().yellow(),
        if failed > 0 {
            failed.to_string().red().to_string()
        } else {
            "0".to_string()
        }
    );
}

fn print_coverage_line(coverage: &types::CoverageReport) {
    let pct = coverage.coverage_percent;
    let pct_str = format!("{pct}%");
    let colored_pct = if pct == 100 {
        pct_str.green().to_string()
    } else if pct >= 80 {
        pct_str.yellow().to_string()
    } else {
        pct_str.red().to_string()
    };

    let loc_pct = coverage.loc_coverage_percent;
    let loc_pct_str = format!("{loc_pct}%");
    let colored_loc_pct = if loc_pct == 100 {
        loc_pct_str.green().to_string()
    } else if loc_pct >= 80 {
        loc_pct_str.yellow().to_string()
    } else {
        loc_pct_str.red().to_string()
    };

    println!(
        "File coverage: {}/{} ({colored_pct})",
        coverage.specced_file_count, coverage.total_source_files
    );
    println!(
        "LOC coverage:  {}/{} ({colored_loc_pct})",
        coverage.specced_loc, coverage.total_loc
    );
}

fn print_coverage_report(coverage: &types::CoverageReport) {
    println!(
        "\n--- {} ------------------------------------------------",
        "Coverage Report".bold()
    );

    if coverage.unspecced_modules.is_empty() {
        println!(
            "\n  {} All source modules have spec directories",
            "✓".green()
        );
    } else {
        println!(
            "\n  Modules without specs ({}):",
            coverage.unspecced_modules.len()
        );
        for module in &coverage.unspecced_modules {
            println!("    {} {module}/", "⚠".yellow());
        }
    }

    if coverage.unspecced_files.is_empty() {
        println!("  {} All source files referenced by specs", "✓".green());
    } else {
        let uncovered_loc: usize = coverage.unspecced_file_loc.iter().map(|(_, l)| l).sum();
        println!(
            "\n  Files not in any spec ({}, {} LOC uncovered):",
            coverage.unspecced_files.len(),
            uncovered_loc
        );
        for (file, loc) in &coverage.unspecced_file_loc {
            println!("    {} {file} ({loc} LOC)", "⚠".yellow());
        }
    }
}

fn exit_with_status(
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
