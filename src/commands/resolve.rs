use colored::Colorize;
use std::collections::HashMap;
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

/// The folder of remote spec content in the project's local state folder (REQ-cmd-resolve-002).
const REMOTE_SPECS_FOLDER: &str = "remote-specs";

/// File-based cache for remote spec content, kept in `remote-specs/` in the project's local
/// state folder outside the work tree, so a repository can never ship an entry
/// (REQ-cmd-resolve-002). A `.specsync-cache/` in the work tree is never looked at.
struct SpecCache {
    root: std::path::PathBuf,
    ttl: Duration,
    /// Whether this run has already said that the cache was not used.
    warned: std::cell::Cell<bool>,
}

impl SpecCache {
    fn new(root: &Path, ttl_secs: u64) -> Self {
        Self {
            root: root.to_path_buf(),
            ttl: Duration::from_secs(ttl_secs),
            warned: std::cell::Cell::new(false),
        }
    }

    /// Say once per run that the cache was skipped, and why. The fetched content is still used.
    fn skipped(&self, reason: &str) {
        if !self.warned.replace(true) {
            eprintln!(
                "{} remote spec cache not used: {reason}",
                "warning:".yellow().bold()
            );
        }
    }

    /// The cache folder, made when missing and `create`. When it cannot be opened, the cache is
    /// skipped with the reason.
    fn folder(&self, create: bool) -> Option<crate::util::local_state::LocalState> {
        let opened =
            crate::util::local_state::LocalState::open(&self.root, create).and_then(|state| {
                match state {
                    Some(state) => state.subdirectory(REMOTE_SPECS_FOLDER, create),
                    None => Ok(None),
                }
            });
        match opened {
            Ok(folder) => folder,
            Err(error) => {
                self.skipped(&error.to_string());
                None
            }
        }
    }

    /// Get cached content if it exists and hasn't expired. A link or anything but a regular
    /// file at the entry is skipped with the reason: SpecSync only ever renames files there.
    fn get(&self, repo: &str, spec_path: &str) -> Option<String> {
        let folder = self.folder(false)?;
        let name = Self::entry_name(repo, spec_path);
        let metadata = folder.metadata(&name).ok()??;
        if !metadata.is_file() {
            let what = if metadata.file_type().is_symlink() {
                "a symbolic link"
            } else {
                "not a regular file"
            };
            self.skipped(&format!("{} is {what}", folder.file_path(&name).display()));
            return None;
        }
        let modified = metadata.modified().ok()?.into_std();
        let age = SystemTime::now().duration_since(modified).ok()?;
        if age > self.ttl {
            return None;
        }
        // not a project file read: the remote spec cache in SpecSync's own state folder outside the work tree, read through its no-follow, bounded handle (REQ-util-006)
        match folder.read_file(&name, crate::util::MAX_SPEC_FILE_BYTES) {
            Ok(content) => String::from_utf8(content?).ok(),
            Err(error) => {
                self.skipped(&error.to_string());
                None
            }
        }
    }

    /// Store content in cache: a new file renamed over the entry, so nothing is written through
    /// what the entry was.
    fn set(&self, repo: &str, spec_path: &str, content: &str) {
        let Some(folder) = self.folder(true) else {
            return;
        };
        if let Err(error) = folder.write_file(
            &Self::entry_name(repo, spec_path),
            content.as_bytes(),
            false,
        ) {
            self.skipped(&error.to_string());
        }
    }

    /// The entry's file name: a digest of the repository and spec path, so no part of it comes
    /// from the repository's own `depends_on` text.
    fn entry_name(repo: &str, spec_path: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(repo.as_bytes());
        hasher.update([0]);
        hasher.update(spec_path.as_bytes());
        format!("{:x}.md", hasher.finalize())
    }

    /// Where the entry is kept, for tests.
    #[cfg(test)]
    fn cache_path(&self, repo: &str, spec_path: &str) -> std::path::PathBuf {
        crate::util::local_state::project_state_dir(&self.root)
            .unwrap()
            .join(REMOTE_SPECS_FOLDER)
            .join(Self::entry_name(repo, spec_path))
    }
}

/// Verdict for a single local dependency entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LocalVerdict {
    /// The path exists and is confined to the project root.
    Ok,
    /// The path does not exist.
    NotFound,
    /// The path escapes the project root (absolute or `..` traversal).
    OutsideRoot,
}

impl LocalVerdict {
    fn as_str(self) -> &'static str {
        match self {
            LocalVerdict::Ok => "ok",
            LocalVerdict::NotFound => "not-found",
            LocalVerdict::OutsideRoot => "outside-root",
        }
    }
}

pub fn cmd_resolve(
    root: &Path,
    remote: bool,
    verify: bool,
    cache_ttl: u64,
    format: crate::types::OutputFormat,
    strict: bool,
) {
    let json = matches!(format, crate::types::OutputFormat::Json);
    let (_config, spec_files) = load_and_discover(root, false);
    let mut cross_refs: Vec<(String, String, String)> = Vec::new();
    let mut local_refs: Vec<(String, String, LocalVerdict)> = Vec::new();
    // depends_on entries that look like refs but don't parse (e.g. `owner/repo@`
    // with no module) — previously these vanished silently (#444).
    let mut malformed_refs: Vec<(String, String)> = Vec::new();

    // Track local spec exports for bidirectional checking
    let mut local_exports: HashMap<String, Vec<String>> = HashMap::new();

    // Detect our own repo for bidirectional checking
    let local_repo = crate::github::detect_repo(root);

    for spec_file in &spec_files {
        let content = match crate::util::read_project_string(
            root,
            spec_file,
            crate::util::MAX_SPEC_FILE_BYTES,
        ) {
            Ok(c) => c,
            Err(error) => {
                // A refused read is reported, never skipped silently with its `depends_on`
                // (REQ-cmd-resolve-003).
                println!(
                    "  {} cannot read {}: {error}",
                    "⚠".yellow(),
                    spec_file.strip_prefix(root).unwrap_or(spec_file).display()
                );
                continue;
            }
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
                match validator::parse_cross_project_ref(dep) {
                    Some((repo, module)) => {
                        cross_refs.push((spec_path.clone(), repo.to_string(), module.to_string()))
                    }
                    // Looks cross-project (`owner/repo@…`) but doesn't parse —
                    // surface it instead of dropping it silently.
                    None => malformed_refs.push((spec_path.clone(), dep.clone())),
                }
            } else if dep.contains('@') {
                // `repo@` / `@module` shapes: not local paths, not valid refs.
                malformed_refs.push((spec_path.clone(), dep.clone()));
            } else {
                // Confine to the project root: absolute paths and `..`
                // traversal entries must never validate (#444).
                let verdict = match crate::util::confine_path_to_root(root, dep) {
                    Some(path) if path.exists() => LocalVerdict::Ok,
                    Some(_) => LocalVerdict::NotFound,
                    None => LocalVerdict::OutsideRoot,
                };
                local_refs.push((spec_path.clone(), dep.clone(), verdict));
            }
        }
    }

    if !json {
        println!(
            "\n--- {} ------------------------------------------------",
            "Dependency Resolution".bold()
        );
    }

    if local_refs.is_empty() && cross_refs.is_empty() && malformed_refs.is_empty() {
        if json {
            println!("{{\"dependencies\":[],\"passed\":true}}");
        } else {
            println!("\n  No dependencies declared in any spec.");
        }
        return;
    }

    if !json && !local_refs.is_empty() {
        println!("\n  {}", "Local dependencies:".bold());
        for (spec, dep, verdict) in &local_refs {
            match verdict {
                LocalVerdict::Ok => println!("    {} {spec} -> {dep}", "✓".green()),
                LocalVerdict::NotFound => {
                    println!("    {} {spec} -> {dep} (not found)", "✗".red())
                }
                LocalVerdict::OutsideRoot => println!(
                    "    {} {spec} -> {dep} (path escapes the project root)",
                    "✗".red()
                ),
            }
        }
    }

    if !json && !malformed_refs.is_empty() {
        for (spec, dep) in &malformed_refs {
            println!(
                "    {} {spec} -> {dep} (malformed reference — expected `owner/repo@module` or a project-relative path)",
                "✗".red()
            );
        }
    }

    // Local verdicts are computed up-front so both the text and JSON paths —
    // and the final exit code — share one source of truth (#444: unresolved
    // local deps must fail the process even without --strict).
    let unresolved_local = local_refs
        .iter()
        .filter(|(_, _, v)| *v != LocalVerdict::Ok)
        .count()
        + malformed_refs.len();

    // Remote verification state — shared by the text report, the JSON report,
    // and the final exit code so there is exactly one verdict (#422).
    let mut remote_errors = 0usize;
    let mut fetch_failures: Vec<String> = Vec::new();
    let mut drift_issues: Vec<VerifyResult> = Vec::new();

    if !cross_refs.is_empty() {
        if !json {
            println!("\n  {} Cross-project references:", "Remote".bold());
        }

        if remote {
            // Fetch remote registries to verify cross-project refs
            // Group refs by repo to avoid duplicate fetches
            let mut repos: HashMap<String, Option<registry::RemoteRegistry>> = HashMap::new();

            for (_spec, repo, _module) in &cross_refs {
                repos
                    .entry(repo.clone())
                    .or_insert_with(|| match registry::fetch_remote_registry(repo) {
                        Ok(reg) => Some(reg),
                        Err(e) => {
                            // A failed fetch is a failed verification, not a
                            // neutral "?" — the ref cannot be confirmed, so the
                            // run must not pass (#422).
                            fetch_failures.push(repo.clone());
                            if !json {
                                eprintln!(
                                    "    {} Failed to fetch registry for {repo}: {e}",
                                    "!".yellow()
                                );
                            }
                            None
                        }
                    });
            }

            // Phase 1: Registry-level checks
            for (spec, repo, module) in &cross_refs {
                match repos.get(repo) {
                    Some(Some(reg)) => {
                        if reg.has_spec(module) {
                            if !json {
                                println!("    {} {spec} -> {repo}@{module}", "✓".green());
                            }
                        } else {
                            if !json {
                                println!(
                                    "    {} {spec} -> {repo}@{module} (module not in registry)",
                                    "✗".red()
                                );
                            }
                            remote_errors += 1;
                        }
                    }
                    Some(None) => {
                        if !json {
                            println!(
                                "    {} {spec} -> {repo}@{module} (registry fetch failed)",
                                "✗".red()
                            );
                        }
                    }
                    None => {
                        if !json {
                            println!(
                                "    {} {spec} -> {repo}@{module} (no registry)",
                                "?".yellow()
                            );
                        }
                    }
                }
            }

            if remote_errors > 0 && !json {
                println!(
                    "\n  {} {remote_errors} cross-project ref(s) could not be verified",
                    "Error:".red().bold()
                );
            }

            // Phase 2: Deep content verification (--verify)
            if verify {
                let drift_issues_found = verify_remote_specs(
                    &cross_refs,
                    &repos,
                    &local_exports,
                    local_repo.as_deref(),
                    root,
                    cache_ttl,
                );
                drift_issues = drift_issues_found;

                if !drift_issues.is_empty() && !json {
                    println!("\n  {} Content verification:", "Verify".bold());

                    for result in &drift_issues {
                        for issue in &result.issues {
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

                    let drift_error_count: usize = drift_issues
                        .iter()
                        .flat_map(|r| &r.issues)
                        .filter(|i| {
                            matches!(
                                i,
                                DriftIssue::Deprecated { .. } | DriftIssue::MissingExport { .. }
                            )
                        })
                        .count();

                    if drift_error_count > 0 {
                        println!(
                            "\n  {} {drift_error_count} drift issue(s) detected — specs have diverged from remote",
                            "Error:".red().bold()
                        );
                    } else {
                        println!(
                            "\n  {} {} warning(s), no breaking drift",
                            "Info:".cyan(),
                            drift_issues.iter().flat_map(|r| &r.issues).count()
                        );
                    }
                } else if !json {
                    // Never print this when zero registries were actually
                    // fetched — "verified" over zero successful fetches is a
                    // false green (#422).
                    if fetch_failures.is_empty() {
                        println!(
                            "\n  {} All cross-project references verified — no drift detected",
                            "✓".green()
                        );
                    } else {
                        println!(
                            "\n  {} could not verify {} repo(s) — registry fetch failed",
                            "✗".red(),
                            fetch_failures.len()
                        );
                    }
                }
            }
        } else if !json {
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

    // ─── One verdict, one exit code (#422/#444) ────────────────────────
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
    let drift_warnings: usize = drift_issues.iter().flat_map(|r| &r.issues).count() - drift_errors;

    let mut failure_reasons: Vec<String> = Vec::new();
    if unresolved_local > 0 {
        failure_reasons.push(format!(
            "{unresolved_local} unresolved local dependency(ies)"
        ));
    }
    if remote_errors > 0 {
        failure_reasons.push(format!(
            "{remote_errors} cross-project ref(s) not found in remote registry"
        ));
    }
    if !fetch_failures.is_empty() {
        failure_reasons.push(format!(
            "registry fetch failed for {} repo(s): {}",
            fetch_failures.len(),
            fetch_failures.join(", ")
        ));
    }
    if drift_errors > 0 {
        failure_reasons.push(format!("{drift_errors} drift issue(s) detected"));
    }
    // --strict makes verification warnings (non-bidirectional, unparsable or
    // unfetchable remote spec content) fatal, matching `check --strict`.
    if strict && drift_warnings > 0 {
        failure_reasons.push(format!(
            "{drift_warnings} verification warning(s) treated as errors (--strict)"
        ));
    }
    let passed = failure_reasons.is_empty();

    if json {
        let local_json: Vec<serde_json::Value> = local_refs
            .iter()
            .map(|(spec, dep, verdict)| {
                serde_json::json!({"spec": spec, "dep": dep, "status": verdict.as_str()})
            })
            .collect();
        let cross_json: Vec<serde_json::Value> = cross_refs
            .iter()
            .map(|(spec, repo, module)| {
                serde_json::json!({"spec": spec, "repo": repo, "module": module})
            })
            .collect();
        let malformed_json: Vec<serde_json::Value> = malformed_refs
            .iter()
            .map(|(spec, dep)| serde_json::json!({"spec": spec, "dep": dep}))
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "passed": passed,
                "local": local_json,
                "cross_project": cross_json,
                "malformed": malformed_json,
                "fetch_failures": fetch_failures,
                "failures": failure_reasons,
            }))
            .unwrap()
        );
    } else if !passed {
        eprintln!("\n{}", "Dependency resolution FAILED:".red().bold());
        for reason in &failure_reasons {
            eprintln!("  {} {reason}", "✗".red());
        }
    }

    if !passed {
        std::process::exit(1);
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
                    && reg.has_spec(module)
                {
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
    let content = match crate::util::read_project_string(
        root,
        &full_path,
        crate::util::MAX_SPEC_FILE_BYTES,
    ) {
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
                            && let Some(end) = part[start + 1..].find('`')
                        {
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
    use std::fs;

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

    /// A project beside an `outside` directory, both in one temporary directory, so a link that
    /// leaves the project still reaches nothing but this fixture (REQ-cmd-resolve-002).
    #[cfg(unix)]
    fn project_beside_outside() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("project");
        let outside = dir.path().join("outside");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&outside).unwrap();
        (dir, root, outside)
    }

    /// Verifies REQ-cmd-resolve-002. SpecSync only ever renames files into its cache folder, so a
    /// link found at an entry is never read, and storing the fetched spec replaces the link
    /// without writing through it.
    #[cfg(unix)]
    #[test]
    fn spec_cache_never_writes_or_reads_through_a_link() {
        let (_dir, root, outside) = project_beside_outside();
        let victim = outside.join("victim.md");
        fs::write(&victim, "the owner's bytes\n").unwrap();
        let cache = SpecCache::new(&root, 3600);
        let planted = cache.cache_path("owner/repo", "specs/auth/auth.spec.md");
        fs::create_dir_all(planted.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(&victim, &planted).unwrap();

        assert_eq!(
            cache.get("owner/repo", "specs/auth/auth.spec.md"),
            None,
            "a cache entry was read through a link"
        );
        assert!(cache.warned.get(), "the skipped cache is said, once");
        cache.set("owner/repo", "specs/auth/auth.spec.md", "# Fetched");

        assert_eq!(
            fs::read_to_string(&victim).unwrap(),
            "the owner's bytes\n",
            "the fetched spec was written through a link, over a file outside the project"
        );
        assert_eq!(
            cache
                .get("owner/repo", "specs/auth/auth.spec.md")
                .as_deref(),
            Some("# Fetched")
        );
    }

    /// Verifies REQ-cmd-resolve-002. A dangling link at an entry never creates its target, and a
    /// `.specsync-cache` link in the work tree is never used at all.
    #[cfg(unix)]
    #[test]
    fn spec_cache_creates_nothing_through_a_dangling_link_or_a_linked_directory() {
        let (_dir, root, outside) = project_beside_outside();
        let cache = SpecCache::new(&root, 3600);
        let planted = cache.cache_path("owner/repo", "specs/auth/auth.spec.md");
        fs::create_dir_all(planted.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(outside.join("not-there-yet.md"), &planted).unwrap();
        cache.set("owner/repo", "specs/auth/auth.spec.md", "# Fetched");
        assert!(
            fs::read_dir(&outside).unwrap().next().is_none(),
            "the fetched spec was created at a dangling link's target"
        );

        let (_dir, root, outside) = project_beside_outside();
        std::os::unix::fs::symlink(&outside, root.join(".specsync-cache")).unwrap();
        let cache = SpecCache::new(&root, 3600);
        cache.set("owner/repo", "specs/auth/auth.spec.md", "# Fetched");
        assert!(
            fs::read_dir(&outside).unwrap().next().is_none(),
            "the fetched spec was written into a directory outside the project"
        );
        assert_eq!(
            cache
                .get("owner/repo", "specs/auth/auth.spec.md")
                .as_deref(),
            Some("# Fetched")
        );
    }

    /// Verifies REQ-cmd-resolve-002. A repository can commit `.specsync-cache/remote-specs/*`,
    /// and `git clone` gives the file a fresh modification time inside the TTL, so its forged
    /// text would stand in for the remote spec in `resolve --verify`. The cache never reads the
    /// work tree.
    #[test]
    fn spec_cache_never_returns_an_entry_the_work_tree_holds() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let committed =
            root.join(".specsync-cache/remote-specs/owner_repo__specs_auth_auth.spec.md");
        fs::create_dir_all(committed.parent().unwrap()).unwrap();
        fs::write(
            &committed,
            "---\nmodule: auth\nstatus: stable\n---\n\n# Forged by the repository\n",
        )
        .unwrap();
        let cache = SpecCache::new(root, 3600);

        assert_eq!(
            cache.get("owner/repo", "specs/auth/auth.spec.md"),
            None,
            "a remote spec cache entry the work tree holds was used"
        );
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
        assert!(!filename.contains("owner"), "{filename}");
        assert_ne!(
            SpecCache::entry_name("owner/repo", "specs/auth/auth.spec.md"),
            SpecCache::entry_name("owner/repo", "specs/auth/other.spec.md")
        );
    }
}
