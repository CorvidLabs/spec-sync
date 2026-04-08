use colored::Colorize;
use std::path::Path;
use std::process;

use crate::ai;
use crate::generator::{
    generate_specs_for_unspecced_modules, generate_specs_for_unspecced_modules_paths,
};
use crate::output::{print_coverage_line, print_coverage_report, print_summary};
use crate::types;
use crate::validator::{compute_coverage, get_schema_table_names};

use super::{build_schema_columns, exit_with_status, load_and_discover, run_validation};

pub fn cmd_generate(
    root: &Path,
    strict: bool,
    enforcement: Option<types::EnforcementMode>,
    require_coverage: Option<usize>,
    format: types::OutputFormat,
    provider: Option<String>,
) {
    let json = matches!(format, types::OutputFormat::Json);
    let (config, spec_files) = load_and_discover(root, true);
    let enforcement = enforcement.unwrap_or(if strict {
        types::EnforcementMode::Strict
    } else {
        config.enforcement
    });
    let schema_tables = get_schema_table_names(root, &config);
    let schema_columns = build_schema_columns(root, &config);
    let ignore_rules = crate::ignore::IgnoreRules::default();

    let (mut total_errors, mut total_warnings, mut passed, mut total) = if spec_files.is_empty() {
        println!("No existing specs found. Scanning for source modules...");
        (0, 0, 0, 0)
    } else {
        let (te, tw, p, t, _, _) = run_validation(
            root,
            &spec_files,
            &schema_tables,
            &schema_columns,
            &config,
            json,
            false,
            &ignore_rules,
        );
        (te, tw, p, t)
    };

    let mut coverage = compute_coverage(root, &spec_files, &config);

    // --provider enables AI mode. "auto" means auto-detect.
    let ai = provider.is_some();

    let resolved_provider = if let Some(ref prov) = provider {
        let cli_provider = if prov == "auto" {
            None
        } else {
            Some(prov.as_str())
        };
        match ai::resolve_ai_provider(&config, cli_provider) {
            Ok(p) => Some(p),
            Err(e) => {
                eprintln!("{e}");
                process::exit(1);
            }
        }
    } else {
        None
    };

    if json {
        let generated_paths = generate_specs_for_unspecced_modules_paths(
            root,
            &coverage,
            &config,
            resolved_provider.as_ref(),
        );
        let output = serde_json::json!({
            "generated": generated_paths,
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
        process::exit(0);
    }

    print_coverage_report(&coverage);

    println!(
        "\n--- {} -----------------------------------------------",
        if ai {
            "Generating Specs (AI)"
        } else {
            "Generating Specs"
        }
        .bold()
    );
    let generated =
        generate_specs_for_unspecced_modules(root, &coverage, &config, resolved_provider.as_ref());
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
        let schema_columns = build_schema_columns(root, &config);
        coverage = compute_coverage(root, &spec_files, &config);
        if !spec_files.is_empty() {
            let (te, tw, p, t, _, _) = run_validation(
                root,
                &spec_files,
                &schema_tables,
                &schema_columns,
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
