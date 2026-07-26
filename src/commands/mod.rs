pub mod agents;
pub mod archive_tasks;
pub mod change;
pub mod changelog;
pub mod check;
pub mod comment;
pub mod compact;
pub mod coverage;
pub mod deps;
pub mod diff;
pub mod generate;
pub mod hooks;
pub mod import;
pub mod init;
pub mod init_registry;
pub mod issues;
pub mod lifecycle;
pub mod merge;
pub mod migrate;
pub mod new;
pub mod rehash;
pub mod report;
pub mod resolve;
pub mod rules;
pub mod scaffold;
pub mod score;
pub mod stale;
pub mod view;
pub mod wizard;

use colored::Colorize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process;

use crate::config::load_config;
use crate::hash_cache;
use crate::ignore::IgnoreRules;
use crate::parser;
use crate::schema;
use crate::scoring;
use crate::types;
use crate::types::SpecStatus;
use crate::validator::{find_spec_files, source_within_root, validate_spec};

pub fn load_and_discover(root: &Path, allow_empty: bool) -> (types::SpecSyncConfig, Vec<PathBuf>) {
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
        let abs_specs = root.join(&config.specs_dir);
        println!(
            "No spec files found in {}/. Run `specsync generate` to scaffold specs.",
            abs_specs.display()
        );
        process::exit(0);
    }

    (config, spec_files)
}

fn normalize_project_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn collect_validation_files(root: &Path, directory: &Path, inputs: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    let mut paths = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    paths.sort();
    for path in paths {
        let Ok(metadata) = std::fs::symlink_metadata(&path) else {
            continue;
        };
        if metadata.is_dir() && !metadata.file_type().is_symlink() {
            collect_validation_files(root, &path, inputs);
        } else if metadata.is_file() {
            inputs.push(normalize_project_path(root, &path));
        }
    }
}

/// Inputs that can alter any spec's validation result independently of that
/// spec's own files. The ignore path is retained even when absent so creating
/// it invalidates prior snapshots.
fn global_validation_inputs(root: &Path, config: &types::SpecSyncConfig) -> Vec<String> {
    let mut inputs = vec![".specsyncignore".to_string()];
    if let Some(config_path) = &config.config_path {
        inputs.push(normalize_project_path(root, config_path));
    }
    if let Some(schema_dir) = &config.schema_dir {
        collect_validation_files(root, &root.join(schema_dir), &mut inputs);
    }
    inputs.sort();
    inputs.dedup();
    inputs
}

fn spec_inventory(root: &Path, spec_files: &[PathBuf]) -> Vec<String> {
    let mut inventory = spec_files
        .iter()
        .map(|path| normalize_project_path(root, path))
        .collect::<Vec<_>>();
    inventory.sort();
    inventory.dedup();
    inventory
}

/// Validate a user-supplied module name used by the scaffolding commands
/// (`new`, `add-spec`, `scaffold`, `wizard`). The name is written verbatim into paths
/// like `<specs_dir>/<name>/<name>.spec.md` and joined onto source dirs, so an
/// unvalidated name containing a path separator, `.`/`..`, or an absolute/drive-relative
/// path would let scaffolding create files anywhere on disk (path traversal).
///
/// A valid name is a single plain path segment: exactly one `Component::Normal`, with no
/// raw path separator and no control characters. The component check is platform-aware —
/// it also rejects Windows drive-relative prefixes like `C:foo` that `Path::is_absolute`
/// misses. Control characters are rejected so a name cannot inject into the generated
/// YAML frontmatter or create a control-char directory. Returns `Err` (to be printed and
/// exited on) rather than writing outside the project.
pub fn validate_module_name(module_name: &str) -> Result<(), String> {
    let single_normal_segment = {
        let mut components = Path::new(module_name).components();
        matches!(components.next(), Some(std::path::Component::Normal(_)))
            && components.next().is_none()
    };
    let clean = !module_name.contains('/')
        && !module_name.contains('\\')
        && !module_name.chars().any(char::is_control);
    if single_normal_segment && clean {
        return Ok(());
    }
    Err(format!(
        "invalid module name `{}`: use a single plain name — no path separators (`/`, `\\`), \
         `.`/`..`, drive prefixes, absolute paths, or control characters",
        module_name.escape_default()
    ))
}

/// Filter spec files by user-provided spec names/paths.
/// Matches against: exact file path, relative path, module name (from filename stem).
/// Returns the full list if `filters` is empty.
pub fn filter_specs(root: &Path, spec_files: &[PathBuf], filters: &[String]) -> Vec<PathBuf> {
    if filters.is_empty() {
        return spec_files.to_vec();
    }

    let mut matched: Vec<PathBuf> = Vec::new();
    let mut unmatched: Vec<&String> = Vec::new();

    for filter in filters {
        let mut found = false;
        for spec_file in spec_files {
            let rel = spec_file
                .strip_prefix(root)
                .unwrap_or(spec_file)
                .to_string_lossy()
                .to_string();

            // Match by: exact path, relative path, filename, or module name
            let stem = spec_file.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let module = stem.strip_suffix(".spec").unwrap_or(stem);

            if rel == *filter
                || spec_file.to_string_lossy() == *filter
                || stem == *filter
                || module == *filter
                || filter.ends_with(".spec.md") && rel.ends_with(filter.as_str())
            {
                if !matched.contains(spec_file) {
                    matched.push(spec_file.clone());
                }
                found = true;
            }
        }
        if !found {
            unmatched.push(filter);
        }
    }

    if !unmatched.is_empty() {
        eprintln!(
            "{} No specs matched: {}",
            "Warning:".yellow(),
            unmatched
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    matched
}

/// Read only the YAML frontmatter section of a spec file (up to the closing `---`).
/// Avoids reading the full file body when only metadata is needed, reducing I/O
/// for commands that re-read specs later for full validation.
fn read_frontmatter_section(path: &Path) -> std::io::Result<String> {
    use std::io::{BufRead, BufReader};
    let file = std::fs::File::open(path)?;
    let reader = BufReader::new(file);
    let mut result = String::new();
    let mut found_start = false;

    for line in reader.lines() {
        let line = line?;
        result.push_str(&line);
        result.push('\n');
        if line.trim() == "---" {
            if found_start {
                break; // Found closing ---
            }
            found_start = true;
        }
    }
    Ok(result)
}

/// Filter spec files by lifecycle status.
/// `exclude` removes specs with any of the listed statuses.
/// `only` keeps only specs with one of the listed statuses.
/// If both are empty, returns the full list unchanged.
pub fn filter_by_status(
    spec_files: &[PathBuf],
    exclude: &[String],
    only: &[String],
) -> Vec<PathBuf> {
    if exclude.is_empty() && only.is_empty() {
        return spec_files.to_vec();
    }

    // Warn about unrecognized status values so typos don't silently filter nothing
    for s in exclude.iter().chain(only.iter()) {
        if SpecStatus::from_str_loose(s).is_none() {
            eprintln!(
                "{} unknown status '{}' — valid statuses: draft, review, active, stable, deprecated, archived",
                "warning:".yellow().bold(),
                s
            );
        }
    }

    let exclude_set: HashSet<SpecStatus> = exclude
        .iter()
        .filter_map(|s| SpecStatus::from_str_loose(s))
        .collect();
    let only_set: HashSet<SpecStatus> = only
        .iter()
        .filter_map(|s| SpecStatus::from_str_loose(s))
        .collect();

    spec_files
        .iter()
        .filter(|path| {
            // Read only the frontmatter section (up to closing ---) to avoid
            // re-reading the full file body that callers will parse later.
            let status = read_frontmatter_section(path)
                .ok()
                .and_then(|content| parser::parse_frontmatter(&content.replace("\r\n", "\n")))
                .and_then(|parsed| parsed.frontmatter.parsed_status());

            // If we can't parse status: include when excluding (let validation catch the error),
            // but exclude when --only-status is active (no status ≠ matching status).
            let status = match status {
                Some(s) => s,
                None => return only_set.is_empty(),
            };

            if !exclude_set.is_empty() && exclude_set.contains(&status) {
                return false;
            }
            if !only_set.is_empty() && !only_set.contains(&status) {
                return false;
            }
            true
        })
        .cloned()
        .collect()
}

/// Build column-level schema from migration files (if schema_dir is configured).
pub fn build_schema_columns(
    root: &Path,
    config: &types::SpecSyncConfig,
) -> std::collections::HashMap<String, schema::SchemaTable> {
    match &config.schema_dir {
        Some(dir) => schema::build_schema(&root.join(dir)),
        None => std::collections::HashMap::new(),
    }
}

/// Run validation, returning counts and collected error, warning, and notice strings.
/// `ownership_spec_files` must contain the complete discovered inventory even
/// when `spec_files` is an incremental subset.
/// When `collect` is true, diagnostics are collected into vectors instead of printing inline.
/// When `explain` is true (text mode), shows per-category score breakdown for each spec.
#[allow(clippy::too_many_arguments)]
pub fn run_validation(
    root: &Path,
    spec_files: &[PathBuf],
    ownership_spec_files: &[PathBuf],
    schema_tables: &std::collections::HashSet<String>,
    schema_columns: &std::collections::HashMap<String, schema::SchemaTable>,
    config: &types::SpecSyncConfig,
    collect: bool,
    explain: bool,
    ignore_rules: &IgnoreRules,
) -> (
    usize,
    usize,
    usize,
    usize,
    Vec<String>,
    Vec<String>,
    Vec<String>,
) {
    run_validation_with_cache(
        root,
        spec_files,
        ownership_spec_files,
        schema_tables,
        schema_columns,
        config,
        collect,
        explain,
        ignore_rules,
        None,
        &[],
        &[],
    )
}

#[allow(clippy::too_many_arguments)]
fn run_validation_with_cache(
    root: &Path,
    spec_files: &[PathBuf],
    ownership_spec_files: &[PathBuf],
    schema_tables: &std::collections::HashSet<String>,
    schema_columns: &std::collections::HashMap<String, schema::SchemaTable>,
    config: &types::SpecSyncConfig,
    collect: bool,
    explain: bool,
    ignore_rules: &IgnoreRules,
    mut cache: Option<&mut hash_cache::HashCache>,
    global_inputs: &[String],
    spec_inventory: &[String],
) -> (
    usize,
    usize,
    usize,
    usize,
    Vec<String>,
    Vec<String>,
    Vec<String>,
) {
    let mut total_errors = 0;
    let mut total_warnings = 0;
    let mut passed = 0;
    let mut drafts_skipped = 0;
    let mut all_errors: Vec<String> = Vec::new();
    let mut all_warnings: Vec<String> = Vec::new();
    let mut all_notices: Vec<String> = Vec::new();
    let mut file_owners: HashMap<String, Vec<String>> = HashMap::new();
    let mut spec_files_by_path: HashMap<PathBuf, HashSet<String>> = HashMap::new();
    for spec_file in ownership_spec_files {
        let Ok(content) = std::fs::read_to_string(spec_file) else {
            continue;
        };
        let normalized = if content.contains("\r\n") {
            std::borrow::Cow::Owned(content.replace("\r\n", "\n"))
        } else {
            std::borrow::Cow::Borrowed(content.as_str())
        };
        let Some(parsed) = parser::parse_frontmatter(&normalized) else {
            continue;
        };
        if parsed.frontmatter.parsed_status() == Some(SpecStatus::Archived) {
            continue;
        }
        let owner = spec_file
            .strip_prefix(root)
            .unwrap_or(spec_file)
            .to_string_lossy()
            .replace('\\', "/");
        let mut existing_files = HashSet::new();
        for file in &parsed.frontmatter.files {
            if root.join(file).is_file()
                && source_within_root(root, file)
                && !file.contains('\\')
                && let Some(normalized_file) = crate::validator::normalize_source_mapping(file)
            {
                let owners = file_owners.entry(normalized_file.clone()).or_default();
                if !owners.contains(&owner) {
                    owners.push(owner.clone());
                }
                existing_files.insert(normalized_file);
            }
        }
        spec_files_by_path.insert(spec_file.clone(), existing_files);
    }

    for spec_file in spec_files {
        let input_digest_before = cache.as_deref().map(|_| {
            hash_cache::HashCache::current_validation_input_digest(
                root,
                spec_file,
                global_inputs,
                spec_inventory,
            )
        });
        let mut result = validate_spec(spec_file, root, schema_tables, schema_columns, config);
        let owner = spec_file
            .strip_prefix(root)
            .unwrap_or(spec_file)
            .to_string_lossy()
            .replace('\\', "/");
        for file in spec_files_by_path.get(spec_file).into_iter().flatten() {
            if let Some(owners) = file_owners.get(file).filter(|owners| owners.len() > 1) {
                let others = owners
                    .iter()
                    .filter(|candidate| *candidate != &owner)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ");
                result.errors.push(format!(
                    "Source file has duplicate spec ownership: {file} (also mapped by {others})"
                ));
            }
        }
        let is_draft = result.status == Some(SpecStatus::Draft);
        if is_draft {
            drafts_skipped += 1;
        }

        // Parse inline ignore directives from the spec file
        let inline_ignores = std::fs::read_to_string(spec_file)
            .map(|content| IgnoreRules::parse_inline(&content))
            .unwrap_or_default();

        // Filter out suppressed warnings
        let filtered_warnings: Vec<&String> = result
            .warnings
            .iter()
            .filter(|w| !ignore_rules.is_suppressed(w, &result.spec_path, &inline_ignores))
            .collect();

        // Record the complete, input-bound outcome so a warm-cache run can
        // replay findings without becoming run-history dependent.
        if let (Some(cache), Some(input_digest_before)) =
            (cache.as_deref_mut(), input_digest_before.as_deref())
        {
            cache.record_validation_snapshot(
                root,
                spec_file,
                global_inputs,
                spec_inventory,
                input_digest_before,
                hash_cache::ValidationDiagnostics {
                    errors: result.errors.clone(),
                    warnings: filtered_warnings.iter().map(|w| (*w).clone()).collect(),
                    notices: result.notices.clone(),
                },
            );
        }

        if collect {
            let prefix = &result.spec_path;
            all_errors.extend(result.errors.iter().map(|e| format!("{prefix}: {e}")));
            all_warnings.extend(filtered_warnings.iter().map(|w| format!("{prefix}: {w}")));
            all_notices.extend(
                result
                    .notices
                    .iter()
                    .map(|notice| format!("{prefix}: {notice}")),
            );
            total_errors += result.errors.len();
            total_warnings += filtered_warnings.len();
            if result.errors.is_empty() {
                passed += 1;
            }
            continue;
        }

        // Use filtered warnings for text output
        let warnings: Vec<&str> = filtered_warnings.iter().map(|w| w.as_str()).collect();

        println!("\n{}", result.spec_path.bold());

        // Frontmatter check
        let has_fm_errors = result
            .errors
            .iter()
            .any(|e| e.starts_with("Frontmatter") || e.starts_with("Missing or malformed"));
        if has_fm_errors {
            println!("  {} Frontmatter invalid", "✗".red());
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

        if file_errors.is_empty() && result.notices.is_empty() && has_files_field {
            println!("  {} All source files exist", "✓".green());
        } else {
            for e in &file_errors {
                println!("  {} {e}", "✗".red());
            }
            for notice in &result.notices {
                println!("  {} {notice}", "⊘".cyan());
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
        let col_warnings: Vec<&str> = warnings
            .iter()
            .filter(|w| w.starts_with("Schema column"))
            .copied()
            .collect();
        for e in &col_errors {
            println!("  {} {e}", "✗".red());
        }
        for w in &col_warnings {
            println!("  {} {w}", "⚠".yellow());
        }

        // Section check
        // Drafts skip required-section validation by design — say so instead of
        // printing a misleading "all sections present" checkmark.
        let section_errors: Vec<&str> = result
            .errors
            .iter()
            .filter(|e| e.starts_with("Missing required section"))
            .map(|s| s.as_str())
            .collect();
        if is_draft {
            println!(
                "  {} Section validation skipped (status: draft)",
                "⊘".yellow()
            );
        } else if section_errors.is_empty() {
            println!("  {} All required sections present", "✓".green());
        } else {
            for e in &section_errors {
                println!("  {} {e}", "✗".red());
            }
        }

        // API surface
        // Drafts skip export drift detection by design — make that visible.
        let api_line = warnings.iter().find(|w| {
            w.contains("exports documented")
                && w.chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
        });
        if is_draft {
            println!(
                "  {} Export validation skipped (status: draft)",
                "⊘".yellow()
            );
        } else if let Some(line) = api_line {
            // The partial-coverage summary is recorded (and counted) as a
            // warning — print it as one so the summary's warning count matches
            // the number of ⚠ lines shown.
            println!("  {} {line}", "⚠".yellow());
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

        let undocumented: Vec<&str> = warnings
            .iter()
            .filter(|w| w.starts_with("Export '") || w.starts_with("Undocumented export '"))
            .copied()
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
        for w in warnings.iter().filter(|w| w.starts_with("Consumed By")) {
            println!("  {} {w}", "⚠".yellow());
        }

        // Stub section warnings
        for w in warnings
            .iter()
            .filter(|w| w.starts_with("Section ##") && w.contains("stub"))
        {
            println!("  {} {w}", "⚠".yellow());
        }

        // Requirements companion file warnings
        for w in warnings.iter().filter(|w| w.contains("requirements")) {
            println!("  {} {w}", "⚠".yellow());
        }

        // Custom rule violations and any other uncategorized warnings/errors
        let categorized_error_prefixes = [
            "Frontmatter",
            "Missing or malformed",
            "Source file",
            "DB table",
            "Schema column",
            "Missing required section",
            "Spec documents",
            "Dependency spec",
        ];
        let categorized_warning_prefixes = [
            "exports documented",
            "Export '",
            "Undocumented export '",
            "Consumed By",
            "Schema column",
        ];
        for e in result
            .errors
            .iter()
            .filter(|e| !categorized_error_prefixes.iter().any(|p| e.starts_with(p)))
        {
            println!("  {} {e}", "✗".red());
        }
        for w in warnings.iter().filter(|w| {
            !(categorized_warning_prefixes
                .iter()
                .any(|p| w.starts_with(p) || w.contains(p))
                || (w.starts_with("Section ##") && w.contains("stub"))
                || w.contains("requirements"))
        }) {
            println!("  {} {w}", "⚠".yellow());
        }

        // Show fix suggestions when there are errors or warnings with fixes
        if !result.fixes.is_empty() && (!result.errors.is_empty() || !warnings.is_empty()) {
            println!("  {}", "Suggested fixes:".cyan());
            for fix in &result.fixes {
                println!("    {} {fix}", "->".cyan());
            }
        }

        // --explain: show per-category score breakdown
        if explain {
            let score = scoring::score_spec(spec_file, root, config);
            let grade_colored = match score.grade {
                "A" => score.grade.green().bold().to_string(),
                "B" => score.grade.green().to_string(),
                "C" => score.grade.yellow().to_string(),
                "D" => score.grade.yellow().bold().to_string(),
                _ => score.grade.red().bold().to_string(),
            };
            println!(
                "  {} [{}] {}/100 — {} {}/20  {} {}/20  {} {}/20  {} {}/20  {} {}/20",
                "Score:".dimmed(),
                grade_colored,
                score.total,
                "FM:".dimmed(),
                colorize_subscore(score.frontmatter_score),
                "Sec:".dimmed(),
                colorize_subscore(score.sections_score),
                "API:".dimmed(),
                colorize_subscore(score.api_score),
                "Depth:".dimmed(),
                colorize_subscore(score.depth_score),
                "Fresh:".dimmed(),
                colorize_subscore(score.freshness_score),
            );
            for suggestion in &score.suggestions {
                println!("    {} {suggestion}", "->".cyan());
            }
        }

        total_errors += result.errors.len();
        total_warnings += warnings.len();
        if result.errors.is_empty() {
            passed += 1;
        }
    }

    // Schema/migration files that exist but can't be read as UTF-8 are silently
    // skipped by build_schema — their tables/columns vanish, which can silently
    // disable db_tables/column validation (an all-unreadable schema makes the
    // checks a no-op). Surface them once, project-level (not per spec), as hard
    // errors so the gate fails loud instead of under-validating.
    if let Some(dir) = &config.schema_dir {
        for err in schema::schema_read_errors(&root.join(dir)) {
            total_errors += 1;
            if collect {
                all_errors.push(err);
            } else {
                println!("\n{} {err}", "✗".red());
            }
        }
    }

    if !collect && drafts_skipped > 0 {
        println!(
            "\n{} {drafts_skipped} draft spec(s) skipped section and export validation — set `status: active` to enable full checks",
            "ℹ".yellow()
        );
    }

    (
        total_errors,
        total_warnings,
        passed,
        spec_files.len(),
        all_errors,
        all_warnings,
        all_notices,
    )
}

/// Colorize a subscore (out of 20) — green for 20, yellow for 10-19, red for <10.
fn colorize_subscore(score: u32) -> String {
    let s = score.to_string();
    match score {
        20 => s.green().to_string(),
        10..=19 => s.yellow().to_string(),
        _ => s.red().to_string(),
    }
}

/// Compute exit code without printing or exiting.
pub fn compute_exit_code(
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
    if let Some(req) = require_coverage {
        // A `--require-coverage` gate over zero source files is a vacuous pass:
        // coverage is reported as 100% when there is nothing to measure (an empty or
        // misconfigured `source_dirs`, or an over-broad `exclude_patterns`), silently
        // satisfying the gate. Fail loud so a broken config cannot pass CI.
        if req > 0 && coverage.total_source_files == 0 {
            return 1;
        }
        if coverage.coverage_percent < req {
            return 1;
        }
    }
    0
}

pub fn exit_with_status(
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
                process::exit(1);
            }
        }
        Strict => {
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
        }
    }

    if let Some(req) = require_coverage {
        // Fail loud on a vacuous pass: `--require-coverage` over zero source files
        // reports 100% because there is nothing to measure (empty/misconfigured
        // `source_dirs` or an over-broad `exclude_patterns`), which would otherwise
        // satisfy the gate silently.
        if req > 0 && coverage.total_source_files == 0 {
            println!(
                "\n{} {req}%: no source files were found to measure coverage against — \
                 check `source_dirs` and `exclude_patterns` (an empty or over-broad \
                 config reports coverage as a vacuous 100%)",
                "--require-coverage".red()
            );
            process::exit(1);
        }
        if coverage.coverage_percent < req {
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
}

/// Create GitHub issues for specs with validation errors.
/// `all_errors` contains strings in the format `"spec/path: error message"`.
pub fn create_drift_issues(
    root: &Path,
    config: &types::SpecSyncConfig,
    all_errors: &[String],
    format: types::OutputFormat,
) {
    let repo_config = config.github.as_ref().and_then(|g| g.repo.as_deref());
    let repo = match crate::github::resolve_repo(repo_config, root) {
        Ok(r) => r,
        Err(e) => {
            if matches!(format, types::OutputFormat::Text) {
                eprintln!("{} Cannot create issues: {e}", "error:".red().bold());
            }
            return;
        }
    };

    let labels = config
        .github
        .as_ref()
        .map(|g| g.drift_labels.clone())
        .unwrap_or_else(|| vec!["spec-drift".to_string()]);

    // Group errors by spec path (format: "spec/path: error message")
    let mut errors_by_spec: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for entry in all_errors {
        if let Some((spec, error)) = entry.split_once(": ") {
            errors_by_spec
                .entry(spec.to_string())
                .or_default()
                .push(error.to_string());
        }
    }

    if matches!(format, types::OutputFormat::Text) {
        println!(
            "\n{} Creating GitHub issues for {} spec(s) with errors...",
            "⟳".cyan(),
            errors_by_spec.len()
        );
    }

    for (spec_path, errors) in &errors_by_spec {
        match crate::github::create_drift_issue(&repo, spec_path, errors, &labels) {
            Ok(issue) => {
                if matches!(format, types::OutputFormat::Text) {
                    println!(
                        "  {} Created issue #{} for {spec_path}: {}",
                        "✓".green(),
                        issue.number,
                        issue.url
                    );
                }
            }
            Err(e) => {
                if matches!(format, types::OutputFormat::Text) {
                    eprintln!(
                        "  {} Failed to create issue for {spec_path}: {e}",
                        "✗".red()
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::validate_module_name;

    #[test]
    fn validate_module_name_accepts_plain_names() {
        for name in [
            "auth",
            "auth-service",
            "user_profile",
            "v2",
            "a.b",
            "Módulo",
        ] {
            assert!(
                validate_module_name(name).is_ok(),
                "`{name}` should be a valid module name"
            );
        }
    }

    #[test]
    fn validate_module_name_rejects_traversal_and_injection() {
        // Empty, path separators, parent/current refs, absolute paths, and control
        // characters must all be refused — none may reach a filesystem join.
        for name in [
            "",
            ".",
            "..",
            "../evil",
            "../../PWNED/evil",
            "a/b",
            "a\\b",
            "sub/mod",
            "/tmp/abs",
            "auth/", // trailing separator normalizes to one segment, still refused
            "evil\nversion: 99", // newline → frontmatter injection
            "tab\tname",
            "null\0byte",
        ] {
            assert!(
                validate_module_name(name).is_err(),
                "`{}` must be rejected as an unsafe module name",
                name.escape_default()
            );
        }
    }

    #[test]
    #[cfg(windows)]
    fn validate_module_name_rejects_windows_drive_relative() {
        // `C:foo` has no separator and `is_absolute()` is false (drive-relative), but its
        // components include a Prefix, so the single-Normal-segment check refuses it.
        assert!(validate_module_name("C:foo").is_err());
        assert!(validate_module_name("C:\\abs").is_err());
    }
}
