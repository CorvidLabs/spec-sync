pub(crate) mod check;
pub(crate) mod compact;
pub(crate) mod coverage;
pub(crate) mod diff;
pub(crate) mod generate;
pub(crate) mod hooks;
pub(crate) mod import;
pub(crate) mod init;
pub(crate) mod issues;
pub(crate) mod merge;
pub(crate) mod resolve;
pub(crate) mod score;
pub(crate) mod view;
pub(crate) mod wizard;

use colored::Colorize;
use std::path::{Path, PathBuf};

use crate::config::load_config;
use crate::schema;
use crate::types;
use crate::validator::{find_spec_files, validate_spec};

pub(crate) fn load_and_discover(
    root: &Path,
    allow_empty: bool,
) -> (types::SpecSyncConfig, Vec<PathBuf>) {
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
        std::process::exit(0);
    }

    (config, spec_files)
}

/// Build column-level schema from migration files (if schema_dir is configured).
pub(crate) fn build_schema_columns(
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
pub(crate) fn run_validation(
    root: &Path,
    spec_files: &[PathBuf],
    schema_tables: &std::collections::HashSet<String>,
    schema_columns: &std::collections::HashMap<String, schema::SchemaTable>,
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
pub(crate) fn compute_exit_code(
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

pub(crate) fn exit_with_status(
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
                std::process::exit(1);
            }
        }
        Strict => {
            if total_errors > 0 {
                std::process::exit(1);
            }
            if strict && total_warnings > 0 {
                println!(
                    "\n{}: {total_warnings} warning(s) treated as errors",
                    "--strict mode".red()
                );
                std::process::exit(1);
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
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_coverage() -> types::CoverageReport {
        types::CoverageReport {
            total_source_files: 0,
            specced_file_count: 0,
            unspecced_files: vec![],
            unspecced_modules: vec![],
            coverage_percent: 100,
            total_loc: 0,
            specced_loc: 0,
            loc_coverage_percent: 100,
            unspecced_file_loc: vec![],
        }
    }

    fn coverage_with_unspecced(files: Vec<&str>) -> types::CoverageReport {
        let total = files.len();
        types::CoverageReport {
            total_source_files: total,
            specced_file_count: 0,
            unspecced_files: files.iter().map(|s| s.to_string()).collect(),
            unspecced_modules: vec![],
            coverage_percent: 0,
            total_loc: 0,
            specced_loc: 0,
            loc_coverage_percent: 0,
            unspecced_file_loc: vec![],
        }
    }

    // --- Warn mode ---

    #[test]
    fn warn_mode_exits_0_with_no_errors() {
        let coverage = empty_coverage();
        assert_eq!(
            compute_exit_code(0, 0, false, types::EnforcementMode::Warn, &coverage, None),
            0
        );
    }

    #[test]
    fn warn_mode_exits_0_even_with_errors() {
        let coverage = empty_coverage();
        assert_eq!(
            compute_exit_code(5, 3, false, types::EnforcementMode::Warn, &coverage, None),
            0
        );
    }

    #[test]
    fn warn_mode_exits_0_even_with_strict_flag() {
        let coverage = empty_coverage();
        assert_eq!(
            compute_exit_code(0, 3, true, types::EnforcementMode::Warn, &coverage, None),
            0
        );
    }

    #[test]
    fn warn_mode_respects_require_coverage() {
        let coverage = types::CoverageReport {
            coverage_percent: 50,
            ..empty_coverage()
        };
        assert_eq!(
            compute_exit_code(
                0,
                0,
                false,
                types::EnforcementMode::Warn,
                &coverage,
                Some(80)
            ),
            1
        );
    }

    // --- EnforceNew mode ---

    #[test]
    fn enforce_new_exits_0_when_all_files_specced() {
        let coverage = empty_coverage();
        assert_eq!(
            compute_exit_code(
                0,
                0,
                false,
                types::EnforcementMode::EnforceNew,
                &coverage,
                None
            ),
            0
        );
    }

    #[test]
    fn enforce_new_exits_0_with_errors_if_all_specced() {
        let coverage = empty_coverage();
        assert_eq!(
            compute_exit_code(
                3,
                2,
                false,
                types::EnforcementMode::EnforceNew,
                &coverage,
                None
            ),
            0
        );
    }

    #[test]
    fn enforce_new_exits_1_when_unspecced_files_exist() {
        let coverage = coverage_with_unspecced(vec!["src/foo.rs"]);
        assert_eq!(
            compute_exit_code(
                0,
                0,
                false,
                types::EnforcementMode::EnforceNew,
                &coverage,
                None
            ),
            1
        );
    }

    #[test]
    fn enforce_new_exits_1_with_multiple_unspecced_files() {
        let coverage = coverage_with_unspecced(vec!["src/foo.rs", "src/bar.rs"]);
        assert_eq!(
            compute_exit_code(
                0,
                0,
                false,
                types::EnforcementMode::EnforceNew,
                &coverage,
                None
            ),
            1
        );
    }

    // --- Strict mode ---

    #[test]
    fn strict_mode_exits_0_with_no_errors() {
        let coverage = empty_coverage();
        assert_eq!(
            compute_exit_code(0, 0, false, types::EnforcementMode::Strict, &coverage, None),
            0
        );
    }

    #[test]
    fn strict_mode_exits_1_with_errors() {
        let coverage = empty_coverage();
        assert_eq!(
            compute_exit_code(1, 0, false, types::EnforcementMode::Strict, &coverage, None),
            1
        );
    }

    #[test]
    fn strict_mode_exits_0_with_warnings_only() {
        let coverage = empty_coverage();
        assert_eq!(
            compute_exit_code(0, 3, false, types::EnforcementMode::Strict, &coverage, None),
            0
        );
    }

    #[test]
    fn strict_mode_exits_1_with_warnings_and_strict_flag() {
        let coverage = empty_coverage();
        assert_eq!(
            compute_exit_code(0, 3, true, types::EnforcementMode::Strict, &coverage, None),
            1
        );
    }

    #[test]
    fn strict_mode_respects_require_coverage() {
        let coverage = types::CoverageReport {
            coverage_percent: 70,
            ..empty_coverage()
        };
        assert_eq!(
            compute_exit_code(
                0,
                0,
                false,
                types::EnforcementMode::Strict,
                &coverage,
                Some(80)
            ),
            1
        );
    }

    #[test]
    fn strict_mode_exits_0_when_coverage_meets_threshold() {
        let coverage = types::CoverageReport {
            coverage_percent: 85,
            ..empty_coverage()
        };
        assert_eq!(
            compute_exit_code(
                0,
                0,
                false,
                types::EnforcementMode::Strict,
                &coverage,
                Some(80)
            ),
            0
        );
    }
}
