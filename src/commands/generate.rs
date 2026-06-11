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

/// Resolve the AI provider for `generate`, or `None` for template-only mode.
///
/// AI is used when a provider is configured anywhere — the `--provider` flag,
/// `aiProvider`/`aiCommand` in config, or the `SPECSYNC_AI_PROVIDER` /
/// `SPECSYNC_AI_COMMAND` env vars — so a configured provider "just works"
/// without repeating `--provider`. The `--provider` flag overrides config.
/// Model precedence: `--model` > `SPECSYNC_AI_MODEL` env > `aiModel` config.
fn resolve_provider_for_generate(
    config: &types::SpecSyncConfig,
    provider_flag: Option<&str>,
    model_flag: Option<&str>,
) -> Option<ai::ResolvedProvider> {
    let ai_requested = provider_flag.is_some()
        || config.ai_provider.is_some()
        || config.ai_command.is_some()
        || std::env::var("SPECSYNC_AI_PROVIDER").is_ok_and(|v| !v.is_empty())
        || std::env::var("SPECSYNC_AI_COMMAND").is_ok_and(|v| !v.is_empty());
    if !ai_requested {
        return None;
    }

    // "auto" forces auto-detect (ignore a configured provider); an explicit
    // name overrides config; an absent flag falls through to config/env/auto.
    let cli_provider = match provider_flag {
        Some("auto") => None,
        other => other,
    };

    // Effective model: --model > SPECSYNC_AI_MODEL env > aiModel config.
    let mut cfg = config.clone();
    cfg.ai_model = model_flag
        .map(str::to_string)
        .or_else(|| {
            std::env::var("SPECSYNC_AI_MODEL")
                .ok()
                .filter(|m| !m.is_empty())
        })
        .or_else(|| config.ai_model.clone());

    match ai::resolve_ai_provider(&cfg, cli_provider) {
        Ok(p) => Some(p),
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn cmd_generate(
    root: &Path,
    strict: bool,
    enforcement: Option<types::EnforcementMode>,
    require_coverage: Option<usize>,
    format: types::OutputFormat,
    provider: Option<String>,
    model: Option<String>,
    uncovered: bool,
    batch: Vec<String>,
) {
    let json = matches!(format, types::OutputFormat::Json);

    // --batch mode: generate for a specific list of modules
    if !batch.is_empty() {
        cmd_generate_batch(
            root,
            strict,
            enforcement,
            require_coverage,
            format,
            provider,
            model,
            batch,
        );
        return;
    }

    // --uncovered or default: generate for all unspecced modules
    let _ = uncovered; // explicit flag is accepted but behavior is the same as default
    cmd_generate_all(
        root,
        strict,
        enforcement,
        require_coverage,
        format,
        provider,
        model,
        json,
    );
}

/// Generate specs for all unspecced modules (default behavior, also triggered by --uncovered).
#[allow(clippy::too_many_arguments)]
fn cmd_generate_all(
    root: &Path,
    strict: bool,
    enforcement: Option<types::EnforcementMode>,
    require_coverage: Option<usize>,
    _format: types::OutputFormat,
    provider: Option<String>,
    model: Option<String>,
    json: bool,
) {
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

    // AI mode is enabled by --provider, a configured provider, or env.
    let resolved_provider =
        resolve_provider_for_generate(&config, provider.as_deref(), model.as_deref());
    let ai = resolved_provider.is_some();

    if json {
        let outcome = generate_specs_for_unspecced_modules_paths(
            root,
            &coverage,
            &config,
            resolved_provider.as_ref(),
        );
        let output = serde_json::json!({
            "generated": outcome.generated_paths,
            "ai_errors": outcome.ai_errors,
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
        process::exit(if outcome.ai_errors.is_empty() { 0 } else { 1 });
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

    if !coverage.unspecced_modules.is_empty() {
        println!(
            "  {} {} module(s) without specs\n",
            "→".blue(),
            coverage.unspecced_modules.len()
        );
    }

    let outcome =
        generate_specs_for_unspecced_modules(root, &coverage, &config, resolved_provider.as_ref());
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
    report_ai_failures_and_exit(&outcome.ai_errors);
    exit_with_status(
        total_errors,
        total_warnings,
        strict,
        enforcement,
        &coverage,
        require_coverage,
    );
}

/// If any AI generation failures occurred, print them prominently on stderr
/// (last, after the check report) and exit non-zero. No-op when empty.
fn report_ai_failures_and_exit(ai_errors: &[String]) {
    if ai_errors.is_empty() {
        return;
    }
    eprintln!();
    for e in ai_errors {
        eprintln!("  {} {e}", "✗".red());
    }
    eprintln!(
        "{} AI generation failed for {} module(s) — template fallback was used.",
        "✗".red().bold(),
        ai_errors.len()
    );
    eprintln!(
        "  Check your provider configuration (API key, --provider, aiProvider/aiCommand) and re-run."
    );
    process::exit(1);
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
    provider: Option<String>,
    model: Option<String>,
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

    // Resolve AI provider (flag, configured provider, or env).
    let resolved_provider =
        resolve_provider_for_generate(&config, provider.as_deref(), model.as_deref());

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
        let outcome = generate_specs_for_unspecced_modules_paths(
            root,
            &filtered_coverage,
            &config,
            resolved_provider.as_ref(),
        );
        let output = serde_json::json!({
            "requested": modules,
            "generated": outcome.generated_paths,
            "ai_errors": outcome.ai_errors,
            "skipped_already_specced": already_specced,
            "skipped_not_found": not_found,
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
        process::exit(if outcome.ai_errors.is_empty() { 0 } else { 1 });
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

    let mut ai_errors: Vec<String> = Vec::new();
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

        let outcome = generate_specs_for_unspecced_modules(
            root,
            &filtered_coverage,
            &config,
            resolved_provider.as_ref(),
        );
        ai_errors = outcome.ai_errors;

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

    let schema_tables = get_schema_table_names(root, &config);
    let schema_columns = build_schema_columns(root, &config);
    let ignore_rules = crate::ignore::IgnoreRules::default();
    let (total_errors, total_warnings, passed, total, _, _) = run_validation(
        root,
        &spec_files,
        &schema_tables,
        &schema_columns,
        &config,
        true, // collect
        false,
        &ignore_rules,
    );
    print_summary(total, passed, total_warnings, total_errors);

    report_ai_failures_and_exit(&ai_errors);
    exit_with_status(
        total_errors,
        total_warnings,
        strict,
        enforcement,
        &coverage,
        require_coverage,
    );
}
