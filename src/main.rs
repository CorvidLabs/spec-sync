mod ai;
mod config;
mod exports;
mod generator;
mod hash_cache;
mod hooks;
mod manifest;
mod mcp;
mod parser;
mod registry;
mod schema;
mod scoring;
mod types;
mod validator;
mod watch;

use clap::{Parser, Subcommand};
use colored::Colorize;
use std::fs;
use std::io::{IsTerminal, Write as _};
use std::path::{Path, PathBuf};
use std::process;

use config::{detect_source_dirs, load_config};
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

    /// Output format: text (default), json, or markdown
    #[arg(long, value_enum, global = true, default_value = "text")]
    format: types::OutputFormat,

    /// Output results as JSON (shorthand for --format json)
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Validate all specs against source code (default)
    Check {
        /// Auto-add undocumented exports to spec Public API tables
        #[arg(long)]
        fix: bool,
        /// Skip hash cache and re-validate all specs
        #[arg(long)]
        force: bool,
    },
    /// Show file and module coverage report
    Coverage,
    /// Scaffold spec files for unspecced modules
    Generate {
        /// AI provider to use for spec generation. Without this flag, specs are
        /// generated from templates only.
        ///
        /// Use "auto" to auto-detect an installed provider, or specify one:
        /// claude, anthropic, openai, ollama, copilot.
        #[arg(long, value_name = "PROVIDER")]
        provider: Option<String>,
    },
    /// Create a specsync.json config file
    Init,
    /// Score spec quality (0-100) with letter grades and improvement suggestions
    Score,
    /// Watch spec and source files, re-running check on changes
    Watch,
    /// Run as an MCP (Model Context Protocol) server over stdio
    Mcp,
    /// Scaffold a new spec with companion files (tasks.md, context.md)
    AddSpec {
        /// Module name for the new spec
        name: String,
    },
    /// Generate a specsync-registry.toml for cross-project references
    InitRegistry {
        /// Project name for the registry
        #[arg(long)]
        name: Option<String>,
    },
    /// Resolve cross-project spec references in depends_on
    Resolve {
        /// Fetch remote specsync-registry.toml files from GitHub to verify
        /// cross-project references actually exist. Off by default — no
        /// network calls without this flag.
        #[arg(long)]
        remote: bool,
    },
    /// Show export changes since last commit (useful for CI/PR comments)
    Diff {
        /// Git ref to compare against (default: HEAD)
        #[arg(long, default_value = "HEAD")]
        base: String,
    },
    /// Manage agent instruction files and git hooks for spec awareness
    Hooks {
        #[command(subcommand)]
        action: HooksAction,
    },
}

#[derive(Subcommand)]
enum HooksAction {
    /// Install agent instructions and/or git hooks
    Install {
        /// Install CLAUDE.md instructions
        #[arg(long)]
        claude: bool,
        /// Install .cursorrules instructions
        #[arg(long)]
        cursor: bool,
        /// Install .github/copilot-instructions.md
        #[arg(long)]
        copilot: bool,
        /// Install AGENTS.md instructions
        #[arg(long)]
        agents: bool,
        /// Install git pre-commit hook
        #[arg(long)]
        precommit: bool,
        /// Install Claude Code settings.json hook
        #[arg(long)]
        claude_code_hook: bool,
    },
    /// Remove previously installed hooks
    Uninstall {
        /// Remove CLAUDE.md instructions
        #[arg(long)]
        claude: bool,
        /// Remove .cursorrules instructions
        #[arg(long)]
        cursor: bool,
        /// Remove .github/copilot-instructions.md
        #[arg(long)]
        copilot: bool,
        /// Remove AGENTS.md instructions
        #[arg(long)]
        agents: bool,
        /// Remove git pre-commit hook
        #[arg(long)]
        precommit: bool,
        /// Remove Claude Code settings.json hook
        #[arg(long)]
        claude_code_hook: bool,
    },
    /// Show installation status of all hooks
    Status,
}

fn main() {
    let result = std::panic::catch_unwind(run);
    match result {
        Ok(()) => {}
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else {
                "unknown error".to_string()
            };
            eprintln!(
                "{} specsync panicked: {msg}\n\nThis is a bug — please report it at https://github.com/CorvidLabs/spec-sync/issues",
                "Error:".red().bold()
            );
            process::exit(1);
        }
    }
}

fn run() {
    let cli = Cli::parse();
    let root = cli
        .root
        .unwrap_or_else(|| std::env::current_dir().expect("Cannot determine cwd"));
    let root = root.canonicalize().unwrap_or(root);

    // --json flag is shorthand for --format json (backward compat)
    let format = if cli.json {
        types::OutputFormat::Json
    } else {
        cli.format
    };

    let command = cli.command.unwrap_or(Command::Check {
        fix: false,
        force: false,
    });

    match command {
        Command::Init => cmd_init(&root),
        Command::Check { fix, force } => {
            cmd_check(&root, cli.strict, cli.require_coverage, format, fix, force)
        }
        Command::Coverage => cmd_coverage(&root, cli.strict, cli.require_coverage, format),
        Command::Generate { provider } => {
            cmd_generate(&root, cli.strict, cli.require_coverage, format, provider)
        }
        Command::Score => cmd_score(&root, format),
        Command::Watch => watch::run_watch(&root, cli.strict, cli.require_coverage),
        Command::Mcp => mcp::run_mcp_server(&root),
        Command::AddSpec { name } => cmd_add_spec(&root, &name),
        Command::InitRegistry { name } => cmd_init_registry(&root, name),
        Command::Resolve { remote } => cmd_resolve(&root, remote),
        Command::Diff { base } => cmd_diff(&root, &base, format),
        Command::Hooks { action } => cmd_hooks(&root, action),
    }
}

fn cmd_hooks(root: &Path, action: HooksAction) {
    match action {
        HooksAction::Install {
            claude,
            cursor,
            copilot,
            agents,
            precommit,
            claude_code_hook,
        } => {
            let targets =
                collect_hook_targets(claude, cursor, copilot, agents, precommit, claude_code_hook);
            hooks::cmd_install(root, &targets);
        }
        HooksAction::Uninstall {
            claude,
            cursor,
            copilot,
            agents,
            precommit,
            claude_code_hook,
        } => {
            let targets =
                collect_hook_targets(claude, cursor, copilot, agents, precommit, claude_code_hook);
            hooks::cmd_uninstall(root, &targets);
        }
        HooksAction::Status => hooks::cmd_status(root),
    }
}

fn collect_hook_targets(
    claude: bool,
    cursor: bool,
    copilot: bool,
    agents: bool,
    precommit: bool,
    claude_code_hook: bool,
) -> Vec<hooks::HookTarget> {
    let mut targets = Vec::new();
    if claude {
        targets.push(hooks::HookTarget::Claude);
    }
    if cursor {
        targets.push(hooks::HookTarget::Cursor);
    }
    if copilot {
        targets.push(hooks::HookTarget::Copilot);
    }
    if agents {
        targets.push(hooks::HookTarget::Agents);
    }
    if precommit {
        targets.push(hooks::HookTarget::Precommit);
    }
    if claude_code_hook {
        targets.push(hooks::HookTarget::ClaudeCodeHook);
    }
    // If no specific targets, empty vec means "all"
    targets
}

fn cmd_init(root: &Path) {
    let config_path = root.join("specsync.json");
    let toml_path = root.join(".specsync.toml");
    if config_path.exists() {
        println!("specsync.json already exists");
        return;
    }
    if toml_path.exists() {
        println!(".specsync.toml already exists");
        return;
    }

    let detected_dirs = detect_source_dirs(root);
    let dirs_display = detected_dirs.join(", ");

    let default = serde_json::json!({
        "specsDir": "specs",
        "sourceDirs": detected_dirs,
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
    if let Err(e) = fs::write(&config_path, content) {
        eprintln!(
            "{} Failed to write specsync.json: {e}",
            "error:".red().bold()
        );
        process::exit(1);
    }
    println!("{} Created specsync.json", "✓".green());
    println!("  Detected source directories: {dirs_display}");
}

fn cmd_check(
    root: &Path,
    strict: bool,
    require_coverage: Option<usize>,
    format: types::OutputFormat,
    fix: bool,
    force: bool,
) {
    use hash_cache::{ChangeClassification, ChangeKind};
    use types::OutputFormat::*;

    let (config, spec_files) = load_and_discover(root, fix);

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
        if !answer.trim().eq_ignore_ascii_case("y") {
            // User declined — just continue with normal validation
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

    match format {
        Json => {
            let exit_code = compute_exit_code(
                total_errors,
                effective_warnings,
                strict,
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
                &coverage,
                require_coverage,
            );
        }
    }
}

/// Auto-regenerate specs whose requirements have drifted, using AI if available.
fn auto_regen_stale_specs(
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

        // Find the requirements file
        let parent = match spec_path.parent() {
            Some(p) => p,
            None => continue,
        };
        let req_path = parent.join("requirements.md");
        if !req_path.exists() {
            // Try legacy name
            let stem = spec_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let module = stem.strip_suffix(".spec").unwrap_or(stem);
            let legacy = parent.join(format!("{module}.req.md"));
            if !legacy.exists() {
                continue;
            }
            // Use legacy path
            let module_name = module;
            if matches!(format, types::OutputFormat::Text) {
                println!("  {} Regenerating {spec_rel}...", "⟳".cyan());
            }
            match ai::regenerate_spec_with_ai(
                module_name,
                spec_path,
                &legacy,
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
            continue;
        }

        let stem = spec_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let module_name = stem.strip_suffix(".spec").unwrap_or(stem);
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

fn cmd_coverage(
    root: &Path,
    strict: bool,
    require_coverage: Option<usize>,
    format: types::OutputFormat,
) {
    let json = matches!(format, types::OutputFormat::Json);
    let (config, spec_files) = load_and_discover(root, false);
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
        &coverage,
        require_coverage,
    );
}

fn cmd_generate(
    root: &Path,
    strict: bool,
    require_coverage: Option<usize>,
    format: types::OutputFormat,
    provider: Option<String>,
) {
    let json = matches!(format, types::OutputFormat::Json);
    let (config, spec_files) = load_and_discover(root, true);
    let schema_tables = get_schema_table_names(root, &config);
    let schema_columns = build_schema_columns(root, &config);

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
        &coverage,
        require_coverage,
    );
}

fn cmd_score(root: &Path, format: types::OutputFormat) {
    let json = matches!(format, types::OutputFormat::Json);
    let (config, spec_files) = load_and_discover(root, false);
    let scores: Vec<scoring::SpecScore> = spec_files
        .iter()
        .map(|f| scoring::score_spec(f, root, &config))
        .collect();
    let project = scoring::compute_project_score(scores);

    if json {
        let specs_json: Vec<serde_json::Value> = project
            .spec_scores
            .iter()
            .map(|s| {
                serde_json::json!({
                    "spec": s.spec_path,
                    "total": s.total,
                    "grade": s.grade,
                    "frontmatter": s.frontmatter_score,
                    "sections": s.sections_score,
                    "api": s.api_score,
                    "depth": s.depth_score,
                    "freshness": s.freshness_score,
                    "suggestions": s.suggestions,
                })
            })
            .collect();
        let output = serde_json::json!({
            "average_score": (project.average_score * 10.0).round() / 10.0,
            "grade": project.grade,
            "total_specs": project.total_specs,
            "distribution": {
                "A": project.grade_distribution[0],
                "B": project.grade_distribution[1],
                "C": project.grade_distribution[2],
                "D": project.grade_distribution[3],
                "F": project.grade_distribution[4],
            },
            "specs": specs_json,
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
        return;
    }

    println!(
        "\n--- {} ------------------------------------------------",
        "Spec Quality Scores".bold()
    );

    for s in &project.spec_scores {
        let grade_colored = match s.grade {
            "A" => s.grade.green().bold().to_string(),
            "B" => s.grade.green().to_string(),
            "C" => s.grade.yellow().to_string(),
            "D" => s.grade.yellow().bold().to_string(),
            _ => s.grade.red().bold().to_string(),
        };

        println!(
            "\n  {} [{}] {}/100",
            s.spec_path.bold(),
            grade_colored,
            s.total
        );
        println!(
            "    Frontmatter: {}/20  Sections: {}/20  API: {}/20  Depth: {}/20  Fresh: {}/20",
            s.frontmatter_score, s.sections_score, s.api_score, s.depth_score, s.freshness_score
        );
        if !s.suggestions.is_empty() {
            for suggestion in &s.suggestions {
                println!("    {} {suggestion}", "->".cyan());
            }
        }
    }

    let avg_str = format!("{:.1}", project.average_score);
    let grade_colored = match project.grade {
        "A" => project.grade.green().bold().to_string(),
        "B" => project.grade.green().to_string(),
        "C" => project.grade.yellow().to_string(),
        "D" => project.grade.yellow().bold().to_string(),
        _ => project.grade.red().bold().to_string(),
    };

    println!(
        "\n{} specs scored: average {avg_str}/100 [{}]",
        project.total_specs, grade_colored
    );
    println!(
        "  A: {}  B: {}  C: {}  D: {}  F: {}",
        project.grade_distribution[0],
        project.grade_distribution[1],
        project.grade_distribution[2],
        project.grade_distribution[3],
        project.grade_distribution[4]
    );
}

fn cmd_add_spec(root: &Path, module_name: &str) {
    let config = load_config(root);
    let specs_dir = root.join(&config.specs_dir);
    let spec_dir = specs_dir.join(module_name);
    let spec_file = spec_dir.join(format!("{module_name}.spec.md"));

    if spec_file.exists() {
        println!(
            "{} Spec already exists: {}",
            "!".yellow(),
            spec_file.strip_prefix(root).unwrap_or(&spec_file).display()
        );
        // Still generate companion files if missing
        generator::generate_companion_files_for_spec(&spec_dir, module_name);
        return;
    }

    if let Err(e) = fs::create_dir_all(&spec_dir) {
        eprintln!("Failed to create {}: {e}", spec_dir.display());
        process::exit(1);
    }

    // Use the template-based generator (no AI for add-spec)
    let template_path = specs_dir.join("_template.spec.md");
    let template = if template_path.exists() {
        fs::read_to_string(&template_path).unwrap_or_default()
    } else {
        String::new()
    };

    // Find any matching source files
    let module_files: Vec<String> = config
        .source_dirs
        .iter()
        .flat_map(|src_dir| {
            let module_dir = root.join(src_dir).join(module_name);
            if module_dir.exists() {
                walkdir::WalkDir::new(&module_dir)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.path().is_file()
                            && crate::exports::has_extension(e.path(), &config.source_extensions)
                    })
                    .map(|e| {
                        e.path()
                            .strip_prefix(root)
                            .unwrap_or(e.path())
                            .to_string_lossy()
                            .replace('\\', "/")
                    })
                    .collect::<Vec<_>>()
            } else {
                vec![]
            }
        })
        .collect();

    let _ = template; // Template handling is done by generate_spec internal

    // Generate spec content using the internal generate function
    let spec_content = {
        let title = module_name
            .split('-')
            .map(|w| {
                let mut chars = w.chars();
                match chars.next() {
                    Some(c) => c.to_uppercase().to_string() + chars.as_str(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        let files_yaml = if module_files.is_empty() {
            "  # - path/to/source/file".to_string()
        } else {
            module_files
                .iter()
                .map(|f| format!("  - {f}"))
                .collect::<Vec<_>>()
                .join("\n")
        };

        format!(
            r#"---
module: {module_name}
version: 1
status: draft
files:
{files_yaml}
db_tables: []
depends_on: []
---

# {title}

## Purpose

<!-- TODO: describe what this module does -->

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|

### Exported Types

| Type | Description |
|------|-------------|

## Invariants

1. <!-- TODO -->

## Behavioral Examples

### Scenario: TODO

- **Given** precondition
- **When** action
- **Then** result

## Error Cases

| Condition | Behavior |
|-----------|----------|

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|

### Consumed By

| Module | What is used |
|--------|-------------|

## Change Log

| Date | Author | Change |
|------|--------|--------|
"#
        )
    };

    match fs::write(&spec_file, &spec_content) {
        Ok(_) => {
            let rel = spec_file.strip_prefix(root).unwrap_or(&spec_file).display();
            println!("  {} Created {rel}", "✓".green());
            generator::generate_companion_files_for_spec(&spec_dir, module_name);
        }
        Err(e) => {
            eprintln!("Failed to write {}: {e}", spec_file.display());
            process::exit(1);
        }
    }
}

fn cmd_init_registry(root: &Path, name: Option<String>) {
    let registry_path = root.join("specsync-registry.toml");
    if registry_path.exists() {
        println!("specsync-registry.toml already exists");
        return;
    }

    let config = load_config(root);
    let project_name = name.unwrap_or_else(|| {
        root.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project")
            .to_string()
    });

    let content = registry::generate_registry(root, &project_name, &config.specs_dir);
    match fs::write(&registry_path, &content) {
        Ok(_) => {
            println!("{} Created specsync-registry.toml", "✓".green());
        }
        Err(e) => {
            eprintln!("Failed to write specsync-registry.toml: {e}");
            process::exit(1);
        }
    }
}

fn cmd_resolve(root: &Path, remote: bool) {
    let (_config, spec_files) = load_and_discover(root, false);
    let mut cross_refs: Vec<(String, String, String)> = Vec::new();
    let mut local_refs: Vec<(String, String, bool)> = Vec::new();

    for spec_file in &spec_files {
        let content = match fs::read_to_string(spec_file) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let parsed = match parser::parse_frontmatter(&content) {
            Some(p) => p,
            None => continue,
        };

        let spec_path = spec_file
            .strip_prefix(root)
            .unwrap_or(spec_file)
            .to_string_lossy()
            .to_string();

        for dep in &parsed.frontmatter.depends_on {
            if validator::is_cross_project_ref(dep) {
                if let Some((repo, module)) = validator::parse_cross_project_ref(dep) {
                    cross_refs.push((spec_path.clone(), repo.to_string(), module.to_string()));
                }
            } else {
                let exists = root.join(dep).exists();
                local_refs.push((spec_path.clone(), dep.clone(), exists));
            }
        }
    }

    println!(
        "\n--- {} ------------------------------------------------",
        "Dependency Resolution".bold()
    );

    if local_refs.is_empty() && cross_refs.is_empty() {
        println!("\n  No dependencies declared in any spec.");
        return;
    }

    if !local_refs.is_empty() {
        println!("\n  {} Local dependencies:", "Local".bold());
        for (spec, dep, exists) in &local_refs {
            if *exists {
                println!("    {} {spec} -> {dep}", "✓".green());
            } else {
                println!("    {} {spec} -> {dep} (not found)", "✗".red());
            }
        }
    }

    if !cross_refs.is_empty() {
        println!("\n  {} Cross-project references:", "Remote".bold());

        if remote {
            // Fetch remote registries to verify cross-project refs
            let mut remote_errors = 0;
            // Group refs by repo to avoid duplicate fetches
            let mut repos: std::collections::HashMap<String, Option<registry::RemoteRegistry>> =
                std::collections::HashMap::new();

            for (_spec, repo, _module) in &cross_refs {
                repos
                    .entry(repo.clone())
                    .or_insert_with(|| match registry::fetch_remote_registry(repo) {
                        Ok(reg) => Some(reg),
                        Err(e) => {
                            eprintln!(
                                "    {} Failed to fetch registry for {repo}: {e}",
                                "!".yellow()
                            );
                            None
                        }
                    });
            }

            for (spec, repo, module) in &cross_refs {
                match repos.get(repo) {
                    Some(Some(reg)) => {
                        if reg.has_spec(module) {
                            println!("    {} {spec} -> {repo}@{module}", "✓".green());
                        } else {
                            println!(
                                "    {} {spec} -> {repo}@{module} (module not in registry)",
                                "✗".red()
                            );
                            remote_errors += 1;
                        }
                    }
                    Some(None) => {
                        println!(
                            "    {} {spec} -> {repo}@{module} (registry fetch failed)",
                            "?".yellow()
                        );
                    }
                    None => {
                        println!(
                            "    {} {spec} -> {repo}@{module} (no registry)",
                            "?".yellow()
                        );
                    }
                }
            }

            if remote_errors > 0 {
                println!(
                    "\n  {} {remote_errors} cross-project ref(s) could not be verified",
                    "Warning:".yellow()
                );
            }
        } else {
            for (spec, repo, module) in &cross_refs {
                println!("    {} {spec} -> {repo}@{module}", "→".cyan());
            }
            println!(
                "\n  {} Cross-project refs are not verified by default.",
                "Tip:".cyan()
            );
            println!("  Use --remote to fetch registries and verify they exist.");
        }
    }
}

// ─── Auto-fix: add undocumented exports to spec ─────────────────────────

fn auto_fix_specs(root: &Path, spec_files: &[PathBuf], config: &types::SpecSyncConfig) -> usize {
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

// ─── Diff command ────────────────────────────────────────────────────────

fn cmd_diff(root: &Path, base: &str, format: types::OutputFormat) {
    use crate::exports::get_exported_symbols;
    use crate::parser::parse_frontmatter;

    let (config, spec_files) = load_and_discover(root, false);

    // Get list of files changed since base ref
    let output = match std::process::Command::new("git")
        .args(["diff", "--name-only", base])
        .current_dir(root)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Failed to run git diff: {e}");
            process::exit(1);
        }
    };

    let changed_files: std::collections::HashSet<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| l.to_string())
        .collect();

    if changed_files.is_empty() {
        match format {
            types::OutputFormat::Json => println!("{{\"changes\":[]}}"),
            types::OutputFormat::Markdown => {
                println!("## SpecSync Drift Report\n");
                println!("No files changed since `{base}`.");
            }
            types::OutputFormat::Text => println!("No files changed since {base}"),
        }
        return;
    }

    // Collect structured diff data for all specs
    struct DiffEntry {
        spec: String,
        changed_files: Vec<String>,
        new_exports: Vec<String>,
        removed_exports: Vec<String>,
    }

    let mut entries: Vec<DiffEntry> = Vec::new();

    for spec_file in &spec_files {
        let content = match fs::read_to_string(spec_file) {
            Ok(c) => c.replace("\r\n", "\n"),
            Err(_) => continue,
        };

        let parsed = match parse_frontmatter(&content) {
            Some(p) => p,
            None => continue,
        };

        let spec_rel = spec_file
            .strip_prefix(root)
            .unwrap_or(spec_file)
            .to_string_lossy()
            .replace('\\', "/");

        let affected_files: Vec<String> = parsed
            .frontmatter
            .files
            .iter()
            .filter(|f| changed_files.contains(*f))
            .cloned()
            .collect();

        if affected_files.is_empty() {
            continue;
        }

        // Get current exports from changed files
        let mut current_exports: Vec<String> = Vec::new();
        for file in &parsed.frontmatter.files {
            let full_path = root.join(file);
            current_exports.extend(get_exported_symbols(&full_path));
        }
        let mut seen = std::collections::HashSet::new();
        current_exports.retain(|s| seen.insert(s.clone()));

        // Get spec-documented symbols
        let spec_symbols = crate::parser::get_spec_symbols(&parsed.body);
        let spec_set: std::collections::HashSet<&str> =
            spec_symbols.iter().map(|s| s.as_str()).collect();
        let export_set: std::collections::HashSet<&str> =
            current_exports.iter().map(|s| s.as_str()).collect();

        let new_exports: Vec<String> = current_exports
            .iter()
            .filter(|s| !spec_set.contains(s.as_str()))
            .cloned()
            .collect();

        let removed_exports: Vec<String> = spec_symbols
            .iter()
            .filter(|s| !export_set.contains(s.as_str()))
            .cloned()
            .collect();

        entries.push(DiffEntry {
            spec: spec_rel,
            changed_files: affected_files,
            new_exports,
            removed_exports,
        });
    }

    match format {
        types::OutputFormat::Json => {
            let changes: Vec<serde_json::Value> = entries
                .iter()
                .map(|e| {
                    serde_json::json!({
                        "spec": e.spec,
                        "changed_files": e.changed_files,
                        "new_exports": e.new_exports,
                        "removed_exports": e.removed_exports,
                    })
                })
                .collect();
            let output = serde_json::json!({ "changes": changes });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        types::OutputFormat::Markdown => {
            #[allow(clippy::type_complexity)]
            let tuples: Vec<(String, Vec<String>, Vec<String>, Vec<String>)> = entries
                .iter()
                .map(|e| {
                    (
                        e.spec.clone(),
                        e.changed_files.clone(),
                        e.new_exports.clone(),
                        e.removed_exports.clone(),
                    )
                })
                .collect();
            print_diff_markdown(&tuples, &changed_files, &spec_files, root, &config, base);
        }
        types::OutputFormat::Text => {
            for entry in &entries {
                println!("\n{}", entry.spec.bold());
                println!("  Changed files: {}", entry.changed_files.join(", "));
                if !entry.new_exports.is_empty() {
                    println!(
                        "  {} New exports (not in spec): {}",
                        "+".green(),
                        entry.new_exports.join(", ")
                    );
                }
                if !entry.removed_exports.is_empty() {
                    println!(
                        "  {} Removed exports (still in spec): {}",
                        "-".red(),
                        entry.removed_exports.join(", ")
                    );
                }
                if entry.new_exports.is_empty() && entry.removed_exports.is_empty() {
                    println!("  {} Spec is up to date", "✓".green());
                }
            }

            if entries.is_empty() {
                // Check if any changed files are NOT covered by specs
                let specced_files: std::collections::HashSet<String> = spec_files
                    .iter()
                    .filter_map(|f| fs::read_to_string(f).ok())
                    .filter_map(|c| parse_frontmatter(&c.replace("\r\n", "\n")))
                    .flat_map(|p| p.frontmatter.files)
                    .collect();

                let untracked: Vec<&String> = changed_files
                    .iter()
                    .filter(|f| {
                        let path = std::path::Path::new(f.as_str());
                        crate::exports::has_extension(path, &config.source_extensions)
                            && !specced_files.contains(*f)
                    })
                    .collect();

                if untracked.is_empty() {
                    println!("No spec-tracked source files changed since {base}.");
                } else {
                    println!("Changed files not covered by any spec:");
                    for f in &untracked {
                        println!("  {} {f}", "?".yellow());
                    }
                }
            }
        }
    }
}

// ─── Markdown formatters ─────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn print_check_markdown(
    total: usize,
    passed: usize,
    warnings: usize,
    errors: usize,
    all_errors: &[String],
    all_warnings: &[String],
    coverage: &types::CoverageReport,
    overall_passed: bool,
) {
    let status = if overall_passed { "Passed" } else { "Failed" };
    let icon = if overall_passed { "✅" } else { "❌" };

    println!("## SpecSync Check Results\n");
    println!(
        "**{icon} {status}** — {total} specs checked, {passed} passed, {warnings} warning(s), {errors} error(s)\n"
    );

    if !all_errors.is_empty() {
        println!("### Errors\n");
        for e in all_errors {
            println!("- {e}");
        }
        println!();
    }

    if !all_warnings.is_empty() {
        println!("### Warnings\n");
        for w in all_warnings {
            println!("- {w}");
        }
        println!();
    }

    println!("### Coverage\n");
    println!(
        "- **Files:** {}/{} ({}%)",
        coverage.specced_file_count, coverage.total_source_files, coverage.coverage_percent
    );
    println!(
        "- **LOC:** {}/{} ({}%)",
        coverage.specced_loc, coverage.total_loc, coverage.loc_coverage_percent
    );
}

/// Print diff results as markdown. Each entry is (spec, changed_files, new_exports, removed_exports).
#[allow(clippy::type_complexity)]
fn print_diff_markdown(
    entries: &[(String, Vec<String>, Vec<String>, Vec<String>)],
    changed_files: &std::collections::HashSet<String>,
    spec_files: &[PathBuf],
    _root: &Path,
    config: &types::SpecSyncConfig,
    base: &str,
) {
    println!("## SpecSync Drift Report\n");

    if entries.is_empty() {
        // Check for untracked files
        let specced_files: std::collections::HashSet<String> = spec_files
            .iter()
            .filter_map(|f| fs::read_to_string(f).ok())
            .filter_map(|c| crate::parser::parse_frontmatter(&c.replace("\r\n", "\n")))
            .flat_map(|p| p.frontmatter.files)
            .collect();

        let untracked: Vec<&String> = changed_files
            .iter()
            .filter(|f| {
                let path = std::path::Path::new(f.as_str());
                crate::exports::has_extension(path, &config.source_extensions)
                    && !specced_files.contains(*f)
            })
            .collect();

        if untracked.is_empty() {
            println!("No spec-tracked source files changed since `{base}`.");
        } else {
            println!("**Changed files not covered by any spec:**\n");
            for f in &untracked {
                println!("- `{f}`");
            }
        }
        return;
    }

    let has_drift = entries
        .iter()
        .any(|(_, _, new, removed)| !new.is_empty() || !removed.is_empty());

    if has_drift {
        println!(
            "Spec drift detected in {} module(s) since `{base}`.\n",
            entries.len()
        );
    } else {
        println!("All specs are up to date with source code.\n");
    }

    for (spec, files, new_exports, removed_exports) in entries {
        println!("### `{spec}`\n");
        println!(
            "**Changed files:** {}\n",
            files
                .iter()
                .map(|f| format!("`{f}`"))
                .collect::<Vec<_>>()
                .join(", ")
        );

        if !new_exports.is_empty() || !removed_exports.is_empty() {
            println!("| Change | Export |");
            println!("|--------|--------|");
            for e in new_exports {
                println!("| Added | `{e}` |");
            }
            for e in removed_exports {
                println!("| Removed | `{e}` |");
            }
            println!();
        } else {
            println!("No drift — spec is up to date.\n");
        }
    }
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

/// Build column-level schema from migration files (if schema_dir is configured).
fn build_schema_columns(
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
fn run_validation(
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
