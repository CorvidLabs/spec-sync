use std::path::Path;
use std::process;

use crate::output::{percent_json, print_coverage_line, print_coverage_report, print_summary};
use crate::types;
use crate::validator::compute_coverage_checked;

use super::{compute_exit_code, exit_with_status, load_and_discover, run_validation};

pub fn cmd_coverage(
    root: &Path,
    strict: bool,
    enforcement: Option<types::EnforcementMode>,
    require_coverage: Option<usize>,
    format: types::OutputFormat,
) {
    let json = matches!(format, types::OutputFormat::Json);
    // `coverage` is a gate/report command: it must evaluate the requested gate
    // even on a project with source but NO specs (0% coverage). Passing
    // allow_empty=true stops load_and_discover from taking its no-spec early-exit
    // (which returned exit 0 and bypassed --require-coverage/--enforcement/--strict
    // and any config enforcement — finding M1).
    let (config, spec_files) = load_and_discover(root, true);
    let enforcement = enforcement.unwrap_or(if strict {
        types::EnforcementMode::Strict
    } else {
        config.enforcement
    });
    let ignore_rules = crate::ignore::IgnoreRules::default();
    let (total_errors, total_warnings, passed, total, _all_errors, _all_warnings, _all_notices) =
        run_validation(
            root,
            &spec_files,
            &spec_files,
            &config,
            json,
            false,
            &ignore_rules,
        );
    let coverage = match compute_coverage_checked(root, &spec_files, &config) {
        Ok(coverage) => coverage,
        Err(error) => {
            if json {
                let output = serde_json::json!({
                    "valid": false,
                    "inconclusive": true,
                    "error": format!("Coverage inconclusive: {error}"),
                    "file_coverage": serde_json::Value::Null,
                    "files_covered": 0,
                    "files_total": 0,
                    "loc_coverage": serde_json::Value::Null,
                    "loc_covered": 0,
                    "loc_total": 0,
                    "modules": [],
                    "uncovered_files": [],
                });
                println!("{}", serde_json::to_string_pretty(&output).unwrap());
            } else {
                eprintln!("Coverage inconclusive: {error}");
            }
            process::exit(1);
        }
    };

    if json {
        // `null`, not `100.0`, when there was nothing to measure — matching the
        // inconclusive-discovery payload above, which already reports absence
        // as null. This site used to re-derive its own percentage with a
        // hardcoded 100.0 fallback (#582).
        let file_coverage = percent_json(coverage.file_coverage());
        let loc_coverage = percent_json(coverage.loc_coverage());

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
            "file_coverage": file_coverage,
            "files_covered": coverage.specced_file_count,
            "files_total": coverage.measured_file_total(),
            "loc_coverage": loc_coverage,
            "loc_covered": coverage.specced_loc,
            "loc_total": coverage.total_loc,
            "modules": modules,
            "uncovered_files": uncovered_files,
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
        // Gate the exit code for machine consumers too (was an unconditional
        // exit 0, so `coverage --format json --require-coverage N` never failed —
        // finding M1). compute_exit_code prints nothing, so stdout stays valid JSON.
        process::exit(compute_exit_code(
            total_errors,
            total_warnings,
            strict,
            enforcement,
            &coverage,
            require_coverage,
        ));
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
