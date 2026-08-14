use std::path::Path;
use std::process;

use crate::output::{
    CoverageFindings, coverage_json, print_coverage_line, print_coverage_report, print_summary,
};
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
    let (total_errors, total_warnings, passed, total, all_errors, all_warnings, all_notices) =
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
                let message = format!("Coverage inconclusive: {error}");
                let output = serde_json::json!({
                    "valid": false,
                    "inconclusive": true,
                    // `passed` and `errors` match the successful payload's
                    // shape. Without them a consumer had to infer failure from
                    // the ABSENCE of fields, on the one path where the command
                    // knows for certain that it failed.
                    "passed": false,
                    "errors": [message],
                    "warnings": [],
                    "notices": [],
                    "error": message,
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
        // Gate the exit code for machine consumers too (was an unconditional
        // exit 0, so `coverage --format json --require-coverage N` never failed —
        // finding M1). compute_exit_code prints nothing, so stdout stays valid JSON.
        let exit_code = compute_exit_code(
            total_errors,
            total_warnings,
            strict,
            enforcement,
            &coverage,
            require_coverage,
        );
        // The findings `run_validation` just produced used to be dropped on the
        // floor here (`_all_errors`, `_all_warnings`, `_all_notices`), so
        // `coverage --format json` printed a spotless report and exited 1 on
        // the very tree whose `check --format text` names two problems. Same
        // run, same validation, opposite conclusions — the format decided
        // whether the problems existed. `passed` is the gate verdict, so it can
        // never contradict the exit code beside it.
        let output = coverage_json(
            &coverage,
            &CoverageFindings {
                passed: exit_code == 0,
                specs_checked: total,
                specs_passed: passed,
                errors: all_errors,
                warnings: all_warnings,
                notices: all_notices,
            },
        );
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
        process::exit(exit_code);
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
