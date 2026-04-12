use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Name of the cache directory (relative to project root).
const CACHE_DIR: &str = ".specsync";
/// Name of the hash cache file inside the cache directory.
const CACHE_FILE: &str = "hashes.json";

/// Normalize a relative path to use forward slashes on all platforms.
/// This ensures cache keys are consistent across Windows and Unix.
fn normalize_rel(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Stored content hashes for spec and source files.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct HashCache {
    /// Map from relative file path to its SHA-256 hex digest.
    pub hashes: HashMap<String, String>,
}

impl HashCache {
    /// Load the hash cache from disk.  Returns an empty cache if the file
    /// does not exist or cannot be parsed.
    pub fn load(root: &Path) -> Self {
        let path = cache_path(root);
        match fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
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
    }
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
}

impl ChangeClassification {
    pub fn is_changed(&self) -> bool {
        !self.changes.is_empty()
    }

    pub fn has(&self, kind: &ChangeKind) -> bool {
        self.changes.contains(kind)
    }
}

/// Companion file names to check — both the plain names (actual convention)
/// and the legacy `{module}.` prefixed names.
const COMPANION_REQ_NAMES: &[&str] = &["requirements.md"];
const COMPANION_REQ_LEGACY_SUFFIX: &str = "req.md";
const COMPANION_OTHER_NAMES: &[&str] = &["context.md", "tasks.md", "testing.md"];
/// Legacy suffixes check for `{module}.suffix` naming. testing.md is included
/// for forward-consistency even though no legacy-named testing files exist yet.
const COMPANION_OTHER_LEGACY_SUFFIXES: &[&str] = &["context.md", "tasks.md", "testing.md"];

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
                files.push(item.trim().to_string());
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
}
