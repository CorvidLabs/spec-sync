use cap_std::ambient_authority;
use cap_std::fs::Dir;
use colored::Colorize;
use std::io;
use std::path::{Path, PathBuf};
use std::process;

use crate::generator::generate_specs_for_unspecced_modules_retained;
use crate::output::{print_coverage_line, print_coverage_report, print_summary};
use crate::types;
use crate::validator::compute_coverage_checked;

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
    let retained_root = RetainedGenerateRoot::open(root)
        .unwrap_or_else(|error| exit_generation_inconclusive(json, error));

    // --batch mode: generate for a specific list of modules
    if !batch.is_empty() {
        cmd_generate_batch(
            root,
            &retained_root,
            strict,
            enforcement,
            require_coverage,
            format,
            batch,
        );
        return;
    }

    // --uncovered or default: generate for all unspecced modules
    let _ = uncovered; // explicit flag is accepted but behavior is the same as default
    cmd_generate_all(
        root,
        &retained_root,
        strict,
        enforcement,
        require_coverage,
        format,
        json,
    );
}

/// Generate specs for all unspecced modules (default behavior, also triggered by --uncovered).
#[allow(clippy::too_many_arguments)]
fn cmd_generate_all(
    root: &Path,
    retained_root: &RetainedGenerateRoot,
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
            &config,
            json,
            false,
            &ignore_rules,
        );
        (te, tw, p, t)
    };

    let mut coverage = checked_coverage_or_exit(root, &spec_files, &config, json);

    if json {
        let outcome =
            generate_with_retained_root(retained_root, root, &coverage, &config, json, false);
        // Recompute coverage + validation post-generation so the gate reflects the
        // newly written specs, then honor the same gate flags the text path does.
        // Without this, `--strict`/`--enforcement`/`--require-coverage` were silently
        // ignored on the JSON path — a machine-consumer false pass on the exact
        // states (validation errors, unspecced files, sub-threshold/vacuous coverage)
        // a gate exists to catch.
        let (config, spec_files) = load_and_discover(root, true);
        let coverage = checked_coverage_or_exit(root, &spec_files, &config, true);
        let (total_errors, total_warnings) = if spec_files.is_empty() {
            (0, 0)
        } else {
            let (te, tw, _, _, _, _, _) = run_validation(
                root,
                &spec_files,
                &spec_files,
                &config,
                true,
                false,
                &ignore_rules,
            );
            (te, tw)
        };
        verify_generate_root_or_exit(retained_root, json);
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

    let outcome = generate_with_retained_root(retained_root, root, &coverage, &config, json, true);
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
        coverage = checked_coverage_or_exit(root, &spec_files, &config, json);
        if !spec_files.is_empty() {
            let (te, tw, p, t, _, _, _) = run_validation(
                root,
                &spec_files,
                &spec_files,
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

    verify_generate_root_or_exit(retained_root, json);
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
    retained_root: &RetainedGenerateRoot,
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

    let coverage = checked_coverage_or_exit(root, &spec_files, &config, json);

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
        let outcome = generate_with_retained_root(
            retained_root,
            root,
            &filtered_coverage,
            &config,
            json,
            false,
        );
        // Recompute coverage + validation post-generation and honor the gate flags,
        // matching the text path — the JSON path previously ignored
        // --strict/--enforcement/--require-coverage (a machine-consumer false pass).
        let (config, spec_files) = load_and_discover(root, true);
        let coverage = checked_coverage_or_exit(root, &spec_files, &config, true);
        let (total_errors, total_warnings) = if spec_files.is_empty() {
            (0, 0)
        } else {
            let ignore_rules = crate::ignore::IgnoreRules::default();
            let (te, tw, _, _, _, _, _) = run_validation(
                root,
                &spec_files,
                &spec_files,
                &config,
                true,
                false,
                &ignore_rules,
            );
            (te, tw)
        };
        verify_generate_root_or_exit(retained_root, json);
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

        let outcome = generate_with_retained_root(
            retained_root,
            root,
            &filtered_coverage,
            &config,
            json,
            true,
        );

        println!(
            "\n  {} Batch generate complete: {}/{} spec(s) generated",
            "✓".green(),
            outcome.generated,
            to_generate.len()
        );
    }

    // Final coverage + exit status
    let (config, spec_files) = load_and_discover(root, true);
    let coverage = checked_coverage_or_exit(root, &spec_files, &config, json);

    let ignore_rules = crate::ignore::IgnoreRules::default();
    let (total_errors, total_warnings, passed, total, _, _, _) = run_validation(
        root,
        &spec_files,
        &spec_files,
        &config,
        true, // collect
        false,
        &ignore_rules,
    );
    verify_generate_root_or_exit(retained_root, json);
    print_coverage_line(&coverage);
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

fn checked_coverage_or_exit(
    root: &Path,
    spec_files: &[std::path::PathBuf],
    config: &types::SpecSyncConfig,
    json: bool,
) -> types::CoverageReport {
    compute_coverage_checked(root, spec_files, config).unwrap_or_else(|error| {
        if json {
            let output = serde_json::json!({
                "valid": false,
                "inconclusive": true,
                "error": format!("Coverage inconclusive: {error}"),
                "generated": [],
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        } else {
            eprintln!("Coverage inconclusive: {error}");
        }
        process::exit(1);
    })
}

fn generate_with_retained_root(
    retained_root: &RetainedGenerateRoot,
    root: &Path,
    coverage: &types::CoverageReport,
    config: &types::SpecSyncConfig,
    json: bool,
    progress: bool,
) -> crate::generator::GenerationOutcome {
    generate_after_coverage_test_barrier_or_exit(json);
    verify_generate_root_or_exit(retained_root, json);
    let outcome = generate_specs_for_unspecced_modules_retained(
        &retained_root.directory,
        root,
        coverage,
        config,
        progress,
    )
    .unwrap_or_else(|error| exit_generation_inconclusive(json, error));
    verify_generate_root_or_exit(retained_root, json);
    outcome
}

fn verify_generate_root_or_exit(retained_root: &RetainedGenerateRoot, json: bool) {
    retained_root
        .verify_public_path()
        .unwrap_or_else(|error| exit_generation_inconclusive(json, error));
}

fn exit_generation_inconclusive(json: bool, error: String) -> ! {
    if json {
        let output = serde_json::json!({
            "valid": false,
            "inconclusive": true,
            "error": format!("Generation inconclusive: {error}"),
            "generated": [],
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else {
        eprintln!("Generation inconclusive: {error}");
    }
    process::exit(1);
}

#[cfg(unix)]
type GenerateRootIdentity = (u64, u64);

#[cfg(windows)]
type GenerateRootIdentity = (u32, u64);

#[cfg(not(any(unix, windows)))]
type GenerateRootIdentity = (u64, Option<std::time::SystemTime>);

struct RetainedGenerateRoot {
    requested_path: PathBuf,
    directory: Dir,
    identity: GenerateRootIdentity,
}

impl RetainedGenerateRoot {
    fn open(root: &Path) -> Result<Self, String> {
        let directory = Dir::open_ambient_dir(root, ambient_authority()).map_err(|error| {
            format!(
                "Cannot retain Generate project root {}: {error}",
                root.display()
            )
        })?;
        let identity = generate_directory_identity(&directory)
            .map_err(|error| format!("Cannot identify retained Generate project root: {error}"))?;
        let retained = Self {
            requested_path: root.to_path_buf(),
            directory,
            identity,
        };
        retained.verify_public_path()?;
        Ok(retained)
    }

    fn verify_public_path(&self) -> Result<(), String> {
        let observed =
            Dir::open_ambient_dir(&self.requested_path, ambient_authority()).map_err(|error| {
                format!(
                    "Generate project root {} changed after coverage snapshot: {error}",
                    self.requested_path.display()
                )
            })?;
        let observed = generate_directory_identity(&observed).map_err(|error| {
            format!(
                "Cannot reidentify Generate project root {} after coverage snapshot: {error}",
                self.requested_path.display()
            )
        })?;
        if observed != self.identity {
            return Err(format!(
                "Generate project root {} changed after coverage snapshot",
                self.requested_path.display()
            ));
        }
        Ok(())
    }
}

#[cfg(unix)]
fn generate_directory_identity(directory: &Dir) -> io::Result<GenerateRootIdentity> {
    use cap_std::fs::MetadataExt;

    let metadata = directory.dir_metadata()?;
    Ok((metadata.dev(), metadata.ino()))
}

#[cfg(windows)]
fn generate_directory_identity(directory: &Dir) -> io::Result<GenerateRootIdentity> {
    use cap_primitives::fs::_WindowsByHandle;

    let metadata = directory.dir_metadata()?;
    let volume = metadata
        .volume_serial_number()
        .ok_or_else(|| io::Error::other("Windows volume serial number is unavailable"))?;
    let index = metadata
        .file_index()
        .ok_or_else(|| io::Error::other("Windows file index is unavailable"))?;
    Ok((volume, index))
}

#[cfg(not(any(unix, windows)))]
fn generate_directory_identity(directory: &Dir) -> io::Result<GenerateRootIdentity> {
    let metadata = directory.dir_metadata()?;
    Ok((metadata.len(), metadata.modified().ok()))
}

fn generate_after_coverage_test_barrier_or_exit(json: bool) {
    if let Err(error) = generate_after_coverage_test_barrier() {
        exit_generation_inconclusive(json, error);
    }
}

#[cfg(debug_assertions)]
fn generate_after_coverage_test_barrier() -> Result<(), String> {
    use std::io::Write;

    const BARRIER_ENV: &str = "SPECSYNC_TEST_GENERATE_ROOT_IDENTITY_BARRIER";
    const TEST_CONTEXT_ENV: &str = "SPECSYNC_TEST_CONTEXT";
    const TEST_CONTEXT: &str = "generate-root-identity";
    const MARKER: &str = "coverage-complete";

    let Some(directory) = std::env::var_os(BARRIER_ENV) else {
        return Ok(());
    };
    if std::env::var(TEST_CONTEXT_ENV).as_deref() != Ok(TEST_CONTEXT) {
        return Err(format!(
            "Generate test barrier requires {TEST_CONTEXT_ENV}={TEST_CONTEXT}"
        ));
    }

    let directory = std::path::PathBuf::from(directory);
    let mut ready = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(directory.join(MARKER))
        .map_err(|error| format!("Cannot create generate test barrier: {error}"))?;
    ready
        .write_all(b"coverage-complete\n")
        .and_then(|_| ready.sync_all())
        .map_err(|error| format!("Cannot publish generate test barrier: {error}"))?;
    drop(ready);

    let resume = directory.join("resume");
    let started = std::time::Instant::now();
    loop {
        match std::fs::symlink_metadata(&resume) {
            Ok(metadata) if metadata.is_file() => return Ok(()),
            Ok(_) => {
                return Err("Generate test barrier resume marker is not a file".to_string());
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "Cannot inspect generate test barrier resume marker: {error}"
                ));
            }
        }
        if started.elapsed() >= std::time::Duration::from_secs(10) {
            return Err("Timed out waiting for generate test barrier".to_string());
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[cfg(not(debug_assertions))]
fn generate_after_coverage_test_barrier() -> Result<(), String> {
    Ok(())
}
