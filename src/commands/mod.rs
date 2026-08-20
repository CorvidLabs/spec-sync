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
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::process;

use crate::ignore::IgnoreRules;
use crate::parser;
use crate::scoring;
use crate::types;
use crate::types::SpecStatus;
use crate::validator::{
    find_spec_files, load_config_and_discover_retained, schema_config_problems_for_snapshot,
    schema_table_names_from_snapshot, source_within_root, validate_spec,
};

pub fn load_and_discover(root: &Path, allow_empty: bool) -> (types::SpecSyncConfig, Vec<PathBuf>) {
    let (config, spec_files) = load_config_and_discover_retained(root).unwrap_or_else(|error| {
        eprintln!("SpecSync discovery is inconclusive: {error}");
        process::exit(1);
    });

    // Single choke point: every command that reads specs comes through here, so
    // none of them can report a verdict over rules that failed to load (#570).
    refuse_unloadable_config(&config);

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

#[allow(dead_code)]
fn normalize_project_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[allow(dead_code)]
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
#[allow(dead_code)]
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

/// List normalized inventory paths for selected specs (validation snapshot inputs).
#[allow(dead_code)]
pub(crate) fn spec_inventory(root: &Path, spec_files: &[PathBuf]) -> Vec<String> {
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
/// A valid name is a single portable path segment: exactly one `Component::Normal`, with no
/// raw path separator, control characters, trailing spaces/dots, or Windows reserved device
/// basename. The generated `<name>.spec.md` component must also fit within the portable 255-byte
/// limit. The component check is platform-aware — it also rejects Windows drive-relative prefixes
/// like `C:foo` that `Path::is_absolute` misses. Returns `Err` (to be printed and exited on) rather
/// than writing outside the project.
pub fn validate_module_name(module_name: &str) -> Result<(), String> {
    const SPEC_SUFFIX: &str = ".spec.md";
    const MAX_COMPONENT_BYTES: usize = 255;

    let single_normal_segment = {
        let mut components = Path::new(module_name).components();
        matches!(components.next(), Some(std::path::Component::Normal(_)))
            && components.next().is_none()
    };
    let clean = !module_name.contains('/')
        && !module_name.contains('\\')
        && !module_name
            .chars()
            .any(|character| matches!(character, '<' | '>' | ':' | '"' | '|' | '?' | '*'))
        && !module_name.chars().any(char::is_control);
    if !single_normal_segment || !clean {
        return Err(format!(
            "invalid module name `{}`: use a single plain name — no path separators (`/`, `\\`), \
             Windows-invalid characters (`<`, `>`, `:`, `\"`, `|`, `?`, `*`), `.`/`..`, drive \
             prefixes, absolute paths, or control characters",
            module_name.escape_default()
        ));
    }

    if module_name != module_name.trim() || module_name.ends_with('.') {
        return Err(format!(
            "invalid module name `{}`: leading/trailing spaces and trailing dots are not portable",
            module_name.escape_default()
        ));
    }

    if module_name.len() + SPEC_SUFFIX.len() > MAX_COMPONENT_BYTES {
        return Err(format!(
            "invalid module name `{}`: UTF-8 name must not exceed {} bytes so `{SPEC_SUFFIX}` fits \
             within a {MAX_COMPONENT_BYTES}-byte path component",
            module_name.escape_default(),
            MAX_COMPONENT_BYTES - SPEC_SUFFIX.len()
        ));
    }

    let basename = module_name.split('.').next().unwrap_or_default();
    let uppercase_basename = basename.to_ascii_uppercase();
    let numbered_device = uppercase_basename.len() == 4
        && (uppercase_basename.starts_with("COM") || uppercase_basename.starts_with("LPT"))
        && matches!(uppercase_basename.as_bytes()[3], b'1'..=b'9');
    let reserved_device =
        matches!(uppercase_basename.as_str(), "CON" | "PRN" | "AUX" | "NUL") || numbered_device;
    if reserved_device {
        return Err(format!(
            "invalid module name `{}`: `{basename}` is a Windows reserved device basename",
            module_name.escape_default()
        ));
    }

    Ok(())
}

/// Stricter naming rules for names the scaffolding commands (`new`, `add-spec`,
/// `scaffold`, `wizard`) are about to mint. On top of the traversal-safety base
/// rules, a newly scaffolded name must satisfy the documented naming rules:
/// length, character set, reserved words. Internal callers validating
/// pre-existing module names use [`validate_module_name`] so older repos with
/// legacy names keep working.
/// Maximum module-name length.
pub(crate) const MAX_MODULE_NAME_LEN: usize = 64;

/// Names that cannot be a directory component on a platform we ship for.
///
/// Shared with `change::slugify`, which mints directory names from free text and would
/// otherwise produce `nul` from the description "NUL". Windows device names are matched
/// case-insensitively by the OS, so a lowercase slug is not an escape. Reused rather than
/// reimplemented — a second copy of this list is exactly how these drift apart.
pub(crate) fn is_reserved_module_name(lower: &str) -> bool {
    const RESERVED: &[&str] = &[
        "change", "changes", "spec", "specs", "con", "prn", "aux", "nul", "com1", "com2", "com3",
        "com4", "com5", "com6", "com7", "com8", "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5",
        "lpt6", "lpt7", "lpt8", "lpt9",
    ];
    RESERVED.contains(&lower)
}

/// Documented naming rules for scaffold errors.
pub(crate) const MODULE_NAME_RULES: &str = "1-64 chars: letters, digits, `-`, `_`, `.`; must start with a letter or digit; no spaces, path separators, or reserved names";

pub(crate) fn validate_scaffold_module_name(module_name: &str) -> Result<(), String> {
    validate_module_name(module_name)?;

    // Length limit (character count, not bytes).
    let char_count = module_name.chars().count();
    if char_count == 0 || char_count > MAX_MODULE_NAME_LEN {
        return Err(format!(
            "invalid module name `{}`: must be 1-{MAX_MODULE_NAME_LEN} characters (got {char_count}). Rules: {MODULE_NAME_RULES}",
            module_name.escape_default()
        ));
    }

    // Character set: letters/digits plus `-`, `_`, `.`; must start with a
    // letter or digit (no leading dash, dot, emoji, or whitespace).
    let mut chars = module_name.chars();
    let first = chars.next().unwrap_or(' ');
    let charset_ok = first.is_alphanumeric()
        && module_name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.');
    if !charset_ok {
        return Err(format!(
            "invalid module name `{}`: {MODULE_NAME_RULES}",
            module_name.escape_default()
        ));
    }

    // Reserved words (case-insensitive — `CON` is as unwritable as `con`).
    if is_reserved_module_name(&module_name.to_lowercase()) {
        return Err(format!(
            "invalid module name `{module_name}`: reserved name. Rules: {MODULE_NAME_RULES}"
        ));
    }

    Ok(())
}

/// Reject a module name that would collide with an existing spec directory
/// under a case-folding filesystem (e.g. `Lib` vs existing `lib`). Linux
/// allows both, but the repo then breaks on macOS/Windows checkouts.
pub(crate) fn check_case_collision(specs_dir: &Path, module_name: &str) -> Result<(), String> {
    let entries = match std::fs::read_dir(specs_dir) {
        Ok(e) => e,
        Err(_) => return Ok(()), // no specs dir yet — nothing to collide with
    };
    for entry in entries.flatten() {
        let existing = entry.file_name().to_string_lossy().to_string();
        if existing != module_name && existing.eq_ignore_ascii_case(module_name) {
            return Err(format!(
                "invalid module name `{module_name}`: collides with existing spec directory \
                 `{existing}` on case-insensitive filesystems (macOS/Windows) — pick a distinct name"
            ));
        }
    }
    Ok(())
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
) -> std::collections::HashMap<String, crate::schema::SchemaTable> {
    match &config.schema_dir {
        Some(dir) => crate::schema::build_schema_snapshot(&root.join(dir))
            .map(|snapshot| snapshot.tables)
            .unwrap_or_default(),
        None => std::collections::HashMap::new(),
    }
}

struct SchemaValidationInput {
    table_names: HashSet<String>,
    tables: HashMap<String, crate::schema::SchemaTable>,
    errors: Vec<String>,
}

fn build_schema_validation_input(
    root: &Path,
    config: &types::SpecSyncConfig,
) -> SchemaValidationInput {
    let Some(directory) = &config.schema_dir else {
        return SchemaValidationInput {
            table_names: HashSet::new(),
            tables: HashMap::new(),
            errors: schema_config_problems_for_snapshot(config, None),
        };
    };

    match crate::schema::build_schema_snapshot(&root.join(directory)) {
        Ok(snapshot) => {
            let (table_names, errors) = match schema_table_names_from_snapshot(&snapshot, config) {
                Ok(table_names) => (table_names, Vec::new()),
                Err(error) => (HashSet::new(), vec![error]),
            };
            SchemaValidationInput {
                table_names,
                tables: snapshot.tables,
                errors,
            }
        }
        Err(error) => {
            let mut errors = schema_config_problems_for_snapshot(config, None);
            errors.push(error.to_string());
            SchemaValidationInput {
                table_names: HashSet::new(),
                tables: HashMap::new(),
                errors,
            }
        }
    }
}

/// Run validation, returning counts and collected error, warning, and notice strings.
/// `ownership_spec_files` must contain the complete discovered inventory even
/// when `spec_files` is an incremental subset.
/// When `collect` is true, diagnostics are collected into vectors instead of printing inline.
/// When `explain` is true (text mode), shows per-category score breakdown for each spec.
#[derive(Debug, Default)]
struct ValidationErrors {
    rendered: Vec<String>,
    drift_by_spec: std::collections::BTreeMap<String, Vec<String>>,
}

impl ValidationErrors {
    fn from_rendered(rendered: &[String], spec_paths: &[String]) -> Self {
        let mut spec_paths = spec_paths.to_vec();
        spec_paths.sort_by_key(|path| std::cmp::Reverse(path.len()));

        let mut errors = Self {
            rendered: rendered.to_vec(),
            drift_by_spec: std::collections::BTreeMap::new(),
        };
        for entry in rendered {
            let attributed = spec_paths.iter().find_map(|spec_path| {
                entry
                    .strip_prefix(spec_path)
                    .and_then(|suffix| suffix.strip_prefix(": "))
                    .map(|error| (spec_path, error))
            });
            let fallback = || entry.split_once(": ");
            if let Some((spec_path, error)) = attributed
                .map(|(spec_path, error)| (spec_path.as_str(), error))
                .or_else(fallback)
            {
                errors
                    .drift_by_spec
                    .entry(spec_path.to_string())
                    .or_default()
                    .push(error.to_string());
            }
        }
        errors
    }

    fn push_for_spec(&mut self, spec_path: &str, error: &str) {
        self.rendered.push(format!("{spec_path}: {error}"));
        self.drift_by_spec
            .entry(spec_path.to_string())
            .or_default()
            .push(error.to_string());
    }

    fn push_unattributed(&mut self, error: String) {
        self.rendered.push(error);
    }

    fn is_empty(&self) -> bool {
        self.rendered.is_empty()
    }

    fn into_rendered(self) -> Vec<String> {
        self.rendered
    }
}

impl Deref for ValidationErrors {
    type Target = [String];

    fn deref(&self) -> &Self::Target {
        &self.rendered
    }
}

impl AsRef<[String]> for ValidationErrors {
    fn as_ref(&self) -> &[String] {
        &self.rendered
    }
}

#[allow(clippy::too_many_arguments)]
pub fn run_validation(
    root: &Path,
    spec_files: &[PathBuf],
    ownership_spec_files: &[PathBuf],
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
    let (errors, warnings, passed, total, rendered_errors, rendered_warnings, notices, _) =
        run_validation_with_suppressions(
            root,
            spec_files,
            ownership_spec_files,
            config,
            collect,
            explain,
            ignore_rules,
            None,
        );
    (
        errors,
        warnings,
        passed,
        total,
        rendered_errors,
        rendered_warnings,
        notices,
    )
}

type ValidationWithSuppressions = (
    usize,
    usize,
    usize,
    usize,
    Vec<String>,
    Vec<String>,
    Vec<String>,
    Vec<serde_json::Value>,
);

/// Per-spec diagnostics collected so `check` can persist them for warm replay.
pub(super) struct SpecValidationOutcome {
    pub spec_file: PathBuf,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub notices: Vec<String>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn run_validation_with_suppressions(
    root: &Path,
    spec_files: &[PathBuf],
    ownership_spec_files: &[PathBuf],
    config: &types::SpecSyncConfig,
    collect: bool,
    explain: bool,
    ignore_rules: &IgnoreRules,
    mut outcomes: Option<&mut Vec<SpecValidationOutcome>>,
) -> ValidationWithSuppressions {
    let schema_input = build_schema_validation_input(root, config);
    let mut total_errors = 0;
    let mut total_warnings = ignore_rules.warnings.len();
    let mut passed = 0;
    let mut drafts_skipped = 0;
    let mut all_errors = ValidationErrors::default();
    let mut all_warnings = ignore_rules
        .warnings
        .iter()
        .map(|warning| format!(".specsyncignore: {warning}"))
        .collect::<Vec<_>>();
    let mut all_notices: Vec<String> = Vec::new();
    let mut all_suppressed_warnings: Vec<serde_json::Value> = Vec::new();
    if !collect {
        for warning in &ignore_rules.warnings {
            println!("{} {warning}", "warning:".yellow().bold());
        }
    }
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
        let mut result = validate_spec(
            spec_file,
            root,
            &schema_input.table_names,
            &schema_input.tables,
            config,
        );
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
            // `status: draft` means two very different things, and only one of
            // them is a problem.
            //
            // A draft whose files do not exist yet is spec-first authoring —
            // the spec is deliberately written before the code, nothing could
            // have been validated, and it rightly passes `--strict` (see
            // `draft_planned_mapping_passes_strict_and_is_absent_from_coverage`).
            //
            // A draft whose files *are* present skipped section and export
            // validation over real source that could have been checked. Every
            // machine-readable channel — exit code, the `N passed` count,
            // coverage percent, `"passed": true` in JSON — then reports success
            // for a spec never compared to its source; a Public API documenting
            // a function that exists nowhere still passed green. Since
            // `generate` writes new specs as draft, that is also the day-one
            // state of an adopting project.
            //
            // Warn only where all three hold — the source is present, and the
            // spec documents a contract over it. An empty Public API is an
            // honest stub that claims nothing and is left alone; only a draft
            // asserting a checkable contract and skipping the check is called
            // out. Bare `check` stays exit 0; `--strict` gates.
            if result.had_present_source && result.documents_contract {
                result.warnings.push(
                    "Spec is `status: draft` — section and export validation were skipped, so a documented Public API was never compared to source that is present; set `status: active` to validate it"
                        .to_string(),
                );
            }
        }

        // Parse inline ignore directives from the spec file
        let inline_ignores = std::fs::read_to_string(spec_file)
            .map(|content| IgnoreRules::parse_inline(&content))
            .unwrap_or_default();

        // Filter only classified warnings, while retaining deterministic
        // machine-readable details for every suppression.
        let mut filtered_warnings = Vec::new();
        for warning in &result.warnings {
            if let Some((category, source)) =
                ignore_rules.suppression_source(warning, &result.spec_path, &inline_ignores)
            {
                all_suppressed_warnings.push(serde_json::json!({
                    "spec": result.spec_path,
                    "warning": warning,
                    "category": category.as_str(),
                    "source": source,
                }));
            } else {
                filtered_warnings.push(warning);
            }
        }

        if let Some(outcomes) = outcomes.as_mut() {
            outcomes.push(SpecValidationOutcome {
                spec_file: spec_file.clone(),
                errors: result.errors.clone(),
                warnings: filtered_warnings
                    .iter()
                    .map(|warning| (*warning).clone())
                    .collect(),
                notices: result.notices.clone(),
            });
        }

        if collect {
            let prefix = &result.spec_path;
            for error in &result.errors {
                all_errors.push_for_spec(prefix, error);
            }
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

        // Frontmatter check — on failure, print each specific frontmatter
        // error inline; a bare "Frontmatter invalid" with the cause buried in
        // Suggested fixes leaves users guessing.
        let fm_errors: Vec<&str> = result
            .errors
            .iter()
            .filter(|e| e.starts_with("Frontmatter") || e.starts_with("Missing or malformed"))
            .map(|s| s.as_str())
            .collect();
        if fm_errors.is_empty() {
            println!("  {} Frontmatter valid", "✓".green());
        } else {
            println!("  {} Frontmatter invalid", "✗".red());
            for e in &fm_errors {
                println!("    {} {e}", "✗".red());
            }
        }

        // File existence
        let file_errors: Vec<&str> = result
            .errors
            .iter()
            .filter(|e| e.starts_with("Source file"))
            .map(|s| s.as_str())
            .collect();
        let has_files_field = !result.errors.iter().any(|e| e.contains("files (must be"));

        // Every check below infers success from the ABSENCE of errors in its
        // category. When frontmatter cannot be parsed there are no inputs, so
        // there are no errors, so each one reported a green line for work that
        // never happened — and `✓ All required sections present` was not merely
        // vacuous but false, on a spec missing six of them. Say "not checked",
        // the same way the draft path already does (#553).
        let frontmatter_invalid = !fm_errors.is_empty();

        if frontmatter_invalid {
            println!(
                "  {} Source file check skipped (frontmatter invalid)",
                "⊘".yellow()
            );
        } else if file_errors.is_empty() && result.notices.is_empty() && has_files_field {
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
        } else if frontmatter_invalid {
            // Guarded on the PROJECT schema having tables, not on the spec's
            // `db_tables:` being readable — which is why matching the shape of
            // the other three checks missed this one entirely.
            println!(
                "  {} DB table check skipped (frontmatter invalid)",
                "⊘".yellow()
            );
        } else if !schema_input.table_names.is_empty() {
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
        } else if frontmatter_invalid {
            println!(
                "  {} Section validation skipped (frontmatter invalid)",
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
        let suppressed_api_summary = result.export_summary.as_ref().is_some_and(|summary| {
            result.warnings.iter().any(|warning| warning == summary)
                && ignore_rules.is_suppressed(summary, &result.spec_path, &inline_ignores)
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
        } else if !suppressed_api_summary && let Some(ref summary) = result.export_summary {
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
        if frontmatter_invalid {
            println!(
                "  {} Dependency check skipped (frontmatter invalid)",
                "⊘".yellow()
            );
        } else if dep_errors.is_empty() {
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

    // Surface schema snapshot and configured-pattern failures once at project
    // scope. The compatibility table/column APIs remain infallible for existing
    // callers, but no failed input may become a vacuous validation pass.
    for error in schema_input.errors {
        total_errors += 1;
        if collect {
            all_errors.push_unattributed(error);
        } else {
            println!("\n{} {error}", "✗".red());
        }
    }

    // .specsyncignore problems (unknown categories, invalid UTF-8 lines):
    // dead rules must be visible, not silently inert.
    for warning in &ignore_rules.warnings {
        total_warnings += 1;
        if collect {
            all_warnings.push(warning.clone());
        } else {
            println!("\n{} {warning}", "⚠".yellow());
        }
    }

    // Suppression must be visible in every output format — otherwise a typo'd
    // rule is indistinguishable from a working one.
    let suppressed_warnings = all_suppressed_warnings.len();
    if suppressed_warnings > 0 {
        let notice = format!("{suppressed_warnings} warning(s) suppressed by .specsyncignore");
        if collect {
            all_notices.push(notice);
        } else {
            println!("\n{} {notice}", "ℹ".cyan());
        }
    }

    if !collect && drafts_skipped > 0 {
        println!(
            "\n{} {drafts_skipped} draft spec(s) skipped section and export validation — set `status: active` to enable full checks",
            "ℹ".yellow()
        );
    }

    fn suppression_sort_key(value: &serde_json::Value) -> (&str, &str, &str, &str) {
        (
            value["spec"].as_str().unwrap_or_default(),
            value["category"].as_str().unwrap_or_default(),
            value["warning"].as_str().unwrap_or_default(),
            value["source"].as_str().unwrap_or_default(),
        )
    }
    all_suppressed_warnings
        .sort_by(|left, right| suppression_sort_key(left).cmp(&suppression_sort_key(right)));

    (
        total_errors,
        total_warnings,
        passed,
        spec_files.len(),
        all_errors.into_rendered(),
        all_warnings,
        all_notices,
        all_suppressed_warnings,
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

/// Resolve the enforcement mode when the user did NOT pass `--enforcement`.
///
/// Consistent convention across all gate commands (`check`, `coverage`,
/// `report`, `score`, `generate`, `comment`):
/// - an explicit `enforcement` key in the config file is honored as-is
///   (including `warn`, which always exits 0);
/// - when enforcement is not configured anywhere, the default GATES ON
///   ERRORS (`Strict`-like): validation errors exit 1. Warnings remain
///   non-blocking unless `--strict` is also passed. This is the only safe
///   default for CI — a command that prints "1 failed" must not exit 0.
pub(crate) fn default_enforcement(config: &types::SpecSyncConfig) -> types::EnforcementMode {
    if config.enforcement_set {
        config.enforcement
    } else {
        types::EnforcementMode::Strict
    }
}

/// Compute exit code without printing or exiting.
/// Refuse to run when a config file exists but could not be loaded.
///
/// The rules in force would be the built-in defaults, not the project's. Every
/// `required_sections` entry, `[rules]` threshold and `exclude_patterns` line it
/// configured is silently absent — and the previous behaviour warned on stderr
/// and carried on, so stdout reported `✓ All required sections present` over a
/// section list that had been thrown away, and a CI job capturing stdout saw a
/// clean pass (#570).
///
/// Reporting success over rules that were never loaded is the same defect as
/// reporting success over checks that never ran, and worse: it disables every
/// configured rule at once. A project writes that file precisely because the
/// defaults are not enough, so substituting the defaults is never the safe
/// reading of a typo.
pub fn refuse_unloadable_config(config: &types::SpecSyncConfig) {
    let Some(reason) = &config.load_error else {
        return;
    };
    eprintln!("{} {reason}", "error:".red().bold());
    eprintln!("  fix the config file, or remove it to use the built-in defaults deliberately");
    process::exit(1);
}

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
            // Discovery skips symlinked entries rather than aborting (#546),
            // which is safe — a link points either outside the root, where it
            // must not be followed, or inside it, where the target is already
            // counted under its real path. What is not safe is the denominator
            // shrinking silently: a repo that symlinks a vendored tree would
            // report a *higher* percentage than before, a number that improved
            // because measurement stopped. Bare `check` reports it; `--strict`
            // refuses to call a partially-measured tree clean.
            if strict && !coverage.skipped_links.is_empty() {
                return 1;
            }
        }
    }
    if let Some(req) = require_coverage {
        // A `--require-coverage` gate over zero source files is a vacuous pass:
        // there is nothing to measure (an empty or misconfigured `source_dirs`,
        // or an over-broad `exclude_patterns`), so the gate has no evidence to
        // act on. Fail closed so a broken config cannot pass CI.
        //
        // `file_coverage_percent()` returns `None` for exactly that case rather
        // than the fabricated `100` the old field held, so the gate can no
        // longer be satisfied by an unmeasured tree even if this branch were
        // removed.
        match coverage.file_coverage_percent() {
            None => {
                if req > 0 {
                    return 1;
                }
            }
            Some(pct) => {
                if pct < req {
                    return 1;
                }
            }
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
            // Mirrors `compute_exit_code`. Discovery skips symlinked entries
            // rather than aborting (#546), which is safe — but the coverage
            // denominator shrinks with them, so a tree that symlinks its
            // vendored source would report a *higher* percentage than before, a
            // number that improved because measurement stopped. Bare `check`
            // reports it; `--strict` refuses to call a partially-measured tree
            // clean.
            if strict && !coverage.skipped_links.is_empty() {
                println!(
                    "\n{}: {} symlinked path(s) were excluded from coverage — the percentages above are measured over the remainder",
                    "--strict mode".red(),
                    coverage.skipped_links.len()
                );
                process::exit(1);
            }
        }
    }

    if let Some(req) = require_coverage {
        // Mirrors `compute_exit_code`. Fail closed on a vacuous pass:
        // `--require-coverage` over zero source files has nothing to measure
        // (empty/misconfigured `source_dirs` or an over-broad
        // `exclude_patterns`), so there is no percentage to compare against.
        match coverage.file_coverage_percent() {
            None => {
                if req > 0 {
                    println!(
                        "\n{} {req}%: no source files were found to measure coverage against — \
                         check `source_dirs` and `exclude_patterns` (nothing was measured, so \
                         the gate has no coverage to compare against)",
                        "--require-coverage".red()
                    );
                    process::exit(1);
                }
            }
            Some(pct) => {
                if pct < req {
                    println!(
                        "\n{} {req}%: actual coverage is {pct}% ({} file(s) missing specs)",
                        "--require-coverage".red(),
                        coverage.unspecced_files.len()
                    );
                    for f in &coverage.unspecced_files {
                        println!("  {} {f}", "✗".red());
                    }
                    process::exit(1);
                }
            }
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
    let spec_paths = find_spec_files(&root.join(&config.specs_dir))
        .into_iter()
        .map(|path| {
            path.strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string()
        })
        .collect::<Vec<_>>();
    let diagnostics = ValidationErrors::from_rendered(all_errors, &spec_paths);
    create_drift_issues_with_diagnostics(root, config, &diagnostics, format);
}

fn create_drift_issues_with_diagnostics(
    root: &Path,
    config: &types::SpecSyncConfig,
    all_errors: &ValidationErrors,
    format: types::OutputFormat,
) {
    let repo_config = config.github.as_ref().and_then(|g| g.repo.as_deref());
    let repo = match crate::github::resolve_repo(repo_config, root) {
        Ok(r) => r,
        Err(e) => {
            if matches!(format, types::OutputFormat::Text) {
                eprintln!(
                    "{} Cannot create issues: {}",
                    "error:".red().bold(),
                    issues::safe_diagnostic(&e)
                );
            }
            return;
        }
    };

    let labels = config
        .github
        .as_ref()
        .map(|g| g.drift_labels.clone())
        .unwrap_or_else(|| vec!["spec-drift".to_string()]);

    if matches!(format, types::OutputFormat::Text) {
        println!(
            "\n{} Creating GitHub issues for {} spec(s) with errors...",
            "⟳".cyan(),
            all_errors.drift_by_spec.len()
        );
    }

    for (spec_path, errors) in &all_errors.drift_by_spec {
        match crate::github::create_drift_issue(&repo, spec_path, errors, &labels) {
            Ok(issue) => {
                if matches!(format, types::OutputFormat::Text) {
                    println!(
                        "  {} Created issue #{} for {}: {}",
                        "✓".green(),
                        issue.number,
                        issues::safe_diagnostic(spec_path),
                        issues::safe_diagnostic(&issue.url)
                    );
                }
            }
            Err(e) => {
                if matches!(format, types::OutputFormat::Text) {
                    eprintln!(
                        "  {} Failed to create issue for {}: {}",
                        "✗".red(),
                        issues::safe_diagnostic(spec_path),
                        issues::safe_diagnostic(&e)
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ValidationErrors, create_drift_issues, run_validation, run_validation_with_suppressions,
        validate_module_name,
    };
    use crate::ignore::{IgnoreRules, WarningCategory};
    use crate::types::{OutputFormat, SpecSyncConfig};
    use std::fs;
    use std::path::{Path, PathBuf};

    type RunValidationSignature = fn(
        &Path,
        &[PathBuf],
        &[PathBuf],
        &SpecSyncConfig,
        bool,
        bool,
        &IgnoreRules,
    ) -> (
        usize,
        usize,
        usize,
        usize,
        Vec<String>,
        Vec<String>,
        Vec<String>,
    );
    type CreateDriftIssuesSignature = fn(&Path, &SpecSyncConfig, &[String], OutputFormat);

    const _: RunValidationSignature = run_validation;
    const _: CreateDriftIssuesSignature = create_drift_issues;

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
            "  spaced  ", // padded names would create literal whitespace directories
            " spaced",
            "spaced ",
        ] {
            assert!(
                validate_module_name(name).is_err(),
                "`{}` must be rejected as an unsafe module name",
                name.escape_default()
            );
        }
    }

    #[test]
    fn validate_module_name_rejects_windows_invalid_characters_portably() {
        for name in [
            "auth<api",
            "auth>api",
            "auth:api",
            "auth\"api",
            "auth|api",
            "auth?api",
            "auth*api",
        ] {
            assert!(
                validate_module_name(name).is_err(),
                "`{name}` must be rejected on every host"
            );
        }
    }

    #[test]
    fn validate_module_name_rejects_windows_reserved_basenames_portably() {
        for name in [
            "CON",
            "con",
            "Con.txt",
            "PRN",
            "prn.backup",
            "AUX",
            "aux.rs",
            "NUL",
            "nul.spec",
            "COM1",
            "com9.log",
            "LPT1",
            "lPt9.anything",
        ] {
            assert!(
                validate_module_name(name).is_err(),
                "`{name}` must be rejected as a Windows reserved basename"
            );
        }
        for number in 1..=9 {
            for prefix in ["COM", "LPT"] {
                let bare = format!("{prefix}{number}");
                let with_extension = format!("{}.txt", bare.to_ascii_lowercase());
                assert!(
                    validate_module_name(&bare).is_err(),
                    "`{bare}` must be rejected"
                );
                assert!(
                    validate_module_name(&with_extension).is_err(),
                    "`{with_extension}` must be rejected"
                );
            }
        }

        for name in ["console", "com0", "com10", "lpt0", "lpt10", "module.con"] {
            assert!(
                validate_module_name(name).is_ok(),
                "`{name}` is not a Windows reserved basename"
            );
        }
    }

    #[test]
    fn validate_module_name_rejects_trailing_spaces_and_dots() {
        for name in ["auth ", "auth.", "auth. ", "Módulo."] {
            assert!(
                validate_module_name(name).is_err(),
                "`{name}` must be rejected because Windows strips trailing spaces/dots"
            );
        }
    }

    #[test]
    fn validate_module_name_enforces_portable_spec_filename_byte_limit() {
        let ascii_boundary = "a".repeat(247);
        let ascii_too_long = "a".repeat(248);
        let multibyte_boundary = format!("{}a", "é".repeat(123));
        let multibyte_too_long = "é".repeat(124);

        assert_eq!(ascii_boundary.len(), 247);
        assert!(validate_module_name(&ascii_boundary).is_ok());
        assert_eq!(ascii_too_long.len(), 248);
        assert!(validate_module_name(&ascii_too_long).is_err());

        assert_eq!(multibyte_boundary.len(), 247);
        assert!(validate_module_name(&multibyte_boundary).is_ok());
        assert_eq!(multibyte_too_long.len(), 248);
        assert!(validate_module_name(&multibyte_too_long).is_err());
    }

    #[test]
    fn rendered_drift_errors_prefer_longest_discovered_spec_path() {
        let colon_path = "specs/team: api/auth.spec.md";
        let prefix_path = "specs/team";
        let rendered = vec![
            format!("{colon_path}: Missing required section: Purpose"),
            format!("{prefix_path}: independent failure"),
        ];
        let discovered_spec_paths = vec![prefix_path.to_string(), colon_path.to_string()];
        let errors = ValidationErrors::from_rendered(&rendered, &discovered_spec_paths);

        assert_eq!(errors.drift_by_spec.len(), 2);
        assert_eq!(
            errors.drift_by_spec.get(colon_path),
            Some(&vec!["Missing required section: Purpose".to_string()])
        );
        assert_eq!(
            errors.drift_by_spec.get(prefix_path),
            Some(&vec!["independent failure".to_string()])
        );
        assert_eq!(errors.as_ref(), rendered);
    }

    #[test]
    fn validation_reports_structured_suppression_details_without_counting_finding() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        let spec_path = root.join("specs/demo/demo.spec.md");
        fs::create_dir_all(spec_path.parent().unwrap()).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/demo.rs"), "pub fn visible() {}\n").unwrap();
        fs::write(
            &spec_path,
            r#"---
module: demo
version: 1
status: stable
files:
  - src/demo.rs
---

# Demo

## Purpose

Demonstrate visible suppression.

## Requirements

- The module remains deterministic.

## Public API

| Export | Description |
|---|---|

## Invariants

1. Validation remains deterministic.

## Behavioral Examples

The module can be checked.

## Error Cases

Invalid input is rejected.

## Dependencies

None.

## Change Log

| Date | Change |
|---|---|
| 2026-07-29 | Initial |
"#,
        )
        .unwrap();
        let mut ignore_rules = IgnoreRules::default();
        ignore_rules
            .global
            .insert(WarningCategory::UndocumentedExport);

        let (_, warning_count, _, _, _, warnings, _, suppressed) = run_validation_with_suppressions(
            root,
            std::slice::from_ref(&spec_path),
            std::slice::from_ref(&spec_path),
            &SpecSyncConfig::default(),
            true,
            false,
            &ignore_rules,
            None,
        );

        assert!(
            warnings
                .iter()
                .all(|warning| !warning.contains("Undocumented export"))
        );
        assert_eq!(warning_count, warnings.len());
        assert!(suppressed.iter().any(|detail| {
            detail["spec"] == "specs/demo/demo.spec.md"
                && detail["category"] == "undocumented-export"
                && detail["source"] == "global"
                && detail["warning"]
                    .as_str()
                    .is_some_and(|warning| warning.contains("Undocumented export"))
        }));
    }

    #[test]
    #[cfg(windows)]
    fn validate_module_name_rejects_windows_drive_relative() {
        // `C:foo` has no separator and `is_absolute()` is false (drive-relative), but its
        // components include a Prefix, so the single-Normal-segment check refuses it.
        assert!(validate_module_name("C:foo").is_err());
        assert!(validate_module_name("C:\\abs").is_err());
    }

    // ── #442 regression: naming rules ──────────────────────────────────

    #[test]
    fn validate_scaffold_module_name_rejects_overlong_names() {
        let long = "a".repeat(200);
        let err = super::validate_scaffold_module_name(&long).unwrap_err();
        assert!(err.contains("1-64"), "{err}");
        let max_ok = "a".repeat(64);
        assert!(super::validate_scaffold_module_name(&max_ok).is_ok());
    }

    #[test]
    fn validate_scaffold_module_name_rejects_reserved_names() {
        for name in ["change", "specs", "con", "CON", "nul", "com1", "Changes"] {
            assert!(
                super::validate_scaffold_module_name(name).is_err(),
                "`{name}` is reserved and must be rejected"
            );
        }
    }

    #[test]
    fn validate_scaffold_module_name_rejects_spaces_leading_dash_emoji() {
        for name in ["my module", "-dash", "🚀rocket", ".hidden", "trailing "] {
            assert!(
                super::validate_scaffold_module_name(name).is_err(),
                "`{name}` must be rejected"
            );
        }
        // Still accepted: documented charset.
        for name in [
            "auth",
            "auth-service",
            "user_profile",
            "a.b",
            "v2",
            "Módulo",
        ] {
            assert!(
                super::validate_scaffold_module_name(name).is_ok(),
                "`{name}` should stay valid"
            );
        }
    }

    #[test]
    fn case_collision_detects_case_fold_duplicates() {
        let tmp = tempfile::tempdir().unwrap();
        let specs_dir = tmp.path().join("specs");
        std::fs::create_dir_all(specs_dir.join("lib")).unwrap();
        assert!(super::check_case_collision(&specs_dir, "Lib").is_err());
        assert!(super::check_case_collision(&specs_dir, "LIB").is_err());
        assert!(super::check_case_collision(&specs_dir, "lib").is_ok()); // same name: normal "already exists" path
        assert!(super::check_case_collision(&specs_dir, "other").is_ok());
    }
}
