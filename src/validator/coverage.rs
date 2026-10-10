use super::mappings::normalize_source_mapping;
use crate::exports::has_configured_extension;
use crate::parser::parse_frontmatter;
use crate::types::{CoverageReport, SpecSyncConfig};
use access::{
    CoverageEntryIdentity, coverage_relative_path_text, open_coverage_project_root,
    verify_coverage_project_root,
};
use cap_std::fs::Dir;
use configuration::retained_config;
use ownership::{
    CoverageModuleOwnership, coverage_module_directory, coverage_source_modules,
    declared_files_ownership, normalized_coverage_source_dir,
};
use sources::{
    display_coverage_byte_limit, select_coverage_source_directories,
    snapshot_selected_coverage_sources_with_hook,
};
use specs::{
    collect_specced_files, discover_coverage_spec_files, get_spec_module_dirs,
    select_coverage_spec_files,
};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use test_barriers::{CoverageSnapshotCheckpoint, coverage_snapshot_test_barrier};

mod access;
mod configuration;
mod ownership;
mod sources;
mod specs;
mod test_barriers;

const STATIC_COVERAGE_EXTENSIONS: &[&str] = &["html", "htm", "css"];
// Keep checked CLI coverage within the same source-input envelope as MCP snapshots.
const MAX_COVERAGE_FILE_BYTES: u64 = 8 * 1024 * 1024;
const MAX_COVERAGE_INPUT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_COVERAGE_ENTRIES: usize = 100_000;
const MAX_COVERAGE_DEPTH: usize = 256;

/// Load CLI discovery inputs through one retained project-root capability.
///
/// Configuration bytes, manifest-based source autodetection, fallback source
/// scanning, and spec enumeration all remain beneath the same retained root.
pub(super) fn load_config_and_discover_retained(
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

/// Compute file and module coverage.
#[allow(dead_code)]
pub(super) fn compute_coverage(
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
        manifest_notices: Vec::new(),
    })
}

/// Retained manifest discovery for coverage, and what a failure of it means.
///
/// Discovery exists to INFER a source list the project did not state, and to
/// name the modules a manifest declares. Which of those a given project is
/// actually relying on decides whether a manifest that cannot be parsed is a
/// verdict or a note:
///
/// - **`source_dirs` omitted** — the list coverage would measure came out of
///   this discovery. There is nothing trustworthy to measure over, so the error
///   propagates and the command is inconclusive, exactly as before.
/// - **`source_dirs` stated** — the user already said where the sources are.
///   A failure to infer what they stated cannot overrule them: it costs only
///   manifest-declared module names, so it degrades to a notice and coverage
///   proceeds over the declared list.
///
/// The second case used to abort too. A Gradle project with an ordinary in-repo
/// `includeBuild("vendor/…")` could not run `check --strict` or `coverage` at
/// all — on any 6.0 candidate — even with `source_dirs` explicitly configured,
/// because an input the parser could not read was treated as a verdict about
/// the project (#723). Fixing only the parser would have left the class intact:
/// any manifest, in any ecosystem, that a future parser cannot read would again
/// override an explicit declaration.
///
/// The notice is not optional. Manifest modules seed module attribution, so a
/// degraded run reports fewer modules without specs than the tree has — a
/// report that improved because part of the measurement stopped. It travels
/// with the figures rather than to stderr, because stderr is precisely what a
/// CI job capturing stdout does not read (#570).
fn retained_coverage_manifest(
    root: &Path,
    project: &Dir,
    config: &SpecSyncConfig,
) -> Result<(crate::manifest::ManifestDiscovery, Vec<String>), String> {
    match crate::manifest::discover_from_manifests_checked_with_root(root, project) {
        Ok(discovery) => Ok((discovery, Vec::new())),
        Err(error) if config.source_dirs_set => Ok((
            crate::manifest::ManifestDiscovery::default(),
            vec![format!(
                "Manifest discovery was skipped ({error}); coverage used the configured source_dirs, \
                 so modules declared only by that manifest are not reported"
            )],
        )),
        Err(error) => Err(error),
    }
}

/// Compute file and module coverage while surfacing malformed manifest inputs.
pub(super) fn compute_coverage_checked(
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
    let (manifest, manifest_notices) = retained_coverage_manifest(root, &project, config)?;
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
        manifest_notices,
    })
}
