use std::path::Path;
use std::process;

use crate::output::{print_coverage_line, print_coverage_report, print_summary};
use crate::types;
use crate::validator::{compute_coverage, get_schema_table_names};

use super::{build_schema_columns, exit_with_status, load_and_discover, run_validation};

pub fn cmd_coverage(
    root: &Path,
    strict: bool,
    enforcement: Option<types::EnforcementMode>,
    require_coverage: Option<usize>,
    format: types::OutputFormat,
) {
    let json = matches!(format, types::OutputFormat::Json);
    let (config, spec_files) = load_and_discover(root, false);
    let enforcement = enforcement.unwrap_or(if strict {
        types::EnforcementMode::Strict
    } else {
        config.enforcement
    });
    let schema_tables = get_schema_table_names(root, &config);
    let schema_columns = build_schema_columns(root, &config);
    let (total_errors, total_warnings, passed, total, _all_errors, _all_warnings) = run_validation(
        root,
        &spec_files,
        &schema_tables,
        &schema_columns,
        &config,
        json,
    );
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
        enforcement,
        &coverage,
        require_coverage,
    );
}
