use crate::util::local_state::LocalState;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Name of the hash cache file in the project's local state folder (REQ-hash-cache-009).
const CACHE_FILE: &str = "hashes.json";
/// The largest cache read or kept: the project input bound (REQ-hash-cache-008). A larger
/// `hashes.json` is refused like a link, so it is neither read whole nor saved over.
const MAX_CACHE_BYTES: u64 = crate::util::MAX_PROJECT_INPUT_BYTES;
/// On-disk cache schema. Unknown versions are discarded conservatively.
const CACHE_FORMAT_VERSION: u32 = 1;
/// Per-spec validation snapshot schema.
const SNAPSHOT_VERSION: u32 = 1;
/// SHA-256 of every source and manifest this binary was compiled from (`build.rs`, #878). Two
/// builds that report the same package version can still judge the same bytes differently, so a
/// cached result is replayed only by the build whose fingerprint it carries (REQ-hash-cache-005).
pub(crate) const BUILD_FINGERPRINT: &str = env!("SPECSYNC_BUILD_FINGERPRINT");

/// Normalize a relative path to use forward slashes on all platforms.
/// This ensures cache keys are consistent across Windows and Unix.
fn normalize_rel(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Complete validation result bound to the exact inputs that produced it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CachedValidationSnapshot {
    /// Snapshot schema version.
    pub snapshot_version: u32,
    /// Platform-native project-relative path used by cold validation output.
    pub spec_path: String,
    /// Validation errors without a duplicated spec-path prefix.
    pub errors: Vec<String>,
    /// Unsuppressed validation warnings without a spec-path prefix.
    pub warnings: Vec<String>,
    /// Informational notices without a spec-path prefix.
    pub notices: Vec<String>,
    /// SHA-256 binding the result to spec, companion, source, global, and inventory inputs.
    pub input_digest: String,
    /// Integrity digest over every snapshot field except this digest.
    pub snapshot_digest: String,
}

/// User-visible diagnostics captured for one spec validation.
#[derive(Debug, Clone)]
pub(crate) struct ValidationDiagnostics {
    /// Validation errors without a duplicated spec-path prefix.
    pub errors: Vec<String>,
    /// Unsuppressed validation warnings without a spec-path prefix.
    pub warnings: Vec<String>,
    /// Informational notices without a spec-path prefix.
    pub notices: Vec<String>,
}

/// Stored content hashes for spec and source files.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HashCache {
    /// On-disk cache format version.
    pub format_version: u32,
    /// Map from relative file path to its SHA-256 hex digest.
    pub hashes: BTreeMap<String, String>,
    /// Complete per-spec validation snapshots.
    #[serde(default)]
    pub snapshots: BTreeMap<String, CachedValidationSnapshot>,
}

impl CachedValidationSnapshot {
    /// Whether this snapshot carries any user-visible diagnostic.
    pub(crate) fn has_findings(&self) -> bool {
        !self.errors.is_empty() || !self.warnings.is_empty() || !self.notices.is_empty()
    }
}

impl Default for HashCache {
    fn default() -> Self {
        Self {
            format_version: CACHE_FORMAT_VERSION,
            hashes: BTreeMap::new(),
            snapshots: BTreeMap::new(),
        }
    }
}

impl HashCache {
    /// Load the hash cache from the project's local state folder outside the work tree
    /// (REQ-hash-cache-009). Returns an empty cache if the file does not exist, cannot be
    /// parsed, uses an unsupported schema, or cannot be read there: a link, anything but a
    /// regular file, or more than the bound (REQ-hash-cache-007, REQ-hash-cache-008). A
    /// `.specsync/hashes.json` in the work tree is never looked at, so a cache that came with a
    /// repository, under any spelling, can never replay a pass.
    pub fn load(root: &Path) -> Self {
        Self::read(root).unwrap_or_default()
    }

    /// The cache in the local state folder, or `None` when there is none to use.
    fn read(root: &Path) -> Option<Self> {
        let state = LocalState::open(root, false).ok()??;
        // not a project file read: SpecSync's own state folder outside the work tree, read through its no-follow, bounded handle (REQ-util-006)
        let contents = state.read_file(CACHE_FILE, MAX_CACHE_BYTES).ok()??;
        let contents = String::from_utf8(contents).ok()?;
        serde_json::from_str::<Self>(&contents)
            .ok()
            .filter(|cache| cache.format_version == CACHE_FORMAT_VERSION)
    }

    /// Persist the cache to the project's local state folder outside the work tree, making the
    /// folder if needed (REQ-hash-cache-009). The cache is written to a new file there and
    /// renamed over `hashes.json`, so nothing is ever written through what that name was. It
    /// fails, writing nothing, when no local state folder can be found or SpecSync's own folders
    /// there are not plain folders (REQ-util-006).
    pub fn save(&self, root: &Path) -> io::Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        let state = LocalState::open(root, true)?.ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "the local state folder is missing")
        })?;
        state.write_file(CACHE_FILE, json.as_bytes(), false)
    }

    /// Where the project's cache is kept, outside the work tree, for messages; `None` when no
    /// local state folder can be found.
    pub fn cache_path(root: &Path) -> Option<PathBuf> {
        crate::util::local_state::project_state_dir(root)
            .ok()
            .map(|folder| folder.join(CACHE_FILE))
    }

    /// Why this project's cache is not used, in words for the user: no local state folder can be
    /// found, or what is at `hashes.json` there is a symbolic link, not a regular file, or
    /// larger than the bound. `None` when the cache is usable or absent. `load` reads such a
    /// cache as empty and `save` refuses where it cannot write (REQ-hash-cache-007,
    /// REQ-hash-cache-009).
    pub fn refusal(root: &Path) -> Option<String> {
        let state = match LocalState::open(root, false) {
            Ok(Some(state)) => state,
            Ok(None) => return None,
            Err(error) => return Some(error.to_string()),
        };
        let shown = state.file_path(CACHE_FILE);
        match state.metadata(CACHE_FILE) {
            Ok(Some(metadata)) if metadata.file_type().is_symlink() => Some(format!(
                "{} is a symbolic link, and the hash cache is never read or written through one",
                shown.display()
            )),
            Ok(Some(metadata)) if !metadata.is_file() => {
                Some(format!("{} is not a regular file", shown.display()))
            }
            Ok(Some(metadata)) if metadata.len() > MAX_CACHE_BYTES => Some(format!(
                "{} is larger than {MAX_CACHE_BYTES} bytes",
                shown.display()
            )),
            Ok(_) => None,
            Err(error) => Some(error.to_string()),
        }
    }

    /// Compute the SHA-256 hex digest of a file's contents.
    /// Returns `None` if the file cannot be read: missing, a link out of the project, not a
    /// regular file, or larger than 8 MiB (REQ-hash-cache-008). The cache treats that as
    /// changed, which only ever causes more validation.
    pub fn hash_file(root: &Path, path: &Path) -> Option<String> {
        let content =
            crate::util::read_project_bytes(root, path, crate::util::MAX_SOURCE_FILE_BYTES).ok()?;
        Some(format!("{:x}", Sha256::digest(&content)))
    }

    /// Check whether a file has changed since the last cached hash.
    /// Returns `true` if the file is new, modified, or unreadable.
    /// Whether a prior hash exists for this path — i.e. whether "changed" can
    /// be distinguished from "never seen".
    pub fn has_baseline(&self, rel_path: &str) -> bool {
        self.hashes.contains_key(rel_path)
    }

    pub fn is_changed(&self, root: &Path, rel_path: &str) -> bool {
        let current = match Self::hash_file(root, &root.join(rel_path)) {
            Some(h) => h,
            None => return true, // unreadable → treat as changed
        };
        match self.hashes.get(rel_path) {
            Some(cached) => cached != &current,
            None => true, // new file
        }
    }

    /// Update the stored hash for a file (computes fresh hash from disk).
    pub fn update(&mut self, root: &Path, rel_path: &str) {
        if let Some(hash) = Self::hash_file(root, &root.join(rel_path)) {
            self.hashes.insert(rel_path.to_string(), hash);
        }
    }

    /// Remove entries for files that no longer exist in the project. A key is looked up only
    /// when it names a path inside the project, and only through the project rule, so a key a
    /// committed cache chose (absolute, with `..`, or behind a link out) is dropped without its
    /// path ever being looked at (REQ-hash-cache-010).
    pub fn prune(&mut self, root: &Path) {
        let present = |rel_path: &str| {
            crate::util::confine_path_to_root(root, rel_path)
                .is_some_and(|path| crate::util::is_project_file(root, &path))
        };
        self.hashes.retain(|rel_path, _| present(rel_path));
        self.snapshots.retain(|rel_path, _| present(rel_path));
    }

    /// Record one complete validation snapshot against the current inputs.
    pub(crate) fn record_validation_snapshot(
        &mut self,
        root: &Path,
        spec_path: &Path,
        global_inputs: &[String],
        spec_inventory: &[String],
        expected_input_digest: &str,
        diagnostics: ValidationDiagnostics,
    ) -> bool {
        let rel = normalize_rel(spec_path.strip_prefix(root).unwrap_or(spec_path));
        let input_digest = validation_input_digest(root, spec_path, global_inputs, spec_inventory);
        if input_digest != expected_input_digest {
            self.snapshots.remove(&rel);
            return false;
        }
        let mut snapshot = CachedValidationSnapshot {
            snapshot_version: SNAPSHOT_VERSION,
            spec_path: spec_path
                .strip_prefix(root)
                .unwrap_or(spec_path)
                .to_string_lossy()
                .to_string(),
            errors: diagnostics.errors,
            warnings: diagnostics.warnings,
            notices: diagnostics.notices,
            input_digest,
            snapshot_digest: String::new(),
        };
        snapshot.snapshot_digest = validation_snapshot_digest(&snapshot);
        self.snapshots.insert(rel, snapshot);
        true
    }

    /// Compute the exact current input digest used to guard a validation run.
    pub(crate) fn current_validation_input_digest(
        root: &Path,
        spec_path: &Path,
        global_inputs: &[String],
        spec_inventory: &[String],
    ) -> String {
        validation_input_digest(root, spec_path, global_inputs, spec_inventory)
    }

    /// Bind the just-computed diagnostics to the current inputs and store them.
    pub(crate) fn record_current_validation_snapshot(
        &mut self,
        root: &Path,
        spec_path: &Path,
        global_inputs: &[String],
        spec_inventory: &[String],
        diagnostics: ValidationDiagnostics,
    ) -> bool {
        let digest =
            Self::current_validation_input_digest(root, spec_path, global_inputs, spec_inventory);
        self.record_validation_snapshot(
            root,
            spec_path,
            global_inputs,
            spec_inventory,
            &digest,
            diagnostics,
        )
    }

    /// Return a snapshot only when its schema, integrity digest, and every
    /// validation input still match the current workspace.
    pub(crate) fn replayable_validation_snapshot(
        &self,
        root: &Path,
        spec_path: &Path,
        global_inputs: &[String],
        spec_inventory: &[String],
    ) -> Option<&CachedValidationSnapshot> {
        if self.format_version != CACHE_FORMAT_VERSION {
            return None;
        }
        let rel = normalize_rel(spec_path.strip_prefix(root).unwrap_or(spec_path));
        let snapshot = self.snapshots.get(&rel)?;
        if snapshot.snapshot_version != SNAPSHOT_VERSION
            || snapshot.snapshot_digest != validation_snapshot_digest(snapshot)
            || snapshot.input_digest
                != validation_input_digest(root, spec_path, global_inputs, spec_inventory)
        {
            return None;
        }
        Some(snapshot)
    }
}

fn update_digest(hasher: &mut Sha256, field: &str, value: &str) {
    hasher.update((field.len() as u64).to_le_bytes());
    hasher.update(field.as_bytes());
    hasher.update((value.len() as u64).to_le_bytes());
    hasher.update(value.as_bytes());
}

fn validation_snapshot_digest(snapshot: &CachedValidationSnapshot) -> String {
    let mut hasher = Sha256::new();
    update_digest(
        &mut hasher,
        "snapshot_version",
        &snapshot.snapshot_version.to_string(),
    );
    update_digest(&mut hasher, "spec_path", &snapshot.spec_path);
    update_digest(&mut hasher, "input_digest", &snapshot.input_digest);
    for error in &snapshot.errors {
        update_digest(&mut hasher, "error", error);
    }
    for warning in &snapshot.warnings {
        update_digest(&mut hasher, "warning", warning);
    }
    for notice in &snapshot.notices {
        update_digest(&mut hasher, "notice", notice);
    }
    format!("{:x}", hasher.finalize())
}

fn validation_input_digest(
    root: &Path,
    spec_path: &Path,
    global_inputs: &[String],
    spec_inventory: &[String],
) -> String {
    validation_input_digest_for_build(
        BUILD_FINGERPRINT,
        root,
        spec_path,
        global_inputs,
        spec_inventory,
    )
}

/// [`validation_input_digest`] as the build with `build_fingerprint` computes it.
fn validation_input_digest_for_build(
    build_fingerprint: &str,
    root: &Path,
    spec_path: &Path,
    global_inputs: &[String],
    spec_inventory: &[String],
) -> String {
    let mut hasher = Sha256::new();
    update_digest(
        &mut hasher,
        "cache_format_version",
        &CACHE_FORMAT_VERSION.to_string(),
    );
    update_digest(&mut hasher, "specsync_version", env!("CARGO_PKG_VERSION"));
    update_digest(&mut hasher, "build_fingerprint", build_fingerprint);

    let mut inventory = spec_inventory.to_vec();
    inventory.sort();
    inventory.dedup();
    for spec in inventory {
        update_digest(&mut hasher, "spec_inventory", &spec);
    }

    let spec_rel = normalize_rel(spec_path.strip_prefix(root).unwrap_or(spec_path));
    let mut input_paths = BTreeSet::new();
    input_paths.insert(spec_rel);
    let (requirements, companions) = find_companion_files(spec_path);
    for companion in requirements.iter().chain(companions.iter()) {
        input_paths.insert(normalize_rel(
            companion.strip_prefix(root).unwrap_or(companion),
        ));
    }
    if let Ok(content) =
        crate::util::read_project_string(root, spec_path, crate::util::MAX_SPEC_FILE_BYTES)
    {
        input_paths.extend(extract_frontmatter_files(&content));
    }
    // `check` validates the spec's history fragments (REQ-validator-048), so the verdict binds
    // the `history/` path and the name and bytes of every entry in it (REQ-hash-cache-006).
    let history = crate::change::spec_history_dir(spec_path);
    input_paths.insert(normalize_rel(
        history.strip_prefix(root).unwrap_or(&history),
    ));
    if let Ok(entries) = fs::read_dir(&history) {
        for entry in entries.flatten() {
            let path = entry.path();
            input_paths.insert(normalize_rel(path.strip_prefix(root).unwrap_or(&path)));
        }
    }
    input_paths.extend(global_inputs.iter().cloned());

    for input in input_paths {
        update_digest(&mut hasher, "input_path", &input);
        let state = HashCache::hash_file(root, &root.join(&input))
            .map(|hash| format!("sha256:{hash}"))
            .unwrap_or_else(|| "missing-or-unreadable".to_string());
        update_digest(&mut hasher, "input_state", &state);
    }
    format!("{:x}", hasher.finalize())
}

/// What kind of change was detected for a spec.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeKind {
    /// The spec file itself was modified.
    Spec,
    /// A requirements companion file changed (requirements.md or {module}.req.md).
    Requirements,
    /// A non-requirements companion file changed (context.md, tasks.md).
    Companion,
    /// One or more source files listed in frontmatter changed.
    Source,
}

impl fmt::Display for ChangeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChangeKind::Spec => write!(f, "spec"),
            ChangeKind::Requirements => write!(f, "requirements"),
            ChangeKind::Companion => write!(f, "companion"),
            ChangeKind::Source => write!(f, "source"),
        }
    }
}

/// Result of classifying changes for a single spec file.
#[derive(Debug, Clone)]
pub struct ChangeClassification {
    pub spec_path: PathBuf,
    pub changes: Vec<ChangeKind>,
    /// Whether the cache held a prior hash for this spec.
    ///
    /// `is_changed` treats an absent entry as changed, which is correct for
    /// deciding what to re-validate — with no baseline everything must be
    /// re-checked — and wrong for telling a person something drifted, because
    /// there is nothing it could have drifted from. `.specsync/hashes.json` is
    /// untracked, so CI always starts cold: without this, every CI run reported
    /// a requirements-drift warning for every spec that owns a companion.
    pub baseline_known: bool,
}

impl ChangeClassification {
    pub fn is_changed(&self) -> bool {
        !self.changes.is_empty()
    }

    pub fn has(&self, kind: &ChangeKind) -> bool {
        self.changes.contains(kind)
    }

    /// Whether `kind` is a change observed against a real baseline, and so is
    /// worth reporting rather than merely acting on.
    pub fn reportable(&self, kind: &ChangeKind) -> bool {
        self.baseline_known && self.has(kind)
    }
}

/// Companion file names to check — both the plain names (actual convention)
/// and the legacy `{module}.` prefixed names.
const COMPANION_REQ_NAMES: &[&str] = &["requirements.md"];
const COMPANION_REQ_LEGACY_SUFFIX: &str = "req.md";
const COMPANION_OTHER_NAMES: &[&str] = &["context.md", "tasks.md", "testing.md", "design.md"];
/// Legacy suffixes check for `{module}.suffix` naming. testing.md and design.md
/// are included for forward-consistency even though no legacy-named files exist yet.
const COMPANION_OTHER_LEGACY_SUFFIXES: &[&str] =
    &["context.md", "tasks.md", "testing.md", "design.md"];

/// Find all companion files for a spec, checking both naming conventions.
fn find_companion_files(spec_path: &Path) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let parent = match spec_path.parent() {
        Some(p) => p,
        None => return (vec![], vec![]),
    };
    let stem = spec_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let module = stem.strip_suffix(".spec").unwrap_or(stem);

    let mut req_files = Vec::new();
    let mut other_files = Vec::new();

    // Check plain companion names (current convention)
    for name in COMPANION_REQ_NAMES {
        let path = parent.join(name);
        if path.exists() {
            req_files.push(path);
        }
    }
    for name in COMPANION_OTHER_NAMES {
        let path = parent.join(name);
        if path.exists() {
            other_files.push(path);
        }
    }

    // Check legacy prefixed names ({module}.req.md, etc.)
    let legacy_req = parent.join(format!("{module}.{COMPANION_REQ_LEGACY_SUFFIX}"));
    if legacy_req.exists() && !req_files.contains(&legacy_req) {
        req_files.push(legacy_req);
    }
    for suffix in COMPANION_OTHER_LEGACY_SUFFIXES {
        let legacy = parent.join(format!("{module}.{suffix}"));
        if legacy.exists() && !other_files.contains(&legacy) {
            other_files.push(legacy);
        }
    }

    (req_files, other_files)
}

/// Classify what changed for a single spec file.
pub fn classify_changes(root: &Path, spec_path: &Path, cache: &HashCache) -> ChangeClassification {
    let mut changes = Vec::new();

    let rel = normalize_rel(spec_path.strip_prefix(root).unwrap_or(spec_path));

    // Check spec file itself
    if cache.is_changed(root, &rel) {
        changes.push(ChangeKind::Spec);
    }

    // Check companion files
    let (req_files, other_files) = find_companion_files(spec_path);
    for companion in &req_files {
        let comp_rel = normalize_rel(companion.strip_prefix(root).unwrap_or(companion));
        if cache.is_changed(root, &comp_rel) {
            if !changes.contains(&ChangeKind::Requirements) {
                changes.push(ChangeKind::Requirements);
            }
            break;
        }
    }
    for companion in &other_files {
        let comp_rel = normalize_rel(companion.strip_prefix(root).unwrap_or(companion));
        if cache.is_changed(root, &comp_rel) {
            if !changes.contains(&ChangeKind::Companion) {
                changes.push(ChangeKind::Companion);
            }
            break;
        }
    }

    // Check source files listed in frontmatter
    if let Ok(content) =
        crate::util::read_project_string(root, spec_path, crate::util::MAX_SPEC_FILE_BYTES)
    {
        for source_file in extract_frontmatter_files(&content) {
            if cache.is_changed(root, &source_file) {
                changes.push(ChangeKind::Source);
                break;
            }
        }
    }

    ChangeClassification {
        spec_path: spec_path.to_path_buf(),
        changes,
        baseline_known: cache.has_baseline(&rel),
    }
}

/// Filter a list of spec files down to only those whose content (or backing
/// source files) has changed since the last cached hash.
///
/// After validation, call `update_cache` with the full spec list to persist
/// the new hashes.
#[allow(dead_code)]
pub fn filter_unchanged(root: &Path, spec_files: &[PathBuf], cache: &HashCache) -> Vec<PathBuf> {
    spec_files
        .iter()
        .filter(|spec_path| classify_changes(root, spec_path, cache).is_changed())
        .cloned()
        .collect()
}

/// Classify changes for all spec files, returning only those with changes.
pub fn classify_all_changes(
    root: &Path,
    spec_files: &[PathBuf],
    cache: &HashCache,
) -> Vec<ChangeClassification> {
    spec_files
        .iter()
        .map(|spec_path| classify_changes(root, spec_path, cache))
        .filter(|c| c.is_changed())
        .collect()
}

/// After a validation run, update the cache with current hashes for all
/// spec files and their backing source files.
pub fn update_cache(root: &Path, spec_files: &[PathBuf], cache: &mut HashCache) {
    for spec_path in spec_files {
        let rel = normalize_rel(spec_path.strip_prefix(root).unwrap_or(spec_path));
        cache.update(root, &rel);

        // Update companion files (both naming conventions)
        let (req_files, other_files) = find_companion_files(spec_path);
        for companion in req_files.iter().chain(other_files.iter()) {
            let comp_rel = normalize_rel(companion.strip_prefix(root).unwrap_or(companion));
            cache.update(root, &comp_rel);
        }

        // Update source files from frontmatter
        if let Ok(content) =
            crate::util::read_project_string(root, spec_path, crate::util::MAX_SPEC_FILE_BYTES)
        {
            for source_file in extract_frontmatter_files(&content) {
                cache.update(root, &source_file);
            }
        }
    }
    cache.prune(root);
}

/// Quick extraction of the `files:` list from YAML frontmatter without
/// pulling in the full parser (avoids circular dependency).
pub fn extract_frontmatter_files(content: &str) -> Vec<String> {
    let mut files = Vec::new();
    let mut in_frontmatter = false;
    let mut in_files = false;

    for line in content.lines() {
        if line.trim() == "---" {
            if in_frontmatter {
                break; // end of frontmatter
            }
            in_frontmatter = true;
            continue;
        }
        if !in_frontmatter {
            continue;
        }
        let trimmed = line.trim();
        if trimmed.starts_with("files:") {
            in_files = true;
            continue;
        }
        if in_files {
            if let Some(item) = trimmed.strip_prefix("- ") {
                // Match the parser: a quoted entry names the path inside the
                // quotes, so the cache must key on that and not on the literal
                // `"src/a.rs"`, which no file will ever be found at.
                let item = item.trim();
                let unquoted = item
                    .strip_prefix('"')
                    .and_then(|rest| rest.split_once('"').map(|(inner, _)| inner))
                    .or_else(|| {
                        item.strip_prefix('\'')
                            .and_then(|rest| rest.split_once('\'').map(|(inner, _)| inner))
                    })
                    .unwrap_or(item);
                files.push(unquoted.to_string());
            } else if !trimmed.is_empty() && !trimmed.starts_with('-') {
                // New key — stop collecting files
                in_files = false;
            }
        }
    }
    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn cache_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        let mut cache = HashCache::default();
        cache
            .hashes
            .insert("specs/auth.spec.md".into(), "abc123".into());
        cache.save(root).unwrap();

        let loaded = HashCache::load(root);
        assert_eq!(loaded.hashes.get("specs/auth.spec.md").unwrap(), "abc123");
    }

    #[test]
    fn snapshots_round_trip_and_old_caches_revalidate() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("specs/auth")).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("specs/auth/auth.spec.md"),
            "---\nfiles:\n  - src/auth.rs\n---\n",
        )
        .unwrap();
        fs::write(root.join("src/auth.rs"), "pub fn login() {}\n").unwrap();

        let mut cache = HashCache::default();
        let spec_path = root.join("specs/auth/auth.spec.md");
        let inventory = vec!["specs/auth/auth.spec.md".to_string()];
        let input_digest =
            HashCache::current_validation_input_digest(root, &spec_path, &[], &inventory);
        cache.record_validation_snapshot(
            root,
            &spec_path,
            &[],
            &inventory,
            &input_digest,
            ValidationDiagnostics {
                errors: vec![],
                warnings: vec!["Undocumented export 'login'".into()],
                notices: vec![],
            },
        );
        cache.save(root).unwrap();

        let loaded = HashCache::load(root);
        let snapshot = loaded
            .replayable_validation_snapshot(root, &spec_path, &[], &inventory)
            .unwrap();
        assert_eq!(
            snapshot.warnings,
            vec!["Undocumented export 'login'".to_string()]
        );

        // Caches written before versioned snapshots are safely invalidated.
        fs::write(state_cache_file(root), r#"{"hashes":{"a":"b"}}"#).unwrap();
        let loaded = HashCache::load(root);
        assert!(loaded.hashes.is_empty());
        assert!(loaded.snapshots.is_empty());
        assert_eq!(loaded.format_version, CACHE_FORMAT_VERSION);
    }

    #[test]
    fn snapshot_integrity_and_input_changes_are_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("specs/auth")).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        let spec_path = root.join("specs/auth/auth.spec.md");
        fs::write(&spec_path, "---\nfiles:\n  - src/auth.rs\n---\n").unwrap();
        fs::write(root.join("src/auth.rs"), "pub fn login() {}\n").unwrap();
        let inventory = vec!["specs/auth/auth.spec.md".to_string()];

        let mut cache = HashCache::default();
        let input_digest =
            HashCache::current_validation_input_digest(root, &spec_path, &[], &inventory);
        cache.record_validation_snapshot(
            root,
            &spec_path,
            &[],
            &inventory,
            &input_digest,
            ValidationDiagnostics {
                errors: vec![],
                warnings: vec!["warning".to_string()],
                notices: vec![],
            },
        );
        assert!(
            cache
                .replayable_validation_snapshot(root, &spec_path, &[], &inventory)
                .is_some()
        );

        cache
            .snapshots
            .get_mut("specs/auth/auth.spec.md")
            .unwrap()
            .warnings
            .clear();
        assert!(
            cache
                .replayable_validation_snapshot(root, &spec_path, &[], &inventory)
                .is_none()
        );

        let input_digest =
            HashCache::current_validation_input_digest(root, &spec_path, &[], &inventory);
        cache.record_validation_snapshot(
            root,
            &spec_path,
            &[],
            &inventory,
            &input_digest,
            ValidationDiagnostics {
                errors: vec![],
                warnings: vec!["warning".to_string()],
                notices: vec![],
            },
        );
        fs::write(root.join("src/auth.rs"), "pub fn changed() {}\n").unwrap();
        assert!(
            cache
                .replayable_validation_snapshot(root, &spec_path, &[], &inventory)
                .is_none()
        );
        assert!(!cache.record_validation_snapshot(
            root,
            &spec_path,
            &[],
            &inventory,
            &input_digest,
            ValidationDiagnostics {
                errors: vec![],
                warnings: vec!["stale validation".to_string()],
                notices: vec![],
            },
        ));
        assert!(!cache.snapshots.contains_key("specs/auth/auth.spec.md"));
    }

    #[test]
    fn is_changed_detects_new_file() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::write(root.join("test.txt"), "hello").unwrap();

        let cache = HashCache::default();
        assert!(cache.is_changed(root, "test.txt"));
    }

    #[test]
    fn is_changed_detects_modification() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::write(root.join("test.txt"), "hello").unwrap();

        let mut cache = HashCache::default();
        cache.update(root, "test.txt");
        assert!(!cache.is_changed(root, "test.txt"));

        fs::write(root.join("test.txt"), "world").unwrap();
        assert!(cache.is_changed(root, "test.txt"));
    }

    #[test]
    fn extract_files_from_frontmatter() {
        let content = "---\nmodule: auth\nversion: 1\nfiles:\n  - src/auth.ts\n  - src/types.ts\ndb_tables: []\n---\n# Auth";
        let files = extract_frontmatter_files(content);
        assert_eq!(files, vec!["src/auth.ts", "src/types.ts"]);
    }

    #[test]
    fn prune_removes_missing() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::write(root.join("exists.txt"), "hi").unwrap();

        let mut cache = HashCache::default();
        cache.hashes.insert("exists.txt".into(), "aaa".into());
        cache.hashes.insert("gone.txt".into(), "bbb".into());

        cache.prune(root);
        assert!(cache.hashes.contains_key("exists.txt"));
        assert!(!cache.hashes.contains_key("gone.txt"));
    }

    fn git(root: &Path, args: &[&str]) {
        let status = std::process::Command::new("git")
            .args(args)
            .current_dir(root)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?}");
    }

    /// Verifies REQ-hash-cache-009. A cache a repository commits can carry a pass for a spec that
    /// fails: every digest in it is over public inputs. Committed or not, the work tree's
    /// `.specsync/hashes.json` is never read, never saved over, and not reported as a refusal.
    #[test]
    fn a_forged_pass_in_a_cache_git_tracks_is_never_replayed() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        git(root, &["init", "-q", "-b", "main"]);
        let (spec_path, inventory, forged) = forged_pass(root);
        fs::create_dir_all(root.join(".specsync")).unwrap();
        fs::write(
            root.join(".specsync/hashes.json"),
            serde_json::to_string_pretty(&forged).unwrap(),
        )
        .unwrap();
        git(root, &["add", "-f", ".specsync/hashes.json"]);
        let committed = fs::read(root.join(".specsync/hashes.json")).unwrap();

        let loaded = HashCache::load(root);

        assert!(
            loaded
                .replayable_validation_snapshot(root, &spec_path, &[], &inventory)
                .is_none(),
            "a tracked cache replayed a forged pass"
        );
        assert_eq!(HashCache::refusal(root), None);
        HashCache::default().save(root).unwrap();
        assert_eq!(
            fs::read(root.join(".specsync/hashes.json")).unwrap(),
            committed
        );
    }

    /// A cache that passes `specs/auth/auth.spec.md`, which fails (its file is missing), with
    /// digests anyone can compute: the spec, its inventory and the forged cache.
    fn forged_pass(root: &Path) -> (PathBuf, Vec<String>, HashCache) {
        fs::create_dir_all(root.join("specs/auth")).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        let spec_path = root.join("specs/auth/auth.spec.md");
        fs::write(&spec_path, "---\nfiles:\n  - src/missing.rs\n---\n").unwrap();
        let inventory = vec!["specs/auth/auth.spec.md".to_string()];
        let mut forged = HashCache::default();
        let input_digest =
            HashCache::current_validation_input_digest(root, &spec_path, &[], &inventory);
        forged.record_validation_snapshot(
            root,
            &spec_path,
            &[],
            &inventory,
            &input_digest,
            ValidationDiagnostics {
                errors: vec![],
                warnings: vec![],
                notices: vec![],
            },
        );
        (spec_path, inventory, forged)
    }

    /// Verifies REQ-hash-cache-009 and CHECK-1.f. A ZIP or tarball of a repository has no Git,
    /// and neither does a checkout where Git cannot answer, so nothing can say that a
    /// `.specsync/hashes.json` there came with the repository. Its forged pass is never replayed.
    #[test]
    fn a_forged_pass_an_unpacked_tree_ships_is_never_replayed() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let (spec_path, inventory, forged) = forged_pass(root);
        fs::create_dir_all(root.join(".specsync")).unwrap();
        fs::write(
            root.join(".specsync/hashes.json"),
            serde_json::to_string_pretty(&forged).unwrap(),
        )
        .unwrap();

        let loaded = HashCache::load(root);

        assert!(
            loaded
                .replayable_validation_snapshot(root, &spec_path, &[], &inventory)
                .is_none(),
            "a cache an unpacked tree shipped replayed a forged pass"
        );
    }

    /// Verifies REQ-hash-cache-009 and CHECK-1.f. On a case-insensitive disk (macOS's default),
    /// Git checks out `.specsync/Hashes.json` under that spelling, and opening `hashes.json`
    /// finds it, while Git, asked about `.specsync/hashes.json`, says it tracks nothing there.
    /// Its forged pass is never replayed. Skipped on a case-sensitive disk, where that file is
    /// simply another name.
    #[test]
    fn a_forged_pass_committed_in_another_letter_case_is_never_replayed() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::write(root.join("Probe"), "").unwrap();
        if !root.join("probe").exists() {
            return;
        }
        fs::remove_file(root.join("Probe")).unwrap();
        let (spec_path, inventory, forged) = forged_pass(root);
        fs::create_dir_all(root.join(".specsync")).unwrap();
        fs::write(
            root.join(".specsync/Hashes.json"),
            serde_json::to_string_pretty(&forged).unwrap(),
        )
        .unwrap();
        git(root, &["init", "-q", "-b", "main"]);
        git(root, &["add", "-A"]);
        git(root, &["add", "-f", ".specsync/Hashes.json"]);

        let loaded = HashCache::load(root);

        assert!(
            loaded
                .replayable_validation_snapshot(root, &spec_path, &[], &inventory)
                .is_none(),
            "a cache committed as .specsync/Hashes.json replayed a forged pass"
        );
    }

    /// Verifies REQ-hash-cache-010. `prune` keeps an entry only for a project file it can reach:
    /// a key the repository chose as an absolute path or with `..` is dropped without looking at
    /// the path it names.
    #[test]
    fn prune_drops_keys_that_leave_the_project() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("project");
        fs::create_dir_all(&root).unwrap();
        let outside = dir.path().join("outside.txt");
        fs::write(&outside, "hi").unwrap();
        fs::write(root.join("exists.txt"), "hi").unwrap();
        let mut cache = HashCache::default();
        cache.hashes.insert("exists.txt".into(), "aaa".into());
        cache
            .hashes
            .insert(outside.to_string_lossy().into_owned(), "bbb".into());
        cache.hashes.insert("../outside.txt".into(), "ccc".into());

        cache.prune(&root);

        assert!(cache.hashes.contains_key("exists.txt"));
        assert_eq!(cache.hashes.len(), 1, "{:?}", cache.hashes.keys());
    }

    #[test]
    fn classify_detects_spec_change() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let specs = root.join("specs/auth");
        fs::create_dir_all(&specs).unwrap();
        fs::write(specs.join("auth.spec.md"), "---\nmodule: auth\n---").unwrap();

        let cache = HashCache::default(); // empty = everything is new
        let result = classify_changes(root, &specs.join("auth.spec.md"), &cache);
        assert!(result.has(&ChangeKind::Spec));
    }

    #[test]
    fn classify_detects_requirements_change() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let specs = root.join("specs/auth");
        fs::create_dir_all(&specs).unwrap();
        let spec_path = specs.join("auth.spec.md");
        fs::write(&spec_path, "---\nmodule: auth\nfiles:\n---").unwrap();
        fs::write(specs.join("requirements.md"), "# Requirements v1").unwrap();

        // Cache the spec but not the requirements file
        let mut cache = HashCache::default();
        cache.update(root, "specs/auth/auth.spec.md");
        let result = classify_changes(root, &spec_path, &cache);
        assert!(!result.has(&ChangeKind::Spec));
        assert!(result.has(&ChangeKind::Requirements));
    }

    #[test]
    fn classify_detects_companion_change() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let specs = root.join("specs/auth");
        fs::create_dir_all(&specs).unwrap();
        let spec_path = specs.join("auth.spec.md");
        fs::write(&spec_path, "---\nmodule: auth\nfiles:\n---").unwrap();
        fs::write(specs.join("context.md"), "# Context").unwrap();

        let mut cache = HashCache::default();
        cache.update(root, "specs/auth/auth.spec.md");
        let result = classify_changes(root, &spec_path, &cache);
        assert!(result.has(&ChangeKind::Companion));
        assert!(!result.has(&ChangeKind::Requirements));
    }

    #[test]
    fn classify_detects_testing_companion_change() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let specs = root.join("specs/auth");
        fs::create_dir_all(&specs).unwrap();
        let spec_path = specs.join("auth.spec.md");
        fs::write(&spec_path, "---\nmodule: auth\nfiles:\n---").unwrap();
        fs::write(specs.join("testing.md"), "# Testing").unwrap();

        let mut cache = HashCache::default();
        cache.update(root, "specs/auth/auth.spec.md");
        let result = classify_changes(root, &spec_path, &cache);
        assert!(result.has(&ChangeKind::Companion));
    }

    #[test]
    fn classify_detects_source_change() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let specs = root.join("specs/auth");
        fs::create_dir_all(&specs).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        let spec_path = specs.join("auth.spec.md");
        fs::write(
            &spec_path,
            "---\nmodule: auth\nfiles:\n  - src/auth.ts\n---",
        )
        .unwrap();
        fs::write(root.join("src/auth.ts"), "export function login() {}").unwrap();

        let mut cache = HashCache::default();
        cache.update(root, "specs/auth/auth.spec.md");
        let result = classify_changes(root, &spec_path, &cache);
        assert!(result.has(&ChangeKind::Source));
    }

    #[test]
    fn companion_files_found_with_plain_names() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let specs = root.join("specs/auth");
        fs::create_dir_all(&specs).unwrap();
        fs::write(specs.join("auth.spec.md"), "").unwrap();
        fs::write(specs.join("requirements.md"), "").unwrap();
        fs::write(specs.join("context.md"), "").unwrap();
        fs::write(specs.join("tasks.md"), "").unwrap();
        fs::write(specs.join("testing.md"), "").unwrap();

        let (req, other) = find_companion_files(&specs.join("auth.spec.md"));
        assert_eq!(req.len(), 1);
        assert!(req[0].ends_with("requirements.md"));
        assert_eq!(other.len(), 3);
        assert!(other.iter().any(|p| p.ends_with("testing.md")));
    }

    #[test]
    fn update_cache_tracks_plain_companion_files() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let specs = root.join("specs/auth");
        fs::create_dir_all(&specs).unwrap();
        let spec_path = specs.join("auth.spec.md");
        fs::write(&spec_path, "---\nmodule: auth\nfiles:\n---").unwrap();
        fs::write(specs.join("requirements.md"), "# Req").unwrap();
        fs::write(specs.join("context.md"), "# Ctx").unwrap();

        let mut cache = HashCache::default();
        update_cache(root, &[spec_path], &mut cache);

        assert!(cache.hashes.contains_key("specs/auth/auth.spec.md"));
        assert!(cache.hashes.contains_key("specs/auth/requirements.md"));
        assert!(cache.hashes.contains_key("specs/auth/context.md"));
    }

    #[test]
    fn a_cold_cache_selects_for_revalidation_without_claiming_drift() {
        // #548: an absent entry is classified changed so the spec is
        // re-validated, which is right — but CI always starts cold, so
        // reporting it as drift put one phantom warning per spec in every run.
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let specs = root.join("specs/auth");
        fs::create_dir_all(&specs).unwrap();
        let spec_path = specs.join("auth.spec.md");
        fs::write(&spec_path, "---\nmodule: auth\nfiles:\n---").unwrap();
        fs::write(specs.join("requirements.md"), "# Req").unwrap();

        let cold = HashCache::default();
        let classification = classify_changes(root, &spec_path, &cold);
        assert!(
            classification.is_changed(),
            "a cold cache must still select the spec for validation"
        );
        assert!(
            !classification.baseline_known,
            "there is no baseline to have drifted from"
        );
        assert!(
            !classification.reportable(&ChangeKind::Requirements),
            "drift must not be reported without a baseline"
        );

        // Warm the cache, then make a real edit: the warning must come back.
        let mut warm = HashCache::default();
        update_cache(root, std::slice::from_ref(&spec_path), &mut warm);
        fs::write(specs.join("requirements.md"), "# Req\n\nNew criterion.\n").unwrap();
        let edited = classify_changes(root, &spec_path, &warm);
        assert!(edited.baseline_known);
        assert!(
            edited.reportable(&ChangeKind::Requirements),
            "a real requirements edit against a known baseline must still report"
        );
    }

    #[test]
    fn quoted_frontmatter_files_are_cached_under_the_real_path() {
        // #545: the cache's own `files:` extractor must agree with the parser,
        // or it keys entries on a path no file exists at.
        let files = extract_frontmatter_files(
            "---\nmodule: auth\nfiles:\n  - \"src/auth.ts\"\n  - 'src/b.ts'\n  - src/c.ts\n---\n",
        );
        assert_eq!(files, vec!["src/auth.ts", "src/b.ts", "src/c.ts"]);
    }

    /// The validation input digest every build wrote before #878: cache format and package
    /// version, then the inputs, and nothing that names the code that judged them. Frozen on
    /// purpose, because it stands for caches already on disk; do not update it with the live one.
    fn validation_input_digest_before_878(
        root: &Path,
        spec_path: &Path,
        global_inputs: &[String],
        spec_inventory: &[String],
    ) -> String {
        let mut hasher = Sha256::new();
        update_digest(&mut hasher, "cache_format_version", "1");
        update_digest(&mut hasher, "specsync_version", env!("CARGO_PKG_VERSION"));
        let mut inventory = spec_inventory.to_vec();
        inventory.sort();
        inventory.dedup();
        for spec in inventory {
            update_digest(&mut hasher, "spec_inventory", &spec);
        }
        let spec_rel = normalize_rel(spec_path.strip_prefix(root).unwrap_or(spec_path));
        let mut input_paths = BTreeSet::new();
        input_paths.insert(spec_rel);
        let (requirements, companions) = find_companion_files(spec_path);
        for companion in requirements.iter().chain(companions.iter()) {
            input_paths.insert(normalize_rel(
                companion.strip_prefix(root).unwrap_or(companion),
            ));
        }
        if let Ok(content) = fs::read_to_string(spec_path) {
            input_paths.extend(extract_frontmatter_files(&content));
        }
        input_paths.extend(global_inputs.iter().cloned());
        for input in input_paths {
            update_digest(&mut hasher, "input_path", &input);
            let state = HashCache::hash_file(root, &root.join(&input))
                .map(|hash| format!("sha256:{hash}"))
                .unwrap_or_else(|| "missing-or-unreadable".to_string());
            update_digest(&mut hasher, "input_state", &state);
        }
        format!("{:x}", hasher.finalize())
    }

    /// A spec whose `requirements.md` repeats one ID, which this build refuses (#865).
    fn duplicated_requirement_project(root: &Path) -> (PathBuf, Vec<String>) {
        fs::create_dir_all(root.join("specs/auth")).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        let spec_path = root.join("specs/auth/auth.spec.md");
        fs::write(&spec_path, "---\nfiles:\n  - src/auth.rs\n---\n").unwrap();
        fs::write(root.join("src/auth.rs"), "pub fn login() {}\n").unwrap();
        fs::write(
            root.join("specs/auth/requirements.md"),
            "# Requirements\n\n### REQ-auth-001\n\nLog in.\n\n### REQ-auth-001\n\nLog in again.\n",
        )
        .unwrap();
        (spec_path, vec!["specs/auth/auth.spec.md".to_string()])
    }

    /// A clean snapshot for the duplicated-ID project, bound to the build with `build_fingerprint`.
    fn clean_snapshot_from_build(
        build_fingerprint: &str,
        root: &Path,
        spec_path: &Path,
        inventory: &[String],
    ) -> CachedValidationSnapshot {
        let mut snapshot = CachedValidationSnapshot {
            snapshot_version: SNAPSHOT_VERSION,
            spec_path: "specs/auth/auth.spec.md".into(),
            errors: vec![],
            warnings: vec![],
            notices: vec![],
            input_digest: validation_input_digest_for_build(
                build_fingerprint,
                root,
                spec_path,
                &[],
                inventory,
            ),
            snapshot_digest: String::new(),
        };
        snapshot.snapshot_digest = validation_snapshot_digest(&snapshot);
        snapshot
    }

    /// Verifies REQ-hash-cache-005 (#878). A snapshot from another build of this same package
    /// version is not replayed, and the same snapshot from this build is: the build fingerprint,
    /// and nothing else, decides.
    #[test]
    fn only_the_build_that_recorded_a_snapshot_replays_it() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let (spec_path, inventory) = duplicated_requirement_project(root);
        assert_eq!(
            BUILD_FINGERPRINT.len(),
            64,
            "build.rs compiles in a SHA-256"
        );
        let mut cache = HashCache::default();

        cache.snapshots.insert(
            "specs/auth/auth.spec.md".into(),
            clean_snapshot_from_build(&"0".repeat(64), root, &spec_path, &inventory),
        );
        assert!(
            cache
                .replayable_validation_snapshot(root, &spec_path, &[], &inventory)
                .is_none(),
            "another build of this version must re-validate"
        );

        cache.snapshots.insert(
            "specs/auth/auth.spec.md".into(),
            clean_snapshot_from_build(BUILD_FINGERPRINT, root, &spec_path, &inventory),
        );
        assert!(
            cache
                .replayable_validation_snapshot(root, &spec_path, &[], &inventory)
                .is_some(),
            "this build replays its own snapshot"
        );
    }

    /// Verifies REQ-hash-cache-005 (#878). DISCRIMINATOR: a build from before the duplicate-ID
    /// rule, reporting this same package version, recorded a clean result for these exact bytes.
    /// Replaying it would hide the refusal this build makes until a file changed.
    #[test]
    fn a_clean_snapshot_written_before_878_does_not_hide_a_new_refusal() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let (spec_path, inventory) = duplicated_requirement_project(root);
        let mut snapshot = CachedValidationSnapshot {
            snapshot_version: SNAPSHOT_VERSION,
            spec_path: "specs/auth/auth.spec.md".into(),
            errors: vec![],
            warnings: vec![],
            notices: vec![],
            input_digest: validation_input_digest_before_878(root, &spec_path, &[], &inventory),
            snapshot_digest: String::new(),
        };
        snapshot.snapshot_digest = validation_snapshot_digest(&snapshot);
        let mut cache = HashCache::default();
        cache
            .snapshots
            .insert("specs/auth/auth.spec.md".into(), snapshot);

        assert!(
            cache
                .replayable_validation_snapshot(root, &spec_path, &[], &inventory)
                .is_none(),
            "a clean result recorded before #878 must not replay over this build's refusal"
        );
        let requirements = fs::read_to_string(root.join("specs/auth/requirements.md")).unwrap();
        assert_eq!(
            crate::validator::duplicate_requirement_errors_in(
                &requirements,
                "specs/auth/requirements.md"
            ),
            vec![
                "requirement ID `REQ-auth-001` appears 2 times in specs/auth/requirements.md"
                    .to_string()
            ],
            "re-validation is what reports the refusal the replay would have hidden"
        );
    }

    /// Verifies REQ-hash-cache-006 (#1084). `check` validates a spec's history fragments, so a
    /// snapshot is replayed only while every entry of its `history/` directory is as it was:
    /// adding, editing or removing a fragment re-validates the spec.
    #[test]
    fn a_snapshot_is_not_replayed_over_a_changed_history_fragment() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let (spec_path, inventory) = duplicated_requirement_project(root);
        let replays = |snapshot: &CachedValidationSnapshot| {
            let mut cache = HashCache::default();
            cache
                .snapshots
                .insert("specs/auth/auth.spec.md".into(), snapshot.clone());
            cache
                .replayable_validation_snapshot(root, &spec_path, &[], &inventory)
                .is_some()
        };
        let fragment = root.join("specs/auth/history/add-passkeys.md");

        let before = clean_snapshot_from_build(BUILD_FINGERPRINT, root, &spec_path, &inventory);
        assert!(replays(&before), "nothing changed, so the snapshot replays");
        fs::create_dir_all(fragment.parent().unwrap()).unwrap();
        fs::write(
            &fragment,
            "---\nchange: add-passkeys\ndate: 2026-11-19\n---\n\nAdd\n",
        )
        .unwrap();
        assert!(!replays(&before), "an added fragment re-validates");

        let added = clean_snapshot_from_build(BUILD_FINGERPRINT, root, &spec_path, &inventory);
        assert!(replays(&added));
        fs::write(
            &fragment,
            "---\nchange: add-passkeys\ndate: not-a-date\n---\n\nAdd\n",
        )
        .unwrap();
        assert!(!replays(&added), "an edited fragment re-validates");

        let edited = clean_snapshot_from_build(BUILD_FINGERPRINT, root, &spec_path, &inventory);
        fs::remove_file(&fragment).unwrap();
        assert!(!replays(&edited), "a removed fragment re-validates");
    }

    /// A project beside an `outside` directory, both inside one temporary directory, so a link
    /// that leaves the project still reaches nothing but this fixture (REQ-hash-cache-007).
    #[cfg(unix)]
    fn project_beside_outside() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("project");
        let outside = dir.path().join("outside");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&outside).unwrap();
        (dir, root, outside)
    }

    #[cfg(unix)]
    fn cache_with_one_hash() -> HashCache {
        let mut cache = HashCache::default();
        cache
            .hashes
            .insert("specs/auth.spec.md".into(), "abc123".into());
        cache
    }

    /// The names in `directory`, sorted.
    #[cfg(unix)]
    fn entry_names(directory: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(directory)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    /// The cache file in the project's local state folder, which is made (REQ-hash-cache-009).
    fn state_cache_file(root: &Path) -> PathBuf {
        LocalState::open(root, true)
            .unwrap()
            .unwrap()
            .file_path(CACHE_FILE)
    }

    /// Verifies REQ-hash-cache-007 and REQ-hash-cache-009 (GHSA-7q5v-8jcc-rx54). A repository can
    /// commit `.specsync/hashes.json` as a link to a file of whoever clones it. The cache lives
    /// outside the work tree, so that link is never followed, left as it is, and its target keeps
    /// its bytes.
    #[cfg(unix)]
    #[test]
    fn a_link_at_the_work_tree_cache_path_is_never_followed() {
        let (_dir, root, outside) = project_beside_outside();
        let victim = outside.join("victim.txt");
        fs::write(&victim, "the owner's bytes\n").unwrap();
        fs::create_dir_all(root.join(".specsync")).unwrap();
        let link = root.join(".specsync/hashes.json");
        std::os::unix::fs::symlink("../../outside/victim.txt", &link).unwrap();

        cache_with_one_hash().save(&root).unwrap();

        assert_eq!(
            fs::read_to_string(&victim).unwrap(),
            "the owner's bytes\n",
            "the cache was written through the link, over a file outside the project"
        );
        assert!(
            fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(entry_names(&outside), ["victim.txt"]);
        assert_eq!(HashCache::refusal(&root), None);
        assert_eq!(
            HashCache::load(&root)
                .hashes
                .get("specs/auth.spec.md")
                .unwrap(),
            "abc123"
        );
    }

    /// Verifies REQ-hash-cache-007 and REQ-hash-cache-009. A dangling link at the work tree's
    /// cache path, or a linked `.specsync`, never receives the cache.
    #[cfg(unix)]
    #[test]
    fn save_creates_nothing_through_a_dangling_link_or_a_linked_specsync() {
        let (_dir, root, outside) = project_beside_outside();
        let target = outside.join("not-there-yet.json");
        fs::create_dir_all(root.join(".specsync")).unwrap();
        std::os::unix::fs::symlink(&target, root.join(".specsync/hashes.json")).unwrap();
        cache_with_one_hash().save(&root).unwrap();
        assert!(
            fs::symlink_metadata(&target).is_err(),
            "the cache was created at the link's target, outside the project"
        );

        let (_dir, root, outside) = project_beside_outside();
        std::os::unix::fs::symlink(&outside, root.join(".specsync")).unwrap();
        cache_with_one_hash().save(&root).unwrap();
        assert!(
            entry_names(&outside).is_empty(),
            "the cache was written into a directory outside the project: {:?}",
            entry_names(&outside)
        );
    }

    /// Verifies REQ-hash-cache-007 and REQ-hash-cache-009. A valid cache behind a link in the
    /// work tree is not this project's cache, so it is not read.
    #[cfg(unix)]
    #[test]
    fn load_reads_no_cache_through_a_link() {
        let valid = serde_json::to_string(&cache_with_one_hash()).unwrap();

        let (_dir, root, outside) = project_beside_outside();
        fs::write(outside.join("elsewhere.json"), &valid).unwrap();
        fs::create_dir_all(root.join(".specsync")).unwrap();
        std::os::unix::fs::symlink(
            outside.join("elsewhere.json"),
            root.join(".specsync/hashes.json"),
        )
        .unwrap();
        assert!(
            HashCache::load(&root).hashes.is_empty(),
            "a cache was read through a linked cache file"
        );

        let (_dir, root, outside) = project_beside_outside();
        fs::write(outside.join(CACHE_FILE), &valid).unwrap();
        std::os::unix::fs::symlink(&outside, root.join(".specsync")).unwrap();
        assert!(
            HashCache::load(&root).hashes.is_empty(),
            "a cache was read through a linked cache directory"
        );

        // In the local state folder, SpecSync only ever renames a file over the name, so a link
        // found there is not read either, and is said.
        let (_dir, root, outside) = project_beside_outside();
        fs::write(outside.join("elsewhere.json"), &valid).unwrap();
        std::os::unix::fs::symlink(outside.join("elsewhere.json"), state_cache_file(&root))
            .unwrap();
        assert!(HashCache::load(&root).hashes.is_empty());
        assert!(
            HashCache::refusal(&root)
                .unwrap()
                .ends_with("hashes.json is a symbolic link, and the hash cache is never read or written through one")
        );
    }

    /// Verifies REQ-hash-cache-007. Anything at the cache's name in the local state folder that
    /// is not a regular file is refused, not only a link.
    #[cfg(unix)]
    #[test]
    fn a_directory_at_the_cache_path_is_refused() {
        let (_dir, root, _outside) = project_beside_outside();
        let cache_file = state_cache_file(&root);
        fs::create_dir_all(&cache_file).unwrap();

        assert!(cache_with_one_hash().save(&root).is_err());
        assert!(HashCache::load(&root).hashes.is_empty());
        assert!(cache_file.is_dir());
        assert!(
            HashCache::refusal(&root)
                .unwrap()
                .ends_with("hashes.json is not a regular file")
        );
    }

    /// Verifies REQ-hash-cache-007. The cache is replaced by a rename, never written in place: a
    /// second name for the old file keeps the old bytes, and no temporary file is left behind.
    #[cfg(unix)]
    #[test]
    fn save_replaces_the_cache_by_rename() {
        let (_dir, root, outside) = project_beside_outside();
        assert_eq!(
            HashCache::refusal(&root),
            None,
            "an absent cache is not refused"
        );
        HashCache::default().save(&root).unwrap();
        assert_eq!(
            HashCache::refusal(&root),
            None,
            "a regular cache is not refused"
        );
        let cache_file = state_cache_file(&root);
        let before = fs::read_to_string(&cache_file).unwrap();
        let other_name = outside.join("other-name.json");
        fs::hard_link(&cache_file, &other_name).unwrap();

        cache_with_one_hash().save(&root).unwrap();

        assert_eq!(
            fs::read_to_string(&other_name).unwrap(),
            before,
            "the old cache file was written in place, through its other name"
        );
        assert_eq!(
            HashCache::load(&root)
                .hashes
                .get("specs/auth.spec.md")
                .unwrap(),
            "abc123"
        );
        assert_eq!(entry_names(cache_file.parent().unwrap()), [CACHE_FILE]);
        assert!(
            !root.join(".specsync").exists(),
            "the work tree got a cache"
        );
    }
}
