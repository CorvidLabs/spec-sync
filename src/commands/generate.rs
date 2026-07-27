use colored::Colorize;
use std::path::Path;
use std::process;

use crate::generator::{
    generate_specs_for_unspecced_modules, generate_specs_for_unspecced_modules_paths,
};
use crate::output::{print_coverage_line, print_coverage_report, print_summary};
use crate::types;
use crate::validator::{compute_coverage, load_schema_validation};

use super::{compute_exit_code, exit_with_status, load_and_discover, run_validation};

#[allow(clippy::too_many_arguments)]
pub fn cmd_generate(
    root: &Path,
    strict: bool,
    enforcement: Option<types::EnforcementMode>,
    require_coverage: Option<usize>,
    format: types::OutputFormat,
    uncovered: bool,
    batch: Vec<String>,
) {
    let json = matches!(format, types::OutputFormat::Json);

    // --batch mode: generate for a specific list of modules
    if !batch.is_empty() {
        cmd_generate_batch(root, strict, enforcement, require_coverage, format, batch);
        return;
    }

    // --uncovered or default: generate for all unspecced modules
    let _ = uncovered; // explicit flag is accepted but behavior is the same as default
    cmd_generate_all(root, strict, enforcement, require_coverage, format, json);
}

/// Generate specs for all unspecced modules (default behavior, also triggered by --uncovered).
#[allow(clippy::too_many_arguments)]
fn cmd_generate_all(
    root: &Path,
    strict: bool,
    enforcement: Option<types::EnforcementMode>,
    require_coverage: Option<usize>,
    _format: types::OutputFormat,
    json: bool,
) {
    let (config, spec_files) = load_and_discover(root, true);
    let enforcement = enforcement.unwrap_or(if strict {
        types::EnforcementMode::Strict
    } else {
        config.enforcement
    });
    let schema = load_schema_validation(root, &config);
    let ignore_rules = crate::ignore::IgnoreRules::default();

    let (mut total_errors, mut total_warnings, mut passed, mut total) = if spec_files.is_empty() {
        // Diagnostic only — never on stdout under --format json, which must stay a
        // clean JSON document.
        if !json {
            println!("No existing specs found. Scanning for source modules...");
        }
        (0, 0, 0, 0)
    } else {
        let (te, tw, p, t, _, _, _) = run_validation(
            root,
            &spec_files,
            &spec_files,
            &schema,
            &config,
            json,
            false,
            &ignore_rules,
        );
        (te, tw, p, t)
    };

    let mut coverage = compute_coverage(root, &spec_files, &config);

    if json {
        let outcome = generate_specs_for_unspecced_modules_paths(root, &coverage, &config);
        // Recompute coverage + validation post-generation so the gate reflects the
        // newly written specs, then honor the same gate flags the text path does.
        // Without this, `--strict`/`--enforcement`/`--require-coverage` were silently
        // ignored on the JSON path — a machine-consumer false pass on the exact
        // states (validation errors, unspecced files, sub-threshold/vacuous coverage)
        // a gate exists to catch.
        let (config, spec_files) = load_and_discover(root, true);
        let coverage = compute_coverage(root, &spec_files, &config);
        let (total_errors, total_warnings) = if spec_files.is_empty() {
            (0, 0)
        } else {
            let schema = load_schema_validation(root, &config);
            let (te, tw, _, _, _, _, _) = run_validation(
                root,
                &spec_files,
                &spec_files,
                &schema,
                &config,
                true,
                false,
                &ignore_rules,
            );
            (te, tw)
        };
        let output = serde_json::json!({
            "generated": outcome.generated_paths,
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
        let gate = compute_exit_code(
            total_errors,
            total_warnings,
            strict,
            enforcement,
            &coverage,
            require_coverage,
        );
        process::exit(gate);
    }

    print_coverage_report(&coverage);

    println!(
        "\n--- {} -----------------------------------------------",
        "Generating Specs".bold()
    );

    if !coverage.unspecced_modules.is_empty() {
        println!(
            "  {} {} module(s) without specs\n",
            "→".blue(),
            coverage.unspecced_modules.len()
        );
    }

    let outcome = generate_specs_for_unspecced_modules(root, &coverage, &config);
    let generated = outcome.generated;
    if generated == 0 && coverage.unspecced_modules.is_empty() {
        println!(
            "  {} No specs to generate — full module coverage",
            "✓".green()
        );
    } else if generated > 0 {
        println!(
            "\n  Generated {} spec file(s) — review and complete the guided sections",
            generated
        );

        // Recompute coverage and validation now that new specs exist
        let (config, spec_files) = load_and_discover(root, true);
        let schema = load_schema_validation(root, &config);
        coverage = compute_coverage(root, &spec_files, &config);
        if !spec_files.is_empty() {
            let (te, tw, p, t, _, _, _) = run_validation(
                root,
                &spec_files,
                &spec_files,
                &schema,
                &config,
                json,
                false,
                &ignore_rules,
            );
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
        enforcement,
        &coverage,
        require_coverage,
    );
}

/// Generate specs for a specific batch of module names.
/// Parses comma-separated values within each entry (e.g. "foo,bar" or ["foo", "bar"]).
#[allow(clippy::too_many_arguments)]
fn cmd_generate_batch(
    root: &Path,
    strict: bool,
    enforcement: Option<types::EnforcementMode>,
    require_coverage: Option<usize>,
    format: types::OutputFormat,
    batch: Vec<String>,
) {
    let json = matches!(format, types::OutputFormat::Json);

    // Expand comma-separated entries
    let modules: Vec<String> = batch
        .iter()
        .flat_map(|s| s.split(','))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let (config, spec_files) = load_and_discover(root, true);
    let enforcement = enforcement.unwrap_or(if strict {
        types::EnforcementMode::Strict
    } else {
        config.enforcement
    });

    let coverage = compute_coverage(root, &spec_files, &config);

    // Filter the coverage report to only the requested modules
    let unspecced_set: std::collections::HashSet<&str> = coverage
        .unspecced_modules
        .iter()
        .map(|s| s.as_str())
        .collect();

    let mut to_generate: Vec<String> = Vec::new();
    let mut already_specced: Vec<String> = Vec::new();
    let mut not_found: Vec<String> = Vec::new();

    for module in &modules {
        if unspecced_set.contains(module.as_str()) {
            to_generate.push(module.clone());
        } else {
            // Check if a spec already exists
            let specs_dir = root.join(&config.specs_dir);
            let spec_file = specs_dir.join(module).join(format!("{module}.spec.md"));
            if spec_file.exists() {
                already_specced.push(module.clone());
            } else {
                not_found.push(module.clone());
            }
        }
    }

    if json {
        // In JSON mode, build a filtered coverage report and generate
        let filtered_coverage = types::CoverageReport {
            unspecced_modules: to_generate.clone(),
            ..coverage.clone()
        };
        let outcome = generate_specs_for_unspecced_modules_paths(root, &filtered_coverage, &config);
        // Recompute coverage + validation post-generation and honor the gate flags,
        // matching the text path — the JSON path previously ignored
        // --strict/--enforcement/--require-coverage (a machine-consumer false pass).
        let (config, spec_files) = load_and_discover(root, true);
        let coverage = compute_coverage(root, &spec_files, &config);
        let (total_errors, total_warnings) = if spec_files.is_empty() {
            (0, 0)
        } else {
            let schema = load_schema_validation(root, &config);
            let ignore_rules = crate::ignore::IgnoreRules::default();
            let (te, tw, _, _, _, _, _) = run_validation(
                root,
                &spec_files,
                &spec_files,
                &schema,
                &config,
                true,
                false,
                &ignore_rules,
            );
            (te, tw)
        };
        let output = serde_json::json!({
            "requested": modules,
            "generated": outcome.generated_paths,
            "skipped_already_specced": already_specced,
            "skipped_not_found": not_found,
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
        let gate = compute_exit_code(
            total_errors,
            total_warnings,
            strict,
            enforcement,
            &coverage,
            require_coverage,
        );
        process::exit(gate);
    }

    println!(
        "\n--- {} -----------------------------------------------",
        "Batch Generate".bold()
    );
    println!("  {} {} module(s) requested", "→".blue(), modules.len());

    if !already_specced.is_empty() {
        println!(
            "  {} {} already have specs (skipped): {}",
            "~".yellow(),
            already_specced.len(),
            already_specced.join(", ")
        );
    }
    if !not_found.is_empty() {
        println!(
            "  {} {} not found in coverage report (skipped): {}",
            "~".yellow(),
            not_found.len(),
            not_found.join(", ")
        );
    }

    if to_generate.is_empty() {
        println!("  {} Nothing to generate.", "i".blue());
    } else {
        println!(
            "  {} Generating {} spec(s)...\n",
            "→".blue(),
            to_generate.len()
        );

        // Build a filtered coverage report with only the requested modules
        let filtered_coverage = types::CoverageReport {
            unspecced_modules: to_generate.clone(),
            ..coverage
        };

        let outcome = generate_specs_for_unspecced_modules(root, &filtered_coverage, &config);

        println!(
            "\n  {} Batch generate complete: {}/{} spec(s) generated",
            "✓".green(),
            outcome.generated,
            to_generate.len()
        );
    }

    // Final coverage + exit status
    let (config, spec_files) = load_and_discover(root, true);
    let coverage = compute_coverage(root, &spec_files, &config);
    print_coverage_line(&coverage);

    let schema = load_schema_validation(root, &config);
    let ignore_rules = crate::ignore::IgnoreRules::default();
    let (total_errors, total_warnings, passed, total, _, _, _) = run_validation(
        root,
        &spec_files,
        &spec_files,
        &schema,
        &config,
        true, // collect
        false,
        &ignore_rules,
    );
    print_summary(total, passed, total_warnings, total_errors);

    exit_with_status(
        total_errors,
        total_warnings,
        strict,
        enforcement,
        &coverage,
        require_coverage,
    );
}
