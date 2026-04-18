use colored::Colorize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime};

use crate::parser;
use crate::registry;
use crate::validator;

use super::load_and_discover;

/// Result of verifying a single cross-project reference.
#[derive(Debug)]
struct VerifyResult {
    spec: String,
    repo: String,
    module: String,
    issues: Vec<DriftIssue>,
}

/// A specific drift issue found during verification.
#[derive(Debug)]
#[allow(dead_code)]
enum DriftIssue {
    /// The remote spec has been deprecated.
    Deprecated { status: String },
    /// An export referenced locally no longer exists in the remote spec.
    MissingExport { export: String },
    /// The dependency is not bidirectional — remote spec doesn't depend back.
    NotBidirectional { local_repo: String },
    /// The remote spec file could not be fetched.
    FetchFailed { reason: String },
    /// The remote spec file could not be parsed.
    ParseFailed,
}

/// Simple file-based cache for remote spec content.
struct SpecCache {
    cache_dir: std::path::PathBuf,
    ttl: Duration,
}

impl SpecCache {
    fn new(root: &Path, ttl_secs: u64) -> Self {
        let cache_dir = root.join(".specsync-cache").join("remote-specs");
        Self {
            cache_dir,
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    /// Get cached content if it exists and hasn't expired.
    fn get(&self, repo: &str, spec_path: &str) -> Option<String> {
        let cache_file = self.cache_path(repo, spec_path);
        let metadata = fs::metadata(&cache_file).ok()?;
        let modified = metadata.modified().ok()?;
        let age = SystemTime::now().duration_since(modified).ok()?;
        if age > self.ttl {
            return None;
        }
        fs::read_to_string(&cache_file).ok()
    }

    /// Store content in cache.
    fn set(&self, repo: &str, spec_path: &str, content: &str) {
        let cache_file = self.cache_path(repo, spec_path);
        if let Some(parent) = cache_file.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&cache_file, content);
    }

    fn cache_path(&self, repo: &str, spec_path: &str) -> std::path::PathBuf {
        // Sanitize repo and path for filesystem: owner/repo -> owner_repo
        let safe_repo = repo.replace('/', "_");
        let safe_path = spec_path.replace('/', "_");
        self.cache_dir.join(format!("{safe_repo}__{safe_path}"))
    }
}

pub fn cmd_resolve(root: &Path, remote: bool, verify: bool, cache_ttl: u64) {
    let (_config, spec_files) = load_and_discover(root, false);
    let mut cross_refs: Vec<(String, String, String)> = Vec::new();
    let mut local_refs: Vec<(String, String, bool)> = Vec::new();

    // Track local spec exports for bidirectional checking
    let mut local_exports: HashMap<String, Vec<String>> = HashMap::new();

    // Detect our own repo for bidirectional checking
    let local_repo = crate::github::detect_repo(root);

    for spec_file in &spec_files {
        let content = match fs::read_to_string(spec_file) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let parsed = match parser::parse_frontmatter(&content) {
            Some(p) => p,
            None => continue,
        };

        let spec_path = spec_file
            .strip_prefix(root)
            .unwrap_or(spec_file)
            .to_string_lossy()
            .to_string();

        // Collect local exports for bidirectional checking
        if let Some(module) = &parsed.frontmatter.module {
            let exports = parser::get_spec_symbols(&parsed.body);
            local_exports.insert(module.clone(), exports);
        }

        for dep in &parsed.frontmatter.depends_on {
            if validator::is_cross_project_ref(dep) {
                if let Some((repo, module)) = validator::parse_cross_project_ref(dep) {
                    cross_refs.push((spec_path.clone(), repo.to_string(), module.to_string()));
                }
            } else {
                let exists = root.join(dep).exists();
                local_refs.push((spec_path.clone(), dep.clone(), exists));
            }
        }
    }

    println!(
        "\n--- {} ------------------------------------------------",
        "Dependency Resolution".bold()
    );

    if local_refs.is_empty() && cross_refs.is_empty() {
        println!("\n  No dependencies declared in any spec.");
        return;
    }

    if !local_refs.is_empty() {
        println!("\n  {} Local dependencies:", "Local".bold());
        for (spec, dep, exists) in &local_refs {
            if *exists {
                println!("    {} {spec} -> {dep}", "✓".green());
            } else {
                println!("    {} {spec} -> {dep} (not found)", "✗".red());
            }
        }
    }

    if !cross_refs.is_empty() {
        println!("\n  {} Cross-project references:", "Remote".bold());

        if remote {
            // Fetch remote registries to verify cross-project refs
            let mut remote_errors = 0;
            // Group refs by repo to avoid duplicate fetches
            let mut repos: HashMap<String, Option<registry::RemoteRegistry>> = HashMap::new();

            for (_spec, repo, _module) in &cross_refs {
                repos
                    .entry(repo.clone())
                    .or_insert_with(|| match registry::fetch_remote_registry(repo) {
                        Ok(reg) => Some(reg),
                        Err(e) => {
                            eprintln!(
                                "    {} Failed to fetch registry for {repo}: {e}",
                                "!".yellow()
                            );
                            None
                        }
                    });
            }

            // Phase 1: Registry-level checks
            for (spec, repo, module) in &cross_refs {
                match repos.get(repo) {
                    Some(Some(reg)) => {
                        if reg.has_spec(module) {
                            println!("    {} {spec} -> {repo}@{module}", "✓".green());
                        } else {
                            println!(
                                "    {} {spec} -> {repo}@{module} (module not in registry)",
                                "✗".red()
                            );
                            remote_errors += 1;
                        }
                    }
                    Some(None) => {
                        println!(
                            "    {} {spec} -> {repo}@{module} (registry fetch failed)",
                            "?".yellow()
                        );
                    }
                    None => {
                        println!(
                            "    {} {spec} -> {repo}@{module} (no registry)",
                            "?".yellow()
                        );
                    }
                }
            }

            if remote_errors > 0 {
                println!(
                    "\n  {} {remote_errors} cross-project ref(s) could not be verified",
                    "Warning:".yellow()
                );
            }

            // Phase 2: Deep content verification (--verify)
            if verify {
                let drift_issues = verify_remote_specs(
                    &cross_refs,
                    &repos,
                    &local_exports,
                    local_repo.as_deref(),
                    root,
                    cache_ttl,
                );

                if !drift_issues.is_empty() {
                    println!("\n  {} Content verification:", "Verify".bold());

                    let mut drift_count = 0;
                    for result in &drift_issues {
                        for issue in &result.issues {
                            drift_count += 1;
                            match issue {
                                DriftIssue::Deprecated { status } => {
                                    println!(
                                        "    {} {repo}@{module}: remote spec status is \"{status}\"",
                                        "DRIFT".red().bold(),
                                        repo = result.repo,
                                        module = result.module,
                                    );
                                }
                                DriftIssue::MissingExport { export } => {
                                    println!(
                                        "    {} {spec} references {repo}@{module}, but export `{export}` no longer exists in remote spec",
                                        "DRIFT".red().bold(),
                                        spec = result.spec,
                                        repo = result.repo,
                                        module = result.module,
                                    );
                                }
                                DriftIssue::NotBidirectional { local_repo } => {
                                    println!(
                                        "    {} {spec} depends on {repo}@{module}, but remote spec does not reference {local_repo}",
                                        "WARN".yellow().bold(),
                                        spec = result.spec,
                                        repo = result.repo,
                                        module = result.module,
                                    );
                                }
                                DriftIssue::FetchFailed { reason } => {
                                    println!(
                                        "    {} {repo}@{module}: could not fetch spec content — {reason}",
                                        "WARN".yellow().bold(),
                                        repo = result.repo,
                                        module = result.module,
                                    );
                                }
                                DriftIssue::ParseFailed => {
                                    println!(
                                        "    {} {repo}@{module}: remote spec could not be parsed",
                                        "WARN".yellow().bold(),
                                        repo = result.repo,
                                        module = result.module,
                                    );
                                }
                            }
                        }
                    }

                    let drift_errors: usize = drift_issues
                        .iter()
                        .flat_map(|r| &r.issues)
                        .filter(|i| {
                            matches!(
                                i,
                                DriftIssue::Deprecated { .. } | DriftIssue::MissingExport { .. }
                            )
                        })
                        .count();

                    if drift_errors > 0 {
                        println!(
                            "\n  {} {drift_errors} drift issue(s) detected — specs have diverged from remote",
                            "Error:".red().bold()
                        );
                        std::process::exit(1);
                    } else {
                        println!(
                            "\n  {} {drift_count} warning(s), no breaking drift",
                            "Info:".cyan()
                        );
                    }
                } else {
                    println!(
                        "\n  {} All cross-project references verified — no drift detected",
                        "✓".green()
                    );
                }
            }
        } else {
            for (spec, repo, module) in &cross_refs {
                println!("    {} {spec} -> {repo}@{module}", "→".cyan());
            }
            println!(
                "\n  {} Cross-project refs are not verified by default.",
                "Tip:".cyan()
            );
            println!("  Use --remote to fetch registries and verify they exist.");
            println!("  Use --verify for deep content verification and drift detection.");
        }
    }
}

/// Deep-verify remote spec content for drift.
fn verify_remote_specs(
    cross_refs: &[(String, String, String)],
    repos: &HashMap<String, Option<registry::RemoteRegistry>>,
    local_exports: &HashMap<String, Vec<String>>,
    local_repo: Option<&str>,
    root: &Path,
    cache_ttl: u64,
) -> Vec<VerifyResult> {
    let cache = SpecCache::new(root, cache_ttl);
    let mut results: Vec<VerifyResult> = Vec::new();

    // Cache fetched+parsed remote specs to avoid re-fetching for duplicate refs
    let mut remote_specs: HashMap<(String, String), Option<registry::RemoteSpec>> = HashMap::new();

    for (spec, repo, module) in cross_refs {
        let key = (repo.clone(), module.clone());

        // Get or fetch the remote spec
        let remote_spec = remote_specs.entry(key).or_insert_with(|| {
            // Look up the spec path from the registry
            let reg = repos.get(repo)?.as_ref()?;
            let spec_path = reg.spec_path(module)?;

            // Try cache first
            let content = if let Some(cached) = cache.get(repo, spec_path) {
                cached
            } else {
                match registry::fetch_remote_spec(repo, spec_path) {
                    Ok(content) => {
                        cache.set(repo, spec_path, &content);
                        content
                    }
                    Err(_) => return None,
                }
            };

            registry::parse_remote_spec(module, &content)
        });

        let mut issues = Vec::new();

        match remote_spec {
            Some(remote) => {
                // Check 1: Status — is the remote spec deprecated?
                if let Some(status) = &remote.status {
                    let s = status.to_lowercase();
                    if s == "deprecated" || s == "removed" || s == "archived" {
                        issues.push(DriftIssue::Deprecated {
                            status: status.clone(),
                        });
                    }
                }

                // Check 2: Exports — do the local spec's consumed exports still exist?
                if !remote.exports.is_empty() {
                    let consumed = find_consumed_exports(root, spec, module);
                    for export in &consumed {
                        if !remote.exports.iter().any(|e| e == export) {
                            issues.push(DriftIssue::MissingExport {
                                export: export.clone(),
                            });
                        }
                    }
                }

                // Check 3: Bidirectional — does the remote depend back on us?
                if let Some(our_repo) = local_repo {
                    let remote_refs_us = remote.depends_on.iter().any(|dep| {
                        if let Some((dep_repo, _)) = validator::parse_cross_project_ref(dep) {
                            dep_repo == our_repo
                        } else {
                            false
                        }
                    });
                    // Only warn if we have exports that the remote could consume
                    if !remote_refs_us && !local_exports.is_empty() {
                        issues.push(DriftIssue::NotBidirectional {
                            local_repo: our_repo.to_string(),
                        });
                    }
                }
            }
            None => {
                // Could not fetch or parse — only flag if registry said it existed
                if let Some(Some(reg)) = repos.get(repo)
                    && reg.has_spec(module) {
                        issues.push(DriftIssue::FetchFailed {
                            reason: "spec listed in registry but content unavailable".to_string(),
                        });
                    }
            }
        }

        if !issues.is_empty() {
            results.push(VerifyResult {
                spec: spec.clone(),
                repo: repo.clone(),
                module: module.clone(),
                issues,
            });
        }
    }

    results
}

/// Find export names consumed from a specific remote module.
///
/// Scans the local spec's "### Consumes" table for rows matching the module name.
/// Expected table format: `| module | What is used |`
fn find_consumed_exports(root: &Path, spec_path: &str, remote_module: &str) -> Vec<String> {
    let full_path = root.join(spec_path);
    let content = match fs::read_to_string(&full_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let mut in_consumes = false;
    let mut exports = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "### Consumes" {
            in_consumes = true;
            continue;
        }
        if in_consumes && trimmed.starts_with("### ") {
            break;
        }
        if !in_consumes {
            continue;
        }

        // Parse table rows: | module | what_is_used |
        if trimmed.starts_with('|') && !trimmed.contains("---") {
            let cols: Vec<&str> = trimmed.split('|').map(|c| c.trim()).collect();
            // cols[0] is empty (before first |), cols[1] is module, cols[2] is what's used
            if cols.len() >= 3 {
                let module_col = cols[1].trim_matches('`').trim();
                if module_col.eq_ignore_ascii_case(remote_module) {
                    // Parse "what is used" — may contain backtick-wrapped function names
                    let usage = cols[2];
                    for part in usage.split(',') {
                        let part = part.trim();
                        // Extract backtick-wrapped identifiers
                        if let Some(start) = part.find('`')
                            && let Some(end) = part[start + 1..].find('`') {
                                let name = &part[start + 1..start + 1 + end];
                                if !name.is_empty() {
                                    exports.push(name.to_string());
                                }
                            }
                    }
                }
            }
        }
    }

    exports
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_consumed_exports_parses_table() {
        let spec_content = r#"---
module: my_module
version: 1
status: stable
files:
  - src/my_module.rs
depends_on:
  - corvid-labs/algochat@auth
---

# My Module

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| auth | `verify_token`, `create_session` |
| messaging | `send_message` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli | Entry point |
"#;

        let dir = tempfile::tempdir().unwrap();
        let spec_path = "specs/my_module/my_module.spec.md";
        let full_path = dir.path().join(spec_path);
        fs::create_dir_all(full_path.parent().unwrap()).unwrap();
        fs::write(&full_path, spec_content).unwrap();

        let exports = find_consumed_exports(dir.path(), spec_path, "auth");
        assert_eq!(exports, vec!["verify_token", "create_session"]);

        let msg_exports = find_consumed_exports(dir.path(), spec_path, "messaging");
        assert_eq!(msg_exports, vec!["send_message"]);

        let none = find_consumed_exports(dir.path(), spec_path, "nonexistent");
        assert!(none.is_empty());
    }

    #[test]
    fn test_find_consumed_exports_skips_header_row() {
        let spec_content = r#"---
module: test
version: 1
status: stable
files: []
depends_on: []
---

# Test

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| auth | `login` |
"#;

        let dir = tempfile::tempdir().unwrap();
        let spec_path = "specs/test/test.spec.md";
        let full_path = dir.path().join(spec_path);
        fs::create_dir_all(full_path.parent().unwrap()).unwrap();
        fs::write(&full_path, spec_content).unwrap();

        // "Module" is the header, should not match
        let exports = find_consumed_exports(dir.path(), spec_path, "Module");
        assert!(exports.is_empty());

        let exports = find_consumed_exports(dir.path(), spec_path, "auth");
        assert_eq!(exports, vec!["login"]);
    }

    #[test]
    fn test_spec_cache_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let cache = SpecCache::new(dir.path(), 3600);

        cache.set("owner/repo", "specs/auth/auth.spec.md", "# Auth spec");
        let cached = cache.get("owner/repo", "specs/auth/auth.spec.md");
        assert_eq!(cached, Some("# Auth spec".to_string()));
    }

    #[test]
    fn test_spec_cache_miss() {
        let dir = tempfile::tempdir().unwrap();
        let cache = SpecCache::new(dir.path(), 3600);

        let cached = cache.get("owner/repo", "specs/auth/auth.spec.md");
        assert!(cached.is_none());
    }

    #[test]
    fn test_spec_cache_expired() {
        let dir = tempfile::tempdir().unwrap();
        let cache = SpecCache::new(dir.path(), 0); // TTL = 0 seconds

        cache.set("owner/repo", "specs/auth/auth.spec.md", "# Auth spec");
        std::thread::sleep(Duration::from_millis(10));
        let cached = cache.get("owner/repo", "specs/auth/auth.spec.md");
        assert!(cached.is_none());
    }

    #[test]
    fn test_verify_detects_deprecated_status() {
        let remote = registry::RemoteSpec {
            module: "auth".to_string(),
            status: Some("deprecated".to_string()),
            depends_on: vec![],
            exports: vec![],
            body: String::new(),
        };
        assert_eq!(remote.status.as_deref(), Some("deprecated"));
    }

    #[test]
    fn test_cache_path_sanitizes_slashes() {
        let dir = tempfile::tempdir().unwrap();
        let cache = SpecCache::new(dir.path(), 3600);

        let path = cache.cache_path("owner/repo", "specs/auth/auth.spec.md");
        let filename = path.file_name().unwrap().to_str().unwrap();
        assert!(!filename.contains('/'));
        assert!(filename.contains("owner_repo__specs_auth_auth.spec.md"));
    }
}
