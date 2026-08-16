use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Name of the cache directory (relative to project root).
const CACHE_DIR: &str = ".specsync";
/// Name of the hash cache file inside the cache directory.
const CACHE_FILE: &str = "hashes.json";
/// On-disk cache schema. Unknown versions are discarded conservatively.
const CACHE_FORMAT_VERSION: u32 = 1;
/// Per-spec validation snapshot schema.
const SNAPSHOT_VERSION: u32 = 1;

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
    /// Load the hash cache from disk.  Returns an empty cache if the file
    /// does not exist, cannot be parsed, or uses an unsupported schema.
    pub fn load(root: &Path) -> Self {
        let path = cache_path(root);
        match fs::read_to_string(&path) {
            Ok(contents) => match serde_json::from_str::<Self>(&contents) {
                Ok(cache) if cache.format_version == CACHE_FORMAT_VERSION => cache,
                Ok(_) | Err(_) => Self::default(),
            },
            Err(_) => Self::default(),
        }
    }

    /// Persist the cache to disk, creating the `.specsync/` directory if needed.
    pub fn save(&self, root: &Path) -> io::Result<()> {
        let dir = root.join(CACHE_DIR);
        fs::create_dir_all(&dir)?;
        let path = dir.join(CACHE_FILE);
        let json = serde_json::to_string_pretty(self)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        fs::write(path, json)
    }

    /// Compute the SHA-256 hex digest of a file's contents.
    /// Returns `None` if the file cannot be read.
    pub fn hash_file(path: &Path) -> Option<String> {
        use std::io::Read;
        let mut file = fs::File::open(path).ok()?;
        let mut hasher = Sha256::new();
        let mut buf = [0u8; 8192];
        loop {
            let n = file.read(&mut buf).ok()?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        Some(format!("{:x}", hasher.finalize()))
    }

    /// Check whether a file has changed since the last cached hash.
    /// Returns `true` if the file is new, modified, or unreadable.
    /// Whether a prior hash exists for this path — i.e. whether "changed" can
    /// be distinguished from "never seen".
    pub fn has_baseline(&self, rel_path: &str) -> bool {
        self.hashes.contains_key(rel_path)
    }

    pub fn is_changed(&self, root: &Path, rel_path: &str) -> bool {
        let current = match Self::hash_file(&root.join(rel_path)) {
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
        if let Some(hash) = Self::hash_file(&root.join(rel_path)) {
            self.hashes.insert(rel_path.to_string(), hash);
        }
    }

    /// Remove entries for files that no longer exist on disk.
    pub fn prune(&mut self, root: &Path) {
        self.hashes
            .retain(|rel_path, _| root.join(rel_path).exists());
        self.snapshots
            .retain(|rel_path, _| root.join(rel_path).exists());
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
    let mut hasher = Sha256::new();
    update_digest(
        &mut hasher,
        "cache_format_version",
        &CACHE_FORMAT_VERSION.to_string(),
    );
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
        let state = HashCache::hash_file(&root.join(&input))
            .map(|hash| format!("sha256:{hash}"))
            .unwrap_or_else(|| "missing-or-unreadable".to_string());
        update_digest(&mut hasher, "input_state", &state);
    }
    format!("{:x}", hasher.finalize())
}

/// Full path to the cache file.
fn cache_path(root: &Path) -> PathBuf {
    root.join(CACHE_DIR).join(CACHE_FILE)
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
    if let Ok(content) = fs::read_to_string(spec_path) {
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
        if let Ok(content) = fs::read_to_string(spec_path) {
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
        fs::write(
            root.join(CACHE_DIR).join(CACHE_FILE),
            r#"{"hashes":{"a":"b"}}"#,
        )
        .unwrap();
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
        update_cache(root, &[spec_path.clone()], &mut warm);
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
}
