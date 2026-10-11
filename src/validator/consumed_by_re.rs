use crate::config::{
    CONFIG_PATH_CANDIDATES, default_schema_pattern, is_detectable_source_file,
    parse_config_content_checked_with_source_dirs, source_detection_ignores_directory,
};
use crate::exports::{has_configured_extension, is_test_file};
use crate::parser::{
    body_has_section, find_section_offset, find_stub_sections, get_duplicate_spec_symbols,
    get_missing_sections, get_near_miss_sections, get_spec_symbols, parse_frontmatter,
};
use crate::schema::{self, SchemaTable};
use crate::types::{
    CoverageReport, CustomRuleType, Frontmatter, RuleSeverity, SpecSyncConfig, ValidationResult,
};
use crate::util::{levenshtein, safe_regex};
use cap_primitives::fs::FollowSymlinks;
use cap_std::ambient_authority;
use cap_std::fs::{Dir, Metadata, OpenOptions};
use regex::Regex;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::ffi::{OsStr, OsString};
use std::fs;
#[cfg(debug_assertions)]
use std::io::Write;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use walkdir::WalkDir;

static CONSUMED_BY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)### Consumed By\s*\n(.*?)(?:\n## |\n### |$)").unwrap());
static FILE_REF_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\|\s*`([^`]+\.\w+)`\s*\|").unwrap());
static NUMBERED_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^\d+\.\s+\S").unwrap());
const STATIC_COVERAGE_EXTENSIONS: &[&str] = &["html", "htm", "css"];
// Keep checked CLI coverage within the same source-input envelope as MCP snapshots.
const MAX_COVERAGE_FILE_BYTES: u64 = 8 * 1024 * 1024;
const MAX_COVERAGE_INPUT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_COVERAGE_ENTRIES: usize = 100_000;
const MAX_COVERAGE_DEPTH: usize = 256;
#[cfg(debug_assertions)]
const COVERAGE_SNAPSHOT_TEST_BARRIER_ENV: &str = "SPECSYNC_TEST_COVERAGE_SNAPSHOT_IDENTITY_BARRIER";
#[cfg(debug_assertions)]
const COVERAGE_SNAPSHOT_TEST_BARRIER_PHASE_ENV: &str =
    "SPECSYNC_TEST_COVERAGE_SNAPSHOT_IDENTITY_BARRIER_PHASE";
#[cfg(debug_assertions)]
const COVERAGE_SNAPSHOT_TEST_CONTEXT_ENV: &str = "SPECSYNC_TEST_CONTEXT";
#[cfg(debug_assertions)]
const COVERAGE_SNAPSHOT_TEST_CONTEXT: &str = "coverage-snapshot-identity";
#[cfg(debug_assertions)]
const COVERAGE_SNAPSHOT_TEST_BARRIER_TIMEOUT: std::time::Duration =
    std::time::Duration::from_secs(30);

/// Check if a dependency reference is a cross-project reference.
/// Cross-project refs use the format `owner/repo@module` (e.g. `corvid-labs/algochat@auth`).
pub fn is_cross_project_ref(dep: &str) -> bool {
    dep.contains('/') && dep.contains('@')
}

/// Parse a cross-project reference into (owner/repo, module).
/// Returns None if not a valid cross-project ref.
pub fn parse_cross_project_ref(dep: &str) -> Option<(&str, &str)> {
    if !is_cross_project_ref(dep) {
        return None;
    }
    let at_pos = dep.find('@')?;
    let repo = &dep[..at_pos];
    let module = &dep[at_pos + 1..];
    if repo.is_empty() || module.is_empty() {
        return None;
    }
    Some((repo, module))
}

// ─── Schema Table Discovery ──────────────────────────────────────────────

/// Extract canonical table names from the ordered schema replay. An explicit
/// schema pattern may supplement replayed names, but cannot restore a table
/// identity retired by DROP TABLE or ALTER TABLE RENAME.
pub fn get_schema_table_names(root: &Path, config: &SpecSyncConfig) -> HashSet<String> {
    // Replay is authoritative; retain the public default-pattern contract as a
    // compile-checked compatibility value without re-scanning CREATE statements.
    debug_assert!(safe_regex(default_schema_pattern()).is_some());

    let schema_dir = match &config.schema_dir {
        Some(d) => root.join(d),
        None => return HashSet::new(),
    };

    let snapshot = match schema::build_schema_snapshot(&schema_dir) {
        Ok(snapshot) => snapshot,
        // The schema could not be replayed. Returning an empty set here would be read by
        // `add_missing_db_table_error` as "these tables do not exist", which is a different
        // claim entirely — see `schema_table_names_available`.
        Err(_) => return HashSet::new(),
    };
    schema_table_names_from_snapshot(&snapshot, config).unwrap_or_default()
}

/// Whether the declared table set is KNOWN, as opposed to merely empty.
///
/// `get_schema_table_names` collapses three outcomes into one empty `HashSet`: a schema that
/// genuinely declares no tables, a schema that failed to replay, and a `schema_pattern` that
/// failed to compile. Only the first justifies telling a user their table is missing.
///
/// Conflating them is how one unparseable migration reports EVERY declared table as absent,
/// including tables created correctly in an unrelated file. A parse failure must degrade to
/// "unknown", never to "not there".
fn schema_table_names_available(root: &Path, config: &SpecSyncConfig) -> bool {
    let Some(schema_dir) = config.schema_dir.as_ref().map(|dir| root.join(dir)) else {
        return false;
    };
    let Ok(snapshot) = schema::build_schema_snapshot(&schema_dir) else {
        return false;
    };
    schema_table_names_from_snapshot(&snapshot, config).is_ok()
}

pub(crate) fn schema_table_names_from_snapshot(
    snapshot: &schema::SchemaSnapshot,
    config: &SpecSyncConfig,
) -> Result<HashSet<String>, String> {
    let mut tables: HashSet<String> = snapshot.tables.keys().cloned().collect();
    if let Some(pattern) = config.schema_pattern.as_deref() {
        let regex = safe_regex(pattern).ok_or_else(|| {
            "Invalid `schema_pattern`: the regex could not be compiled within safety limits"
                .to_string()
        })?;
        if regex.captures_len() < 2 {
            return Err(
                "Invalid `schema_pattern`: it must contain a capture group for the table name"
                    .to_string(),
            );
        }
        tables.extend(
            snapshot
                .pattern_table_names(&regex)
                .map_err(|error| format!("Invalid `schema_pattern` result: {error}"))?,
        );
    }
    Ok(tables)
}

pub(crate) fn schema_config_problems_for_snapshot(
    config: &SpecSyncConfig,
    snapshot: Option<&schema::SchemaSnapshot>,
) -> Vec<String> {
    let Some(pattern) = config.schema_pattern.as_deref() else {
        return Vec::new();
    };

    let mut problems = Vec::new();
    let regex = match safe_regex(pattern) {
        Some(regex) if regex.captures_len() >= 2 => Some(regex),
        Some(_) => {
            problems.push(
                "Invalid `schema_pattern`: it must contain a capture group for the table name"
                    .to_string(),
            );
            None
        }
        None => {
            problems.push(
                "Invalid `schema_pattern`: the regex could not be compiled within safety limits"
                    .to_string(),
            );
            None
        }
    };

    let has_schema_dir = match &config.schema_dir {
        Some(_) => true,
        None => {
            problems.push(
                "`schema_pattern` is configured but `schema_dir` is not configured; DB schema validation cannot scan for tables"
                    .to_string(),
            );
            false
        }
    };

    if has_schema_dir
        && let (Some(regex), Some(snapshot)) = (regex, snapshot)
        && let Err(error) = snapshot.pattern_table_names(&regex)
    {
        problems.push(format!("Invalid `schema_pattern` result: {error}"));
    }

    problems
}

fn schema_table_exists(declaration: &str, schema_tables: &HashSet<String>) -> Result<bool, String> {
    for discovered in schema_tables {
        if schema::table_reference_matches(declaration, discovered)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn resolve_schema_value<'a, Value>(
    declaration: &str,
    values: &'a HashMap<String, Value>,
) -> Result<Option<(&'a str, &'a Value)>, String> {
    let canonical_declaration = schema::canonicalize_table_name(declaration)?;
    let mut exact = Vec::new();
    let mut compatible = Vec::new();

    for (name, value) in values {
        let canonical_name = schema::canonicalize_table_name(name)?;
        if canonical_name == canonical_declaration {
            exact.push((name.as_str(), value));
        } else if schema::table_reference_matches(&canonical_declaration, &canonical_name)? {
            compatible.push((name.as_str(), value));
        }
    }

    let candidates = if exact.is_empty() { compatible } else { exact };
    if candidates.len() > 1 {
        let mut names: Vec<&str> = candidates.iter().map(|(name, _)| *name).collect();
        names.sort_unstable();
        return Err(format!(
            "DB table reference `{declaration}` is ambiguous across schema tables: {}",
            names.join(", ")
        ));
    }

    Ok(candidates.into_iter().next())
}

fn validate_db_table_identifier(table: &str, result: &mut ValidationResult) -> Result<(), String> {
    match schema::canonicalize_table_name(table) {
        Ok(_) => Ok(()),
        Err(error) => {
            result
                .errors
                .push(format!("Invalid DB table identifier `{table}`: {error}"));
            result.fixes.push(format!(
                "Use a bare, quoted, or schema-qualified SQL identifier instead of `{table}`"
            ));
            Err(error)
        }
    }
}

fn add_missing_db_table_error(table: &str, result: &mut ValidationResult) {
    result
        .errors
        .push(format!("DB table not found in schema: {table}"));
    result.fixes.push(format!(
        "Remove `{table}` from db_tables or add a CREATE TABLE migration"
    ));
}

fn add_schema_resolution_error(error: String, result: &mut ValidationResult) {
    if !result.errors.contains(&error) {
        result.errors.push(error);
    }
}

// ─── File Discovery ──────────────────────────────────────────────────────

/// Find all *.spec.md files in a directory recursively.
pub fn find_spec_files(dir: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();
    if !dir.exists() {
        return results;
    }

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file()
            && let Some(name) = path.file_name().and_then(|n| n.to_str())
            && name.ends_with(".spec.md")
        {
            results.push(path.to_path_buf());
        }
    }

    results.sort();
    results
}

/// Load CLI discovery inputs through one retained project-root capability.
///
/// Configuration bytes, manifest-based source autodetection, fallback source
/// scanning, and spec enumeration all remain beneath the same retained root.
pub(crate) fn load_config_and_discover_retained(
    root: &Path,
) -> Result<(SpecSyncConfig, Vec<PathBuf>), String> {
    let project = open_coverage_project_root(root)?;
    let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());
    let config = retained_config(&project, root, &mut budget)?;
    let spec_files = discover_coverage_spec_files(&project, &config.specs_dir, &mut budget)?
        .into_iter()
        .map(|spec| root.join(spec.relative_path))
        .collect();
    verify_coverage_project_root(root, &project)?;
    Ok((config, spec_files))
}

#[derive(Debug)]
struct CoverageSourceFile {
    relative_path: PathBuf,
    loc: usize,
}

#[derive(Clone, Debug)]
struct CoverageSpecFile {
    relative_path: PathBuf,
    identity: CoverageEntryIdentity,
}

#[derive(Debug, Default)]
struct CoverageSourceSnapshot {
    files: Vec<CoverageSourceFile>,
    immediate_modules: HashMap<String, Vec<OsString>>,
}

#[derive(Clone, Copy)]
struct CoverageTraversalLimits {
    max_file_bytes: u64,
    max_input_bytes: u64,
    max_entries: usize,
    max_depth: usize,
}

impl Default for CoverageTraversalLimits {
    fn default() -> Self {
        Self {
            max_file_bytes: MAX_COVERAGE_FILE_BYTES,
            max_input_bytes: MAX_COVERAGE_INPUT_BYTES,
            max_entries: MAX_COVERAGE_ENTRIES,
            max_depth: MAX_COVERAGE_DEPTH,
        }
    }
}

struct CoverageTraversalBudget {
    limits: CoverageTraversalLimits,
    entries: usize,
    bytes: u64,
    charged_files: HashSet<PathBuf>,
    source_files: HashSet<PathBuf>,
    /// Symlinked entries met during discovery, skipped rather than traversed.
    ///
    /// These walks run behind retained directory capabilities and use
    /// `symlink_metadata` precisely so a link can never redirect discovery
    /// outside the retained root, and that must not change. But aborting the
    /// whole command on the first link bought no correctness: a link either
    /// points outside the root, where it must not be followed anyway, or inside
    /// it, where the target is already discovered under its real path and
    /// following it would double-count. Skipping loses nothing (#546).
    ///
    /// Collected rather than discarded because skipping silently shrinks the
    /// coverage denominator — a repo whose `src/vendor` is a link would report a
    /// *higher* percentage after this change, a number that improved because
    /// measurement stopped. Callers surface these next to the coverage line.
    skipped_links: BTreeSet<String>,
}

impl CoverageTraversalBudget {
    fn new(limits: CoverageTraversalLimits) -> Self {
        Self {
            limits,
            entries: 0,
            bytes: 0,
            charged_files: HashSet::new(),
            source_files: HashSet::new(),
            skipped_links: BTreeSet::new(),
        }
    }

    /// Record a symlinked entry that discovery skipped instead of traversing.
    ///
    /// Normalized to forward slashes so the reported path is stable across
    /// platforms, and deduplicated: the same link can be met by both the
    /// source-detection scan and the coverage walk.
    fn record_skipped_link(&mut self, relative: &Path) {
        self.skipped_links
            .insert(relative.to_string_lossy().replace('\\', "/"));
    }

    fn remaining_entries(&self) -> usize {
        self.limits.max_entries.saturating_sub(self.entries)
    }

    fn charge_entries(&mut self, count: usize) -> Result<(), String> {
        self.entries = self.entries.saturating_add(count);
        if self.entries > self.limits.max_entries {
            return Err(format!(
                "Coverage traversal exceeds the {}-entry limit",
                self.limits.max_entries
            ));
        }
        Ok(())
    }

    fn remaining_bytes(&self) -> u64 {
        self.limits.max_input_bytes.saturating_sub(self.bytes)
    }

    fn charge_input_file(&mut self, relative: &Path, bytes: u64) -> Result<(), String> {
        if !self.charged_files.insert(relative.to_path_buf()) {
            return Ok(());
        }
        self.bytes = self.bytes.saturating_add(bytes);
        if self.bytes > self.limits.max_input_bytes {
            return Err(format!(
                "Coverage source inputs exceed the {} cumulative limit",
                display_coverage_byte_limit(self.limits.max_input_bytes)
            ));
        }
        Ok(())
    }

    fn charge_source_file(&mut self, relative: &Path, bytes: u64) -> Result<bool, String> {
        if !self.source_files.insert(relative.to_path_buf()) {
            return Ok(false);
        }
        self.charge_input_file(relative, bytes)?;
        Ok(true)
    }
}

fn retained_source_dirs(
    project: &Dir,
    root: &Path,
    budget: &mut CoverageTraversalBudget,
) -> Result<Vec<String>, String> {
    match crate::manifest::discover_from_manifests_checked_with_root(root, project) {
        Ok(mut discovery) if !discovery.source_dirs.is_empty() => {
            discovery.source_dirs.sort();
            discovery.source_dirs.dedup();
            Ok(discovery.source_dirs)
        }
        Ok(_) => retained_source_dirs_by_scan(project, root, budget),
        // Coverage itself repeats retained manifest discovery and owns the
        // command-specific inconclusive JSON. Preserve legacy omitted-source
        // compatibility for shared CLI discovery by using the bounded retained
        // scan here; the checked coverage pass still reports the manifest error.
        Err(_) => retained_source_dirs_by_scan(project, root, budget),
    }
}

fn retained_source_dirs_by_scan(
    project: &Dir,
    root: &Path,
    budget: &mut CoverageTraversalBudget,
) -> Result<Vec<String>, String> {
    let mut source_dirs = Vec::new();
    let mut has_root_source_files = false;
    let names = read_coverage_entry_names(project, Path::new(""), budget)?;
    for name in names {
        let name_text = name.to_str().ok_or_else(|| {
            "Coverage source-detection path beneath . is not valid UTF-8".to_string()
        })?;
        if source_detection_ignores_directory(name_text) {
            continue;
        }
        let relative = PathBuf::from(&name);
        let metadata = project.symlink_metadata(&name).map_err(|error| {
            format!(
                "Cannot inspect retained source-detection path {}: {error}",
                relative.display()
            )
        })?;
        if coverage_metadata_is_link(&metadata) {
            budget.record_skipped_link(&relative);
            continue;
        }
        if metadata.is_file() {
            if is_detectable_source_file(&root.join(&relative)) {
                has_root_source_files = true;
            }
            continue;
        }
        if !metadata.is_dir() {
            continue;
        }
        let directory = open_coverage_child_directory(project, &name, &relative)?;
        if retained_directory_contains_source(project, root, directory, &relative, 0, budget)? {
            source_dirs.push(name_text.to_string());
        }
    }
    verify_coverage_directory_edge(project, Path::new(""), project)?;
    if has_root_source_files && source_dirs.is_empty() {
        return Ok(vec![".".to_string()]);
    }
    if source_dirs.is_empty() {
        return Ok(vec!["src".to_string()]);
    }
    source_dirs.sort();
    source_dirs.dedup();
    Ok(source_dirs)
}

fn retained_directory_contains_source(
    project: &Dir,
    root: &Path,
    directory: Dir,
    relative: &Path,
    depth: usize,
    budget: &mut CoverageTraversalBudget,
) -> Result<bool, String> {
    let retained = directory.try_clone().map_err(|error| {
        format!(
            "Cannot retain source-detection directory {}: {error}",
            relative.display()
        )
    })?;
    let names = read_coverage_entry_names(&directory, relative, budget)?;
    let mut found = false;
    for name in names {
        let name_text = name.to_str().ok_or_else(|| {
            format!(
                "Coverage source-detection path beneath {} is not valid UTF-8",
                relative.display()
            )
        })?;
        if source_detection_ignores_directory(name_text) {
            continue;
        }
        let child = relative.join(&name);
        ensure_coverage_depth(&child, budget.limits.max_depth)?;
        let metadata = directory.symlink_metadata(&name).map_err(|error| {
            format!(
                "Cannot inspect retained source-detection path {}: {error}",
                child.display()
            )
        })?;
        if coverage_metadata_is_link(&metadata) {
            budget.record_skipped_link(&child);
            continue;
        }
        if metadata.is_file() && is_detectable_source_file(&root.join(&child)) {
            found = true;
            break;
        }
        if metadata.is_dir() && depth < 2 {
            let child_directory = open_coverage_child_directory(&directory, &name, &child)?;
            if retained_directory_contains_source(
                project,
                root,
                child_directory,
                &child,
                depth + 1,
                budget,
            )? {
                found = true;
                break;
            }
        }
    }
    verify_coverage_directory_edge(project, relative, &retained)?;
    Ok(found)
}

fn retained_config(
    project: &Dir,
    root: &Path,
    budget: &mut CoverageTraversalBudget,
) -> Result<SpecSyncConfig, String> {
    retained_config_with_hook(project, root, budget, |_| {})
}

fn retained_config_with_hook<BeforeRead>(
    project: &Dir,
    root: &Path,
    budget: &mut CoverageTraversalBudget,
    mut before_read: BeforeRead,
) -> Result<SpecSyncConfig, String>
where
    BeforeRead: FnMut(&Path),
{
    for configured in CONFIG_PATH_CANDIDATES {
        let relative = Path::new(configured);
        let parent = relative.parent().unwrap_or_else(|| Path::new(""));
        let Some(retained_directory) = open_retained_coverage_directory(project, parent)? else {
            continue;
        };
        let directory = &retained_directory.directory;
        let name = relative.file_name().ok_or_else(|| {
            format!(
                "Configuration path {} has no terminal filename",
                relative.display()
            )
        })?;
        let metadata = match directory.symlink_metadata(name) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(format!(
                    "Cannot inspect retained configuration file {}: {error}",
                    relative.display()
                ));
            }
        };
        if coverage_metadata_is_link(&metadata) || !metadata.is_file() {
            return Err(format!(
                "Configuration file {} must be a regular file beneath the retained project root",
                relative.display()
            ));
        }
        verify_retained_coverage_directory(project, &retained_directory)?;
        before_read(relative);
        let bytes = read_coverage_file(
            directory,
            name,
            relative,
            budget.limits.max_file_bytes,
            budget.remaining_bytes(),
            budget.limits.max_input_bytes,
        )?;
        verify_retained_coverage_directory(project, &retained_directory)?;
        budget.charge_input_file(relative, bytes.len() as u64)?;
        let content = match std::str::from_utf8(&bytes) {
            Ok(content) => content,
            Err(_) => {
                eprintln!(
                    "Warning: config file {} exists but could not be read (not valid UTF-8 or unreadable); \
                     using built-in defaults — its settings (enforcement, required sections, etc.) are NOT applied",
                    root.join(relative).display()
                );
                let source_dirs = retained_source_dirs(project, root, budget)?;
                return Ok(SpecSyncConfig {
                    source_dirs,
                    config_path: Some(root.join(relative)),
                    // The project configured rules that are now NOT in effect.
                    // Warning on stderr and carrying on meant stdout reported
                    // `✓ All required sections present` over a section list that
                    // had been thrown away (#570).
                    load_error: Some(format!(
                        "config file {} exists but could not be loaded; built-in defaults are in use",
                        root.join(relative).display()
                    )),
                    ..SpecSyncConfig::default()
                });
            }
        };
        let parsed = parse_config_content_checked_with_source_dirs(
            &root.join(relative),
            content,
            root,
            Some(Vec::new()),
        );
        let mut config = match parsed {
            Ok(config) => config,
            Err(error) => {
                eprintln!(
                    "Warning: failed to parse config file {}: {error}; using built-in defaults",
                    root.join(relative).display()
                );
                let source_dirs = retained_source_dirs(project, root, budget)?;
                return Ok(SpecSyncConfig {
                    source_dirs,
                    config_path: Some(root.join(relative)),
                    // The project configured rules that are now NOT in effect.
                    // Warning on stderr and carrying on meant stdout reported
                    // `✓ All required sections present` over a section list that
                    // had been thrown away (#570).
                    load_error: Some(format!(
                        "config file {} exists but could not be loaded; built-in defaults are in use",
                        root.join(relative).display()
                    )),
                    ..SpecSyncConfig::default()
                });
            }
        };
        // One predicate decides both the fallback and the record of it. If the
        // flag were left to the parser while the override were decided here,
        // the two could disagree and coverage would treat a SCANNED list as a
        // stated one — degrading a manifest error over a guess, which is the
        // one thing the flag exists to prevent (#723).
        config.source_dirs_set = retained_config_has_source_dirs(relative, content)?;
        if !config.source_dirs_set {
            config.source_dirs = retained_source_dirs(project, root, budget)?;
        }
        return Ok(config);
    }

    let source_dirs = retained_source_dirs(project, root, budget)?;
    Ok(SpecSyncConfig {
        source_dirs,
        ..SpecSyncConfig::default()
    })
}

fn retained_config_has_source_dirs(relative: &Path, content: &str) -> Result<bool, String> {
    let content = content.trim_start_matches('\u{feff}');
    if relative
        .extension()
        .and_then(|extension| extension.to_str())
        == Some("toml")
    {
        let table = toml::from_str::<toml::Table>(content).map_err(|error| error.to_string())?;
        return Ok(table.contains_key("source_dirs"));
    }
    let value =
        serde_json::from_str::<serde_json::Value>(content).map_err(|error| error.to_string())?;
    let object = value
        .as_object()
        .ok_or_else(|| "configuration root must be an object".to_string())?;
    Ok(object.contains_key("sourceDirs"))
}

fn discover_coverage_spec_files(
    project: &Dir,
    configured: &str,
    budget: &mut CoverageTraversalBudget,
) -> Result<Vec<CoverageSpecFile>, String> {
    discover_coverage_spec_files_with_hook(project, configured, budget, |_| {})
}

fn discover_coverage_spec_files_with_hook<AfterEnumeration>(
    project: &Dir,
    configured: &str,
    budget: &mut CoverageTraversalBudget,
    mut after_enumeration: AfterEnumeration,
) -> Result<Vec<CoverageSpecFile>, String>
where
    AfterEnumeration: FnMut(&Path),
{
    let relative = if configured == "." {
        PathBuf::new()
    } else {
        PathBuf::from(normalize_source_mapping(configured).ok_or_else(|| {
            format!("Coverage specs directory must remain beneath the project root: {configured}")
        })?)
    };
    ensure_coverage_depth(&relative, budget.limits.max_depth)?;
    let Some(retained_directory) = open_retained_coverage_directory(project, &relative)? else {
        return Ok(Vec::new());
    };
    let mut specs = Vec::new();
    discover_coverage_spec_directory(
        &retained_directory.directory,
        &relative,
        budget,
        &mut after_enumeration,
        &mut specs,
    )?;
    verify_retained_coverage_directory(project, &retained_directory)?;
    specs.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    specs.dedup_by(|left, right| left.relative_path == right.relative_path);
    Ok(specs)
}

fn discover_coverage_spec_directory<AfterEnumeration>(
    directory: &Dir,
    relative: &Path,
    budget: &mut CoverageTraversalBudget,
    after_enumeration: &mut AfterEnumeration,
    specs: &mut Vec<CoverageSpecFile>,
) -> Result<(), String>
where
    AfterEnumeration: FnMut(&Path),
{
    let names = read_coverage_entry_names(directory, relative, budget)?;
    let mut children = Vec::new();
    for name in names {
        let name_text = name.to_str().ok_or_else(|| {
            format!(
                "Coverage spec path beneath {} is not valid UTF-8",
                display_coverage_path(relative)
            )
        })?;
        let child = relative.join(&name);
        ensure_coverage_depth(&child, budget.limits.max_depth)?;
        let metadata = directory.symlink_metadata(&name).map_err(|error| {
            format!(
                "Cannot inspect retained coverage spec path {}: {error}",
                child.display()
            )
        })?;
        if coverage_metadata_is_link(&metadata) {
            return Err(format!(
                "Coverage spec path {} must not traverse a symlink or reparse point",
                child.display()
            ));
        }
        if metadata.is_dir() {
            let identity = coverage_metadata_identity(&metadata).map_err(|error| {
                format!(
                    "Cannot identify retained coverage spec directory {}: {error}",
                    child.display()
                )
            })?;
            children.push(DiscoveredCoverageDirectory {
                name,
                relative: child,
                identity,
            });
            continue;
        }
        if !metadata.is_file() {
            return Err(format!(
                "Coverage spec path {} must be a regular file or directory",
                child.display()
            ));
        }
        if name_text.ends_with(".spec.md") && !name_text.starts_with('_') {
            let identity = coverage_metadata_identity(&metadata).map_err(|error| {
                format!(
                    "Cannot identify retained coverage spec file {}: {error}",
                    child.display()
                )
            })?;
            specs.push(CoverageSpecFile {
                relative_path: child,
                identity,
            });
        }
    }
    after_enumeration(relative);
    for child in children {
        let child_directory = reopen_discovered_coverage_directory(directory, &child)?;
        discover_coverage_spec_directory(
            &child_directory,
            &child.relative,
            budget,
            after_enumeration,
            specs,
        )?;
        verify_coverage_child_directory(directory, &child.name, &child.relative, &child_directory)?;
    }
    Ok(())
}

struct DiscoveredCoverageDirectory {
    name: OsString,
    relative: PathBuf,
    identity: CoverageEntryIdentity,
}

struct SelectedCoverageSourceDirectory {
    configured: String,
    normalized: PathBuf,
    identity: CoverageEntryIdentity,
}

#[cfg(test)]
fn snapshot_coverage_sources(
    project: &Dir,
    root: &Path,
    config: &SpecSyncConfig,
    exclude_dirs: &HashSet<String>,
    budget: &mut CoverageTraversalBudget,
) -> Result<CoverageSourceSnapshot, String> {
    snapshot_coverage_sources_with_hook(project, root, config, exclude_dirs, budget, |_| {})
}

#[cfg(test)]
fn snapshot_coverage_sources_with_hook<AfterEnumeration>(
    project: &Dir,
    root: &Path,
    config: &SpecSyncConfig,
    exclude_dirs: &HashSet<String>,
    budget: &mut CoverageTraversalBudget,
    mut after_enumeration: AfterEnumeration,
) -> Result<CoverageSourceSnapshot, String>
where
    AfterEnumeration: FnMut(&Path),
{
    let selected_sources =
        select_coverage_source_directories(project, config, budget.limits.max_depth)?;
    snapshot_selected_coverage_sources_with_hook(
        project,
        root,
        config,
        exclude_dirs,
        budget,
        selected_sources,
        &mut after_enumeration,
    )
}

fn select_coverage_source_directories(
    project: &Dir,
    config: &SpecSyncConfig,
    max_depth: usize,
) -> Result<Vec<SelectedCoverageSourceDirectory>, String> {
    let mut selected_sources = Vec::new();
    for configured in &config.source_dirs {
        let normalized = if configured == "." {
            PathBuf::new()
        } else {
            PathBuf::from(normalize_source_mapping(configured).ok_or_else(|| {
                format!(
                    "Coverage source directory must remain beneath the project root: {configured}"
                )
            })?)
        };
        ensure_coverage_depth(&normalized, max_depth)?;
        let Some(retained) = open_retained_coverage_directory(project, &normalized)? else {
            continue;
        };
        let identity = coverage_directory_identity(&retained.directory).map_err(|error| {
            format!(
                "Cannot identify selected coverage source directory {}: {error}",
                display_coverage_path(&normalized)
            )
        })?;
        verify_retained_coverage_directory(project, &retained)?;
        selected_sources.push(SelectedCoverageSourceDirectory {
            configured: configured.clone(),
            normalized,
            identity,
        });
    }
    Ok(selected_sources)
}

fn snapshot_selected_coverage_sources_with_hook<AfterEnumeration>(
    project: &Dir,
    root: &Path,
    config: &SpecSyncConfig,
    exclude_dirs: &HashSet<String>,
    budget: &mut CoverageTraversalBudget,
    selected_sources: Vec<SelectedCoverageSourceDirectory>,
    after_enumeration: &mut AfterEnumeration,
) -> Result<CoverageSourceSnapshot, String>
where
    AfterEnumeration: FnMut(&Path),
{
    let mut snapshot = CoverageSourceSnapshot::default();
    for source in selected_sources {
        let Some(retained) = open_retained_coverage_directory(project, &source.normalized)? else {
            return Err(format!(
                "Selected coverage source directory {} changed during retained traversal",
                display_coverage_path(&source.normalized)
            ));
        };
        let observed_identity =
            coverage_directory_identity(&retained.directory).map_err(|error| {
                format!(
                    "Cannot identify reopened coverage source directory {}: {error}",
                    display_coverage_path(&source.normalized)
                )
            })?;
        if observed_identity != source.identity {
            return Err(format!(
                "Selected coverage source directory {} changed during retained traversal",
                display_coverage_path(&source.normalized)
            ));
        }
        let mut immediate_modules = Vec::new();
        let mut accumulator = CoverageSourceAccumulator {
            exclude_dirs,
            config,
            root,
            immediate_modules: &mut immediate_modules,
            files: &mut snapshot.files,
            budget,
        };
        snapshot_coverage_directory_with_hook(
            &retained.directory,
            &source.normalized,
            &mut accumulator,
            after_enumeration,
        )?;
        verify_retained_coverage_directory(project, &retained)?;
        immediate_modules.sort();
        immediate_modules.dedup();
        snapshot
            .immediate_modules
            .insert(source.configured, immediate_modules);
    }
    snapshot
        .files
        .sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    snapshot
        .files
        .dedup_by(|left, right| left.relative_path == right.relative_path);
    Ok(snapshot)
}

#[derive(Clone, Copy)]
enum CoverageSnapshotCheckpoint {
    RootRetained,
    ManifestDiscovered,
}

#[cfg(debug_assertions)]
impl CoverageSnapshotCheckpoint {
    fn marker(self) -> &'static str {
        match self {
            Self::RootRetained => "root-retained",
            Self::ManifestDiscovered => "manifest-discovered",
        }
    }
}

#[cfg(debug_assertions)]
fn coverage_snapshot_test_barrier(checkpoint: CoverageSnapshotCheckpoint) -> Result<(), String> {
    let Some(directory) = std::env::var_os(COVERAGE_SNAPSHOT_TEST_BARRIER_ENV) else {
        return Ok(());
    };
    if std::env::var(COVERAGE_SNAPSHOT_TEST_CONTEXT_ENV).as_deref()
        != Ok(COVERAGE_SNAPSHOT_TEST_CONTEXT)
    {
        return Err(format!(
            "Coverage snapshot test barrier requires {COVERAGE_SNAPSHOT_TEST_CONTEXT_ENV}={COVERAGE_SNAPSHOT_TEST_CONTEXT}"
        ));
    }
    let selected_checkpoint = std::env::var(COVERAGE_SNAPSHOT_TEST_BARRIER_PHASE_ENV)
        .unwrap_or_else(|_| {
            CoverageSnapshotCheckpoint::RootRetained
                .marker()
                .to_string()
        });
    if selected_checkpoint != checkpoint.marker() {
        return Ok(());
    }
    let directory = PathBuf::from(directory);
    let marker = checkpoint.marker();
    let mut ready = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(directory.join(marker))
        .map_err(|error| format!("Cannot create coverage snapshot test barrier: {error}"))?;
    ready
        .write_all(format!("{marker}\n").as_bytes())
        .and_then(|_| ready.sync_all())
        .map_err(|error| format!("Cannot publish coverage snapshot test barrier: {error}"))?;
    drop(ready);
    let resume = directory.join("resume");
    let started = std::time::Instant::now();
    loop {
        match fs::symlink_metadata(&resume) {
            Ok(metadata) if metadata.is_file() => return Ok(()),
            Ok(_) => {
                return Err(
                    "Coverage snapshot test barrier resume marker is not a file".to_string()
                );
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "Cannot inspect coverage snapshot test barrier resume marker: {error}"
                ));
            }
        }
        if started.elapsed() >= COVERAGE_SNAPSHOT_TEST_BARRIER_TIMEOUT {
            return Err("Timed out waiting for coverage snapshot test barrier".to_string());
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[cfg(not(debug_assertions))]
fn coverage_snapshot_test_barrier(_checkpoint: CoverageSnapshotCheckpoint) -> Result<(), String> {
    Ok(())
}

struct CoverageSourceAccumulator<'a> {
    exclude_dirs: &'a HashSet<String>,
    config: &'a SpecSyncConfig,
    root: &'a Path,
    immediate_modules: &'a mut Vec<OsString>,
    files: &'a mut Vec<CoverageSourceFile>,
    budget: &'a mut CoverageTraversalBudget,
}

fn snapshot_coverage_directory_with_hook<AfterEnumeration>(
    directory: &Dir,
    relative: &Path,
    accumulator: &mut CoverageSourceAccumulator<'_>,
    after_enumeration: &mut AfterEnumeration,
) -> Result<(), String>
where
    AfterEnumeration: FnMut(&Path),
{
    snapshot_coverage_directory(directory, relative, true, accumulator, after_enumeration)
}

fn snapshot_coverage_directory<AfterEnumeration>(
    directory: &Dir,
    relative: &Path,
    source_root: bool,
    accumulator: &mut CoverageSourceAccumulator<'_>,
    after_enumeration: &mut AfterEnumeration,
) -> Result<(), String>
where
    AfterEnumeration: FnMut(&Path),
{
    let names = read_coverage_entry_names(directory, relative, accumulator.budget)?;
    let mut children = Vec::new();
    for name in names {
        if coverage_name_is_excluded(&name, accumulator.exclude_dirs) {
            continue;
        }
        name.to_str().ok_or_else(|| {
            format!(
                "Coverage source path beneath {} is not valid UTF-8",
                display_coverage_path(relative)
            )
        })?;
        let child = relative.join(&name);
        ensure_coverage_depth(&child, accumulator.budget.limits.max_depth)?;
        let metadata = directory.symlink_metadata(&name).map_err(|error| {
            format!(
                "Cannot inspect retained coverage source {}: {error}",
                child.display()
            )
        })?;
        if coverage_metadata_is_link(&metadata) {
            accumulator.budget.record_skipped_link(&child);
            continue;
        }
        if metadata.is_dir() {
            if source_root {
                accumulator.immediate_modules.push(name.clone());
            }
            let identity = coverage_metadata_identity(&metadata).map_err(|error| {
                format!(
                    "Cannot identify retained coverage source directory {}: {error}",
                    child.display()
                )
            })?;
            children.push(DiscoveredCoverageDirectory {
                name,
                relative: child,
                identity,
            });
            continue;
        }
        if !metadata.is_file() {
            return Err(format!(
                "Coverage source {} must be a regular file or directory",
                child.display()
            ));
        }
        let logical_path = accumulator.root.join(&child);
        if !has_coverage_extension(&logical_path, accumulator.config)
            || is_test_file(&logical_path, accumulator.root)
        {
            continue;
        }
        if accumulator.budget.source_files.contains(&child) {
            continue;
        }
        if metadata.len() > accumulator.budget.limits.max_file_bytes {
            return Err(format!(
                "Coverage source file {} exceeds the {} per-file limit",
                child.display(),
                display_coverage_byte_limit(accumulator.budget.limits.max_file_bytes)
            ));
        }
        if metadata.len() > accumulator.budget.remaining_bytes() {
            return Err(format!(
                "Coverage source inputs exceed the {} cumulative limit",
                display_coverage_byte_limit(accumulator.budget.limits.max_input_bytes)
            ));
        }
        let bytes = read_coverage_file(
            directory,
            &name,
            &child,
            accumulator.budget.limits.max_file_bytes,
            accumulator.budget.remaining_bytes(),
            accumulator.budget.limits.max_input_bytes,
        )?;
        let content = std::str::from_utf8(&bytes).map_err(|_| {
            format!(
                "Coverage source file {} is not valid UTF-8",
                child.display()
            )
        })?;
        if accumulator
            .budget
            .charge_source_file(&child, bytes.len() as u64)?
        {
            accumulator.files.push(CoverageSourceFile {
                relative_path: child,
                loc: content.lines().count(),
            });
        }
    }
    after_enumeration(relative);
    for child in children {
        let child_directory = reopen_discovered_coverage_directory(directory, &child)?;
        snapshot_coverage_directory(
            &child_directory,
            &child.relative,
            false,
            accumulator,
            after_enumeration,
        )?;
        verify_coverage_child_directory(directory, &child.name, &child.relative, &child_directory)?;
    }
    Ok(())
}

fn read_coverage_entry_names(
    directory: &Dir,
    relative: &Path,
    budget: &mut CoverageTraversalBudget,
) -> Result<Vec<OsString>, String> {
    let entries = directory.read_dir(".").map_err(|error| {
        format!(
            "Cannot read retained coverage source directory {}: {error}",
            display_coverage_path(relative)
        )
    })?;
    let mut names = Vec::new();
    let remaining = budget.remaining_entries();
    for entry in entries {
        let entry = entry.map_err(|error| {
            format!(
                "Cannot inspect retained coverage source beneath {}: {error}",
                display_coverage_path(relative)
            )
        })?;
        names.push(entry.file_name());
        if names.len() > remaining {
            return Err(format!(
                "Coverage traversal exceeds the {}-entry limit",
                budget.limits.max_entries
            ));
        }
    }
    names.sort();
    budget.charge_entries(names.len())?;
    Ok(names)
}

fn coverage_name_is_excluded(name: &OsStr, exclude_dirs: &HashSet<String>) -> bool {
    exclude_dirs
        .iter()
        .any(|excluded| name == OsStr::new(excluded))
}

fn ensure_coverage_depth(path: &Path, max_depth: usize) -> Result<(), String> {
    let depth = path
        .components()
        .filter(|component| matches!(component, std::path::Component::Normal(_)))
        .count();
    if depth > max_depth {
        return Err(format!(
            "Coverage source path {} exceeds the {max_depth}-component depth limit",
            path.display()
        ));
    }
    Ok(())
}

fn display_coverage_byte_limit(bytes: u64) -> String {
    if bytes.is_multiple_of(1024 * 1024) {
        format!("{} MiB", bytes / (1024 * 1024))
    } else if bytes == 1 {
        "1 byte".to_string()
    } else {
        format!("{bytes} bytes")
    }
}

struct RetainedCoverageDirectory {
    root: Dir,
    directory: Dir,
    edges: Vec<RetainedCoverageDirectoryEdge>,
}

struct RetainedCoverageDirectoryEdge {
    parent: Dir,
    name: OsString,
    relative: PathBuf,
    child: Dir,
}

fn open_retained_coverage_directory(
    root: &Dir,
    relative: &Path,
) -> Result<Option<RetainedCoverageDirectory>, String> {
    let retained_root = root
        .try_clone()
        .map_err(|error| format!("Cannot retain coverage project root: {error}"))?;
    if relative.as_os_str().is_empty() {
        let directory = root
            .try_clone()
            .map_err(|error| format!("Cannot retain coverage project root: {error}"))?;
        return Ok(Some(RetainedCoverageDirectory {
            root: retained_root,
            directory,
            edges: Vec::new(),
        }));
    }
    let mut directory = root
        .try_clone()
        .map_err(|error| format!("Cannot retain coverage project root: {error}"))?;
    let mut edges = Vec::new();
    let mut inspected = PathBuf::new();
    for component in relative.components() {
        let std::path::Component::Normal(name) = component else {
            return Err(format!(
                "Coverage source directory must remain beneath the project root: {}",
                relative.display()
            ));
        };
        inspected.push(name);
        let before = match directory.symlink_metadata(name) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(format!(
                    "Cannot inspect coverage source directory {}: {error}",
                    inspected.display()
                ));
            }
        };
        // Deliberately still fatal, unlike the discovery walks (#546). This
        // opens a path the project *configured* — a `source_dirs` entry — not
        // one discovery happened to meet. Skipping an incidentally-encountered
        // link loses nothing; skipping a configured source directory silently
        // drops everything the author asked to be measured, which is the
        // failure this whole change exists to prevent.
        if coverage_metadata_is_link(&before) {
            return Err(format!(
                "Coverage source directory {} must not traverse a symlink or reparse point",
                inspected.display()
            ));
        }
        if !before.is_dir() {
            if !before.is_file() {
                return Err(format!(
                    "Coverage source directory {} must be a regular directory",
                    inspected.display()
                ));
            }
            return Ok(None);
        }
        let child = open_coverage_child_directory(&directory, name, &inspected)?;
        edges.push(RetainedCoverageDirectoryEdge {
            parent: directory.try_clone().map_err(|error| {
                format!(
                    "Cannot retain coverage source directory {}: {error}",
                    display_coverage_path(inspected.parent().unwrap_or_else(|| Path::new("")))
                )
            })?,
            name: name.to_os_string(),
            relative: inspected.clone(),
            child: child.try_clone().map_err(|error| {
                format!(
                    "Cannot retain coverage source directory {}: {error}",
                    inspected.display()
                )
            })?,
        });
        directory = child;
    }
    Ok(Some(RetainedCoverageDirectory {
        root: retained_root,
        directory,
        edges,
    }))
}

fn verify_retained_coverage_directory(
    project: &Dir,
    retained: &RetainedCoverageDirectory,
) -> Result<(), String> {
    verify_coverage_directory_edge(project, Path::new(""), &retained.root)?;
    for edge in &retained.edges {
        verify_coverage_child_directory(&edge.parent, &edge.name, &edge.relative, &edge.child)?;
    }
    Ok(())
}

fn open_coverage_directory(root: &Dir, relative: &Path) -> Result<Option<Dir>, String> {
    open_retained_coverage_directory(root, relative)
        .map(|retained| retained.map(|retained| retained.directory))
}

fn open_coverage_child_directory(
    parent: &Dir,
    name: &std::ffi::OsStr,
    display: &Path,
) -> Result<Dir, String> {
    let child = parent.open_dir(name).map_err(|error| {
        format!(
            "Cannot open retained coverage source directory {}: {error}",
            display.display()
        )
    })?;
    verify_coverage_child_directory(parent, name, display, &child)?;
    Ok(child)
}

fn reopen_discovered_coverage_directory(
    parent: &Dir,
    discovered: &DiscoveredCoverageDirectory,
) -> Result<Dir, String> {
    let metadata = parent.symlink_metadata(&discovered.name).map_err(|error| {
        format!(
            "Cannot re-inspect coverage source directory {}: {error}",
            discovered.relative.display()
        )
    })?;
    if coverage_metadata_is_link(&metadata) || !metadata.is_dir() {
        return Err(format!(
            "Coverage source directory {} changed during retained traversal",
            discovered.relative.display()
        ));
    }
    let identity = coverage_metadata_identity(&metadata).map_err(|error| {
        format!(
            "Cannot identify re-inspected coverage source directory {}: {error}",
            discovered.relative.display()
        )
    })?;
    if identity != discovered.identity {
        return Err(format!(
            "Coverage source directory {} changed during retained traversal",
            discovered.relative.display()
        ));
    }
    open_coverage_child_directory(parent, &discovered.name, &discovered.relative)
}

fn verify_coverage_directory_edge(
    project: &Dir,
    relative: &Path,
    expected: &Dir,
) -> Result<(), String> {
    if relative.as_os_str().is_empty() {
        let observed = project
            .try_clone()
            .map_err(|error| format!("Cannot re-open coverage project root: {error}"))?;
        let observed_identity = coverage_directory_identity(&observed)
            .map_err(|error| format!("Cannot identify re-opened coverage project root: {error}"))?;
        let expected_identity = coverage_directory_identity(expected)
            .map_err(|error| format!("Cannot identify retained coverage project root: {error}"))?;
        if observed_identity != expected_identity {
            return Err("Coverage project root changed during retained traversal".to_string());
        }
        return Ok(());
    }
    let parent = relative.parent().unwrap_or_else(|| Path::new(""));
    let name = relative
        .file_name()
        .ok_or_else(|| "Coverage source directory has no terminal component".to_string())?;
    let Some(parent_directory) = open_coverage_directory(project, parent)? else {
        return Err(format!(
            "Coverage source directory {} changed during retained traversal",
            relative.display()
        ));
    };
    verify_coverage_child_directory(&parent_directory, name, relative, expected)
}

fn verify_coverage_child_directory(
    parent: &Dir,
    name: &std::ffi::OsStr,
    display: &Path,
    expected: &Dir,
) -> Result<(), String> {
    let metadata = parent.symlink_metadata(name).map_err(|error| {
        format!(
            "Cannot re-inspect coverage source directory {}: {error}",
            display.display()
        )
    })?;
    if coverage_metadata_is_link(&metadata) || !metadata.is_dir() {
        return Err(format!(
            "Coverage source directory {} changed during retained traversal",
            display.display()
        ));
    }
    let observed = parent.open_dir(name).map_err(|error| {
        format!(
            "Cannot re-open coverage source directory {}: {error}",
            display.display()
        )
    })?;
    let observed_identity = coverage_directory_identity(&observed).map_err(|error| {
        format!(
            "Cannot identify re-opened coverage source directory {}: {error}",
            display.display()
        )
    })?;
    let expected_identity = coverage_directory_identity(expected).map_err(|error| {
        format!(
            "Cannot identify retained coverage source directory {}: {error}",
            display.display()
        )
    })?;
    if observed_identity != expected_identity {
        return Err(format!(
            "Coverage source directory {} changed during retained traversal",
            display.display()
        ));
    }
    Ok(())
}

fn read_coverage_file(
    directory: &Dir,
    name: &OsStr,
    display: &Path,
    max_file_bytes: u64,
    remaining_input_bytes: u64,
    max_input_bytes: u64,
) -> Result<Vec<u8>, String> {
    read_coverage_file_with_hook(
        directory,
        name,
        display,
        max_file_bytes,
        remaining_input_bytes,
        max_input_bytes,
        || {},
    )
}

fn read_coverage_file_with_hook<BeforeOpen>(
    directory: &Dir,
    name: &OsStr,
    display: &Path,
    max_file_bytes: u64,
    remaining_input_bytes: u64,
    max_input_bytes: u64,
    before_open: BeforeOpen,
) -> Result<Vec<u8>, String>
where
    BeforeOpen: FnOnce(),
{
    read_coverage_file_with_expected_identity_and_hook(
        directory,
        name,
        display,
        max_file_bytes,
        remaining_input_bytes,
        max_input_bytes,
        None,
        before_open,
    )
}

#[allow(clippy::too_many_arguments)]
fn read_coverage_file_with_expected_identity_and_hook<BeforeOpen>(
    directory: &Dir,
    name: &OsStr,
    display: &Path,
    max_file_bytes: u64,
    remaining_input_bytes: u64,
    max_input_bytes: u64,
    expected_identity: Option<CoverageEntryIdentity>,
    before_open: BeforeOpen,
) -> Result<Vec<u8>, String>
where
    BeforeOpen: FnOnce(),
{
    let before = directory.symlink_metadata(name).map_err(|error| {
        format!(
            "Cannot inspect retained coverage source file {} before open: {error}",
            display.display()
        )
    })?;
    if coverage_metadata_is_link(&before) || !before.is_file() {
        return Err(format!(
            "Coverage source file {} must be a regular non-link file",
            display.display()
        ));
    }
    if before.len() > max_file_bytes {
        return Err(format!(
            "Coverage source file {} exceeds the {} per-file limit",
            display.display(),
            display_coverage_byte_limit(max_file_bytes)
        ));
    }
    if before.len() > remaining_input_bytes {
        return Err(format!(
            "Coverage source inputs exceed the {} cumulative limit",
            display_coverage_byte_limit(max_input_bytes)
        ));
    }
    let before_identity = coverage_metadata_identity(&before).map_err(|error| {
        format!(
            "Cannot identify retained coverage source file {} before open: {error}",
            display.display()
        )
    })?;
    if expected_identity.is_some_and(|expected| expected != before_identity) {
        return Err(format!(
            "Coverage source file {} changed after retained inventory",
            display.display()
        ));
    }
    before_open();
    let mut options = OpenOptions::new();
    options
        .read(true)
        ._cap_fs_ext_nonblock(true)
        ._cap_fs_ext_follow(FollowSymlinks::No);
    let mut file = directory.open_with(name, &options).map_err(|error| {
        format!(
            "Cannot open retained coverage source file {}: {error}",
            display.display()
        )
    })?;
    let identity = coverage_file_identity(&file).map_err(|error| {
        format!(
            "Cannot identify retained coverage source file {}: {error}",
            display.display()
        )
    })?;
    if identity != before_identity {
        return Err(format!(
            "Coverage source file {} changed during retained open",
            display.display()
        ));
    }
    verify_coverage_file_edge(directory, name, display, identity)?;
    let mut bytes = Vec::new();
    let read_limit = max_file_bytes.min(remaining_input_bytes);
    Read::by_ref(&mut file)
        .take(read_limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| {
            format!(
                "Cannot read retained coverage source file {}: {error}",
                display.display()
            )
        })?;
    if bytes.len() as u64 > max_file_bytes {
        return Err(format!(
            "Coverage source file {} exceeds the {} per-file limit",
            display.display(),
            display_coverage_byte_limit(max_file_bytes)
        ));
    }
    if bytes.len() as u64 > remaining_input_bytes {
        return Err(format!(
            "Coverage source inputs exceed the {} cumulative limit",
            display_coverage_byte_limit(max_input_bytes)
        ));
    }
    if coverage_file_identity(&file).ok() != Some(identity) {
        return Err(format!(
            "Coverage source file {} changed while it was being read",
            display.display()
        ));
    }
    verify_coverage_file_edge(directory, name, display, identity)?;
    Ok(bytes)
}

fn verify_coverage_file_edge(
    directory: &Dir,
    name: &std::ffi::OsStr,
    display: &Path,
    expected: CoverageEntryIdentity,
) -> Result<(), String> {
    let metadata = directory.symlink_metadata(name).map_err(|error| {
        format!(
            "Cannot re-inspect coverage source file {}: {error}",
            display.display()
        )
    })?;
    if coverage_metadata_is_link(&metadata) || !metadata.is_file() {
        return Err(format!(
            "Coverage source file {} changed during retained traversal",
            display.display()
        ));
    }
    let mut options = OpenOptions::new();
    options
        .read(true)
        ._cap_fs_ext_nonblock(true)
        ._cap_fs_ext_follow(FollowSymlinks::No);
    let observed = directory.open_with(name, &options).map_err(|error| {
        format!(
            "Cannot re-open coverage source file {}: {error}",
            display.display()
        )
    })?;
    if coverage_file_identity(&observed).ok() != Some(expected) {
        return Err(format!(
            "Coverage source file {} changed during retained traversal",
            display.display()
        ));
    }
    Ok(())
}

fn display_coverage_path(path: &Path) -> String {
    if path.as_os_str().is_empty() {
        ".".to_string()
    } else {
        path.display().to_string()
    }
}

fn coverage_relative_path_text(path: &Path) -> Result<String, String> {
    let mut components = Vec::new();
    for component in path.components() {
        let std::path::Component::Normal(name) = component else {
            return Err(format!(
                "Coverage source path must remain relative to the project root: {}",
                path.display()
            ));
        };
        let name = name
            .to_str()
            .ok_or_else(|| format!("Coverage source path {} is not valid UTF-8", path.display()))?;
        components.push(name);
    }
    Ok(components.join("/"))
}

fn open_coverage_project_root(root: &Path) -> Result<Dir, String> {
    Dir::open_ambient_dir(root, ambient_authority()).map_err(|error| {
        format!(
            "Cannot open coverage project root {} as a retained directory: {error}",
            root.display()
        )
    })
}

fn verify_coverage_project_root(root: &Path, retained: &Dir) -> Result<(), String> {
    let expected = coverage_directory_identity(retained)
        .map_err(|error| format!("Cannot identify retained coverage project root: {error}"))?;
    let observed = Dir::open_ambient_dir(root, ambient_authority()).map_err(|error| {
        format!(
            "Coverage project root {} changed during retained traversal: {error}",
            root.display()
        )
    })?;
    let observed = coverage_directory_identity(&observed)
        .map_err(|error| format!("Cannot identify re-opened coverage project root: {error}"))?;
    if expected != observed {
        return Err(format!(
            "Coverage project root {} changed during retained traversal",
            root.display()
        ));
    }
    Ok(())
}

#[cfg(unix)]
type CoverageEntryIdentity = (u64, u64);

#[cfg(windows)]
type CoverageEntryIdentity = (u32, u64);

#[cfg(not(any(unix, windows)))]
type CoverageEntryIdentity = (u64, Option<std::time::SystemTime>);

#[cfg(unix)]
fn coverage_directory_identity(directory: &Dir) -> io::Result<CoverageEntryIdentity> {
    use cap_std::fs::MetadataExt;

    let metadata = directory.dir_metadata()?;
    Ok((metadata.dev(), metadata.ino()))
}

#[cfg(windows)]
fn coverage_directory_identity(directory: &Dir) -> io::Result<CoverageEntryIdentity> {
    use std::os::windows::io::AsRawHandle;

    let file = directory.try_clone()?.into_std_file();
    coverage_windows_handle_identity(file.as_raw_handle().cast())
}

#[cfg(not(any(unix, windows)))]
fn coverage_directory_identity(directory: &Dir) -> io::Result<CoverageEntryIdentity> {
    let metadata = directory.dir_metadata()?;
    Ok((metadata.len(), metadata.modified().ok()))
}

#[cfg(unix)]
fn coverage_file_identity(file: &cap_std::fs::File) -> io::Result<CoverageEntryIdentity> {
    use cap_std::fs::MetadataExt;

    let metadata = file.metadata()?;
    Ok((metadata.dev(), metadata.ino()))
}

#[cfg(unix)]
fn coverage_metadata_identity(
    metadata: &cap_std::fs::Metadata,
) -> io::Result<CoverageEntryIdentity> {
    use cap_std::fs::MetadataExt;

    Ok((metadata.dev(), metadata.ino()))
}

#[cfg(windows)]
fn coverage_file_identity(file: &cap_std::fs::File) -> io::Result<CoverageEntryIdentity> {
    use std::os::windows::io::AsRawHandle;

    coverage_windows_handle_identity(file.as_raw_handle().cast())
}

#[cfg(windows)]
fn coverage_metadata_identity(
    metadata: &cap_std::fs::Metadata,
) -> io::Result<CoverageEntryIdentity> {
    use cap_primitives::fs::_WindowsByHandle;

    let volume = metadata
        .volume_serial_number()
        .ok_or_else(|| io::Error::other("Windows volume serial number is unavailable"))?;
    let index = metadata
        .file_index()
        .ok_or_else(|| io::Error::other("Windows file index is unavailable"))?;
    Ok((volume, index))
}

#[cfg(not(any(unix, windows)))]
fn coverage_file_identity(file: &cap_std::fs::File) -> io::Result<CoverageEntryIdentity> {
    let metadata = file.metadata()?;
    Ok((metadata.len(), metadata.modified().ok()))
}

#[cfg(not(any(unix, windows)))]
fn coverage_metadata_identity(
    metadata: &cap_std::fs::Metadata,
) -> io::Result<CoverageEntryIdentity> {
    Ok((metadata.len(), metadata.modified().ok()))
}

#[cfg(windows)]
fn coverage_windows_handle_identity(
    handle: *mut std::ffi::c_void,
) -> io::Result<CoverageEntryIdentity> {
    use std::ffi::c_void;
    use std::mem::MaybeUninit;

    #[repr(C)]
    struct FileTime {
        low: u32,
        high: u32,
    }

    #[repr(C)]
    struct ByHandleFileInformation {
        file_attributes: u32,
        creation_time: FileTime,
        last_access_time: FileTime,
        last_write_time: FileTime,
        volume_serial_number: u32,
        file_size_high: u32,
        file_size_low: u32,
        number_of_links: u32,
        file_index_high: u32,
        file_index_low: u32,
    }

    unsafe extern "system" {
        fn GetFileInformationByHandle(
            file: *mut c_void,
            information: *mut ByHandleFileInformation,
        ) -> i32;
    }

    let mut information = MaybeUninit::<ByHandleFileInformation>::uninit();
    // SAFETY: `handle` is owned by the live retained file/directory for the duration of the call,
    // and `information` has the exact writable Win32 structure layout.
    let success = unsafe { GetFileInformationByHandle(handle, information.as_mut_ptr()) };
    if success == 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: a successful GetFileInformationByHandle call initializes every field.
    let information = unsafe { information.assume_init() };
    let file_index =
        (u64::from(information.file_index_high) << 32) | u64::from(information.file_index_low);
    Ok((information.volume_serial_number, file_index))
}

#[cfg(windows)]
fn coverage_metadata_is_link(metadata: &Metadata) -> bool {
    use cap_std::fs::MetadataExt;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn coverage_metadata_is_link(metadata: &Metadata) -> bool {
    metadata.file_type().is_symlink()
}

fn has_coverage_extension(path: &Path, config: &SpecSyncConfig) -> bool {
    if has_configured_extension(
        path,
        &config.source_extensions,
        config.include_extensionless,
    ) {
        return true;
    }
    if !config.source_extensions.is_empty() {
        return false;
    }
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| STATIC_COVERAGE_EXTENSIONS.contains(&extension))
}

// ─── Single Spec Validation ──────────────────────────────────────────────

/// A capability-confined source-file observation supplied by a snapshot caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SourceSnapshot {
    Present(Vec<u8>),
    Missing,
    Rejected,
    Unreadable,
    /// The mapping resolved to a directory inside the project. Kept distinct from
    /// `Rejected` so validation reports the real cause instead of a security escape.
    Directory,
}

/// Validate a single spec file against source code.
pub fn validate_spec(
    spec_path: &Path,
    root: &Path,
    schema_tables: &HashSet<String>,
    schema_columns: &HashMap<String, SchemaTable>,
    config: &SpecSyncConfig,
) -> ValidationResult {
    let content = match fs::read_to_string(spec_path) {
        Ok(content) => content,
        Err(error) => {
            let rel_path = spec_path
                .strip_prefix(root)
                .unwrap_or(spec_path)
                .to_string_lossy()
                .to_string();
            let mut result = ValidationResult::new(rel_path);
            result.errors.push(format!("Cannot read spec: {error}"));
            return result;
        }
    };

    validate_spec_content_internal(
        spec_path,
        &content,
        root,
        schema_tables,
        schema_columns,
        config,
        true,
        None,
    )
}

/// Validate already-read spec content against source code.
///
/// `spec_path` is used only as the logical location for diagnostics. Neither the
/// spec nor adjacent companion files are opened, allowing callers with a confined
/// snapshot to validate without crossing back into ambient path resolution.
#[allow(dead_code)]
pub fn validate_spec_content(
    spec_path: &Path,
    content: &str,
    root: &Path,
    schema_tables: &HashSet<String>,
    schema_columns: &HashMap<String, SchemaTable>,
    config: &SpecSyncConfig,
) -> ValidationResult {
    validate_spec_content_internal(
        spec_path,
        content,
        root,
        schema_tables,
        schema_columns,
        config,
        false,
        None,
    )
}

/// Validate supplied spec bytes and supplied source observations without reopening
/// either through ambient project paths.
pub(crate) fn validate_spec_content_with_sources(
    spec_path: &Path,
    content: &str,
    root: &Path,
    schema_tables: &HashSet<String>,
    schema_columns: &HashMap<String, SchemaTable>,
    config: &SpecSyncConfig,
    sources: &HashMap<String, SourceSnapshot>,
) -> ValidationResult {
    validate_spec_content_internal(
        spec_path,
        content,
        root,
        schema_tables,
        schema_columns,
        config,
        false,
        Some(sources),
    )
}

#[allow(clippy::too_many_arguments)]
fn validate_spec_content_internal(
    spec_path: &Path,
    content: &str,
    root: &Path,
    schema_tables: &HashSet<String>,
    schema_columns: &HashMap<String, SchemaTable>,
    config: &SpecSyncConfig,
    validate_companions: bool,
    source_snapshots: Option<&HashMap<String, SourceSnapshot>>,
) -> ValidationResult {
    let rel_path = spec_path
        .strip_prefix(root)
        .unwrap_or(spec_path)
        .to_string_lossy()
        .to_string();

    let mut result = ValidationResult::new(rel_path);
    let content_size = content.len() as u64;
    let normalized = if content.contains("\r\n") {
        std::borrow::Cow::Owned(content.replace("\r\n", "\n"))
    } else {
        std::borrow::Cow::Borrowed(content)
    };
    let content = normalized.as_ref();

    // ─── Level 0: Is this file a document at all? ─────────────────────
    // A spec carrying an unresolved conflict is not a spec; every section and
    // symbol check below would assert a contract over text nobody has read.
    // Reported before frontmatter parsing so a conflict inside the frontmatter
    // is named as a conflict rather than incidentally as a `duplicate key`.
    //
    // Only complete opener/separator/closer triples outside fenced code blocks
    // count. A bare `=======` line is a setext `<h1>` underline in Markdown, so
    // a guard on marker *shape* would fail ordinary specs.
    if let Some(hunk) = crate::merge::document_conflict_hunks(content)
        .into_iter()
        .next()
    {
        result.errors.push(format!(
            "Unresolved merge conflict in spec body ({} ↔ {}) — resolve it before this file can be validated",
            hunk.ours_label, hunk.theirs_label
        ));
        result.fixes.push(
            "Run `spec-sync merge --all` to attempt auto-resolution, or resolve the conflict by hand"
                .to_string(),
        );
        return result;
    }
    if let Some(unmerged) = crate::merge::cached_unmerged_paths(root) {
        let spec_rel = spec_path
            .strip_prefix(root)
            .unwrap_or(spec_path)
            .to_string_lossy()
            .replace('\\', "/");
        if unmerged.contains(&spec_rel) {
            result.errors.push(format!(
                "git reports {spec_rel} as unmerged — resolve the merge conflict before validating"
            ));
            return result;
        }
    }

    let parsed = match parse_frontmatter(content) {
        Some(p) => p,
        None => {
            result.errors.push(
                "Missing or malformed YAML frontmatter (expected --- delimiters)".to_string(),
            );
            return result;
        }
    };

    // Surface parse-time frontmatter diagnostics before structural checks.
    result.errors.extend(parsed.errors.iter().cloned());
    result.warnings.extend(parsed.warnings.iter().cloned());

    let fm = &parsed.frontmatter;
    let body = &parsed.body;
    result.status = fm.parsed_status();

    // File size guard: warn if the spec exceeds the configurable limit (default 512 KB)
    {
        let limit_kb = config.rules.max_spec_size_kb.unwrap_or(512) as u64;
        let size_kb = content_size / 1024;
        if size_kb > limit_kb {
            result.warnings.push(format!(
                "Spec file is {size_kb} KB — exceeds limit of {limit_kb} KB, consider splitting into smaller specs"
            ));
        }
    }

    // Archived specs: skip all validation with zero diagnostics.
    // Must be invisible to --strict mode.
    if fm.parsed_status() == Some(crate::types::SpecStatus::Archived) {
        return result;
    }

    if validate_companions {
        validate_companion_scaffold_markers(spec_path, root, &mut result);
    }

    let config_hint = config
        .config_path
        .as_deref()
        .map(|p| format!(" (check config: {})", p.display()))
        .unwrap_or_default();

    // ─── Level 1: Structural ──────────────────────────────────────────

    if fm.module.is_none() {
        result
            .errors
            .push("Frontmatter missing required field: module".to_string());
        result
            .fixes
            .push("Add `module: your-module-name` to the YAML frontmatter block".to_string());
    }
    if fm.version.is_none() {
        result
            .errors
            .push("Frontmatter missing required field: version".to_string());
        result
            .fixes
            .push("Add `version: 1` to the YAML frontmatter block".to_string());
    }
    if fm.status.is_none() {
        result
            .errors
            .push("Frontmatter missing required field: status".to_string());
        result.fixes.push(
            "Add `status: active` (or draft/review/stable/deprecated/archived) to the frontmatter"
                .to_string(),
        );
    } else if let Some(status_str) = &fm.status
        && fm.parsed_status().is_none()
    {
        result.warnings.push(format!(
                "Unknown status '{}' — expected one of: draft, review, active, stable, deprecated, archived",
                status_str
            ));
    }

    // Status lifecycle warnings
    let spec_status = fm.parsed_status();
    if spec_status == Some(crate::types::SpecStatus::Deprecated) {
        result
            .warnings
            .push("Spec is deprecated — consider archiving with `specsync lifecycle promote <spec>` or `specsync lifecycle set <spec> archived`".to_string());
    }

    // Validate agent_policy if present
    if let Some(policy) = &fm.agent_policy {
        match policy.as_str() {
            "read-only" | "suggest-only" | "full-access" => {}
            _ => {
                result.warnings.push(format!(
                    "Unknown agent_policy '{}' — expected: read-only, suggest-only, or full-access",
                    policy
                ));
            }
        }
    }
    if fm.files.is_empty() {
        result.errors.push(
            "Frontmatter missing required field: files (must be a non-empty list)".to_string(),
        );
        result.fixes.push(
            "Add a `files:` list with relative paths to source files this spec covers".to_string(),
        );
    }

    // Check files exist and can be read as UTF-8.
    // A file that exists but cannot be decoded must fail loud here: otherwise
    // export extraction silently yields zero symbols (see exports::mod read path),
    // so undocumented exports are never checked and the spec passes — a silent
    // false-PASS of the core API-surface guarantee.
    for file in &fm.files {
        let full_path = root.join(file);
        let safe_project_relative = planned_source_path_is_safe(file);
        let snapshot = source_snapshots.and_then(|sources| sources.get(file));
        let ambient_existing_escape =
            source_snapshots.is_none() && full_path.exists() && !source_within_root(root, file);
        let confined_rejection = matches!(snapshot, Some(SourceSnapshot::Rejected))
            || (source_snapshots.is_none() && !source_within_root(root, file));
        // A `files:` entry that resolves to a directory extracts zero exports, so
        // leaving it unreported lets a spec document nothing and still pass the
        // Public API comparison — the gate goes green while measuring nothing.
        let directory_mapping = matches!(snapshot, Some(SourceSnapshot::Directory))
            || (source_snapshots.is_none()
                && safe_project_relative
                && crate::exports::files_entry_is_directory(&full_path)
                && source_within_root(root, file));
        if ambient_existing_escape || (safe_project_relative && confined_rejection) {
            result.errors.push(format!(
                "Source file `{file}` resolves outside the project root and is ignored for security"
            ));
            result.fixes.push(format!(
                "Use a path inside the project (no absolute paths, `..` escapes, or symlinks that leave the project), or remove `{file}` from the files list"
            ));
        } else if !safe_project_relative {
            result.errors.push(format!(
                "Source mapping is not a safe project-relative path: {file}"
            ));
            result.fixes.push(format!(
                "Use a safe project-relative path for `{file}` (no absolute paths, `..`, drive prefixes, or backslashes)"
            ));
        } else if directory_mapping {
            result.errors.push(format!(
                "Source file `{file}` is a directory — `files:` must list source files, not directories"
            ));
            result
                .fixes
                .push(directory_mapping_fix(root, file, config, source_snapshots));
        } else if matches!(snapshot, Some(SourceSnapshot::Unreadable)) {
            result.errors.push(format!(
                "Source file `{file}` could not be read for validation"
            ));
            result.fixes.push(format!(
                "Remove `{file}` from the files list or fix permissions"
            ));
        } else if (source_snapshots.is_some() && snapshot.is_none())
            || matches!(snapshot, Some(SourceSnapshot::Missing))
            || (source_snapshots.is_none() && !full_path.exists())
        {
            let planned_draft_mapping =
                spec_status == Some(crate::types::SpecStatus::Draft) && !config.require_draft_files;
            if planned_draft_mapping {
                result.notices.push(format!(
                    "Planned source mapping (draft; file not created yet): {file}"
                ));
            } else {
                result.errors.push(format!("Source file not found: {file}"));
                if source_snapshots.is_none()
                    && let Some(suggestion) = suggest_similar_file(root, file)
                {
                    result.fixes.push(format!(
                        "Did you mean `{suggestion}`? Update the path in frontmatter"
                    ));
                } else {
                    result.fixes.push(format!(
                        "Remove `{file}` from files list or create the source file"
                    ));
                }
            }
        } else if let Some(SourceSnapshot::Present(bytes)) = snapshot {
            result.had_present_source = true;
            let ext = full_path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let is_supported = crate::types::Language::from_extension(ext).is_some();
            if is_supported && std::str::from_utf8(bytes).is_err() {
                result.errors.push(format!(
                    "Source file `{file}` could not be read as UTF-8 for validation"
                ));
                result.fixes.push(format!(
                    "Re-save `{file}` as UTF-8 (specsync validates UTF-8 source), or remove it from the files list"
                ));
            }
        } else if source_snapshots.is_none() && full_path.is_file() {
            result.had_present_source = true;
            let ext = full_path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let is_supported = crate::types::Language::from_extension(ext).is_some();
            if is_supported {
                if let Err(err) = fs::read_to_string(&full_path) {
                    result.errors.push(format!(
                        "Source file `{file}` could not be read as UTF-8 for validation: {err}"
                    ));
                    result.fixes.push(format!(
                        "Re-save `{file}` as UTF-8 (specsync validates UTF-8 source), or remove it from the files list"
                    ));
                }
            } else if let Err(err) = fs::File::open(&full_path) {
                result.errors.push(format!(
                    "Source file `{file}` could not be read for validation: {err}"
                ));
                result.fixes.push(format!(
                    "Remove `{file}` from the files list or fix permissions"
                ));
            }
        }
    }

    // Check db_tables exist in the canonical replayed schema. An omitted `schema_dir` stays
    // VISIBLE rather than silently disabling every declaration — but it is a NOTICE, not a
    // warning, because it describes what this run did not do rather than a defect in the spec.
    //
    // Warnings escalate to errors under `--strict` (`compute_exit_code`). That made this
    // unactionable for any project whose schema lives in application code rather than `.sql`
    // migrations: there is nothing to point `schema_dir` at, `schema_dir` is not a spec
    // frontmatter field, and the only ways to clear it were deleting the `db_tables`
    // documentation or abandoning `strict`. Gating on advice the reader cannot take is not
    // drift detection.
    if !fm.db_tables.is_empty() && config.schema_dir.is_none() {
        result.notices.push(
            "DB table validation skipped: `db_tables` is declared but `schema_dir` is not configured"
                .to_string(),
        );
        result.fixes.push(
            "Configure `schema_dir` to validate declared database tables against migrations, or leave it unset if the schema is defined in application code"
                .to_string(),
        );
    }

    let mut schema_tables_known: Option<bool> = None;
    for table in &fm.db_tables {
        if validate_db_table_identifier(table, &mut result).is_err() {
            continue;
        }
        if config.schema_dir.is_some() {
            match schema_table_exists(table, schema_tables) {
                Ok(true) => {}
                // Only claim a table is MISSING when the schema was actually readable. An
                // unparseable migration already reports its own error; adding "every declared
                // table is absent" on top buries the real cause and suggests writing a
                // `CREATE TABLE` that is already present and correct. Resolved lazily so the
                // happy path never re-replays the schema.
                Ok(false) => {
                    let known = *schema_tables_known
                        .get_or_insert_with(|| schema_table_names_available(root, config));
                    if known {
                        add_missing_db_table_error(table, &mut result);
                    }
                }
                Err(error) => add_schema_resolution_error(
                    format!("Invalid DB table identifier `{table}`: {error}"),
                    &mut result,
                ),
            }
        }
    }

    // ─── Level 1.5: Schema Columns ──────────────────────────────────────
    if !schema_columns.is_empty() {
        let spec_schema = schema::parse_spec_schema(body);
        for table_name in &fm.db_tables {
            let actual_table = match resolve_schema_value(table_name, schema_columns) {
                Ok(Some((_, table))) => table,
                Ok(None) => continue,
                Err(error) => {
                    add_schema_resolution_error(error, &mut result);
                    continue;
                }
            };
            let spec_cols = match resolve_schema_value(table_name, &spec_schema) {
                Ok(Some((_, columns))) => columns,
                Ok(None) => continue,
                Err(error) => {
                    add_schema_resolution_error(error, &mut result);
                    continue;
                }
            };

            let actual_names: HashSet<&str> = actual_table
                .columns
                .iter()
                .map(|c| c.name.as_str())
                .collect();
            let spec_names: HashSet<&str> = spec_cols.iter().map(|c| c.name.as_str()).collect();

            // Spec documents a column that doesn't exist = ERROR
            for sc in spec_cols {
                if !actual_names.contains(sc.name.as_str()) {
                    result.errors.push(format!(
                        "Schema column `{}.{}` documented in spec but not found in migrations",
                        table_name, sc.name
                    ));
                    result.fixes.push(format!(
                        "Remove `{}` from the ### Schema section or add it via ALTER TABLE",
                        sc.name
                    ));
                }
            }

            // Column exists in schema but not in spec = WARNING
            for ac in &actual_table.columns {
                if !spec_names.contains(ac.name.as_str()) {
                    result.warnings.push(format!(
                        "Schema column `{}.{}` exists in migrations but not documented in spec",
                        table_name, ac.name
                    ));
                }
            }

            // Type mismatch = WARNING
            for sc in spec_cols {
                if let Some(ac) = actual_table.columns.iter().find(|c| c.name == sc.name) {
                    // Normalise both to uppercase for comparison
                    let spec_type = sc.col_type.to_uppercase();
                    let actual_type = ac.col_type.to_uppercase();
                    if spec_type != actual_type {
                        result.warnings.push(format!(
                                "Schema column `{}.{}` type mismatch: spec says {} but migrations say {}",
                                table_name, sc.name, spec_type, actual_type
                            ));
                    }
                }
            }
            // If spec has db_tables but no ### Schema section, that's fine —
            // column-level docs are optional. Only validate when present.
        }
    }

    // Required markdown sections
    // - draft: structure only — skip all required section checks
    // - review: structure + sections, but skip "Public API"
    // - active/stable/deprecated: all sections required
    let is_draft = spec_status == Some(crate::types::SpecStatus::Draft);
    let is_review = spec_status == Some(crate::types::SpecStatus::Review);

    if !is_draft {
        let missing = get_missing_sections(body, &config.required_sections);
        let near_misses = get_near_miss_sections(body, &config.required_sections);
        for section in &missing {
            if is_review && section == "Public API" {
                continue; // review specs can skip Public API
            }
            // Check if a near-miss heading exists — give a targeted hint
            if let Some((_, found)) = near_misses.iter().find(|(req, _)| req == section) {
                result.errors.push(format!(
                    "Missing required section: ## {section} (found '## {found}' — typo? Run --fix to rename)"
                ));
                result.fixes.push(format!(
                    "Run `spec-sync check --fix` to rename `## {found}` → `## {section}`"
                ));
            } else {
                result.errors.push(format!(
                    "Missing required section: ## {section}{config_hint}"
                ));
                result
                    .fixes
                    .push(format!("Add `## {section}` heading to the spec body"));
            }
        }
    }

    // Recorded even when export validation is skipped: a draft that documents
    // a contract over source that exists is opting out of a check it could
    // have passed, which is what separates it from an honest stub.
    result.documents_contract = !crate::parser::get_spec_symbols(body).is_empty();

    // ─── Level 1.7: Empty-Draft Section Detection ───────────────────
    let stub_sections = find_stub_sections(body, &config.required_sections);
    if !stub_sections.is_empty() {
        for section in &stub_sections {
            result.warnings.push(format!(
                "Section ## {section} contains only unfinished draft text"
            ));
            result.fixes.push(format!(
                "Replace draft content in ## {section} with real documentation"
            ));
        }
    }

    // ─── Level 2: API Surface ─────────────────────────────────────────
    // Draft and review specs skip API surface validation — exports may not exist yet.
    let skip_api = matches!(
        spec_status,
        Some(crate::types::SpecStatus::Draft) | Some(crate::types::SpecStatus::Review)
    );

    if !skip_api {
        for symbol in get_duplicate_spec_symbols(body) {
            result.errors.push(format!(
                "Public API lists `{symbol}` more than once — remove the duplicate row"
            ));
        }
    }

    if !fm.files.is_empty() && !skip_api {
        // Track exports with their source file for attribution
        let mut exports_by_file: Vec<(String, String)> = Vec::new(); // (symbol, file)
        let mut all_exports: Vec<String> = Vec::new();
        // Source files whose symbol list is fiction. Collected rather than acted
        // on inline: once ANY mapped file is conflicted, the spec-versus-source
        // comparison below is meaningless, so it is skipped entirely instead of
        // emitting a page of "documents X but no export found" noise derived
        // from a tree that does not exist.
        let mut conflicted_sources: Vec<String> = Vec::new();
        let unmerged = crate::merge::cached_unmerged_paths(root);
        for file in &fm.files {
            // Never extract exports from a path that escapes the project root — it
            // would leak arbitrary host-file identifiers. The files-exist check
            // above already reports such entries as errors.
            let full_path = root.join(file);
            // git's own unmerged list is authoritative and costs no heuristic: a
            // path in it IS a conflict, whatever the extractor made of the bytes.
            if unmerged
                .as_ref()
                .is_some_and(|paths| paths.contains(&file.replace('\\', "/")))
            {
                conflicted_sources.push(format!("{file} — git reports this path as unmerged"));
                continue;
            }
            let scan = if let Some(sources) = source_snapshots {
                let Some(SourceSnapshot::Present(bytes)) = sources.get(file) else {
                    continue;
                };
                let Ok(content) = std::str::from_utf8(bytes) else {
                    continue;
                };
                crate::exports::scan_exported_symbols_from_content(
                    &full_path,
                    content,
                    config.export_level,
                    config.parse_mode,
                )
            } else {
                if !source_within_root(root, file) {
                    continue;
                }
                crate::exports::scan_exported_symbols_full(
                    &full_path,
                    config.export_level,
                    config.parse_mode,
                )
            };
            let exports = match scan {
                crate::exports::ExportScan::Conflicted(evidence) => {
                    conflicted_sources.push(format!("{file} — {}", evidence.describe()));
                    continue;
                }
                crate::exports::ExportScan::Parsed(symbols) => symbols,
                crate::exports::ExportScan::UnknownLanguage
                | crate::exports::ExportScan::Unreadable
                | crate::exports::ExportScan::Directory => Vec::new(),
            };
            for sym in &exports {
                exports_by_file.push((sym.clone(), file.clone()));
            }
            all_exports.extend(exports);
        }

        if !conflicted_sources.is_empty() {
            for detail in &conflicted_sources {
                result
                    .errors
                    .push(format!("Unresolved merge conflict in source: {detail}"));
            }
            result.fixes.push(
                "Resolve the merge conflict in the listed source file(s) before re-running check"
                    .to_string(),
            );
        }

        // Deduplicate (keep first occurrence for file attribution)
        let mut seen = HashSet::new();
        all_exports.retain(|s| seen.insert(s.clone()));

        // A conflicted mapping makes the whole API surface unknown, so the
        // comparison is not attempted. Reporting `n/n exports documented` over a
        // partial set would be the same fail-open in a different costume.
        if conflicted_sources.is_empty() {
            let spec_symbols = get_spec_symbols(body);
            let spec_set: HashSet<&str> = spec_symbols.iter().map(|s| s.as_str()).collect();
            let export_set: HashSet<&str> = all_exports.iter().map(|s| s.as_str()).collect();

            // Spec documents something that doesn't exist = ERROR
            for sym in &spec_symbols {
                if !export_set.contains(sym.as_str()) {
                    result.errors.push(format!(
                        "Spec documents '{sym}' but no matching export found in source"
                    ));
                }
            }

            // Code exports something not in spec = WARNING (with source file attribution)
            for sym in &all_exports {
                if !spec_set.contains(sym.as_str()) {
                    // Find the source file for this export
                    let source_file = exports_by_file
                        .iter()
                        .find(|(s, _)| s == sym)
                        .map(|(_, f)| f.as_str());
                    match source_file {
                        Some(file) => {
                            result
                                .warnings
                                .push(format!("Undocumented export '{sym}' from {file}"));
                        }
                        None => {
                            result
                                .warnings
                                .push(format!("Export '{sym}' not in spec (undocumented)"));
                        }
                    }
                }
            }

            let documented = spec_symbols
                .iter()
                .filter(|s| export_set.contains(s.as_str()))
                .count();

            if !all_exports.is_empty() {
                let summary = format!("{documented}/{} exports documented", all_exports.len());
                if documented < all_exports.len() {
                    result.warnings.insert(0, summary);
                } else {
                    result.export_summary = Some(summary);
                }
            }
        }
    }

    // ─── Level 3: Dependencies ────────────────────────────────────────

    if !fm.depends_on.is_empty() {
        for dep in &fm.depends_on {
            if is_cross_project_ref(dep) {
                // Cross-project refs (e.g. "owner/repo@module") are validated
                // by `specsync resolve`, not during local checks.
                continue;
            }
            // Bare module names (no path separators or extension) resolve
            // against the specs directory, not the project root.
            let full_path = if !dep.contains('/') && !dep.contains('.') {
                root.join(&config.specs_dir).join(dep)
            } else {
                root.join(dep)
            };
            if !full_path.exists() {
                result
                    .errors
                    .push(format!("Dependency spec not found: {dep}"));
            }
        }
    }

    // Check Consumed By section references
    if let Some(caps) = CONSUMED_BY_RE.captures(body) {
        let section = caps.get(1).unwrap().as_str();
        for caps in FILE_REF_RE.captures_iter(section) {
            if let Some(file_ref) = caps.get(1) {
                let file_path = root.join(file_ref.as_str());
                if !file_path.exists() {
                    result.warnings.push(format!(
                        "Consumed By references missing file: {}",
                        file_ref.as_str()
                    ));
                }
            }
        }
    }

    // ─── Level 4: Requirements Companion File ─────────────────────────
    // spec-sync expects requirements in a separate companion file (requirements.md),
    // not inline in the spec body. Warn if requirements appear inline.
    // Drafts and review specs skip this check.
    if !is_draft && spec_status != Some(crate::types::SpecStatus::Review) {
        let has_inline_requirements = {
            let lower = body.to_ascii_lowercase();
            lower.contains("## requirements") || lower.contains("## acceptance criteria")
        };
        if has_inline_requirements {
            result.warnings.push(
                "Inline requirements detected — specs are technical contracts; user stories and acceptance criteria belong in a companion requirements.md file".to_string()
            );
            result.fixes.push(
                "Run `specsync add-spec <name>` to scaffold requirements.md, then move ## Requirements / ## Acceptance Criteria content there".to_string()
            );
        }

        // Canonical companions are adaptive in the 5.0 SDD model. Technical
        // and internal-only modules do not need an empty requirements file.
    }

    // ─── Custom Validation Rules ─────────────────────────────────────
    apply_custom_rules(spec_path, body, fm, config, &config_hint, &mut result);

    result
}

fn validate_companion_scaffold_markers(
    spec_path: &Path,
    root: &Path,
    result: &mut ValidationResult,
) {
    const MARKERS: &[(&str, &str, &str)] = &[
        (
            "context.md",
            "<!-- Describe the context and motivation for this module. -->",
            "replace the generated context marker with concrete motivation",
        ),
        (
            "context.md",
            "- Record architectural or design decisions relevant to this spec.",
            "record concrete architectural decisions",
        ),
        (
            "context.md",
            "- List the most important files an agent or new developer should read.",
            "list concrete files to read",
        ),
        (
            "context.md",
            "- Summarize implemented behavior, active work, and known blockers.",
            "summarize the current implementation status",
        ),
        (
            "requirements.md",
            "- As a developer, I want to <!-- describe the goal -->",
            "replace the generated user-story marker",
        ),
        (
            "requirements.md",
            "- <!-- List measurable acceptance criteria. -->",
            "list measurable acceptance criteria",
        ),
        (
            "requirements.md",
            "- Define acceptance criteria from the module's source behavior and user-facing responsibilities.",
            "define concrete acceptance criteria",
        ),
        (
            "testing.md",
            "- <!-- List unit test scenarios. -->",
            "list concrete automated test scenarios",
        ),
        (
            "testing.md",
            "List the automated tests and fixtures that protect this module.",
            "list concrete automated tests and fixtures",
        ),
        (
            "testing.md",
            "List manual QA flows, platform checks, and review notes for this module.",
            "record concrete manual QA flows",
        ),
        (
            "tasks.md",
            "- [ ] Add implementation, validation, or release tasks that belong to this spec.",
            "replace the generated task with concrete work or remove it",
        ),
        (
            "design.md",
            "- Document layout structure, responsive breakpoints, and positioning rules.",
            "document concrete layout behavior",
        ),
        (
            "design.md",
            "- Document component tree, inputs, outputs, and slots.",
            "document the concrete component hierarchy",
        ),
        (
            "design.md",
            "- Document color, spacing, typography, and state token overrides.",
            "document concrete design tokens",
        ),
        (
            "design.md",
            "- List icons, images, illustrations, and asset ownership.",
            "list concrete assets and ownership",
        ),
    ];
    let Some(directory) = spec_path.parent() else {
        return;
    };
    for (file_name, marker, correction) in MARKERS {
        let path = directory.join(file_name);
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        let mut fence: Option<char> = None;
        for (index, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            let fence_character = if trimmed.starts_with("```") {
                Some('`')
            } else if trimmed.starts_with("~~~") {
                Some('~')
            } else {
                None
            };
            if let Some(character) = fence_character {
                if fence == Some(character) {
                    fence = None;
                } else if fence.is_none() {
                    fence = Some(character);
                }
                continue;
            }
            if fence.is_none() && trimmed == *marker {
                let relative = path
                    .strip_prefix(root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/");
                result.warnings.push(format!(
                    "Unfilled companion scaffold marker at {relative}:{} — {correction}",
                    index + 1
                ));
            }
        }
    }
}

/// Apply project-specific custom validation rules from config.
fn apply_custom_rules(
    _spec_path: &Path,
    body: &str,
    fm: &Frontmatter,
    config: &SpecSyncConfig,
    config_hint: &str,
    result: &mut ValidationResult,
) {
    let rules = &config.rules;

    // max_spec_size_kb check is handled by the shared validation core to avoid duplicate warnings

    // max_changelog_entries: warn if Change Log has too many rows
    if let Some(max_entries) = rules.max_changelog_entries {
        let count = count_changelog_entries(body);
        if count > max_entries {
            result.warnings.push(format!(
                "Change Log has {count} entries — exceeds limit of {max_entries} (run `specsync compact`)"
            ));
        }
    }

    // require_behavioral_examples: require at least one ### Scenario
    if rules.require_behavioral_examples == Some(true) {
        let scenario_count = body.matches("### Scenario").count();
        if scenario_count == 0 {
            result.errors.push(
                "No behavioral examples found (rule: require_behavioral_examples)".to_string(),
            );
            result.fixes.push(
                "Add at least one `### Scenario:` under `## Behavioral Examples`".to_string(),
            );
        }
    }

    // min_invariants: require a minimum number of numbered invariants
    if let Some(min) = rules.min_invariants {
        let count = count_invariants(body);
        if count < min {
            result.warnings.push(format!(
                "Only {count} invariant(s) found — minimum is {min}"
            ));
        }
    }

    // require_depends_on: require non-empty depends_on in frontmatter
    if rules.require_depends_on == Some(true) && fm.depends_on.is_empty() {
        result
            .warnings
            .push("No consumed dependencies documented (rule: require_depends_on)".to_string());
    }

    // ─── Declarative Custom Rules ────────────────────────────────────
    for rule in &config.custom_rules {
        if !custom_rule_applies(rule, fm) {
            continue;
        }
        if let Some(msg) = evaluate_custom_rule(rule, body) {
            let tagged = format!("{msg} (rule: {}){config_hint}", rule.name);
            match rule.severity {
                RuleSeverity::Error => result.errors.push(tagged),
                RuleSeverity::Warning => result.warnings.push(tagged),
                RuleSeverity::Info => result.warnings.push(format!("[info] {tagged}")),
            }
        }
    }
}

/// Check whether a custom rule applies to the given spec based on its filter.
fn custom_rule_applies(rule: &crate::types::CustomRule, fm: &Frontmatter) -> bool {
    let Some(ref filter) = rule.applies_to else {
        return true;
    };

    if let Some(ref status) = filter.status {
        let spec_status = fm.status.as_deref().unwrap_or("");
        if !spec_status.eq_ignore_ascii_case(status) {
            return false;
        }
    }

    if let Some(ref module_pattern) = filter.module {
        let spec_module = fm.module.as_deref().unwrap_or("");
        if let Some(re) = safe_regex(module_pattern)
            && !re.is_match(spec_module)
        {
            return false;
        }
    }

    true
}

/// Evaluate a single custom rule against the spec body.
/// Returns `Some(message)` if the rule is violated, `None` if it passes.
fn evaluate_custom_rule(rule: &crate::types::CustomRule, body: &str) -> Option<String> {
    match rule.rule_type {
        CustomRuleType::RequireSection => {
            let section = rule.section.as_deref()?;
            if !body_has_section(body, section) {
                let msg = rule
                    .message
                    .clone()
                    .unwrap_or_else(|| format!("Missing required section: ## {section}"));
                return Some(msg);
            }
            None
        }
        CustomRuleType::MinWordCount => {
            let section = rule.section.as_deref()?;
            let min = rule.min_words.unwrap_or(1);
            let header = format!("## {section}");
            let section_start = find_section_offset(body, section)?;
            let after_header = &body[section_start + header.len()..];
            // Bound section to next ## heading
            let section_end = after_header.find("\n## ").unwrap_or(after_header.len());
            let section_body = &after_header[..section_end];
            let word_count = section_body.split_whitespace().count();
            if word_count < min {
                let msg = rule.message.clone().unwrap_or_else(|| {
                    format!("Section ## {section} has {word_count} words — minimum is {min}")
                });
                return Some(msg);
            }
            None
        }
        CustomRuleType::RequirePattern => {
            let pattern = rule.pattern.as_deref()?;
            let re = safe_regex(pattern)?;
            if !re.is_match(body) {
                let msg = rule
                    .message
                    .clone()
                    .unwrap_or_else(|| format!("Required pattern not found: {pattern}"));
                return Some(msg);
            }
            None
        }
        CustomRuleType::ForbidPattern => {
            let pattern = rule.pattern.as_deref()?;
            let re = safe_regex(pattern)?;
            if re.is_match(body) {
                let msg = rule
                    .message
                    .clone()
                    .unwrap_or_else(|| format!("Forbidden pattern found: {pattern}"));
                return Some(msg);
            }
            None
        }
    }
}

/// Count data rows in the Change Log table (excluding header and separator).
fn count_changelog_entries(body: &str) -> usize {
    let changelog_start = match find_section_offset(body, "Change Log") {
        Some(pos) => pos,
        None => return 0,
    };
    let section = &body[changelog_start..];
    // Find next ## heading to bound the section
    let section_end = section[1..]
        .find("\n## ")
        .map(|p| p + 1)
        .unwrap_or(section.len());
    let section = &section[..section_end];

    // Count data rows: skip the first two table lines (header + separator)
    let mut table_line_count = 0usize;
    section
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            if !trimmed.starts_with('|') {
                return false;
            }
            table_line_count += 1;
            table_line_count > 2
        })
        .count()
}

/// Count numbered invariants in the Invariants section.
fn count_invariants(body: &str) -> usize {
    let inv_start = match body.find("## Invariants") {
        Some(pos) => pos,
        None => return 0,
    };
    let section = &body[inv_start..];
    let section_end = section[1..]
        .find("\n## ")
        .map(|p| p + 1)
        .unwrap_or(section.len());
    let section = &section[..section_end];

    NUMBERED_RE.find_iter(section).count()
}

/// Suggest a similar file path when a referenced file doesn't exist.
fn suggest_similar_file(root: &Path, missing_file: &str) -> Option<String> {
    let missing_name = Path::new(missing_file).file_name()?.to_str()?;

    let parent = Path::new(missing_file).parent()?;
    let search_dir = root.join(parent);
    if !search_dir.exists() {
        return None;
    }

    let entries = std::fs::read_dir(&search_dir).ok()?;
    let mut best: Option<(String, usize)> = None;

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !entry.path().is_file() {
            continue;
        }
        let dist = levenshtein(missing_name, &name);
        if dist <= 3 && (best.is_none() || dist < best.as_ref().unwrap().1) {
            let suggestion = parent.join(&name).to_string_lossy().replace('\\', "/");
            best = Some((suggestion, dist));
        }
    }

    best.map(|(s, _)| s)
}

/// How many expanded source files a directory-mapping fix names before summarizing.
const DIRECTORY_MAPPING_FIX_LIMIT: usize = 5;

/// Build the actionable fix for a `files:` entry that resolved to a directory.
///
/// Names the source files the entry should have listed, reusing the same
/// directory expansion `generate`/`scaffold` apply to a `[modules."x"] files`
/// directory (`generator::find_module_source_files`), so the remedy matches what
/// generation would have written. Snapshot callers validate retained bytes only
/// and must not enumerate the ambient filesystem, so they get the shape guidance
/// without the file list.
fn directory_mapping_fix(
    root: &Path,
    file: &str,
    config: &SpecSyncConfig,
    source_snapshots: Option<&HashMap<String, SourceSnapshot>>,
) -> String {
    let expanded = if source_snapshots.is_none() {
        expand_directory_mapping(root, file, config)
    } else {
        Vec::new()
    };
    if expanded.is_empty() {
        return format!(
            "Replace `{file}` in the files list with the source files beneath it (one entry per file)"
        );
    }
    let shown: Vec<String> = expanded
        .iter()
        .take(DIRECTORY_MAPPING_FIX_LIMIT)
        .cloned()
        .collect();
    let remaining = expanded.len().saturating_sub(shown.len());
    let listed = shown.join(", ");
    if remaining > 0 {
        format!(
            "Replace `{file}` in the files list with the {} source files beneath it: {listed}, and {remaining} more",
            expanded.len()
        )
    } else {
        format!("Replace `{file}` in the files list with: {listed}")
    }
}

/// Expand a project-relative directory into the root-relative source files beneath
/// it, excluding configured exclude directories. Ambient filesystem read: callers
/// must confine the entry first (`source_within_root`).
fn expand_directory_mapping(root: &Path, file: &str, config: &SpecSyncConfig) -> Vec<String> {
    let full_path = root.join(file);
    let mut expanded: Vec<String> =
        crate::generator::find_module_source_files(&full_path, config, root)
            .into_iter()
            .filter_map(|path| {
                let relative = Path::new(&path)
                    .strip_prefix(root)
                    .ok()
                    .map(|relative| relative.to_string_lossy().replace('\\', "/"))?;
                let excluded = Path::new(&relative).components().any(|component| {
                    component
                        .as_os_str()
                        .to_str()
                        .is_some_and(|name| config.exclude_dirs.iter().any(|dir| dir == name))
                });
                (!excluded).then_some(relative)
            })
            .collect();
    expanded.sort();
    expanded.dedup();
    expanded
}

/// Whether a spec `files:` entry resolves to a real path INSIDE the project root.
///
/// Source files a spec validates must live within the project. An entry that
/// escapes — an absolute path, `..` traversal, or a symlink pointing outside the
/// project — is rejected by the caller: reading it would count arbitrary host
/// files as covered and leak their exported identifiers into coverage output and
/// PR comments (a hostile-repo info-disclosure vector, the same threat model as a
/// committed executable configuration).
///
/// For a missing leaf, canonicalizes the nearest existing ancestor so a
/// symlinked parent cannot turn a planned in-project path into a future escape.
/// Returns `false` when the project root or every path ancestor is unreadable.
///
/// Shared: every site that reads a `files:` entry's CONTENT (export extraction in
/// `score`, `check --fix`, `diff`, `new`, …) must gate on this, or the out-of-root
/// identifier leak reopens through that command.
pub fn source_within_root(root: &Path, file: &str) -> bool {
    let full = root.join(file);
    let Ok(canon_root) = root.canonicalize() else {
        return false;
    };
    for candidate in full.ancestors() {
        match candidate.symlink_metadata() {
            Ok(_) => {
                return candidate
                    .canonicalize()
                    .is_ok_and(|canonical| canonical.starts_with(&canon_root));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(_) => return false,
        }
    }
    false
}

/// Whether a not-yet-created source mapping is a portable project-relative path.
fn planned_source_path_is_safe(file: &str) -> bool {
    !file.contains('\\') && normalize_source_mapping(file).is_some()
}

/// Normalize a safe project-relative source mapping for ownership and coverage.
pub(crate) fn normalize_source_mapping(file: &str) -> Option<String> {
    let path = Path::new(file);
    if file.is_empty()
        || path.is_absolute()
        || file.as_bytes().get(1).is_some_and(|byte| *byte == b':')
    {
        return None;
    }

    let mut components = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::Normal(value) => {
                components.push(value.to_str()?.to_string());
            }
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir
            | std::path::Component::RootDir
            | std::path::Component::Prefix(_) => return None,
        }
    }
    (!components.is_empty()).then(|| components.join("/"))
}
