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
pub fn schema_table_names_available(root: &Path, config: &SpecSyncConfig) -> bool {
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
        if !retained_config_has_source_dirs(relative, content)? {
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

    // Check db_tables exist in the canonical replayed schema. An omitted
    // schema_dir is visible instead of silently disabling every declaration.
    if !fm.db_tables.is_empty() && config.schema_dir.is_none() {
        result.warnings.push(
            "DB table validation skipped: `db_tables` is declared but `schema_dir` is not configured"
                .to_string(),
        );
        result.fixes.push(
            "Configure `schema_dir` to validate declared database tables against migrations"
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

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_coverage_double_star_exclude_no_panic() {
        // Regression: a `**/**` exclude pattern used to panic in the glob matcher
        // (`&pattern[3..len-3]` reverses to `[3..2]` for the len-5 string). It must
        // instead match every path (empty middle) and exclude all source files.
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("a.ts"), "export function a() {}").unwrap();

        let mut config = SpecSyncConfig {
            source_dirs: vec!["src".to_string()],
            exclude_patterns: vec!["**/**".to_string()],
            ..SpecSyncConfig::default()
        };

        // Must not panic; `**/**` excludes all source files.
        let report = compute_coverage(tmp.path(), &[], &config);
        assert_eq!(
            report.total_source_files, 0,
            "`**/**` should exclude every source file"
        );

        // A normal `**/dir/**` pattern still excludes only that directory.
        fs::create_dir_all(src.join("gen")).unwrap();
        fs::write(src.join("gen/z.ts"), "export function z() {}").unwrap();
        config.exclude_patterns = vec!["**/gen/**".to_string()];
        let report = compute_coverage(tmp.path(), &[], &config);
        assert_eq!(
            report.total_source_files, 1,
            "only src/gen should be excluded, leaving src/a.ts"
        );
    }

    #[test]
    fn gradle_coverage_names_the_root_project_not_the_package_tld() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("payments");
        fs::create_dir_all(root.join("src/main/kotlin/com/example/tool")).unwrap();
        fs::write(root.join("build.gradle.kts"), "plugins {}\n").unwrap();
        fs::write(
            root.join("src/main/kotlin/com/example/tool/Profile.kt"),
            "package com.example.tool\nclass Profile\n",
        )
        .unwrap();
        let config = SpecSyncConfig {
            source_dirs: vec!["src/main/kotlin".to_string()],
            source_extensions: vec!["kt".to_string()],
            ..SpecSyncConfig::default()
        };

        let report = compute_coverage_checked(&root, &[], &config).unwrap();
        assert_eq!(
            report.unspecced_files.len(),
            1,
            "{:?}",
            report.unspecced_files
        );
        assert!(
            report
                .unspecced_modules
                .iter()
                .any(|name| name == "payments"),
            "expected directory identity, got {:?}",
            report.unspecced_modules
        );
        assert!(
            !report.unspecced_modules.iter().any(|name| name == "com"),
            "package TLD must not be a module: {:?}",
            report.unspecced_modules
        );
        assert!(
            !report.unspecced_modules.iter().any(|name| name == "src"),
            "shared src/ leaf must not be a module: {:?}",
            report.unspecced_modules
        );
    }

    #[test]
    fn conventional_src_child_is_still_a_module() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("src/auth")).unwrap();
        fs::write(
            tmp.path().join("src/auth/session.py"),
            "def login():\n    return True\n",
        )
        .unwrap();
        let config = SpecSyncConfig {
            source_dirs: vec!["src".to_string()],
            source_extensions: vec!["py".to_string()],
            ..SpecSyncConfig::default()
        };

        let report = compute_coverage_checked(tmp.path(), &[], &config).unwrap();
        assert!(
            report.unspecced_modules.iter().any(|name| name == "auth"),
            "expected subdirectory module auth, got {:?}",
            report.unspecced_modules
        );
        assert!(!report.unspecced_modules.iter().any(|name| name == "com"));
    }

    // ─── #529: a module name nothing owns is not a missing spec ──────────────

    /// Write a spec at `specs/<module>/<module>.spec.md` mapping `files`.
    fn write_mapping_spec(root: &Path, module: &str, files: &[&str]) -> PathBuf {
        let directory = root.join("specs").join(module);
        fs::create_dir_all(&directory).unwrap();
        let mapped: String = files.iter().map(|file| format!("  - {file}\n")).collect();
        let spec_path = directory.join(format!("{module}.spec.md"));
        fs::write(
            &spec_path,
            format!(
                "---\nmodule: {module}\nversion: 1\nstatus: active\nfiles:\n{mapped}---\n\n# {module}\n"
            ),
        )
        .unwrap();
        spec_path
    }

    fn multi_language_config() -> SpecSyncConfig {
        SpecSyncConfig {
            specs_dir: "specs".to_string(),
            source_dirs: vec!["src".to_string()],
            source_extensions: vec!["py".to_string(), "mjs".to_string()],
            ..SpecSyncConfig::default()
        }
    }

    #[test]
    fn language_specific_specs_do_not_invent_a_parent_module() {
        // #529: `src/strutil.{py,mjs}` are mapped by `strutil_py` and
        // `strutil_js`. No `specs/strutil/` exists — and none should, because
        // nothing is missing. Reporting `strutil` as a module without a spec
        // beside `2/2 files covered` is an answer read off the absence of a
        // NAME, not off any measurement of the files.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/strutil.py"), "def upper(s):\n    return s\n").unwrap();
        fs::write(
            root.join("src/strutil.mjs"),
            "export function upper(s) {\n}\n",
        )
        .unwrap();
        let specs = vec![
            write_mapping_spec(root, "strutil_py", &["src/strutil.py"]),
            write_mapping_spec(root, "strutil_js", &["src/strutil.mjs"]),
        ];

        let report = compute_coverage_checked(root, &specs, &multi_language_config()).unwrap();
        assert_eq!(report.specced_file_count, 2);
        assert!(report.unspecced_files.is_empty());
        assert!(
            !report
                .unspecced_modules
                .iter()
                .any(|name| name == "strutil"),
            "stem of files that are all mapped must not be a module without a spec: {:?}",
            report.unspecced_modules
        );
    }

    #[test]
    fn a_stem_with_one_unmapped_file_is_still_a_module_without_a_spec() {
        // VACUITY CONTROL for the test above. Suppressing every stem would pass
        // that assertion; only a stem whose files were all measured and all
        // found mapped may go quiet. Here `src/strutil.mjs` is mapped by
        // nothing, so `strutil` is a real gap and must survive.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/strutil.py"), "def upper(s):\n    return s\n").unwrap();
        fs::write(
            root.join("src/strutil.mjs"),
            "export function upper(s) {\n}\n",
        )
        .unwrap();
        let specs = vec![write_mapping_spec(root, "strutil_py", &["src/strutil.py"])];

        let report = compute_coverage_checked(root, &specs, &multi_language_config()).unwrap();
        assert_eq!(report.unspecced_files, ["src/strutil.mjs"]);
        assert!(
            report
                .unspecced_modules
                .iter()
                .any(|name| name == "strutil"),
            "a stem with an unmapped file must stay reported: {:?}",
            report.unspecced_modules
        );
    }

    #[test]
    fn a_directory_module_mapped_by_language_specific_specs_is_not_unspecced() {
        // The sibling derivation: the same phantom, one directory up. Nothing
        // in the reported issue named this site, and it fails identically.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src/textkit")).unwrap();
        fs::write(
            root.join("src/textkit/case.py"),
            "def upper(s):\n    return s\n",
        )
        .unwrap();
        fs::write(
            root.join("src/textkit/case.mjs"),
            "export function upper(s) {\n}\n",
        )
        .unwrap();
        let specs = vec![
            write_mapping_spec(root, "textkit_py", &["src/textkit/case.py"]),
            write_mapping_spec(root, "textkit_js", &["src/textkit/case.mjs"]),
        ];

        let report = compute_coverage_checked(root, &specs, &multi_language_config()).unwrap();
        assert_eq!(report.specced_file_count, 2);
        assert!(
            !report
                .unspecced_modules
                .iter()
                .any(|name| name == "textkit"),
            "directory whose files are all mapped must not be a module without a spec: {:?}",
            report.unspecced_modules
        );
    }

    #[test]
    fn a_directory_module_with_an_unmapped_file_is_still_unspecced() {
        // VACUITY CONTROL for the directory site.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src/textkit")).unwrap();
        fs::write(
            root.join("src/textkit/case.py"),
            "def upper(s):\n    return s\n",
        )
        .unwrap();
        fs::write(
            root.join("src/textkit/case.mjs"),
            "export function upper(s) {\n}\n",
        )
        .unwrap();
        let specs = vec![write_mapping_spec(
            root,
            "textkit_py",
            &["src/textkit/case.py"],
        )];

        let report = compute_coverage_checked(root, &specs, &multi_language_config()).unwrap();
        assert_eq!(report.unspecced_files, ["src/textkit/case.mjs"]);
        assert!(
            report
                .unspecced_modules
                .iter()
                .any(|name| name == "textkit"),
            "a directory with an unmapped file must stay reported: {:?}",
            report.unspecced_modules
        );
    }

    #[test]
    fn a_directory_with_no_discovered_source_file_is_still_unspecced() {
        // VACUITY CONTROL for the campaign's own defect class. A directory
        // holding nothing the traversal measures owns zero files, which is the
        // ABSENCE OF INPUT, not a clean bill of health — it keeps its report.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src/assets")).unwrap();
        fs::write(root.join("src/assets/logo.svg"), "<svg/>\n").unwrap();
        fs::write(root.join("src/strutil.py"), "def upper(s):\n    return s\n").unwrap();
        let specs = vec![write_mapping_spec(root, "strutil_py", &["src/strutil.py"])];

        let report = compute_coverage_checked(root, &specs, &multi_language_config()).unwrap();
        assert!(
            report.unspecced_modules.iter().any(|name| name == "assets"),
            "a directory with nothing measured must stay reported: {:?}",
            report.unspecced_modules
        );
    }

    #[test]
    fn a_manifest_module_whose_files_are_all_mapped_is_not_unspecced() {
        // Same phantom from the manifest site: a Cargo package named `toolkit`
        // whose every source file is mapped by `toolkit_core`.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"toolkit\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(root.join("src/lib.rs"), "pub fn upper() {}\n").unwrap();
        let config = SpecSyncConfig {
            specs_dir: "specs".to_string(),
            source_dirs: vec!["src".to_string()],
            source_extensions: vec!["rs".to_string()],
            ..SpecSyncConfig::default()
        };
        let specs = vec![write_mapping_spec(root, "toolkit_core", &["src/lib.rs"])];

        let report = compute_coverage_checked(root, &specs, &config).unwrap();
        assert_eq!(report.specced_file_count, 1);
        assert!(
            !report
                .unspecced_modules
                .iter()
                .any(|name| name == "toolkit"),
            "manifest module whose files are all mapped must not be reported: {:?}",
            report.unspecced_modules
        );

        // VACUITY CONTROL: add a file the spec does not map and the manifest
        // module comes back.
        fs::write(root.join("src/extra.rs"), "pub fn lower() {}\n").unwrap();
        let report = compute_coverage_checked(root, &specs, &config).unwrap();
        assert_eq!(report.unspecced_files, ["src/extra.rs"]);
        assert!(
            report
                .unspecced_modules
                .iter()
                .any(|name| name == "toolkit"),
            "manifest module with an unmapped file must stay reported: {:?}",
            report.unspecced_modules
        );
    }

    #[test]
    fn a_configured_module_whose_files_are_all_mapped_is_not_unspecced() {
        // The configured-module site makes the same claim on the same evidence.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/strutil.py"), "def upper(s):\n    return s\n").unwrap();
        fs::write(
            root.join("src/strutil.mjs"),
            "export function upper(s) {\n}\n",
        )
        .unwrap();
        let mut config = multi_language_config();
        config.modules.insert(
            "strutil".to_string(),
            crate::types::ModuleDefinition {
                files: vec!["src/strutil.py".to_string(), "src/strutil.mjs".to_string()],
                depends_on: Vec::new(),
            },
        );
        let specs = vec![
            write_mapping_spec(root, "strutil_py", &["src/strutil.py"]),
            write_mapping_spec(root, "strutil_js", &["src/strutil.mjs"]),
        ];

        let report = compute_coverage_checked(root, &specs, &config).unwrap();
        assert!(
            !report
                .unspecced_modules
                .iter()
                .any(|name| name == "strutil"),
            "configured module whose files are all mapped must not be reported: {:?}",
            report.unspecced_modules
        );

        // VACUITY CONTROL: a configured module that declares no files at all
        // has nothing measured, so it stays reported.
        let mut declared_nothing = multi_language_config();
        declared_nothing.modules.insert(
            "ghost".to_string(),
            crate::types::ModuleDefinition::default(),
        );
        let report = compute_coverage_checked(root, &specs, &declared_nothing).unwrap();
        assert!(
            report.unspecced_modules.iter().any(|name| name == "ghost"),
            "configured module declaring no files must stay reported: {:?}",
            report.unspecced_modules
        );
    }

    #[test]
    fn malformed_gradle_settings_make_coverage_inconclusive() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("src/main/kotlin")).unwrap();
        fs::write(tmp.path().join("build.gradle.kts"), "plugins {}\n").unwrap();
        fs::write(
            tmp.path().join("settings.gradle.kts"),
            "include(\":member\"\n",
        )
        .unwrap();
        let config = SpecSyncConfig {
            source_dirs: vec!["src".to_string()],
            ..SpecSyncConfig::default()
        };

        let error = compute_coverage_checked(tmp.path(), &[], &config).unwrap_err();
        assert!(error.contains("Cannot parse Gradle settings manifest"));
        let report = compute_coverage(tmp.path(), &[], &config);
        // Nothing was discovered, so there is no percentage to report — not 0
        // and emphatically not 100.
        assert_eq!(report.file_coverage_percent(), None);
        assert!(report.unspecced_modules[0].contains("inconclusive"));
    }

    #[cfg(unix)]
    #[test]
    fn retained_coverage_snapshot_rejects_post_discovery_symlink_replacement() {
        use std::os::unix::fs::symlink;

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("project");
        let outside = tmp.path().join("outside");
        fs::create_dir_all(root.join("member/src/main/kotlin")).unwrap();
        fs::create_dir_all(outside.join("src/main/kotlin")).unwrap();
        let outside_source = outside.join("src/main/kotlin/Secret.kt");
        let sentinel = b"const val SECRET = \"RETAINED_COVERAGE_SENTINEL\"\n";
        fs::write(
            root.join("member/src/main/kotlin/Local.kt"),
            "const val LOCAL = 1\n",
        )
        .unwrap();
        fs::write(&outside_source, sentinel).unwrap();
        let config = SpecSyncConfig {
            source_dirs: vec!["member/src/main/kotlin".to_string()],
            ..SpecSyncConfig::default()
        };

        let project = open_coverage_project_root(&root).unwrap();
        fs::rename(root.join("member"), root.join("original-member")).unwrap();
        symlink(&outside, root.join("member")).unwrap();
        let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());
        let error =
            snapshot_coverage_sources(&project, &root, &config, &HashSet::new(), &mut budget)
                .unwrap_err();

        assert!(error.contains("symlink or reparse point"), "{error}");
        assert!(!error.contains("RETAINED_COVERAGE_SENTINEL"));
        assert_eq!(fs::read(outside_source).unwrap(), sentinel);
    }

    #[test]
    fn retained_coverage_sources_reject_regular_directory_replacement_after_selection() {
        let project = tempfile::tempdir().unwrap();
        let original = project.path().join("original-src");
        fs::create_dir_all(project.path().join("src")).unwrap();
        fs::write(
            project.path().join("src/original.rs"),
            "pub fn original() {}\n",
        )
        .unwrap();
        let retained = open_coverage_project_root(project.path()).unwrap();
        let config = SpecSyncConfig {
            source_dirs: vec!["src".to_string()],
            ..SpecSyncConfig::default()
        };
        let selected_sources = select_coverage_source_directories(
            &retained,
            &config,
            CoverageTraversalLimits::default().max_depth,
        )
        .unwrap();
        fs::rename(project.path().join("src"), &original).unwrap();
        fs::create_dir_all(project.path().join("src")).unwrap();
        fs::write(
            project.path().join("src/replacement.rs"),
            "pub fn replacement() {}\n",
        )
        .unwrap();
        let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());

        let error = snapshot_selected_coverage_sources_with_hook(
            &retained,
            project.path(),
            &config,
            &HashSet::new(),
            &mut budget,
            selected_sources,
            &mut |_| {},
        )
        .unwrap_err();

        assert!(
            error.contains("changed during retained traversal"),
            "{error}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn retained_source_traversal_rejects_directory_replacement_after_enumeration() {
        let project = tempfile::tempdir().unwrap();
        let source_parent = project.path().join("src/module");
        let original = source_parent.join("nested");
        let detached = project.path().join("detached-source");
        let replacement = project.path().join("replacement-source");
        fs::create_dir_all(&original).unwrap();
        fs::create_dir_all(&replacement).unwrap();
        fs::write(original.join("original.rs"), "pub fn original() {}\n").unwrap();
        fs::write(
            replacement.join("replacement.rs"),
            "pub fn replacement() {}\n",
        )
        .unwrap();
        let retained = open_coverage_project_root(project.path()).unwrap();
        let config = SpecSyncConfig {
            source_dirs: vec!["src".to_string()],
            ..SpecSyncConfig::default()
        };
        let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());
        let mut replaced = false;

        let error = snapshot_coverage_sources_with_hook(
            &retained,
            project.path(),
            &config,
            &HashSet::new(),
            &mut budget,
            |relative| {
                if replaced || relative != Path::new("src/module") {
                    return;
                }
                replaced = true;
                fs::rename(&original, &detached).unwrap();
                fs::rename(&replacement, &original).unwrap();
            },
        )
        .unwrap_err();

        assert!(replaced);
        assert!(
            error.contains("changed during retained traversal"),
            "{error}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn retained_spec_traversal_rejects_directory_replacement_after_enumeration() {
        let project = tempfile::tempdir().unwrap();
        let spec_parent = project.path().join("specs/module");
        let original = spec_parent.join("nested");
        let detached = project.path().join("detached-specs");
        let replacement = project.path().join("replacement-specs");
        fs::create_dir_all(&original).unwrap();
        fs::create_dir_all(&replacement).unwrap();
        fs::write(
            original.join("original.spec.md"),
            "---\nmodule: original\nversion: 1\nstatus: stable\nfiles: []\n---\n",
        )
        .unwrap();
        fs::write(
            replacement.join("replacement.spec.md"),
            "---\nmodule: replacement\nversion: 1\nstatus: stable\nfiles: []\n---\n",
        )
        .unwrap();
        let retained = open_coverage_project_root(project.path()).unwrap();
        let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());
        let mut replaced = false;

        let error =
            discover_coverage_spec_files_with_hook(&retained, "specs", &mut budget, |relative| {
                if replaced || relative != Path::new("specs/module") {
                    return;
                }
                replaced = true;
                fs::rename(&original, &detached).unwrap();
                fs::rename(&replacement, &original).unwrap();
            })
            .unwrap_err();

        assert!(replaced);
        assert!(
            error.contains("changed during retained traversal"),
            "{error}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn retained_coverage_spec_mapping_ignores_an_ambient_root_replacement() {
        use std::os::unix::fs::symlink;

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("project");
        let original = tmp.path().join("original-project");
        let replacement = tmp.path().join("replacement-project");
        let relative_spec = Path::new("specs/auth/auth.spec.md");
        fs::create_dir_all(root.join("specs/auth")).unwrap();
        fs::create_dir_all(replacement.join("specs/auth")).unwrap();
        fs::write(
            root.join(relative_spec),
            "---\nmodule: auth\nversion: 1\nstatus: stable\nfiles:\n  - src/retained.rs\n---\n",
        )
        .unwrap();
        fs::write(
            replacement.join(relative_spec),
            "---\nmodule: auth\nversion: 1\nstatus: stable\nfiles:\n  - src/replacement.rs\n---\n",
        )
        .unwrap();
        let project = open_coverage_project_root(&root).unwrap();
        let mut inventory_budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());
        let inventory =
            discover_coverage_spec_files(&project, "specs", &mut inventory_budget).unwrap();
        fs::rename(&root, &original).unwrap();
        symlink(&replacement, &root).unwrap();
        let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());

        let mappings = collect_specced_files(&project, &inventory, &mut budget).unwrap();

        assert!(mappings.contains("src/retained.rs"));
        assert!(!mappings.contains("src/replacement.rs"));
    }

    #[test]
    fn retained_coverage_spec_mapping_rejects_a_path_outside_the_project() {
        let project = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();

        let error = select_coverage_spec_files(
            project.path(),
            &[outside.path().join("outside.spec.md")],
            &[],
        )
        .unwrap_err();

        assert!(
            error.contains("must remain beneath the project root"),
            "{error}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn retained_config_uses_configured_source_dirs_after_root_replacement() {
        use std::os::unix::fs::symlink;

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("project");
        let original = tmp.path().join("original-project");
        let replacement = tmp.path().join("replacement-project");
        fs::create_dir_all(root.join(".specsync")).unwrap();
        fs::create_dir_all(replacement.join(".specsync")).unwrap();
        fs::write(
            root.join(".specsync/config.toml"),
            "specs_dir = \"retained-specs\"\nsource_dirs = [\"retained-source\"]\n",
        )
        .unwrap();
        fs::write(
            replacement.join(".specsync/config.toml"),
            "specs_dir = \"replacement-specs\"\nsource_dirs = [\"replacement-source\"]\n",
        )
        .unwrap();
        let retained = open_coverage_project_root(&root).unwrap();
        fs::rename(&root, &original).unwrap();
        symlink(&replacement, &root).unwrap();
        let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());

        let config = retained_config(&retained, &root, &mut budget).unwrap();

        assert_eq!(config.specs_dir, "retained-specs");
        assert_eq!(config.source_dirs, ["retained-source"]);
    }

    #[test]
    fn retained_explicit_source_dirs_skip_unrelated_manifest_autodetection() {
        let project = tempfile::tempdir().unwrap();
        fs::create_dir_all(project.path().join(".specsync")).unwrap();
        fs::write(
            project.path().join(".specsync/config.toml"),
            "specs_dir = \"specs\"\nsource_dirs = [\"configured-source\"]\n",
        )
        .unwrap();
        fs::write(
            project.path().join("settings.gradle"),
            "include(\":member\"\n",
        )
        .unwrap();
        let retained = open_coverage_project_root(project.path()).unwrap();
        let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());

        let config = retained_config(&retained, project.path(), &mut budget).unwrap();

        assert_eq!(config.source_dirs, ["configured-source"]);
    }

    #[test]
    fn retained_omitted_source_dirs_scan_after_a_malformed_manifest() {
        let project = tempfile::tempdir().unwrap();
        fs::create_dir_all(project.path().join("lib")).unwrap();
        fs::write(project.path().join("lib/source.rs"), "pub fn value() {}\n").unwrap();
        fs::write(
            project.path().join("settings.gradle"),
            "include(\":member\"\n",
        )
        .unwrap();
        let retained = open_coverage_project_root(project.path()).unwrap();
        let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());

        let config = retained_config(&retained, project.path(), &mut budget).unwrap();

        assert_eq!(config.source_dirs, ["lib"]);
        let checked_error = compute_coverage_checked(project.path(), &[], &config).unwrap_err();
        assert!(
            checked_error.contains("Cannot parse Gradle settings manifest"),
            "{checked_error}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn retained_config_rejects_a_detached_parent_before_read() {
        let project = tempfile::tempdir().unwrap();
        let original_parent = project.path().join(".specsync-original");
        fs::create_dir_all(project.path().join(".specsync")).unwrap();
        fs::write(
            project.path().join(".specsync/config.toml"),
            "source_dirs = [\"original-source\"]\n",
        )
        .unwrap();
        let retained = open_coverage_project_root(project.path()).unwrap();
        let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());
        let mut replaced = false;

        let error = retained_config_with_hook(&retained, project.path(), &mut budget, |relative| {
            if replaced || relative != Path::new(".specsync/config.toml") {
                return;
            }
            replaced = true;
            fs::rename(project.path().join(".specsync"), &original_parent).unwrap();
            fs::create_dir_all(project.path().join(".specsync")).unwrap();
            fs::write(
                project.path().join(".specsync/config.toml"),
                "source_dirs = [\"replacement-source\"]\n",
            )
            .unwrap();
        })
        .unwrap_err();

        assert!(
            error.contains("changed during retained traversal"),
            "{error}"
        );
    }

    #[test]
    fn retained_selected_specs_enforce_the_shared_entry_budget() {
        let project = tempfile::tempdir().unwrap();
        fs::create_dir_all(project.path().join("specs")).unwrap();
        let first = project.path().join("specs/first.spec.md");
        let second = project.path().join("specs/second.spec.md");
        let spec = "---\nmodule: fixture\nversion: 1\nstatus: stable\nfiles: []\n---\n";
        fs::write(&first, spec).unwrap();
        fs::write(&second, spec).unwrap();
        let retained = open_coverage_project_root(project.path()).unwrap();
        let mut inventory_budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());
        let inventory =
            discover_coverage_spec_files(&retained, "specs", &mut inventory_budget).unwrap();
        let limits = CoverageTraversalLimits {
            max_entries: 1,
            ..CoverageTraversalLimits::default()
        };
        let mut within_budget = CoverageTraversalBudget::new(limits);
        collect_specced_files(
            &retained,
            std::slice::from_ref(&inventory[0]),
            &mut within_budget,
        )
        .unwrap();

        let mut over_budget = CoverageTraversalBudget::new(limits);
        let error = collect_specced_files(&retained, &inventory, &mut over_budget).unwrap_err();

        assert!(error.contains("1-entry limit"), "{error}");
    }

    #[test]
    fn retained_spec_enumeration_is_bounded_before_returning_paths() {
        let project = tempfile::tempdir().unwrap();
        fs::create_dir_all(project.path().join("specs")).unwrap();
        fs::write(project.path().join("specs/first.spec.md"), "").unwrap();
        fs::write(project.path().join("specs/second.spec.md"), "").unwrap();
        let retained = open_coverage_project_root(project.path()).unwrap();
        let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits {
            max_entries: 1,
            ..CoverageTraversalLimits::default()
        });

        let error = discover_coverage_spec_files(&retained, "specs", &mut budget).unwrap_err();

        assert!(error.contains("1-entry limit"), "{error}");
    }

    #[test]
    fn retained_spec_inventory_applies_caller_selection_before_ownership_parsing() {
        let project = tempfile::tempdir().unwrap();
        fs::create_dir_all(project.path().join("specs")).unwrap();
        fs::create_dir_all(project.path().join("src")).unwrap();
        let first = project.path().join("specs/first.spec.md");
        let second = project.path().join("specs/second.spec.md");
        fs::write(
            &first,
            "---\nmodule: first\nversion: 1\nstatus: stable\nfiles:\n  - src/first.rs\n---\n",
        )
        .unwrap();
        fs::write(
            &second,
            "---\nmodule: second\nversion: 1\nstatus: stable\nfiles:\n  - src/second.rs\n---\n",
        )
        .unwrap();
        fs::write(project.path().join("src/first.rs"), "pub fn first() {}\n").unwrap();
        fs::write(project.path().join("src/second.rs"), "pub fn second() {}\n").unwrap();
        let config = SpecSyncConfig {
            source_dirs: vec!["src".to_string()],
            ..SpecSyncConfig::default()
        };

        let report =
            compute_coverage_checked(project.path(), std::slice::from_ref(&first), &config)
                .unwrap();

        assert_eq!(report.total_source_files, 2);
        assert_eq!(report.specced_file_count, 1);
        assert_eq!(report.unspecced_files, ["src/second.rs"]);
    }

    #[test]
    fn retained_spec_inventory_rejects_replacement_before_ownership_parse() {
        let project = tempfile::tempdir().unwrap();
        fs::create_dir_all(project.path().join("specs")).unwrap();
        let selected = project.path().join("specs/selected.spec.md");
        let replacement = project.path().join("specs/replacement.spec.md");
        fs::write(
            &selected,
            "---\nmodule: selected\nversion: 1\nstatus: stable\nfiles:\n  - src/original.rs\n---\n",
        )
        .unwrap();
        fs::write(
            &replacement,
            "---\nmodule: replacement\nversion: 1\nstatus: stable\nfiles:\n  - src/replacement.rs\n---\n",
        )
        .unwrap();
        let retained = open_coverage_project_root(project.path()).unwrap();
        let mut inventory_budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());
        let inventory =
            discover_coverage_spec_files(&retained, "specs", &mut inventory_budget).unwrap();
        let selected_inventory: Vec<CoverageSpecFile> = inventory
            .into_iter()
            .filter(|spec| spec.relative_path == Path::new("specs/selected.spec.md"))
            .collect();
        let mut read_budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());
        let mut replaced = false;

        let error = collect_specced_files_with_hook(
            &retained,
            &selected_inventory,
            &mut read_budget,
            |_| {
                if replaced {
                    return;
                }
                replaced = true;
                fs::remove_file(&selected).unwrap();
                fs::rename(&replacement, &selected).unwrap();
            },
        )
        .unwrap_err();

        assert!(
            error.contains("changed after retained inventory"),
            "{error}"
        );
    }

    #[test]
    fn retained_coverage_file_read_rejects_preopen_regular_replacement() {
        let project = tempfile::tempdir().unwrap();
        let source = project.path().join("source.rs");
        let replacement = project.path().join("replacement.rs");
        fs::write(&source, "pub fn original() {}\n").unwrap();
        fs::write(&replacement, "pub fn replacement() {}\n").unwrap();
        let retained = open_coverage_project_root(project.path()).unwrap();

        let error = read_coverage_file_with_hook(
            &retained,
            std::ffi::OsStr::new("source.rs"),
            Path::new("source.rs"),
            MAX_COVERAGE_FILE_BYTES,
            MAX_COVERAGE_INPUT_BYTES,
            MAX_COVERAGE_INPUT_BYTES,
            || {
                fs::remove_file(&source).unwrap();
                fs::rename(&replacement, &source).unwrap();
            },
        )
        .unwrap_err();

        assert!(error.contains("changed during retained open"), "{error}");
    }

    fn snapshot_with_limits(
        root: &Path,
        config: &SpecSyncConfig,
        exclude_dirs: &HashSet<String>,
        limits: CoverageTraversalLimits,
    ) -> Result<CoverageSourceSnapshot, String> {
        let project = open_coverage_project_root(root).unwrap();
        let mut budget = CoverageTraversalBudget::new(limits);
        snapshot_coverage_sources(&project, root, config, exclude_dirs, &mut budget)
    }

    #[cfg(unix)]
    #[test]
    fn broad_source_tree_succeeds_with_a_bounded_file_descriptor_limit() {
        const CHILD_ENV: &str = "SPECSYNC_VALIDATOR_BOUNDED_HANDLES_CHILD";
        if std::env::var_os(CHILD_ENV).is_none() {
            let executable = std::env::current_exe().unwrap();
            let status = std::process::Command::new("sh")
                .args([
                    "-c",
                    "ulimit -n 64; exec \"$1\" broad_source_tree_succeeds_with_a_bounded_file_descriptor_limit --nocapture",
                    "sh",
                ])
                .arg(executable)
                .env(CHILD_ENV, "1")
                .status()
                .unwrap();
            assert!(
                status.success(),
                "broad traversal failed with a 64-descriptor soft limit"
            );
            return;
        }

        let project = tempfile::tempdir().unwrap();
        let source = project.path().join("src");
        let specs = project.path().join("specs");
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(&specs).unwrap();
        for index in 0..200 {
            let module = source.join(format!("module-{index:03}"));
            fs::create_dir(&module).unwrap();
            fs::write(module.join("lib.rs"), "pub fn visible() {}\n").unwrap();
            let spec_module = specs.join(format!("module-{index:03}"));
            fs::create_dir(&spec_module).unwrap();
            fs::write(
                spec_module.join("module.spec.md"),
                "---\nmodule: broad\nversion: 1\nstatus: stable\nfiles: []\n---\n",
            )
            .unwrap();
        }
        let retained = open_coverage_project_root(project.path()).unwrap();
        let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());
        let discovered_specs =
            discover_coverage_spec_files(&retained, "specs", &mut budget).unwrap();
        assert_eq!(discovered_specs.len(), 200);
        drop(retained);

        let config = SpecSyncConfig {
            source_dirs: vec!["src".to_string()],
            ..SpecSyncConfig::default()
        };

        let snapshot = snapshot_with_limits(
            project.path(),
            &config,
            &HashSet::new(),
            CoverageTraversalLimits::default(),
        )
        .unwrap();

        assert_eq!(snapshot.files.len(), 200);
    }

    #[cfg(unix)]
    #[test]
    fn broad_configured_source_roots_bound_open_directory_handles() {
        const CHILD_ENV: &str = "SPECSYNC_VALIDATOR_BOUNDED_ROOT_HANDLES_CHILD";
        if std::env::var_os(CHILD_ENV).is_none() {
            let executable = std::env::current_exe().unwrap();
            let status = std::process::Command::new("sh")
                .args([
                    "-c",
                    "ulimit -n 64; exec \"$1\" broad_configured_source_roots_bound_open_directory_handles --nocapture",
                    "sh",
                ])
                .arg(executable)
                .env(CHILD_ENV, "1")
                .status()
                .unwrap();
            assert!(
                status.success(),
                "coverage retained handles across distinct configured roots"
            );
            return;
        }

        let project = tempfile::tempdir().unwrap();
        let mut source_dirs = Vec::new();
        for index in 0..90 {
            let source_dir = format!("source-{index:03}");
            fs::create_dir(project.path().join(&source_dir)).unwrap();
            fs::write(
                project.path().join(&source_dir).join("lib.rs"),
                format!("pub fn visible_{index:03}() {{}}\n"),
            )
            .unwrap();
            source_dirs.push(source_dir);
        }
        let config = SpecSyncConfig {
            source_dirs,
            ..SpecSyncConfig::default()
        };

        let snapshot = snapshot_with_limits(
            project.path(),
            &config,
            &HashSet::new(),
            CoverageTraversalLimits::default(),
        )
        .expect("distinct configured source roots must stay descriptor-bounded");

        assert_eq!(snapshot.files.len(), 90);
    }

    #[cfg(unix)]
    #[test]
    fn excluded_special_names_are_skipped_before_metadata() {
        use std::os::unix::fs::symlink;
        use std::os::unix::net::UnixListener;
        use std::process::Command;

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("project");
        let outside = tmp.path().join("outside");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(root.join("src/kept.rs"), "pub fn kept() {}\n").unwrap();
        symlink(&outside, root.join("src/excluded-link")).unwrap();
        let fifo = root.join("src/excluded-fifo");
        let fifo_status = Command::new("mkfifo").arg(&fifo).status().unwrap();
        assert!(fifo_status.success(), "mkfifo fixture failed");
        let _socket = match UnixListener::bind(root.join("src/excluded-socket")) {
            Ok(socket) => Some(socket),
            Err(error) if error.kind() == io::ErrorKind::PermissionDenied => None,
            Err(error) => panic!("cannot create excluded socket fixture: {error}"),
        };
        let config = SpecSyncConfig {
            source_dirs: vec!["src".to_string()],
            exclude_dirs: vec![
                "excluded-link".to_string(),
                "excluded-fifo".to_string(),
                "excluded-socket".to_string(),
            ],
            ..SpecSyncConfig::default()
        };
        let excludes = config.exclude_dirs.iter().cloned().collect();

        let report = compute_coverage_checked(&root, &[], &config).unwrap();

        assert_eq!(report.total_source_files, 1);
        assert_eq!(report.unspecced_files, ["src/kept.rs"]);
        let snapshot = snapshot_with_limits(
            &root,
            &config,
            &excludes,
            CoverageTraversalLimits::default(),
        )
        .unwrap();
        assert_eq!(snapshot.files.len(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn directly_configured_special_source_roots_are_inconclusive() {
        use std::os::unix::fs::symlink;
        use std::os::unix::net::UnixListener;
        use std::process::Command;

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("project");
        let outside = tmp.path().join("outside");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&outside).unwrap();
        symlink(&outside, root.join("linked")).unwrap();
        let fifo_status = Command::new("mkfifo")
            .arg(root.join("fifo"))
            .status()
            .unwrap();
        assert!(fifo_status.success(), "mkfifo fixture failed");
        let socket = match UnixListener::bind(root.join("socket")) {
            Ok(socket) => Some(socket),
            Err(error) if error.kind() == io::ErrorKind::PermissionDenied => None,
            Err(error) => panic!("cannot create configured socket fixture: {error}"),
        };
        let mut cases = vec![
            ("linked", "symlink or reparse point"),
            ("fifo", "regular directory"),
        ];
        if socket.is_some() {
            cases.push(("socket", "regular directory"));
        }

        for (configured, expected) in cases {
            let config = SpecSyncConfig {
                source_dirs: vec![configured.to_string()],
                exclude_dirs: vec![configured.to_string()],
                ..SpecSyncConfig::default()
            };
            let error = compute_coverage_checked(&root, &[], &config).unwrap_err();
            assert!(error.contains(expected), "{configured}: {error}");
        }
    }

    #[cfg(windows)]
    fn create_coverage_test_junction(junction: &Path, target: &Path) -> Result<(), String> {
        let unicode_path = |path: &Path| -> Result<String, String> {
            path.to_str()
                .map(str::to_string)
                .ok_or_else(|| "junction fixture paths must be valid Unicode".to_string())
        };
        let junction = unicode_path(junction)?;
        let target = unicode_path(target)?;
        let script = "$ErrorActionPreference = 'Stop'; New-Item -ItemType Junction \
                      -Path $env:SPECSYNC_TEST_JUNCTION \
                      -Target $env:SPECSYNC_TEST_TARGET | Out-Null";
        let mut unavailable = Vec::new();
        for executable in ["powershell.exe", "pwsh.exe"] {
            let output = match std::process::Command::new(executable)
                .args([
                    "-NoLogo",
                    "-NoProfile",
                    "-NonInteractive",
                    "-Command",
                    script,
                ])
                .env("SPECSYNC_TEST_JUNCTION", &junction)
                .env("SPECSYNC_TEST_TARGET", &target)
                .output()
            {
                Ok(output) => output,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    unavailable.push(executable);
                    continue;
                }
                Err(error) => {
                    return Err(format!(
                        "failed to launch {executable} junction fixture: {error}"
                    ));
                }
            };
            if output.status.success() {
                return Ok(());
            }
            return Err(format!(
                "{executable} junction fixture exited with {:?}; stdout: {}; stderr: {}",
                output.status.code(),
                String::from_utf8_lossy(&output.stdout).trim(),
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        Err(format!(
            "failed to launch a PowerShell junction fixture; unavailable executables: {}",
            unavailable.join(", ")
        ))
    }

    #[cfg(windows)]
    #[test]
    fn excluded_junction_is_skipped_but_configured_junction_is_inconclusive() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("project");
        let outside = tmp.path().join("outside");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(root.join("src/kept.rs"), "pub fn kept() {}\n").unwrap();
        let target_proof = b"outside junction target\n";
        fs::write(outside.join("junction-target-proof.txt"), target_proof).unwrap();
        let excluded_junction = root.join("src/excluded-junction");
        let configured_junction = root.join("configured-junction");
        create_coverage_test_junction(&excluded_junction, &outside)
            .unwrap_or_else(|error| panic!("failed to create excluded junction: {error}"));
        create_coverage_test_junction(&configured_junction, &outside)
            .unwrap_or_else(|error| panic!("failed to create configured junction: {error}"));
        for junction in [&excluded_junction, &configured_junction] {
            assert_eq!(
                fs::read(junction.join("junction-target-proof.txt")).unwrap(),
                target_proof,
                "{} does not resolve to the intended outside target",
                junction.display()
            );
        }

        let excluded = SpecSyncConfig {
            source_dirs: vec!["src".to_string()],
            exclude_dirs: vec!["excluded-junction".to_string()],
            ..SpecSyncConfig::default()
        };
        let report = compute_coverage_checked(&root, &[], &excluded).unwrap();
        assert_eq!(report.total_source_files, 1);

        let configured = SpecSyncConfig {
            source_dirs: vec!["configured-junction".to_string()],
            exclude_dirs: vec!["configured-junction".to_string()],
            ..SpecSyncConfig::default()
        };
        let error = compute_coverage_checked(&root, &[], &configured).unwrap_err();
        assert!(error.contains("symlink or reparse point"), "{error}");
    }

    #[cfg(unix)]
    #[test]
    fn unix_backslash_and_directory_separator_paths_do_not_collide() {
        assert_eq!(
            normalize_source_mapping("src/collision\\name.rs"),
            Some("src/collision\\name.rs".to_string())
        );
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src/collision")).unwrap();
        fs::write(
            root.join("src").join("collision\\name.rs"),
            "pub fn literal_backslash() {}\n",
        )
        .unwrap();
        fs::write(
            root.join("src/collision/name.rs"),
            "pub fn directory_separator() {}\n",
        )
        .unwrap();
        let config = SpecSyncConfig {
            source_dirs: vec!["src".to_string()],
            ..SpecSyncConfig::default()
        };

        let report = compute_coverage_checked(root, &[], &config).unwrap();

        assert_eq!(report.total_source_files, 2);
        assert!(
            report
                .unspecced_files
                .contains(&"src/collision\\name.rs".to_string())
        );
        assert!(
            report
                .unspecced_files
                .contains(&"src/collision/name.rs".to_string())
        );
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_source_name_is_inconclusive_instead_of_colliding() {
        use std::os::unix::ffi::OsStringExt;

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src")).unwrap();
        let invalid = OsString::from_vec(b"collision-\xff.rs".to_vec());
        let invalid_path = PathBuf::from("src").join(&invalid);
        assert!(coverage_relative_path_text(&invalid_path).is_err());
        if fs::write(root.join("src").join(invalid), "pub fn hidden() {}\n").is_err() {
            return;
        }
        fs::write(
            root.join("src/collision-\u{fffd}.rs"),
            "pub fn visible() {}\n",
        )
        .unwrap();
        let config = SpecSyncConfig {
            source_dirs: vec!["src".to_string()],
            ..SpecSyncConfig::default()
        };

        let error = compute_coverage_checked(root, &[], &config).unwrap_err();

        assert!(error.contains("not valid UTF-8"), "{error}");
    }

    #[test]
    fn invalid_utf8_supported_source_content_is_inconclusive() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/bad.rs"), b"pub fn visible() {}\n\xff").unwrap();
        let config = SpecSyncConfig {
            source_dirs: vec!["src".to_string()],
            ..SpecSyncConfig::default()
        };

        let error = compute_coverage_checked(root, &[], &config).unwrap_err();

        assert!(error.replace('\\', "/").contains("src/bad.rs"), "{error}");
        assert!(error.contains("not valid UTF-8"), "{error}");
    }

    #[test]
    fn coverage_traversal_enforces_per_file_and_cumulative_byte_limits() {
        assert_eq!(MAX_COVERAGE_FILE_BYTES, 8 * 1024 * 1024);
        assert_eq!(MAX_COVERAGE_INPUT_BYTES, 64 * 1024 * 1024);
        let per_file = tempfile::tempdir().unwrap();
        fs::create_dir_all(per_file.path().join("src")).unwrap();
        fs::write(per_file.path().join("src/large.rs"), b"123456789").unwrap();
        let config = SpecSyncConfig {
            source_dirs: vec!["src".to_string()],
            ..SpecSyncConfig::default()
        };
        let per_file_error = snapshot_with_limits(
            per_file.path(),
            &config,
            &HashSet::new(),
            CoverageTraversalLimits {
                max_file_bytes: 8,
                max_input_bytes: 64,
                max_entries: 100,
                max_depth: 16,
            },
        )
        .unwrap_err();
        assert!(per_file_error.contains("8 bytes per-file limit"));

        let cumulative = tempfile::tempdir().unwrap();
        fs::create_dir_all(cumulative.path().join("src")).unwrap();
        fs::write(cumulative.path().join("src/a.rs"), b"123456").unwrap();
        fs::write(cumulative.path().join("src/b.rs"), b"123456").unwrap();
        let cumulative_error = snapshot_with_limits(
            cumulative.path(),
            &config,
            &HashSet::new(),
            CoverageTraversalLimits {
                max_file_bytes: 8,
                max_input_bytes: 10,
                max_entries: 100,
                max_depth: 16,
            },
        )
        .unwrap_err();
        assert!(cumulative_error.contains("10 bytes cumulative limit"));
    }

    #[test]
    fn coverage_traversal_enforces_entry_and_depth_limits_iteratively() {
        assert_eq!(MAX_COVERAGE_ENTRIES, 100_000);
        assert_eq!(MAX_COVERAGE_DEPTH, 256);
        let entries = tempfile::tempdir().unwrap();
        fs::create_dir_all(entries.path().join("src")).unwrap();
        fs::write(entries.path().join("src/a.rs"), "").unwrap();
        fs::write(entries.path().join("src/b.rs"), "").unwrap();
        let config = SpecSyncConfig {
            source_dirs: vec!["src".to_string()],
            ..SpecSyncConfig::default()
        };
        let entry_error = snapshot_with_limits(
            entries.path(),
            &config,
            &HashSet::new(),
            CoverageTraversalLimits {
                max_file_bytes: 8,
                max_input_bytes: 16,
                max_entries: 1,
                max_depth: 16,
            },
        )
        .unwrap_err();
        assert!(entry_error.contains("1-entry limit"));

        let depth = tempfile::tempdir().unwrap();
        fs::create_dir_all(depth.path().join("src/a/b/c")).unwrap();
        fs::write(depth.path().join("src/a/b/c/deep.rs"), "").unwrap();
        let depth_error = snapshot_with_limits(
            depth.path(),
            &config,
            &HashSet::new(),
            CoverageTraversalLimits {
                max_file_bytes: 8,
                max_input_bytes: 16,
                max_entries: 100,
                max_depth: 3,
            },
        )
        .unwrap_err();
        assert!(depth_error.contains("3-component depth limit"));
    }

    #[test]
    fn configured_static_html_has_non_vacuous_coverage() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("landing")).unwrap();
        fs::create_dir_all(root.join("specs/landing")).unwrap();
        fs::write(root.join("landing/index.html"), "<main>Welcome</main>\n").unwrap();
        fs::write(root.join("landing/logo.png"), [0_u8, 1, 2]).unwrap();
        let spec_path = root.join("specs/landing/landing.spec.md");
        fs::write(
            &spec_path,
            "---\nmodule: landing\nversion: 1\nstatus: stable\nfiles:\n  - landing/index.html\n---\n\n# Landing\n",
        )
        .unwrap();
        let config = SpecSyncConfig {
            specs_dir: "specs".into(),
            source_dirs: vec!["landing".into()],
            ..SpecSyncConfig::default()
        };

        let mapped = compute_coverage(root, std::slice::from_ref(&spec_path), &config);
        assert_eq!(mapped.total_source_files, 1);
        assert_eq!(mapped.specced_file_count, 1);
        assert_eq!(mapped.file_coverage_percent(), Some(100));
        assert!(mapped.unspecced_files.is_empty());

        let unmapped = compute_coverage(root, &[], &config);
        assert_eq!(unmapped.total_source_files, 1);
        assert_eq!(unmapped.specced_file_count, 0);
        assert_eq!(unmapped.file_coverage_percent(), Some(0));
        assert_eq!(unmapped.unspecced_files, ["landing/index.html"]);
    }

    #[test]
    fn companion_scaffold_markers_are_precise_and_ignore_fenced_examples() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let directory = root.join("specs/auth");
        fs::create_dir_all(&directory).unwrap();
        let spec_path = directory.join("auth.spec.md");
        fs::write(&spec_path, "---\nmodule: auth\n---\n").unwrap();
        fs::write(
            directory.join("context.md"),
            "# Context\n\n<!-- Describe the context and motivation for this module. -->\n",
        )
        .unwrap();
        fs::write(
            directory.join("requirements.md"),
            "# Requirements\n\n```markdown\n- <!-- List measurable acceptance criteria. -->\n```\n\nThis prose discusses future acceptance criteria without using the generated marker.\n",
        )
        .unwrap();
        fs::write(
            directory.join("testing.md"),
            "# Testing\n\nList the automated tests and fixtures that protect this module.\n",
        )
        .unwrap();
        fs::write(
            directory.join("design.md"),
            "# Design\n\n## Layout\n\n- Document layout structure, responsive breakpoints, and positioning rules.\n\n## Components\n\n- Document component tree, inputs, outputs, and slots.\n\n## Tokens\n\n- Document color, spacing, typography, and state token overrides.\n\n## Assets\n\n- List icons, images, illustrations, and asset ownership.\n",
        )
        .unwrap();
        let mut result = ValidationResult::new("specs/auth/auth.spec.md".into());

        validate_companion_scaffold_markers(&spec_path, root, &mut result);

        assert_eq!(result.warnings.len(), 6, "{:?}", result.warnings);
        assert!(result.warnings.iter().any(|warning| {
            warning.contains("specs/auth/context.md:3") && warning.contains("concrete motivation")
        }));
        assert!(result.warnings.iter().any(|warning| {
            warning.contains("specs/auth/testing.md:3") && warning.contains("automated tests")
        }));
        for line in [5, 9, 13, 17] {
            assert!(
                result
                    .warnings
                    .iter()
                    .any(|warning| warning.contains(&format!("specs/auth/design.md:{line}"))),
                "missing design marker warning at line {line}: {:?}",
                result.warnings
            );
        }
        assert!(
            !result
                .warnings
                .iter()
                .any(|warning| warning.contains("requirements.md"))
        );
    }

    #[test]
    fn content_validation_skips_companions_while_path_validation_preserves_them() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let directory = root.join("specs/auth");
        fs::create_dir_all(&directory).unwrap();
        let spec_path = directory.join("auth.spec.md");
        let content =
            "---\nmodule: auth\nversion: 1\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n";
        fs::write(&spec_path, content).unwrap();
        fs::write(
            directory.join("context.md"),
            "<!-- Describe the context and motivation for this module. -->\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/auth.rs"), "pub fn authenticate() {}\n").unwrap();
        let schema_tables = HashSet::new();
        let schema_columns = HashMap::new();
        let config = SpecSyncConfig::default();

        let path_result = validate_spec(&spec_path, root, &schema_tables, &schema_columns, &config);
        let content_result = validate_spec_content(
            &spec_path,
            content,
            root,
            &schema_tables,
            &schema_columns,
            &config,
        );

        assert!(
            path_result
                .warnings
                .iter()
                .any(|warning| warning.contains("Unfilled companion scaffold marker"))
        );
        assert!(
            content_result
                .warnings
                .iter()
                .all(|warning| !warning.contains("Unfilled companion scaffold marker"))
        );
    }

    #[test]
    fn supplied_content_validation_normalizes_crlf_identically() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/auth.rs"), "pub fn authenticate() {}\n").unwrap();
        let path = root.join("specs/auth/auth.spec.md");
        let lf = "---\nmodule: auth\nversion: 1\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n";
        let crlf = lf.replace('\n', "\r\n");
        let tables = HashSet::new();
        let columns = HashMap::new();
        let config = SpecSyncConfig::default();

        let lf_result = validate_spec_content(&path, lf, root, &tables, &columns, &config);
        let crlf_result = validate_spec_content(&path, &crlf, root, &tables, &columns, &config);

        assert_eq!(crlf_result.errors, lf_result.errors);
        assert_eq!(crlf_result.warnings, lf_result.warnings);
        assert_eq!(crlf_result.notices, lf_result.notices);
    }

    #[test]
    fn supplied_content_size_policy_uses_supplied_bytes_not_path_bytes() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/auth.rs"), "pub fn authenticate() {}\n").unwrap();
        let path = root.join("specs/auth/auth.spec.md");
        let content = format!(
            "---\nmodule: auth\nversion: 1\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n{}\n",
            "x".repeat(3 * 1024)
        );
        let mut config = SpecSyncConfig::default();
        config.rules.max_spec_size_kb = Some(1);

        let result = validate_spec_content(
            &path,
            &content,
            root,
            &HashSet::new(),
            &HashMap::new(),
            &config,
        );

        assert!(
            result
                .warnings
                .iter()
                .any(|warning| warning.contains("exceeds limit of 1 KB")),
            "{:?}",
            result.warnings
        );
    }

    #[test]
    fn test_is_cross_project_ref() {
        assert!(is_cross_project_ref("corvid-labs/algochat@auth"));
        assert!(is_cross_project_ref("owner/repo@module"));
        assert!(!is_cross_project_ref("specs/auth/auth.spec.md"));
        assert!(!is_cross_project_ref("auth"));
        assert!(!is_cross_project_ref("owner/repo")); // no @
        assert!(!is_cross_project_ref("@module")); // no /
    }

    #[test]
    fn test_parse_cross_project_ref() {
        let (repo, module) = parse_cross_project_ref("corvid-labs/algochat@auth").unwrap();
        assert_eq!(repo, "corvid-labs/algochat");
        assert_eq!(module, "auth");

        assert!(parse_cross_project_ref("not-a-ref").is_none());
        assert!(parse_cross_project_ref("/@").is_none()); // empty parts
    }

    #[test]
    fn test_find_spec_files_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let files = find_spec_files(tmp.path());
        assert!(files.is_empty());
    }

    #[test]
    fn test_find_spec_files_nonexistent() {
        let files = find_spec_files(Path::new("/nonexistent/path"));
        assert!(files.is_empty());
    }

    #[test]
    fn test_find_spec_files_with_specs() {
        let tmp = tempfile::tempdir().unwrap();
        let spec_dir = tmp.path().join("auth");
        fs::create_dir_all(&spec_dir).unwrap();
        fs::write(spec_dir.join("auth.spec.md"), "---\nmodule: auth\n---\n").unwrap();
        fs::write(spec_dir.join("not-a-spec.md"), "other").unwrap();

        let files = find_spec_files(tmp.path());
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("auth.spec.md"));
    }

    #[test]
    fn test_validate_spec_missing_frontmatter() {
        let tmp = tempfile::tempdir().unwrap();
        let spec = tmp.path().join("bad.spec.md");
        fs::write(&spec, "# No frontmatter\n\nJust text.").unwrap();

        let tables = HashSet::new();
        let schema_cols = HashMap::new();
        let config = SpecSyncConfig::default();
        let result = validate_spec(&spec, tmp.path(), &tables, &schema_cols, &config);
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].contains("frontmatter"));
    }

    /// #578: a spec body carrying live markers used to pass with
    /// "✓ All required sections present" — the tool asserting a contract over
    /// text that is not a document.
    #[test]
    fn a_spec_body_with_an_unresolved_conflict_fails_validation() {
        let tmp = tempfile::tempdir().unwrap();
        let spec = tmp.path().join("calc.spec.md");
        fs::write(
            &spec,
            "---\nmodule: calc\nversion: 1\nstatus: active\nfiles: []\n---\n\n\
             ## Purpose\n\nMath.\n\n## Invariants\n\n\
             <<<<<<< HEAD\n1. Pure functions.\n=======\n1. Total on i32.\n>>>>>>> feature/other\n",
        )
        .unwrap();

        let result = validate_spec(
            &spec,
            tmp.path(),
            &HashSet::new(),
            &HashMap::new(),
            &SpecSyncConfig::default(),
        );
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("Unresolved merge conflict in spec body")
                    && e.contains("HEAD")
                    && e.contains("feature/other")),
            "{:?}",
            result.errors
        );
    }

    /// The companion false-positive guard: a spec that *documents* markers in a
    /// fenced example is a legitimate document and must still validate.
    #[test]
    fn a_spec_documenting_markers_inside_a_fence_still_validates() {
        let tmp = tempfile::tempdir().unwrap();
        let spec = tmp.path().join("merge.spec.md");
        fs::write(
            &spec,
            "---\nmodule: merge\nversion: 1\nstatus: active\nfiles: []\n---\n\n\
             ## Purpose\n\nDocuments conflict handling.\n\n## Invariants\n\n\
             ```\n<<<<<<< HEAD\nours\n=======\ntheirs\n>>>>>>> branch\n```\n",
        )
        .unwrap();

        let result = validate_spec(
            &spec,
            tmp.path(),
            &HashSet::new(),
            &HashMap::new(),
            &SpecSyncConfig::default(),
        );
        assert!(
            !result
                .errors
                .iter()
                .any(|e| e.contains("Unresolved merge conflict")),
            "{:?}",
            result.errors
        );
    }

    /// The end-to-end shape of the confirmed repro: source conflicted, spec
    /// documenting both sides. Before the fix this reported
    /// `3/3 exports documented` and passed.
    #[test]
    fn a_spec_over_conflicted_source_fails_instead_of_documenting_the_union() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("src")).unwrap();
        fs::write(
            tmp.path().join("src/calc.rs"),
            "pub fn add() {}\n\
             <<<<<<< HEAD\npub fn sub() {}\n=======\npub fn mul() {}\n>>>>>>> feature/other\n",
        )
        .unwrap();
        let spec = tmp.path().join("calc.spec.md");
        fs::write(
            &spec,
            "---\nmodule: calc\nversion: 1\nstatus: active\nfiles:\n  - src/calc.rs\n---\n\n\
             ## Purpose\n\nMath.\n\n## Public API\n\n### Exported Functions\n\n\
             | Function | Description |\n|---|---|\n\
             | `add` | adds |\n| `sub` | subtracts |\n| `mul` | multiplies |\n",
        )
        .unwrap();

        let result = validate_spec(
            &spec,
            tmp.path(),
            &HashSet::new(),
            &HashMap::new(),
            &SpecSyncConfig::default(),
        );
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("Unresolved merge conflict in source")
                    && e.contains("src/calc.rs")),
            "{:?}",
            result.errors
        );
        assert_eq!(
            result.export_summary, None,
            "no export summary may be reported over a tree that does not exist"
        );
    }

    /// The third path. `specsync issues` never calls `validate_spec` — it
    /// pre-reads bytes through `snapshot_source_file` and validates the retained
    /// snapshot. A fix that stopped at the path-based read would leave this
    /// entry point with the original union bug.
    #[test]
    fn the_snapshot_validation_path_also_refuses_a_conflicted_source() {
        let tmp = tempfile::tempdir().unwrap();
        let spec = tmp.path().join("calc.spec.md");
        let spec_content = "---\nmodule: calc\nversion: 1\nstatus: active\nfiles:\n  - src/calc.rs\n---\n\n\
             ## Purpose\n\nMath.\n\n## Public API\n\n### Exported Functions\n\n\
             | Function | Description |\n|---|---|\n\
             | `add` | adds |\n| `sub` | subtracts |\n| `mul` | multiplies |\n";

        let conflicted = "pub fn add() {}\n\
             <<<<<<< HEAD\npub fn sub() {}\n=======\npub fn mul() {}\n>>>>>>> feature/other\n";
        let mut sources = HashMap::new();
        sources.insert(
            "src/calc.rs".to_string(),
            SourceSnapshot::Present(conflicted.as_bytes().to_vec()),
        );

        let result = validate_spec_content_with_sources(
            &spec,
            spec_content,
            tmp.path(),
            &HashSet::new(),
            &HashMap::new(),
            &SpecSyncConfig::default(),
            &sources,
        );
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("Unresolved merge conflict in source")),
            "{:?}",
            result.errors
        );
        assert_eq!(result.export_summary, None);

        // Healthy control on the same entry point: a clean snapshot exporting
        // all three names still validates and still reports its summary.
        let clean = "pub fn add() {}\npub fn sub() {}\npub fn mul() {}\n";
        let mut clean_sources = HashMap::new();
        clean_sources.insert(
            "src/calc.rs".to_string(),
            SourceSnapshot::Present(clean.as_bytes().to_vec()),
        );
        let clean_result = validate_spec_content_with_sources(
            &spec,
            spec_content,
            tmp.path(),
            &HashSet::new(),
            &HashMap::new(),
            &SpecSyncConfig::default(),
            &clean_sources,
        );
        assert!(
            !clean_result
                .errors
                .iter()
                .any(|e| e.contains("Unresolved merge conflict")),
            "{:?}",
            clean_result.errors
        );
        assert_eq!(
            clean_result.export_summary,
            Some("3/3 exports documented".to_string())
        );
    }

    #[test]
    fn test_validate_spec_missing_required_fields() {
        let tmp = tempfile::tempdir().unwrap();
        let spec = tmp.path().join("partial.spec.md");
        fs::write(&spec, "---\nmodule: test\n---\n\n## Purpose\nTest\n").unwrap();

        let tables = HashSet::new();
        let schema_cols = HashMap::new();
        let config = SpecSyncConfig::default();
        let result = validate_spec(&spec, tmp.path(), &tables, &schema_cols, &config);
        // Should have errors for missing version, status, files
        assert!(result.errors.iter().any(|e| e.contains("version")));
        assert!(result.errors.iter().any(|e| e.contains("status")));
        assert!(result.errors.iter().any(|e| e.contains("files")));
    }

    #[test]
    fn test_validate_spec_missing_source_file() {
        let tmp = tempfile::tempdir().unwrap();
        let spec = tmp.path().join("missing.spec.md");
        fs::write(
            &spec,
            "---\nmodule: test\nversion: 1\nstatus: active\nfiles:\n  - src/nonexistent.ts\n---\n\n## Purpose\nTest\n## Requirements\n## Public API\n## Invariants\n## Behavioral Examples\n## Error Cases\n## Dependencies\n## Change Log\n",
        )
        .unwrap();

        let tables = HashSet::new();
        let schema_cols = HashMap::new();
        let config = SpecSyncConfig::default();
        let result = validate_spec(&spec, tmp.path(), &tables, &schema_cols, &config);
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("Source file not found"))
        );
    }

    #[test]
    fn directory_source_mapping_fails_loud_and_names_the_files_to_list() {
        // Regression (#472): a `files:` entry that resolves to a directory yields
        // zero exports, so the whole Public API comparison used to be skipped while
        // the entry still counted as an existing source file — a spec documenting
        // nothing passed `check --strict`. The directory must fail loud instead.
        let tmp = tempfile::tempdir().unwrap();
        let src_dir = tmp.path().join("src/provider");
        fs::create_dir_all(src_dir.join("nested")).unwrap();
        fs::write(src_dir.join("provider.ts"), "export class Provider {}\n").unwrap();
        fs::write(
            src_dir.join("nested/deep.ts"),
            "export function deep() {}\n",
        )
        .unwrap();

        let spec = tmp.path().join("provider.spec.md");
        fs::write(
            &spec,
            "---\nmodule: provider\nversion: 1\nstatus: stable\nfiles:\n  - src/provider\n---\n\n## Purpose\nTest\n## Public API\n## Invariants\n## Behavioral Examples\n## Error Cases\n## Dependencies\n## Change Log\n",
        )
        .unwrap();

        let tables = HashSet::new();
        let schema_cols = HashMap::new();
        let config = SpecSyncConfig::default();
        let result = validate_spec(&spec, tmp.path(), &tables, &schema_cols, &config);
        assert!(
            result
                .errors
                .iter()
                .any(|error| error.contains("`src/provider` is a directory")),
            "expected a directory mapping error, got: {:?}",
            result.errors
        );
        assert!(
            result.fixes.iter().any(|fix| {
                fix.contains("src/provider/provider.ts")
                    && fix.contains("src/provider/nested/deep.ts")
            }),
            "expected the fix to name the source files beneath the directory, got: {:?}",
            result.fixes
        );
    }

    #[test]
    fn directory_source_mapping_does_not_disturb_sibling_file_mappings() {
        // A mixed list keeps validating its file entries: the directory entry is the
        // only failure, and the real file's undocumented export is still reported.
        let tmp = tempfile::tempdir().unwrap();
        let src_dir = tmp.path().join("src/provider");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("provider.ts"), "export class Provider {}\n").unwrap();

        let spec = tmp.path().join("provider.spec.md");
        fs::write(
            &spec,
            "---\nmodule: provider\nversion: 1\nstatus: stable\nfiles:\n  - src/provider\n  - src/provider/provider.ts\n---\n\n## Purpose\nTest\n## Public API\n## Invariants\n## Behavioral Examples\n## Error Cases\n## Dependencies\n## Change Log\n",
        )
        .unwrap();

        let tables = HashSet::new();
        let schema_cols = HashMap::new();
        let config = SpecSyncConfig::default();
        let result = validate_spec(&spec, tmp.path(), &tables, &schema_cols, &config);
        assert_eq!(
            result
                .errors
                .iter()
                .filter(|error| error.contains("is a directory"))
                .count(),
            1,
            "only the directory entry may fail: {:?}",
            result.errors
        );
        assert!(
            result
                .warnings
                .iter()
                .any(|warning| warning.contains("Provider")),
            "the file entry must still be scanned for exports: {:?}",
            result.warnings
        );
    }

    #[test]
    fn draft_directory_source_mapping_is_not_a_planned_mapping_notice() {
        // A draft may map a file that does not exist yet; an existing directory is a
        // wrong-shaped mapping in any status and must not pass as a planned mapping.
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("src/provider")).unwrap();

        let spec = tmp.path().join("provider.spec.md");
        fs::write(
            &spec,
            "---\nmodule: provider\nversion: 1\nstatus: draft\nfiles:\n  - src/provider\n---\n\n## Purpose\nTest\n",
        )
        .unwrap();

        let tables = HashSet::new();
        let schema_cols = HashMap::new();
        let config = SpecSyncConfig::default();
        let result = validate_spec(&spec, tmp.path(), &tables, &schema_cols, &config);
        assert!(
            result
                .errors
                .iter()
                .any(|error| error.contains("is a directory")),
            "expected a directory mapping error for a draft, got: {:?}",
            result.errors
        );
        assert!(
            result.notices.is_empty(),
            "a directory must not be recorded as a planned mapping: {:?}",
            result.notices
        );
    }

    #[test]
    fn test_validate_spec_non_utf8_source_fails_loud() {
        // Regression: a source file that exists but is not valid UTF-8 must
        // produce an ERROR, not silently contribute zero exports (which would
        // let a spec documenting nothing pass — a silent false-PASS).
        let tmp = tempfile::tempdir().unwrap();
        let src_dir = tmp.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();
        // Valid JS export plus a stray Latin-1 byte (0xE9) → invalid UTF-8.
        fs::write(
            src_dir.join("bad.ts"),
            b"export function chargeCard() {}\n// \xE9",
        )
        .unwrap();

        let spec = tmp.path().join("bad.spec.md");
        fs::write(
            &spec,
            "---\nmodule: bad\nversion: 1\nstatus: active\nfiles:\n  - src/bad.ts\n---\n\n## Purpose\nTest\n## Requirements\n## Public API\n## Invariants\n## Behavioral Examples\n## Error Cases\n## Dependencies\n## Change Log\n",
        )
        .unwrap();

        let tables = HashSet::new();
        let schema_cols = HashMap::new();
        let config = SpecSyncConfig::default();
        let result = validate_spec(&spec, tmp.path(), &tables, &schema_cols, &config);
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("could not be read as UTF-8")),
            "expected a UTF-8 read error, got: {:?}",
            result.errors
        );
    }

    #[test]
    fn test_validate_spec_absolute_path_source_is_rejected() {
        // Security: a `files:` entry resolving OUTSIDE the project root (here an
        // absolute path into a different directory) must be rejected, and its
        // exported identifiers must never leak into the report.
        let root_tmp = tempfile::tempdir().unwrap();
        let outside_tmp = tempfile::tempdir().unwrap();
        let secret = outside_tmp.path().join("secret.ts");
        fs::write(&secret, "export const AWS_SECRET_ACCESS_KEY = \"leak\";\n").unwrap();

        let spec = root_tmp.path().join("s.spec.md");
        let body = format!(
            "---\nmodule: s\nversion: 1\nstatus: active\nfiles:\n  - {}\n---\n\n## Purpose\nx\n## Public API\n## Invariants\n## Behavioral Examples\n## Error Cases\n## Dependencies\n## Change Log\n",
            secret.display()
        );
        fs::write(&spec, body).unwrap();

        let tables = HashSet::new();
        let schema_cols = HashMap::new();
        let config = SpecSyncConfig::default();
        let result = validate_spec(&spec, root_tmp.path(), &tables, &schema_cols, &config);

        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("outside the project root")),
            "expected an out-of-root rejection, got: {:?}",
            result.errors
        );
        assert!(
            !result
                .warnings
                .iter()
                .any(|w| w.contains("AWS_SECRET_ACCESS_KEY")),
            "out-of-root export leaked into warnings: {:?}",
            result.warnings
        );
    }

    #[test]
    fn test_validate_spec_parent_escape_source_is_rejected() {
        // Security: `..` traversal that leaves the project root is rejected.
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("project");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            parent.path().join("outside.ts"),
            "export function leakMe() {}\n",
        )
        .unwrap();

        let spec = root.join("s.spec.md");
        fs::write(
            &spec,
            "---\nmodule: s\nversion: 1\nstatus: active\nfiles:\n  - ../outside.ts\n---\n\n## Purpose\nx\n## Public API\n## Invariants\n## Behavioral Examples\n## Error Cases\n## Dependencies\n## Change Log\n",
        )
        .unwrap();

        let tables = HashSet::new();
        let schema_cols = HashMap::new();
        let config = SpecSyncConfig::default();
        let result = validate_spec(&spec, &root, &tables, &schema_cols, &config);

        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("outside the project root")),
            "expected an out-of-root rejection for `..` escape, got: {:?}",
            result.errors
        );
        assert!(
            !result.warnings.iter().any(|w| w.contains("leakMe")),
            "escaped export leaked into warnings: {:?}",
            result.warnings
        );
    }

    #[test]
    fn test_validate_spec_schema_columns() {
        let tmp = tempfile::tempdir().unwrap();
        let src_dir = tmp.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("msg.ts"), "export function send() {}").unwrap();

        let spec = tmp.path().join("msg.spec.md");
        fs::write(
            &spec,
            r#"---
module: msg
version: 1
status: active
files:
  - src/msg.ts
db_tables:
  - messages
---

## Purpose
Messaging

## Requirements

### Schema: messages

| Column | Type | Constraints |
|--------|------|-------------|
| `id` | INTEGER | PRIMARY KEY |
| `content` | TEXT | NOT NULL |
| `ghost_col` | TEXT | NOT NULL |

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `send` | msg: string | void | Sends |

## Invariants
## Behavioral Examples
## Error Cases
## Dependencies
## Change Log
"#,
        )
        .unwrap();

        let mut table_names = HashSet::new();
        table_names.insert("messages".to_string());

        let mut schema_cols = HashMap::new();
        schema_cols.insert(
            "messages".to_string(),
            SchemaTable {
                columns: vec![
                    crate::schema::SchemaColumn {
                        name: "id".to_string(),
                        col_type: "INTEGER".to_string(),
                        nullable: false,
                        has_default: false,
                        is_primary_key: true,
                    },
                    crate::schema::SchemaColumn {
                        name: "content".to_string(),
                        col_type: "TEXT".to_string(),
                        nullable: false,
                        has_default: false,
                        is_primary_key: false,
                    },
                    crate::schema::SchemaColumn {
                        name: "created_at".to_string(),
                        col_type: "TEXT".to_string(),
                        nullable: false,
                        has_default: true,
                        is_primary_key: false,
                    },
                ],
            },
        );

        let config = SpecSyncConfig::default();
        let result = validate_spec(&spec, tmp.path(), &table_names, &schema_cols, &config);

        // ghost_col is in spec but not in schema → ERROR
        assert!(result.errors.iter().any(|e| e.contains("ghost_col")));
        // created_at is in schema but not in spec → WARNING
        assert!(result.warnings.iter().any(|w| w.contains("created_at")));
    }

    #[test]
    fn test_validate_spec_schema_type_mismatch() {
        let tmp = tempfile::tempdir().unwrap();
        let src_dir = tmp.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("t.ts"), "export function f() {}").unwrap();

        let spec = tmp.path().join("t.spec.md");
        fs::write(
            &spec,
            r#"---
module: t
version: 1
status: active
files:
  - src/t.ts
db_tables:
  - items
---

## Purpose
Test

## Requirements

### Schema: items

| Column | Type | Constraints |
|--------|------|-------------|
| `id` | INTEGER | PRIMARY KEY |
| `price` | TEXT | |

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `f` | | void | Does stuff |

## Invariants
## Behavioral Examples
## Error Cases
## Dependencies
## Change Log
"#,
        )
        .unwrap();

        let mut table_names = HashSet::new();
        table_names.insert("items".to_string());

        let mut schema_cols = HashMap::new();
        schema_cols.insert(
            "items".to_string(),
            SchemaTable {
                columns: vec![
                    crate::schema::SchemaColumn {
                        name: "id".to_string(),
                        col_type: "INTEGER".to_string(),
                        nullable: false,
                        has_default: false,
                        is_primary_key: true,
                    },
                    crate::schema::SchemaColumn {
                        name: "price".to_string(),
                        col_type: "REAL".to_string(),
                        nullable: true,
                        has_default: false,
                        is_primary_key: false,
                    },
                ],
            },
        );

        let config = SpecSyncConfig::default();
        let result = validate_spec(&spec, tmp.path(), &table_names, &schema_cols, &config);

        // price type mismatch: spec says TEXT, schema says REAL → WARNING
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.contains("type mismatch") && w.contains("price"))
        );
    }

    #[test]
    fn test_validate_spec_bare_dep_name_resolves_via_specs_dir() {
        let tmp = tempfile::tempdir().unwrap();

        // Create the dependency module directory
        let dep_dir = tmp.path().join("specs").join("run");
        fs::create_dir_all(&dep_dir).unwrap();
        fs::write(dep_dir.join("run.spec.md"), "# fixture content").unwrap();

        let spec = tmp.path().join("deps.spec.md");
        fs::write(
            &spec,
            "---\nmodule: deps\nversion: 1\nstatus: active\nfiles: []\ndepends_on:\n  - run\n---\n\n## Purpose\nTest\n## Requirements\n## Public API\n## Invariants\n## Behavioral Examples\n## Error Cases\n## Dependencies\n## Change Log\n",
        )
        .unwrap();

        let tables = HashSet::new();
        let schema_cols = HashMap::new();
        let config = SpecSyncConfig::default(); // specs_dir = "specs"
        let result = validate_spec(&spec, tmp.path(), &tables, &schema_cols, &config);

        // Bare module name "run" should resolve to specs/run — no dep error
        assert!(
            !result
                .errors
                .iter()
                .any(|e| e.contains("Dependency spec not found")),
            "bare dep name should resolve via specs dir: {:?}",
            result.errors
        );
    }
}

// ─── Coverage ────────────────────────────────────────────────────────────

fn collect_specced_files(
    project: &Dir,
    spec_files: &[CoverageSpecFile],
    budget: &mut CoverageTraversalBudget,
) -> Result<HashSet<String>, String> {
    collect_specced_files_with_hook(project, spec_files, budget, |_| {})
}

fn collect_specced_files_with_hook<BeforeRead>(
    project: &Dir,
    spec_files: &[CoverageSpecFile],
    budget: &mut CoverageTraversalBudget,
    mut before_read: BeforeRead,
) -> Result<HashSet<String>, String>
where
    BeforeRead: FnMut(&Path),
{
    let mut specced = HashSet::new();
    let mut observed_specs = HashSet::new();
    for spec_file in spec_files {
        let relative = &spec_file.relative_path;
        if !observed_specs.insert(relative.clone()) {
            continue;
        }
        budget.charge_entries(1)?;
        ensure_coverage_depth(relative, budget.limits.max_depth)?;
        let parent = relative.parent().unwrap_or_else(|| Path::new(""));
        let name = relative.file_name().ok_or_else(|| {
            format!(
                "Coverage spec path {} has no terminal filename",
                relative.display()
            )
        })?;
        let Some(retained_directory) = open_retained_coverage_directory(project, parent)? else {
            return Err(format!(
                "Coverage spec file {} is missing from the retained project",
                relative.display()
            ));
        };
        let directory = &retained_directory.directory;
        verify_retained_coverage_directory(project, &retained_directory)?;
        before_read(relative);
        let bytes = read_coverage_file_with_expected_identity_and_hook(
            directory,
            name,
            relative,
            budget.limits.max_file_bytes,
            budget.remaining_bytes(),
            budget.limits.max_input_bytes,
            Some(spec_file.identity),
            || {},
        )?;
        verify_retained_coverage_directory(project, &retained_directory)?;
        budget.charge_input_file(relative, bytes.len() as u64)?;
        let content = std::str::from_utf8(&bytes).map_err(|_| {
            format!(
                "Coverage spec file {} is not valid UTF-8",
                relative.display()
            )
        })?;
        let normalized = content.replace("\r\n", "\n");
        if let Some(parsed) = parse_frontmatter(&normalized) {
            if parsed.frontmatter.parsed_status() == Some(crate::types::SpecStatus::Archived) {
                continue;
            }
            for file in &parsed.frontmatter.files {
                if planned_source_path_is_safe(file)
                    && let Some(normalized) = normalize_source_mapping(file)
                {
                    specced.insert(normalized);
                }
            }
        }
    }
    Ok(specced)
}

fn select_coverage_spec_files(
    root: &Path,
    spec_files: &[PathBuf],
    retained_inventory: &[CoverageSpecFile],
) -> Result<Vec<CoverageSpecFile>, String> {
    let selected_spec_files: HashSet<PathBuf> = spec_files
        .iter()
        .map(|spec_file| coverage_relative_spec_path(root, spec_file))
        .collect::<Result<_, _>>()?;
    let retained_spec_files: Vec<CoverageSpecFile> = retained_inventory
        .iter()
        .filter(|spec| selected_spec_files.contains(&spec.relative_path))
        .cloned()
        .collect();
    if retained_spec_files.len() != selected_spec_files.len() {
        let retained_spec_set: HashSet<&PathBuf> = retained_spec_files
            .iter()
            .map(|spec| &spec.relative_path)
            .collect();
        let missing = selected_spec_files
            .iter()
            .find(|selected| !retained_spec_set.contains(selected))
            .expect("selected and retained spec counts differ");
        return Err(format!(
            "Coverage selected spec {} is missing from the retained spec inventory",
            missing.display()
        ));
    }
    Ok(retained_spec_files)
}

fn coverage_relative_spec_path(root: &Path, spec_file: &Path) -> Result<PathBuf, String> {
    let candidate = match spec_file.strip_prefix(root) {
        Ok(relative) => relative,
        Err(_) if spec_file.is_relative() => spec_file,
        Err(_) => {
            return Err(format!(
                "Coverage spec path {} must remain beneath the project root",
                spec_file.display()
            ));
        }
    };
    let mut relative = PathBuf::new();
    for component in candidate.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::Normal(name) => relative.push(name),
            std::path::Component::ParentDir
            | std::path::Component::RootDir
            | std::path::Component::Prefix(_) => {
                return Err(format!(
                    "Coverage spec path {} must remain beneath the project root",
                    spec_file.display()
                ));
            }
        }
    }
    if relative.as_os_str().is_empty() {
        return Err("Coverage spec path cannot be the project root".to_string());
    }
    Ok(relative)
}

/// Get spec module directories that actually contain a .spec.md file.
/// Empty directories (e.g. from a failed prior generation) are ignored.
fn get_spec_module_dirs(
    project: &Dir,
    configured: &str,
    budget: &mut CoverageTraversalBudget,
) -> Result<Vec<String>, String> {
    let relative = if configured == "." {
        PathBuf::new()
    } else {
        PathBuf::from(normalize_source_mapping(configured).ok_or_else(|| {
            format!("Coverage specs directory must remain beneath the project root: {configured}")
        })?)
    };
    ensure_coverage_depth(&relative, budget.limits.max_depth)?;
    let mut modules = Vec::new();
    let Some(retained_directory) = open_retained_coverage_directory(project, &relative)? else {
        return Ok(modules);
    };
    let directory = &retained_directory.directory;
    let names = read_coverage_entry_names(directory, &relative, budget)?;
    for name in names {
        let name_text = name.to_str().ok_or_else(|| {
            format!(
                "Coverage spec-module path beneath {} is not valid UTF-8",
                display_coverage_path(&relative)
            )
        })?;
        let child = relative.join(&name);
        ensure_coverage_depth(&child, budget.limits.max_depth)?;
        let metadata = directory.symlink_metadata(&name).map_err(|error| {
            format!(
                "Cannot inspect retained coverage spec-module path {}: {error}",
                child.display()
            )
        })?;
        if coverage_metadata_is_link(&metadata) {
            return Err(format!(
                "Coverage spec-module path {} must not traverse a symlink or reparse point",
                child.display()
            ));
        }
        if !metadata.is_dir() {
            continue;
        }
        let child_directory = open_coverage_child_directory(directory, &name, &child)?;
        let spec_names = read_coverage_entry_names(&child_directory, &child, budget)?;
        let mut has_spec = false;
        for spec_name in spec_names {
            let spec_name_text = spec_name.to_str().ok_or_else(|| {
                format!(
                    "Coverage spec path beneath {} is not valid UTF-8",
                    child.display()
                )
            })?;
            if !spec_name_text.ends_with(".spec.md") {
                continue;
            }
            let spec_path = child.join(&spec_name);
            ensure_coverage_depth(&spec_path, budget.limits.max_depth)?;
            let spec_metadata = child_directory
                .symlink_metadata(&spec_name)
                .map_err(|error| {
                    format!(
                        "Cannot inspect retained coverage spec path {}: {error}",
                        spec_path.display()
                    )
                })?;
            if coverage_metadata_is_link(&spec_metadata) {
                return Err(format!(
                    "Coverage spec path {} must not traverse a symlink or reparse point",
                    spec_path.display()
                ));
            }
            if spec_metadata.is_file() {
                has_spec = true;
                continue;
            }
            if !spec_metadata.is_dir() {
                return Err(format!(
                    "Coverage spec path {} must be a regular file",
                    spec_path.display()
                ));
            }
        }
        verify_coverage_child_directory(directory, &name, &child, &child_directory)?;
        if has_spec {
            modules.push(name_text.to_string());
        }
    }
    verify_retained_coverage_directory(project, &retained_directory)?;
    modules.sort();
    Ok(modules)
}

fn coverage_source_modules(
    snapshot: &CoverageSourceSnapshot,
    configured: &str,
) -> Result<Vec<String>, String> {
    snapshot
        .immediate_modules
        .get(configured)
        .into_iter()
        .flatten()
        .map(|name| {
            name.to_str()
                .map(str::to_string)
                .ok_or_else(|| "Coverage source module name is not valid UTF-8".to_string())
        })
        .collect()
}

/// A configured source directory as a `/`-joined project-relative prefix, with
/// `.` becoming the empty root prefix.
fn normalized_coverage_source_dir(configured: &str) -> Result<String, String> {
    if configured == "." {
        return Ok(String::new());
    }
    normalize_source_mapping(configured).ok_or_else(|| {
        format!("Coverage source directory must remain beneath the project root: {configured}")
    })
}

/// Join a source-directory prefix with a child directory name.
fn coverage_module_directory(source_dir: &str, child: &str) -> String {
    if source_dir.is_empty() {
        child.to_string()
    } else {
        format!("{source_dir}/{child}")
    }
}

/// How many discovered source files a candidate module owns, and how many of
/// those no spec maps.
#[derive(Clone, Copy, Debug, Default)]
struct ModuleFileOwnership {
    owned: usize,
    unmapped: usize,
}

impl ModuleFileOwnership {
    fn observe(&mut self, mapped: bool) {
        self.owned += 1;
        if !mapped {
            self.unmapped += 1;
        }
    }

    fn absorb(&mut self, other: Self) {
        self.owned += other.owned;
        self.unmapped += other.unmapped;
    }

    /// Whether this module still has a coverage gap worth reporting.
    ///
    /// Owning NO discovered source file is not evidence of coverage: nothing
    /// was measured, so the module keeps its report rather than being declared
    /// covered by default. Only a module whose files were all looked at, and
    /// all found mapped, is silent.
    fn is_uncovered(self) -> bool {
        self.owned == 0 || self.unmapped > 0
    }
}

/// Which discovered source files each candidate module name owns.
///
/// Built in one pass over the measured file list so a candidate can be answered
/// without rescanning it, and keyed only by the directories and stems a
/// candidate can actually name — an unbounded ancestor index would grow with
/// tree depth for names nothing ever asks about.
#[derive(Debug, Default)]
struct CoverageModuleOwnership {
    directories: HashMap<String, ModuleFileOwnership>,
    flat_stems: HashMap<String, HashMap<String, ModuleFileOwnership>>,
}

impl CoverageModuleOwnership {
    fn index(
        source_files: &[String],
        specced_files: &HashSet<String>,
        candidate_directories: &HashSet<String>,
        source_roots: &HashSet<String>,
    ) -> Self {
        let mut ownership = Self {
            directories: candidate_directories
                .iter()
                .map(|directory| (directory.clone(), ModuleFileOwnership::default()))
                .collect(),
            flat_stems: HashMap::new(),
        };
        for file in source_files {
            let mapped = specced_files.contains(file.as_str());
            let (parent, name) = match file.rsplit_once('/') {
                Some((parent, name)) => (parent, name),
                None => ("", file.as_str()),
            };
            if source_roots.contains(parent) {
                // Keyed with the same derivation the flat-module block uses, so
                // a stem always finds its own files.
                let stem = Path::new(name)
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or(name);
                ownership
                    .flat_stems
                    .entry(parent.to_string())
                    .or_default()
                    .entry(stem.to_string())
                    .or_default()
                    .observe(mapped);
            }
            let mut prefix = String::new();
            if let Some(entry) = ownership.directories.get_mut(&prefix) {
                entry.observe(mapped);
            }
            for component in parent.split('/').filter(|component| !component.is_empty()) {
                if !prefix.is_empty() {
                    prefix.push('/');
                }
                prefix.push_str(component);
                if let Some(entry) = ownership.directories.get_mut(&prefix) {
                    entry.observe(mapped);
                }
            }
        }
        ownership
    }

    fn directory(&self, directory: &str) -> ModuleFileOwnership {
        self.directories.get(directory).copied().unwrap_or_default()
    }

    /// Ownership across every source path a manifest declares for a module.
    /// Paths that do not normalize contribute nothing, so a module whose
    /// declared sources cannot be located keeps its report.
    fn declared_directories(&self, source_paths: &[String]) -> ModuleFileOwnership {
        let mut total = ModuleFileOwnership::default();
        for path in source_paths {
            let Some(normalized) = normalize_source_mapping(path) else {
                continue;
            };
            total.absorb(self.directory(&normalized));
        }
        total
    }

    fn flat_stem(&self, source_dir: &str, stem: &str) -> ModuleFileOwnership {
        self.flat_stems
            .get(source_dir)
            .and_then(|stems| stems.get(stem))
            .copied()
            .unwrap_or_default()
    }
}

/// Ownership over the explicit file list a configured module declares. A file
/// the traversal never discovered is not counted as owned, so a declaration
/// pointing at nothing keeps its report.
fn declared_files_ownership(
    files: &[String],
    discovered_files: &HashSet<&str>,
    specced_files: &HashSet<String>,
) -> ModuleFileOwnership {
    let mut ownership = ModuleFileOwnership::default();
    for file in files {
        let Some(normalized) = normalize_source_mapping(file) else {
            continue;
        };
        if !discovered_files.contains(normalized.as_str()) {
            continue;
        }
        ownership.observe(specced_files.contains(&normalized));
    }
    ownership
}

/// Compute file and module coverage.
#[allow(dead_code)]
pub fn compute_coverage(
    root: &Path,
    spec_files: &[PathBuf],
    config: &SpecSyncConfig,
) -> CoverageReport {
    compute_coverage_checked(root, spec_files, config).unwrap_or_else(|error| CoverageReport {
        total_source_files: 0,
        specced_file_count: 0,
        unspecced_files: Vec::new(),
        unspecced_modules: vec![format!("Coverage inconclusive: {error}")],
        total_loc: 0,
        specced_loc: 0,
        unspecced_file_loc: Vec::new(),
        missing_files: Vec::new(),
        skipped_links: Vec::new(),
    })
}

/// Compute file and module coverage while surfacing malformed manifest inputs.
pub fn compute_coverage_checked(
    root: &Path,
    spec_files: &[PathBuf],
    config: &SpecSyncConfig,
) -> Result<CoverageReport, String> {
    let project = open_coverage_project_root(root)?;
    coverage_snapshot_test_barrier(CoverageSnapshotCheckpoint::RootRetained)?;
    let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());
    if spec_files.len() > budget.limits.max_entries {
        return Err(format!(
            "Coverage selected-spec input exceeds the {}-entry limit",
            budget.limits.max_entries
        ));
    }
    let manifest = crate::manifest::discover_from_manifests_checked_with_root(root, &project)?;
    let selected_sources =
        select_coverage_source_directories(&project, config, budget.limits.max_depth)?;
    coverage_snapshot_test_barrier(CoverageSnapshotCheckpoint::ManifestDiscovered)?;
    let retained_spec_inventory =
        discover_coverage_spec_files(&project, &config.specs_dir, &mut budget)?;
    let retained_spec_files =
        select_coverage_spec_files(root, spec_files, &retained_spec_inventory)?;
    let specced_files = collect_specced_files(&project, &retained_spec_files, &mut budget)?;
    let exclude_dirs: HashSet<String> = config.exclude_dirs.iter().cloned().collect();
    let source_snapshot = snapshot_selected_coverage_sources_with_hook(
        &project,
        root,
        config,
        &exclude_dirs,
        &mut budget,
        selected_sources,
        &mut |_| {},
    )?;
    let spec_modules: HashSet<String> =
        get_spec_module_dirs(&project, &config.specs_dir, &mut budget)?
            .into_iter()
            .collect();
    verify_coverage_project_root(root, &project)?;

    let mut all_source_files: Vec<String> = Vec::new();
    let mut file_loc: HashMap<String, usize> = HashMap::new();
    for file in &source_snapshot.files {
        let rel_str = coverage_relative_path_text(&file.relative_path)?;
        let excluded = config.exclude_patterns.iter().any(|pattern| {
            if pattern.starts_with("**/") && pattern.ends_with("/**") {
                let dir_part = pattern
                    .strip_prefix("**/")
                    .and_then(|rest| rest.strip_suffix("/**"))
                    .unwrap_or("");
                rel_str.contains(dir_part)
            } else if let Some(suffix) = pattern.strip_prefix("**/") {
                if let Some(ext) = suffix.strip_prefix('*') {
                    rel_str.ends_with(ext)
                } else {
                    rel_str.ends_with(&format!("/{suffix}")) || rel_str == suffix
                }
            } else {
                rel_str.contains(pattern.as_str())
            }
        });
        if excluded {
            continue;
        }
        all_source_files.push(rel_str.clone());
        file_loc.insert(rel_str.clone(), file.loc);
    }
    all_source_files.sort();
    all_source_files.dedup();

    let total_loc: usize = file_loc.values().sum();
    let specced_loc: usize = all_source_files
        .iter()
        .filter(|f| specced_files.contains(*f))
        .map(|f| file_loc.get(f.as_str()).copied().unwrap_or(0))
        .sum();

    let unspecced_files: Vec<String> = all_source_files
        .iter()
        .filter(|f| !specced_files.contains(*f))
        .cloned()
        .collect();

    let mut unspecced_file_loc: Vec<(String, usize)> = unspecced_files
        .iter()
        .map(|f| (f.clone(), file_loc.get(f.as_str()).copied().unwrap_or(0)))
        .collect();
    unspecced_file_loc.sort_by_key(|b| std::cmp::Reverse(b.1));

    let mut unspecced_modules = Vec::new();
    let mut seen_modules: HashSet<String> = HashSet::new();

    // "This module has no spec" used to mean nothing more than "no spec
    // DIRECTORY carries this name". When language-specific specs own the files
    // — `strutil_py`, `strutil_js`, … mapping `src/strutil.{py,mjs,…}` — no
    // `specs/strutil/` is ever created, so the missing NAME was read as a
    // missing SPEC and coverage invented an uncovered module `strutil` beside
    // `5/5 files covered` (#529). Every candidate below now has to show an
    // actual gap in its own files: a module whose discovered files were all
    // looked at and all found mapped is covered, whatever the spec is called.
    // Owning no discovered file is NOT such a showing — it is the absence of
    // input, and stays reported.
    let discovered_files: HashSet<&str> = all_source_files
        .iter()
        .map(std::string::String::as_str)
        .collect();
    let source_roots: HashSet<String> = config
        .source_dirs
        .iter()
        .map(|src_dir| normalized_coverage_source_dir(src_dir))
        .collect::<Result<_, _>>()?;
    let mut candidate_directories: HashSet<String> = manifest
        .modules
        .values()
        .flat_map(|module| module.source_paths.iter())
        .filter_map(|path| normalize_source_mapping(path))
        .collect();
    for src_dir in &config.source_dirs {
        let normalized = normalized_coverage_source_dir(src_dir)?;
        for module in coverage_source_modules(&source_snapshot, src_dir)? {
            candidate_directories.insert(coverage_module_directory(&normalized, &module));
        }
    }
    let ownership = CoverageModuleOwnership::index(
        &all_source_files,
        &specced_files,
        &candidate_directories,
        &source_roots,
    );

    // User-defined modules from specsync.json take priority
    if !config.modules.is_empty() {
        for (name, definition) in &config.modules {
            if spec_modules.contains(name) {
                continue;
            }
            let files =
                declared_files_ownership(&definition.files, &discovered_files, &specced_files);
            if files.is_uncovered() && seen_modules.insert(name.clone()) {
                unspecced_modules.push(name.clone());
            }
        }
    }

    // Then: detect modules from manifest files (Package.swift, Cargo.toml, etc.)
    for (name, module) in &manifest.modules {
        if spec_modules.contains(name) {
            continue;
        }
        if ownership
            .declared_directories(&module.source_paths)
            .is_uncovered()
            && seen_modules.insert(name.clone())
        {
            unspecced_modules.push(name.clone());
        }
    }

    // Detect subdirectory-based modules. JVM language roots (`src/main/kotlin`,
    // …) hold package segments, not modules — naming from those children is
    // how a conventional Gradle tree became module `com` (#473).
    for src_dir in &config.source_dirs {
        if crate::manifest::is_jvm_package_source_root(src_dir) {
            continue;
        }
        let normalized = normalized_coverage_source_dir(src_dir)?;
        for module in coverage_source_modules(&source_snapshot, src_dir)? {
            if spec_modules.contains(&module) {
                continue;
            }
            let directory = coverage_module_directory(&normalized, &module);
            if ownership.directory(&directory).is_uncovered() && seen_modules.insert(module.clone())
            {
                unspecced_modules.push(module);
            }
        }
    }

    // Detect flat source files as modules (e.g. src/config.rs → module "config")
    let skip_stems: HashSet<&str> = ["main", "lib", "mod", "index", "__init__", "app"]
        .into_iter()
        .collect();
    for src_dir in &config.source_dirs {
        let normalized = normalized_coverage_source_dir(src_dir)?;
        let normalized_path = PathBuf::from(&normalized);
        for file in source_snapshot.files.iter().filter(|file| {
            file.relative_path.parent().unwrap_or_else(|| Path::new("")) == normalized_path
        }) {
            let Some(stem) = file
                .relative_path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .map(str::to_string)
            else {
                continue;
            };
            if skip_stems.contains(stem.as_str()) {
                continue;
            }
            if spec_modules.contains(&stem) {
                continue;
            }
            if ownership.flat_stem(&normalized, &stem).is_uncovered()
                && seen_modules.insert(stem.clone())
            {
                unspecced_modules.push(stem);
            }
        }
    }

    // Files a non-draft spec references in `files:` that do not exist on disk.
    // Draft planned mappings (status: draft, file not created yet) are intentional
    // and already surface as notices — they must not defeat `--require-coverage`.
    // Counting other missing references toward the denominator stops
    // `--require-coverage 100` from passing vacuously over broken references.
    let mut missing_files: Vec<String> = Vec::new();
    for spec_file in &retained_spec_files {
        let Ok(content) = std::fs::read_to_string(root.join(&spec_file.relative_path)) else {
            continue;
        };
        let normalized = content.replace("\r\n", "\n");
        let Some(parsed) = parse_frontmatter(&normalized) else {
            continue;
        };
        if parsed.frontmatter.parsed_status() == Some(crate::types::SpecStatus::Draft)
            && !config.require_draft_files
        {
            continue;
        }
        if parsed.frontmatter.parsed_status() == Some(crate::types::SpecStatus::Archived) {
            continue;
        }
        for file in &parsed.frontmatter.files {
            if let Some(normalized) = normalize_source_mapping(file)
                && !root.join(&normalized).exists()
            {
                missing_files.push(normalized);
            }
        }
    }
    missing_files.sort();
    missing_files.dedup();

    let specced_count = all_source_files.len() - unspecced_files.len();

    // No percentage is computed here. The denominator can be zero — an empty or
    // misconfigured `source_dirs`, an over-broad `exclude_patterns` — and any
    // value stored for that case is a fabrication. `CoverageReport` derives the
    // percentages on demand and returns `None` when there was nothing to
    // measure (#582); the previous `100` fallbacks are gone.
    Ok(CoverageReport {
        total_source_files: all_source_files.len(),
        specced_file_count: specced_count,
        unspecced_files,
        unspecced_modules,
        total_loc,
        specced_loc,
        unspecced_file_loc,
        missing_files,
        skipped_links: budget.skipped_links.iter().cloned().collect(),
    })
}

pub(crate) fn validate_local_dependency(
    dep: &str,
    root: &Path,
    specs_dir: &str,
) -> Result<PathBuf, String> {
    if is_cross_project_ref(dep) {
        // Cross-project refs (e.g. "owner/repo@module") are validated
        // by `specsync resolve`, not during local checks.
        return Ok(root.join(specs_dir));
    }
    let trimmed = dep.trim();
    if trimmed.is_empty() {
        return Err("Dependency spec entry is empty".to_string());
    }
    let as_path = Path::new(trimmed);
    if as_path.is_absolute()
        || as_path.components().any(|c| {
            matches!(
                c,
                std::path::Component::ParentDir | std::path::Component::Prefix(_)
            )
        })
        || trimmed.contains('\\')
    {
        return Err(format!(
            "Dependency spec path escapes the project root: {trimmed} (absolute paths and `..` traversal are not allowed)"
        ));
    }
    // Bare module names (no path separators or extension) resolve
    // against the specs directory, not the project root.
    let full_path = if !trimmed.contains('/') && !trimmed.contains('.') {
        root.join(specs_dir).join(trimmed)
    } else {
        root.join(trimmed)
    };
    if !source_within_root(root, trimmed) {
        return Err(format!(
            "Dependency spec path escapes the project root: {trimmed} (symlink resolution leaves the project)"
        ));
    }
    if !full_path.exists() {
        return Err(format!("Dependency spec not found: {trimmed}"));
    }
    Ok(full_path)
}
