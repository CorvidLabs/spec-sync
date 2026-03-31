use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Name of the cache directory (relative to project root).
const CACHE_DIR: &str = ".specsync";
/// Name of the hash cache file inside the cache directory.
const CACHE_FILE: &str = "hashes.json";

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
        let json = serde_json::to_string_pretty(self)?;
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
        Some(hasher.hex_digest())
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
        self.hashes.retain(|rel_path, _| root.join(rel_path).exists());
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
const COMPANION_OTHER_NAMES: &[&str] = &["context.md", "tasks.md"];
const COMPANION_OTHER_LEGACY_SUFFIXES: &[&str] = &["context.md", "tasks.md"];

/// Find all companion files for a spec, checking both naming conventions.
fn find_companion_files(spec_path: &Path) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let parent = match spec_path.parent() {
        Some(p) => p,
        None => return (vec![], vec![]),
    };
    let stem = spec_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
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

    let rel = spec_path
        .strip_prefix(root)
        .unwrap_or(spec_path)
        .to_string_lossy()
        .to_string();

    // Check spec file itself
    if cache.is_changed(root, &rel) {
        changes.push(ChangeKind::Spec);
    }

    // Check companion files
    let (req_files, other_files) = find_companion_files(spec_path);
    for companion in &req_files {
        let comp_rel = companion
            .strip_prefix(root)
            .unwrap_or(companion)
            .to_string_lossy()
            .to_string();
        if cache.is_changed(root, &comp_rel) {
            if !changes.contains(&ChangeKind::Requirements) {
                changes.push(ChangeKind::Requirements);
            }
            break;
        }
    }
    for companion in &other_files {
        let comp_rel = companion
            .strip_prefix(root)
            .unwrap_or(companion)
            .to_string_lossy()
            .to_string();
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
pub fn filter_unchanged(
    root: &Path,
    spec_files: &[PathBuf],
    cache: &HashCache,
) -> Vec<PathBuf> {
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
        let rel = spec_path
            .strip_prefix(root)
            .unwrap_or(spec_path)
            .to_string_lossy()
            .to_string();
        cache.update(root, &rel);

        // Update companion files (both naming conventions)
        let (req_files, other_files) = find_companion_files(spec_path);
        for companion in req_files.iter().chain(other_files.iter()) {
            let comp_rel = companion
                .strip_prefix(root)
                .unwrap_or(companion)
                .to_string_lossy()
                .to_string();
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

// ---------- Minimal SHA-256 implementation ----------
// Using a small inline implementation to avoid adding a dependency.
// This is the standard FIPS 180-4 algorithm.

struct Sha256 {
    state: [u32; 8],
    buf: Vec<u8>,
    len: u64,
}

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

impl Sha256 {
    fn new() -> Self {
        Self {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
                0x5be0cd19,
            ],
            buf: Vec::new(),
            len: 0,
        }
    }

    fn update(&mut self, data: &[u8]) {
        self.len += data.len() as u64;
        self.buf.extend_from_slice(data);
        while self.buf.len() >= 64 {
            let block: [u8; 64] = self.buf[..64].try_into().unwrap();
            self.compress(&block);
            self.buf.drain(..64);
        }
    }

    fn compress(&mut self, block: &[u8; 64]) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes(block[i * 4..i * 4 + 4].try_into().unwrap());
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.state;

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
        self.state[5] = self.state[5].wrapping_add(f);
        self.state[6] = self.state[6].wrapping_add(g);
        self.state[7] = self.state[7].wrapping_add(h);
    }

    fn hex_digest(mut self) -> String {
        // Padding
        let bit_len = self.len * 8;
        self.buf.push(0x80);
        while self.buf.len() % 64 != 56 {
            self.buf.push(0);
        }
        self.buf.extend_from_slice(&bit_len.to_be_bytes());

        // Process remaining blocks
        while self.buf.len() >= 64 {
            let block: [u8; 64] = self.buf[..64].try_into().unwrap();
            self.compress(&block);
            self.buf.drain(..64);
        }

        self.state
            .iter()
            .map(|word| format!("{word:08x}"))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn sha256_empty() {
        let mut h = Sha256::new();
        h.update(b"");
        assert_eq!(
            h.hex_digest(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn sha256_hello() {
        let mut h = Sha256::new();
        h.update(b"hello");
        assert_eq!(
            h.hex_digest(),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

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

        let (req, other) = find_companion_files(&specs.join("auth.spec.md"));
        assert_eq!(req.len(), 1);
        assert!(req[0].ends_with("requirements.md"));
        assert_eq!(other.len(), 2);
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
