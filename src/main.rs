mod config;
mod exports;
mod generator;
mod parser;
mod types;
mod validator;

use clap::{Parser, Subcommand};
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use config::load_config;
use generator::generate_specs_for_unspecced_modules;
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
}

#[derive(Subcommand)]
enum Command {
    /// Validate all specs against source code (default)
    Check,
    /// Show file and module coverage report
    Coverage,
    /// Scaffold spec files for unspecced modules
    Generate,
    /// Create a specsync.json config file
    Init,
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
        Command::Check => cmd_check(&root, cli.strict, cli.require_coverage),
        Command::Coverage => cmd_coverage(&root, cli.strict, cli.require_coverage),
        Command::Generate => cmd_generate(&root, cli.strict, cli.require_coverage),
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

fn cmd_check(root: &Path, strict: bool, require_coverage: Option<usize>) {
    let (config, spec_files) = load_and_discover(root);
    let schema_tables = get_schema_table_names(root, &config);
    let (total_errors, total_warnings, passed, total) =
        run_validation(root, &spec_files, &schema_tables, &config);
    let coverage = compute_coverage(root, &spec_files, &config);

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

fn cmd_coverage(root: &Path, strict: bool, require_coverage: Option<usize>) {
    let (config, spec_files) = load_and_discover(root);
    let schema_tables = get_schema_table_names(root, &config);
    let (total_errors, total_warnings, passed, total) =
        run_validation(root, &spec_files, &schema_tables, &config);
    let coverage = compute_coverage(root, &spec_files, &config);

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

fn cmd_generate(root: &Path, strict: bool, require_coverage: Option<usize>) {
    let (config, spec_files) = load_and_discover(root);
    let schema_tables = get_schema_table_names(root, &config);
    let (total_errors, total_warnings, passed, total) =
        run_validation(root, &spec_files, &schema_tables, &config);
    let coverage = compute_coverage(root, &spec_files, &config);

    print_coverage_report(&coverage);

    println!(
        "\n--- {} -----------------------------------------------",
        "Generating Specs".bold()
    );
    let generated = generate_specs_for_unspecced_modules(root, &coverage, &config);
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

fn load_and_discover(root: &Path) -> (types::SpecSyncConfig, Vec<PathBuf>) {
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

    if spec_files.is_empty() {
        println!("No spec files found in {}/", config.specs_dir);
        process::exit(0);
    }

    (config, spec_files)
}

fn run_validation(
    root: &Path,
    spec_files: &[PathBuf],
    schema_tables: &std::collections::HashSet<String>,
    config: &types::SpecSyncConfig,
) -> (usize, usize, usize, usize) {
    let mut total_errors = 0;
    let mut total_warnings = 0;
    let mut passed = 0;

    for spec_file in spec_files {
        let result = validate_spec(spec_file, root, schema_tables, config);

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

    (total_errors, total_warnings, passed, spec_files.len())
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

    println!(
        "File coverage: {}/{} ({colored_pct})",
        coverage.specced_file_count, coverage.total_source_files
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
        println!(
            "\n  Files not in any spec ({}):",
            coverage.unspecced_files.len()
        );
        for file in &coverage.unspecced_files {
            println!("    {} {file}", "⚠".yellow());
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
        && coverage.coverage_percent < req {
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
