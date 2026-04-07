use colored::Colorize;
use std::fs;
use std::io::{IsTerminal, Write as _};
use std::path::{Path, PathBuf};
use std::process;

use crate::ai;
use crate::commands::{
    build_schema_columns, compute_exit_code, exit_with_status, load_and_discover, run_validation,
};
use crate::hash_cache;
use crate::output::{print_check_markdown, print_coverage_line, print_summary};
use crate::types;
use crate::validator::compute_coverage;
use crate::validator::get_schema_table_names;

use super::issues::create_drift_issues;

#[allow(clippy::too_many_arguments)]
pub(crate) fn cmd_check(
    root: &Path,
    strict: bool,
    enforcement: Option<types::EnforcementMode>,
    require_coverage: Option<usize>,
    format: types::OutputFormat,
    fix: bool,
    force: bool,
    create_issues: bool,
) {
    use hash_cache::{ChangeClassification, ChangeKind};
    use types::OutputFormat::*;

    let (config, spec_files) = load_and_discover(root, fix);
    // CLI --enforcement flag overrides config; --strict implies strict enforcement.
    let enforcement = enforcement.unwrap_or(if strict {
        types::EnforcementMode::Strict
    } else {
        config.enforcement
    });

    if spec_files.is_empty() {
        match format {
            Json => {
                let output = serde_json::json!({
                    "passed": true,
                    "errors": [],
                    "warnings": [],
                    "stale": [],
                    "specs_checked": 0,
                });
                println!("{}", serde_json::to_string_pretty(&output).unwrap());
            }
            Markdown => {
                println!("## SpecSync Check Results\n");
                println!("No spec files found. Run `specsync generate` to scaffold specs.");
            }
            Text => {
                println!(
                    "No spec files found in {}/. Run `specsync generate` to scaffold specs.",
                    config.specs_dir
                );
            }
        }
        process::exit(0);
    }

    // Load hash cache and classify changes for each spec.
    let mut cache = hash_cache::HashCache::load(root);
    let (specs_to_validate, change_classifications) = if force || strict {
        (spec_files.clone(), Vec::new())
    } else {
        let classifications = hash_cache::classify_all_changes(root, &spec_files, &cache);
        let changed: Vec<PathBuf> = classifications
            .iter()
            .map(|c| c.spec_path.clone())
            .collect();
        (changed, classifications)
    };

    let skipped = spec_files.len() - specs_to_validate.len();
    if skipped > 0 && matches!(format, Text) {
        println!(
            "{} Skipped {skipped} unchanged spec(s) (use --force to re-validate all)\n",
            "⊘".cyan()
        );
    }

    if specs_to_validate.is_empty() && matches!(format, Text) {
        println!("{}", "All specs unchanged — nothing to validate.".green());
        let coverage = compute_coverage(root, &spec_files, &config);
        print_coverage_line(&coverage);
        process::exit(0);
    }

    // Report staleness from change classifications
    let mut stale_entries: Vec<serde_json::Value> = Vec::new();
    let mut staleness_warnings: usize = 0;
    let mut requirements_stale_specs: Vec<ChangeClassification> = Vec::new();

    for classification in &change_classifications {
        let spec_rel = classification
            .spec_path
            .strip_prefix(root)
            .unwrap_or(&classification.spec_path)
            .to_string_lossy()
            .to_string();

        if classification.has(&ChangeKind::Requirements) {
            if matches!(format, Text) {
                println!(
                    "  {} {spec_rel}: requirements changed — spec may need re-validation",
                    "⚠".yellow()
                );
            }
            stale_entries.push(serde_json::json!({
                "spec": spec_rel,
                "reason": "requirements_changed",
                "message": "requirements changed — spec may need re-validation"
            }));
            staleness_warnings += 1;
            requirements_stale_specs.push(classification.clone());
        }

        if classification.has(&ChangeKind::Companion) && matches!(format, Text) {
            println!(
                "  {} {spec_rel}: companion file updated (hash refreshed)",
                "ℹ".cyan()
            );
        }
    }

    if staleness_warnings > 0 && matches!(format, Text) {
        println!(); // spacing after staleness messages
    }

    // Interactive prompting: if TTY and requirements drift detected, offer re-validation
    if !requirements_stale_specs.is_empty()
        && matches!(format, Text)
        && !fix
        && std::io::stdin().is_terminal()
    {
        eprint!(
            "{} Re-validate spec(s) against new requirements? [y/N] ",
            "?".cyan()
        );
        let _ = std::io::stderr().flush();
        let mut answer = String::new();
        let _ = std::io::stdin().read_line(&mut answer);
        if answer.trim().eq_ignore_ascii_case("y") {
            let regen_count =
                auto_regen_stale_specs(root, &requirements_stale_specs, &config, format);
            if regen_count > 0 {
                println!(
                    "{} Re-generated {regen_count} spec(s) from updated requirements\n",
                    "✓".green()
                );
            }
        } else {
            println!("  Skipping re-validation. Use --fix to auto-regenerate.\n");
        }
    }

    let schema_tables = get_schema_table_names(root, &config);
    let schema_columns = build_schema_columns(root, &config);

    // If --fix is requested, auto-add undocumented exports to specs
    if fix {
        let fixed = auto_fix_specs(root, &specs_to_validate, &config);
        if fixed > 0 && matches!(format, Text) {
            println!("{} Auto-added exports to {fixed} spec(s)\n", "✓".green());
        }

        // --fix + requirements changed: regenerate spec via AI
        if !requirements_stale_specs.is_empty() {
            let regen_count =
                auto_regen_stale_specs(root, &requirements_stale_specs, &config, format);
            if regen_count > 0 && matches!(format, Text) {
                println!(
                    "{} Re-generated {regen_count} spec(s) from updated requirements\n",
                    "✓".green()
                );
            }
        }
    }

    let collect = !matches!(format, Text);
    let (total_errors, total_warnings, passed, total, all_errors, all_warnings) = run_validation(
        root,
        &specs_to_validate,
        &schema_tables,
        &schema_columns,
        &config,
        collect,
    );
    // Include staleness warnings in total when --strict
    let effective_warnings = total_warnings + staleness_warnings;
    let coverage = compute_coverage(root, &spec_files, &config);

    // Update hash cache after validation (only when no errors).
    // Specs with warnings are still cached — --strict forces re-validation separately.
    if total_errors == 0 {
        hash_cache::update_cache(root, &specs_to_validate, &mut cache);
        let _ = cache.save(root);
    }

    // --create-issues: create GitHub issues for specs with validation errors
    if create_issues && total_errors > 0 {
        create_drift_issues(root, &config, &all_errors, format);
    }

    match format {
        Json => {
            let exit_code = compute_exit_code(
                total_errors,
                effective_warnings,
                strict,
                enforcement,
                &coverage,
                require_coverage,
            );
            let output = serde_json::json!({
                "passed": exit_code == 0,
                "errors": all_errors,
                "warnings": all_warnings,
                "stale": stale_entries,
                "specs_checked": total,
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
            process::exit(exit_code);
        }
        Markdown => {
            let exit_code = compute_exit_code(
                total_errors,
                effective_warnings,
                strict,
                enforcement,
                &coverage,
                require_coverage,
            );
            print_check_markdown(
                total,
                passed,
                effective_warnings,
                total_errors,
                &all_errors,
                &all_warnings,
                &coverage,
                exit_code == 0,
            );
            process::exit(exit_code);
        }
        Text => {
            print_summary(total, passed, effective_warnings, total_errors);
            print_coverage_line(&coverage);
            exit_with_status(
                total_errors,
                effective_warnings,
                strict,
                enforcement,
                &coverage,
                require_coverage,
            );
        }
    }
}

/// Auto-regenerate specs whose requirements have drifted, using AI if available.
pub(crate) fn auto_regen_stale_specs(
    root: &Path,
    stale: &[hash_cache::ChangeClassification],
    config: &types::SpecSyncConfig,
    format: types::OutputFormat,
) -> usize {
    // Try to resolve an AI provider
    let provider = match ai::resolve_ai_provider(config, None) {
        Ok(p) => p,
        Err(_) => {
            if matches!(format, types::OutputFormat::Text) {
                println!(
                    "  {} Requirements changed but no AI provider configured.",
                    "ℹ".cyan()
                );
                println!("    Configure one in specsync.json (aiProvider/aiCommand) or set");
                println!("    ANTHROPIC_API_KEY / OPENAI_API_KEY to auto-regenerate specs.");
            }
            return 0;
        }
    };

    let mut regen_count = 0;
    for classification in stale {
        let spec_path = &classification.spec_path;
        let spec_rel = spec_path
            .strip_prefix(root)
            .unwrap_or(spec_path)
            .to_string_lossy()
            .to_string();

        // Find the requirements file (current convention, then legacy)
        let parent = match spec_path.parent() {
            Some(p) => p,
            None => continue,
        };
        let stem = spec_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let module_name = stem.strip_suffix(".spec").unwrap_or(stem);

        let req_path = parent.join("requirements.md");
        let req_path = if req_path.exists() {
            req_path
        } else {
            let legacy = parent.join(format!("{module_name}.req.md"));
            if legacy.exists() {
                legacy
            } else {
                continue;
            }
        };

        if matches!(format, types::OutputFormat::Text) {
            println!("  {} Regenerating {spec_rel}...", "⟳".cyan());
        }
        match ai::regenerate_spec_with_ai(
            module_name,
            spec_path,
            &req_path,
            root,
            config,
            &provider,
        ) {
            Ok(new_spec) => {
                if fs::write(spec_path, &new_spec).is_ok() {
                    regen_count += 1;
                }
            }
            Err(e) => {
                if matches!(format, types::OutputFormat::Text) {
                    eprintln!("  {} Failed to regenerate {spec_rel}: {e}", "✗".red());
                }
            }
        }
    }

    regen_count
}

// --- Auto-fix: add undocumented exports to spec ---

pub(crate) fn auto_fix_specs(
    root: &Path,
    spec_files: &[PathBuf],
    config: &types::SpecSyncConfig,
) -> usize {
    use crate::exports::get_exported_symbols_with_level;
    use crate::parser::{get_spec_symbols, parse_frontmatter};

    let mut fixed_count = 0;

    for spec_file in spec_files {
        let content = match fs::read_to_string(spec_file) {
            Ok(c) => c.replace("\r\n", "\n"),
            Err(_) => continue,
        };

        let parsed = match parse_frontmatter(&content) {
            Some(p) => p,
            None => continue,
        };

        if parsed.frontmatter.files.is_empty() {
            continue;
        }

        // Collect all exports from source files
        let mut all_exports: Vec<String> = Vec::new();
        for file in &parsed.frontmatter.files {
            let full_path = root.join(file);
            all_exports.extend(get_exported_symbols_with_level(
                &full_path,
                config.export_level,
            ));
        }
        let mut seen = std::collections::HashSet::new();
        all_exports.retain(|s| seen.insert(s.clone()));

        // Find which exports are already documented
        let spec_symbols = get_spec_symbols(&parsed.body);
        let spec_set: std::collections::HashSet<&str> =
            spec_symbols.iter().map(|s| s.as_str()).collect();

        let undocumented: Vec<&str> = all_exports
            .iter()
            .filter(|s| !spec_set.contains(s.as_str()))
            .map(|s| s.as_str())
            .collect();

        if undocumented.is_empty() {
            continue;
        }

        // Detect primary language for context-aware row format
        let primary_lang = parsed
            .frontmatter
            .files
            .iter()
            .filter_map(|f| {
                std::path::Path::new(f)
                    .extension()
                    .and_then(|e| e.to_str())
                    .and_then(types::Language::from_extension)
            })
            .next();

        // Build new rows with language-appropriate columns
        let new_rows: String = undocumented
            .iter()
            .map(|name| match primary_lang {
                Some(types::Language::Swift)
                | Some(types::Language::Kotlin)
                | Some(types::Language::Java) => {
                    format!("| `{name}` | <!-- kind --> | <!-- TODO: describe --> |")
                }
                Some(types::Language::Rust) => {
                    format!("| `{name}` | <!-- TODO: describe --> |")
                }
                _ => format!("| `{name}` | <!-- TODO: describe --> |"),
            })
            .collect::<Vec<_>>()
            .join("\n");

        // Find insertion point: end of "## Public API" section, before next "## " heading
        let mut new_content = content.clone();
        if let Some(api_start) = content.find("## Public API") {
            let after = &content[api_start..];
            // Find the next ## heading after Public API
            let next_section = after[1..].find("\n## ").map(|pos| api_start + 1 + pos);

            let insert_pos = match next_section {
                Some(pos) => pos,
                None => content.len(),
            };

            // Insert new rows before the next section
            new_content = format!(
                "{}\n{}\n{}",
                content[..insert_pos].trim_end(),
                new_rows,
                &content[insert_pos..]
            );
        } else {
            // No Public API section — append one
            let section = format!(
                "\n## Public API\n\n| Export | Description |\n|--------|-------------|\n{new_rows}\n"
            );
            new_content.push_str(&section);
        }

        if let Ok(()) = fs::write(spec_file, &new_content) {
            fixed_count += 1;
            let rel = spec_file.strip_prefix(root).unwrap_or(spec_file).display();
            println!(
                "  {} {rel}: added {} export(s)",
                "✓".green(),
                undocumented.len()
            );
        }
    }

    fixed_count
}
