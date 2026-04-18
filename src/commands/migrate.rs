use colored::Colorize;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::LazyLock;
use std::time::SystemTime;

use crate::config;
use crate::parser;
use crate::types::OutputFormat;
use crate::validator;

// ─── Types ──────────────────────────────────────────────────────────────────

const V4_VERSION: &str = "4.0.0";

#[derive(Debug, Clone, PartialEq)]
enum StepStatus {
    Done,
    Pending,
    Partial(String), // reason
}

#[allow(dead_code)]
struct MigrationContext {
    root: PathBuf,
    dry_run: bool,
    no_backup: bool,
    format: OutputFormat,
    /// Spec files discovered in the project.
    spec_files: Vec<PathBuf>,
}

#[derive(Debug, Default)]
struct MigrationReport {
    steps_completed: Vec<String>,
    steps_skipped: Vec<String>,
    files_moved: Vec<(String, String)>,
    dirs_created: Vec<String>,
    specs_updated: Vec<String>,
    lifecycle_files_created: Vec<String>,
    warnings: Vec<String>,
}

struct MigrationStep {
    name: &'static str,
    description: &'static str,
    check: fn(&MigrationContext) -> StepStatus,
    apply: fn(&MigrationContext, &mut MigrationReport) -> Result<(), String>,
}

// ─── Regex ──────────────────────────────────────────────────────────────────

static LIFECYCLE_LOG_BLOCK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^lifecycle_log:\n(?:  - [^\n]+\n?)*").unwrap());

// ─── Steps ──────────────────────────────────────────────────────────────────

fn steps() -> Vec<MigrationStep> {
    vec![
        MigrationStep {
            name: "detect_version",
            description: "Detect project version and migration eligibility",
            check: check_detect_version,
            apply: apply_detect_version,
        },
        MigrationStep {
            name: "create_backup",
            description: "Back up 3.x config and spec files",
            check: check_create_backup,
            apply: apply_create_backup,
        },
        MigrationStep {
            name: "create_directories",
            description: "Create .specsync/ directory structure",
            check: check_create_directories,
            apply: apply_create_directories,
        },
        MigrationStep {
            name: "relocate_config",
            description: "Convert config → .specsync/config.toml",
            check: check_relocate_config,
            apply: apply_relocate_config,
        },
        MigrationStep {
            name: "relocate_registry",
            description: "Move specsync-registry.toml → .specsync/registry.toml",
            check: check_relocate_registry,
            apply: apply_relocate_registry,
        },
        MigrationStep {
            name: "extract_lifecycle",
            description: "Extract lifecycle_log from spec frontmatter",
            check: check_extract_lifecycle,
            apply: apply_extract_lifecycle,
        },
        MigrationStep {
            name: "cleanup_frontmatter",
            description: "Remove lifecycle_log field from spec frontmatter",
            check: check_cleanup_frontmatter,
            apply: apply_cleanup_frontmatter,
        },
        MigrationStep {
            name: "write_gitignore",
            description: "Create .specsync/.gitignore",
            check: check_write_gitignore,
            apply: apply_write_gitignore,
        },
        MigrationStep {
            name: "update_root_gitignore",
            description: "Add .specsync/hashes.json to root .gitignore",
            check: check_update_root_gitignore,
            apply: apply_update_root_gitignore,
        },
        MigrationStep {
            name: "scan_cross_project",
            description: "Scan for cross-project registry references",
            check: check_scan_cross_project,
            apply: apply_scan_cross_project,
        },
        MigrationStep {
            name: "stamp_version",
            description: "Write .specsync/version with 4.0.0",
            check: check_stamp_version,
            apply: apply_stamp_version,
        },
    ]
}

// ─── Step: detect_version ───────────────────────────────────────────────────

fn check_detect_version(ctx: &MigrationContext) -> StepStatus {
    let version_file = ctx.root.join(".specsync/version");
    if version_file.exists()
        && let Ok(v) = fs::read_to_string(&version_file)
        && v.trim() == V4_VERSION
    {
        return StepStatus::Done;
    }
    // Check if there's a 3.x project to migrate
    let has_root_config = ctx.root.join("specsync.json").exists();
    let has_new_config = ctx.root.join(".specsync/config.json").exists();
    let has_legacy_toml = ctx.root.join(".specsync.toml").exists();
    let has_legacy_registry = ctx.root.join("specsync-registry.toml").exists();
    if !has_root_config && !has_new_config && !has_legacy_toml && !has_legacy_registry {
        return StepStatus::Partial("No spec-sync project found".to_string());
    }
    StepStatus::Pending
}

fn apply_detect_version(
    ctx: &MigrationContext,
    report: &mut MigrationReport,
) -> Result<(), String> {
    let version_file = ctx.root.join(".specsync/version");
    if version_file.exists()
        && let Ok(v) = fs::read_to_string(&version_file)
        && v.trim() == V4_VERSION
    {
        return Ok(());
    }
    let has_root_config = ctx.root.join("specsync.json").exists();
    let has_new_config = ctx.root.join(".specsync/config.json").exists();
    let has_legacy_toml = ctx.root.join(".specsync.toml").exists();
    let has_legacy_registry = ctx.root.join("specsync-registry.toml").exists();
    if !has_root_config && !has_new_config && !has_legacy_toml && !has_legacy_registry {
        return Err("No spec-sync project found. Run `specsync init` first.".to_string());
    }
    if has_root_config {
        report
            .steps_completed
            .push("Detected 3.x project (specsync.json at root)".to_string());
    } else if has_legacy_toml {
        report
            .steps_completed
            .push("Detected 3.x project (.specsync.toml at root)".to_string());
    } else if has_new_config {
        report
            .steps_completed
            .push("Detected partially migrated project (.specsync/config.json exists)".to_string());
    } else {
        report
            .steps_completed
            .push("Detected 3.x project (specsync-registry.toml at root)".to_string());
    }
    Ok(())
}

// ─── Step: create_backup ────────────────────────────────────────────────────

fn check_create_backup(ctx: &MigrationContext) -> StepStatus {
    if ctx.no_backup {
        return StepStatus::Done;
    }
    let backup_dir = ctx.root.join(".specsync/backup-3x");
    if backup_dir.exists() && backup_dir.join("manifest.json").exists() {
        StepStatus::Done
    } else {
        StepStatus::Pending
    }
}

fn apply_create_backup(ctx: &MigrationContext, report: &mut MigrationReport) -> Result<(), String> {
    if ctx.no_backup {
        report
            .warnings
            .push("Backup skipped (--no-backup)".to_string());
        return Ok(());
    }
    let backup_dir = ctx.root.join(".specsync/backup-3x");
    if backup_dir.exists() && backup_dir.join("manifest.json").exists() {
        return Ok(());
    }

    fs::create_dir_all(&backup_dir).map_err(|e| format!("Failed to create backup dir: {e}"))?;

    let mut manifest_entries: Vec<serde_json::Value> = Vec::new();
    let timestamp = iso_timestamp();

    // Back up root config files
    for filename in &["specsync.json", "specsync-registry.toml"] {
        let src = ctx.root.join(filename);
        if src.exists() {
            let dst = backup_dir.join(filename);
            fs::copy(&src, &dst).map_err(|e| format!("Failed to backup {filename}: {e}"))?;
            manifest_entries.push(serde_json::json!({
                "file": filename,
                "type": "config",
                "backed_up_at": timestamp,
            }));
        }
    }

    // Back up spec files (only those with lifecycle_log)
    let specs_backup_dir = backup_dir.join("specs");
    for spec_file in &ctx.spec_files {
        if let Ok(content) = fs::read_to_string(spec_file)
            && content.contains("lifecycle_log:")
        {
            let rel = spec_file.strip_prefix(&ctx.root).unwrap_or(spec_file);
            let dst = specs_backup_dir.join(rel);
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create backup subdir: {e}"))?;
            }
            fs::copy(spec_file, &dst)
                .map_err(|e| format!("Failed to backup {}: {e}", rel.display()))?;
            manifest_entries.push(serde_json::json!({
                "file": rel.display().to_string(),
                "type": "spec_with_lifecycle_log",
                "backed_up_at": timestamp,
            }));
        }
    }

    // Write manifest
    let manifest = serde_json::json!({
        "version": "3.x",
        "migrating_to": V4_VERSION,
        "timestamp": timestamp,
        "files": manifest_entries,
    });
    let manifest_path = backup_dir.join("manifest.json");
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .map_err(|e| format!("Failed to write backup manifest: {e}"))?;

    report.steps_completed.push(format!(
        "Backed up {} file(s) to .specsync/backup-3x/",
        manifest_entries.len()
    ));
    Ok(())
}

// ─── Step: create_directories ───────────────────────────────────────────────

fn check_create_directories(ctx: &MigrationContext) -> StepStatus {
    let dirs = [
        ".specsync",
        ".specsync/lifecycle",
        ".specsync/changes",
        ".specsync/archive",
    ];
    let all_exist = dirs.iter().all(|d| ctx.root.join(d).exists());
    if all_exist {
        StepStatus::Done
    } else {
        StepStatus::Pending
    }
}

fn apply_create_directories(
    ctx: &MigrationContext,
    report: &mut MigrationReport,
) -> Result<(), String> {
    let dirs = [
        ".specsync",
        ".specsync/lifecycle",
        ".specsync/changes",
        ".specsync/archive",
    ];
    for dir in &dirs {
        let path = ctx.root.join(dir);
        if !path.exists() {
            fs::create_dir_all(&path).map_err(|e| format!("Failed to create {dir}: {e}"))?;
            report.dirs_created.push(dir.to_string());
        }
    }
    report
        .steps_completed
        .push(format!("Created {} directories", report.dirs_created.len()));
    Ok(())
}

// ─── Step: relocate_config ──────────────────────────────────────────────────

fn check_relocate_config(ctx: &MigrationContext) -> StepStatus {
    let old_json = ctx.root.join("specsync.json");
    let old_toml = ctx.root.join(".specsync.toml");
    let new_toml = ctx.root.join(".specsync/config.toml");
    let new_json = ctx.root.join(".specsync/config.json");
    if new_toml.exists() && !old_json.exists() && !old_toml.exists() {
        StepStatus::Done
    } else if new_json.exists() && !old_json.exists() && !old_toml.exists() {
        // v4 JSON exists but not yet converted to TOML
        StepStatus::Pending
    } else if old_json.exists() || old_toml.exists() {
        StepStatus::Pending
    } else {
        StepStatus::Partial("No config file found".to_string())
    }
}

fn apply_relocate_config(
    ctx: &MigrationContext,
    report: &mut MigrationReport,
) -> Result<(), String> {
    let old_json = ctx.root.join("specsync.json");
    let old_toml_root = ctx.root.join(".specsync.toml");
    let new_toml = ctx.root.join(".specsync/config.toml");
    let new_json = ctx.root.join(".specsync/config.json");

    if new_toml.exists() && !old_json.exists() && !old_toml_root.exists() {
        return Ok(());
    }

    // Determine the source config to convert
    let source_path = if old_json.exists() {
        &old_json
    } else if old_toml_root.exists() {
        &old_toml_root
    } else if new_json.exists() {
        &new_json
    } else {
        report
            .warnings
            .push("No config file found — skipping config relocation".to_string());
        return Ok(());
    };

    // Load config from the specific source file (not global precedence) to avoid
    // converting the wrong config when multiple config files exist during partial migration
    let loaded_config = config::load_config_from_path(source_path, &ctx.root);
    let toml_content = config::config_to_toml(&loaded_config);

    // Ensure parent dir exists
    if let Some(parent) = new_toml.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create .specsync/: {e}"))?;
    }

    fs::write(&new_toml, &toml_content)
        .map_err(|e| format!("Failed to write .specsync/config.toml: {e}"))?;

    let source_name = source_path
        .strip_prefix(&ctx.root)
        .unwrap_or(source_path)
        .display()
        .to_string();

    // Remove old config files
    if old_json.exists() {
        let _ = fs::remove_file(&old_json);
    }
    if old_toml_root.exists() {
        let _ = fs::remove_file(&old_toml_root);
    }
    // Remove intermediate v4 JSON if it existed
    if new_json.exists() {
        let _ = fs::remove_file(&new_json);
    }

    report
        .files_moved
        .push((source_name.clone(), ".specsync/config.toml".to_string()));
    report.steps_completed.push(format!(
        "Converted {source_name} → .specsync/config.toml (TOML)"
    ));
    Ok(())
}

// ─── Step: relocate_registry ────────────────────────────────────────────────

fn check_relocate_registry(ctx: &MigrationContext) -> StepStatus {
    let old = ctx.root.join("specsync-registry.toml");
    let new = ctx.root.join(".specsync/registry.toml");
    if !old.exists() && !new.exists() {
        // No registry file — nothing to migrate
        StepStatus::Done
    } else if new.exists() && !old.exists() {
        StepStatus::Done
    } else if new.exists() && old.exists() {
        StepStatus::Partial("Both old and new registry files exist".to_string())
    } else {
        StepStatus::Pending
    }
}

fn apply_relocate_registry(
    ctx: &MigrationContext,
    report: &mut MigrationReport,
) -> Result<(), String> {
    let old = ctx.root.join("specsync-registry.toml");
    let new = ctx.root.join(".specsync/registry.toml");

    if !old.exists() {
        if !new.exists() {
            report
                .warnings
                .push("No registry file found — skipping registry relocation".to_string());
        }
        return Ok(());
    }

    if let Some(parent) = new.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create .specsync/: {e}"))?;
    }
    fs::copy(&old, &new).map_err(|e| format!("Failed to copy registry: {e}"))?;
    fs::remove_file(&old).map_err(|e| format!("Failed to remove old registry: {e}"))?;
    report.files_moved.push((
        "specsync-registry.toml".to_string(),
        ".specsync/registry.toml".to_string(),
    ));
    report
        .steps_completed
        .push("Relocated specsync-registry.toml → .specsync/registry.toml".to_string());
    Ok(())
}

// ─── Step: extract_lifecycle ────────────────────────────────────────────────

fn check_extract_lifecycle(ctx: &MigrationContext) -> StepStatus {
    let lifecycle_dir = ctx.root.join(".specsync/lifecycle");
    if !lifecycle_dir.exists() {
        return StepStatus::Pending;
    }

    // Check if any specs still have lifecycle_log in frontmatter
    let any_remaining = ctx.spec_files.iter().any(|f| {
        fs::read_to_string(f)
            .map(|c| c.contains("lifecycle_log:"))
            .unwrap_or(false)
    });

    if any_remaining {
        // Lifecycle logs exist in frontmatter but lifecycle dir exists — might be partial
        let has_json_files = fs::read_dir(&lifecycle_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .any(|e| e.path().extension().is_some_and(|ext| ext == "json"))
            })
            .unwrap_or(false);

        if has_json_files {
            StepStatus::Partial(
                "Some lifecycle logs extracted but frontmatter not yet cleaned".to_string(),
            )
        } else {
            StepStatus::Pending
        }
    } else {
        // No lifecycle_log in any spec — either already extracted or never existed
        StepStatus::Done
    }
}

fn apply_extract_lifecycle(
    ctx: &MigrationContext,
    report: &mut MigrationReport,
) -> Result<(), String> {
    let lifecycle_dir = ctx.root.join(".specsync/lifecycle");
    fs::create_dir_all(&lifecycle_dir)
        .map_err(|e| format!("Failed to create lifecycle dir: {e}"))?;

    for spec_file in &ctx.spec_files {
        let content = match fs::read_to_string(spec_file) {
            Ok(c) => c,
            Err(e) => {
                report.warnings.push(format!(
                    "Could not read {}: {e}",
                    spec_file
                        .strip_prefix(&ctx.root)
                        .unwrap_or(spec_file)
                        .display()
                ));
                continue;
            }
        };

        let normalized = content.replace("\r\n", "\n");
        let parsed = match parser::parse_frontmatter(&normalized) {
            Some(p) => p,
            None => continue,
        };

        if parsed.frontmatter.lifecycle_log.is_empty() {
            continue;
        }

        let module = match &parsed.frontmatter.module {
            Some(m) => m.clone(),
            None => {
                // Derive from filename
                spec_file
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.strip_suffix(".spec").unwrap_or(s).to_string())
                    .unwrap_or_default()
            }
        };

        // Sanitize module name: strip path separators and traversal components
        let module = module.replace(['/', '\\'], "_").replace("..", "_");
        if module.is_empty() || module == "." {
            report.warnings.push(format!(
                "Could not determine module name for {}",
                spec_file.display()
            ));
            continue;
        }

        // Parse log entries into structured format
        let entries: Vec<serde_json::Value> = parsed
            .frontmatter
            .lifecycle_log
            .iter()
            .map(|entry| {
                // Format: "2026-04-11: draft → review" or "YYYY-MM-DD: from → to"
                let parts: Vec<&str> = entry.splitn(2, ": ").collect();
                if parts.len() == 2 {
                    serde_json::json!({
                        "date": parts[0].trim(),
                        "transition": parts[1].trim(),
                        "raw": entry,
                    })
                } else {
                    serde_json::json!({
                        "raw": entry,
                    })
                }
            })
            .collect();

        let lifecycle_data = serde_json::json!({
            "module": module,
            "extracted_from": spec_file.strip_prefix(&ctx.root)
                .unwrap_or(spec_file)
                .display()
                .to_string(),
            "extracted_at": iso_timestamp(),
            "entries": entries,
        });

        let out_path = lifecycle_dir.join(format!("{module}.json"));
        fs::write(
            &out_path,
            serde_json::to_string_pretty(&lifecycle_data).unwrap(),
        )
        .map_err(|e| format!("Failed to write lifecycle/{module}.json: {e}"))?;

        report
            .lifecycle_files_created
            .push(format!(".specsync/lifecycle/{module}.json"));
    }

    if !report.lifecycle_files_created.is_empty() {
        report.steps_completed.push(format!(
            "Extracted lifecycle history for {} spec(s)",
            report.lifecycle_files_created.len()
        ));
    } else {
        report
            .steps_completed
            .push("No lifecycle_log entries found to extract".to_string());
    }
    Ok(())
}

// ─── Step: cleanup_frontmatter ──────────────────────────────────────────────

fn check_cleanup_frontmatter(ctx: &MigrationContext) -> StepStatus {
    let any_has_log = ctx.spec_files.iter().any(|f| {
        fs::read_to_string(f)
            .map(|c| c.contains("lifecycle_log:"))
            .unwrap_or(false)
    });
    if any_has_log {
        StepStatus::Pending
    } else {
        StepStatus::Done
    }
}

fn apply_cleanup_frontmatter(
    ctx: &MigrationContext,
    report: &mut MigrationReport,
) -> Result<(), String> {
    for spec_file in &ctx.spec_files {
        let content = match fs::read_to_string(spec_file) {
            Ok(c) => c,
            Err(_) => continue,
        };

        if !content.contains("lifecycle_log:") {
            continue;
        }

        // Remove the lifecycle_log block from frontmatter
        let cleaned = LIFECYCLE_LOG_BLOCK_RE.replace(&content, "").to_string();
        // Clean up any resulting double newlines in frontmatter
        let cleaned = cleaned.replace("\n\n---", "\n---");

        fs::write(spec_file, &cleaned).map_err(|e| {
            format!(
                "Failed to clean frontmatter in {}: {e}",
                spec_file.display()
            )
        })?;

        let rel = spec_file
            .strip_prefix(&ctx.root)
            .unwrap_or(spec_file)
            .display()
            .to_string();
        report.specs_updated.push(rel);
    }

    if !report.specs_updated.is_empty() {
        report.steps_completed.push(format!(
            "Cleaned lifecycle_log from {} spec(s)",
            report.specs_updated.len()
        ));
    }
    Ok(())
}

// ─── Step: write_gitignore ──────────────────────────────────────────────────

fn check_write_gitignore(ctx: &MigrationContext) -> StepStatus {
    let gitignore = ctx.root.join(".specsync/.gitignore");
    if gitignore.exists() {
        StepStatus::Done
    } else {
        StepStatus::Pending
    }
}

fn apply_write_gitignore(
    ctx: &MigrationContext,
    report: &mut MigrationReport,
) -> Result<(), String> {
    let gitignore = ctx.root.join(".specsync/.gitignore");
    if gitignore.exists() {
        return Ok(());
    }

    let content = [
        "# spec-sync v4 — generated by `specsync migrate`",
        "# Committed: config.toml, registry.toml, lifecycle/, changes/, archive/",
        "# Ignored: backups, local config, hash cache (regenerated on each run)",
        "",
        "backup-3x/",
        "config.local.toml",
        "hashes.json",
        "",
    ]
    .join("\n");

    fs::write(&gitignore, content)
        .map_err(|e| format!("Failed to write .specsync/.gitignore: {e}"))?;

    report
        .steps_completed
        .push("Created .specsync/.gitignore".to_string());
    Ok(())
}

// ─── Step: update_root_gitignore ────────────────────────────────────────────

fn check_update_root_gitignore(ctx: &MigrationContext) -> StepStatus {
    let gitignore = ctx.root.join(".gitignore");
    let entry = ".specsync/hashes.json";
    match fs::read_to_string(&gitignore) {
        Ok(content) if content.lines().any(|l| l.trim() == entry) => StepStatus::Done,
        _ => StepStatus::Pending,
    }
}

fn apply_update_root_gitignore(
    ctx: &MigrationContext,
    report: &mut MigrationReport,
) -> Result<(), String> {
    super::init::ensure_hashes_gitignored(&ctx.root)?;
    report
        .steps_completed
        .push("Added .specsync/hashes.json to .gitignore".to_string());
    Ok(())
}

// ─── Step: scan_cross_project ───────────────────────────────────────────────

fn check_scan_cross_project(ctx: &MigrationContext) -> StepStatus {
    let xref_path = ctx.root.join(".specsync/cross-project-refs.json");
    if xref_path.exists() {
        StepStatus::Done
    } else {
        // Only needed if any specs have cross-project references
        let has_xrefs = ctx.spec_files.iter().any(|f| {
            fs::read_to_string(f)
                .map(|c| {
                    let normalized = c.replace("\r\n", "\n");
                    if let Some(parsed) = parser::parse_frontmatter(&normalized) {
                        parsed
                            .frontmatter
                            .depends_on
                            .iter()
                            .any(|d| validator::is_cross_project_ref(d))
                    } else {
                        false
                    }
                })
                .unwrap_or(false)
        });
        if has_xrefs {
            StepStatus::Pending
        } else {
            StepStatus::Done
        }
    }
}

fn apply_scan_cross_project(
    ctx: &MigrationContext,
    report: &mut MigrationReport,
) -> Result<(), String> {
    let xref_path = ctx.root.join(".specsync/cross-project-refs.json");
    if xref_path.exists() {
        return Ok(());
    }

    let mut refs: Vec<serde_json::Value> = Vec::new();

    for spec_file in &ctx.spec_files {
        let content = match fs::read_to_string(spec_file) {
            Ok(c) => c.replace("\r\n", "\n"),
            Err(_) => continue,
        };
        let parsed = match parser::parse_frontmatter(&content) {
            Some(p) => p,
            None => continue,
        };

        let module = parsed.frontmatter.module.clone().unwrap_or_else(|| {
            spec_file
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.strip_suffix(".spec").unwrap_or(s).to_string())
                .unwrap_or_default()
        });

        for dep in &parsed.frontmatter.depends_on {
            if validator::is_cross_project_ref(dep)
                && let Some((repo, remote_module)) = validator::parse_cross_project_ref(dep)
            {
                refs.push(serde_json::json!({
                    "local_module": module,
                    "remote_repo": repo,
                    "remote_module": remote_module,
                    "raw": dep,
                    "spec_file": spec_file.strip_prefix(&ctx.root)
                        .unwrap_or(spec_file).display().to_string(),
                }));
            }
        }
    }

    if refs.is_empty() {
        // No cross-project refs — nothing to record
        return Ok(());
    }

    let manifest = serde_json::json!({
        "version": V4_VERSION,
        "scanned_at": iso_timestamp(),
        "note": "Cross-project dependencies detected. Each referenced project should also be migrated to v4 for full compatibility. Run `specsync resolve --remote --verify` after all projects are migrated.",
        "references": refs,
    });

    fs::write(&xref_path, serde_json::to_string_pretty(&manifest).unwrap())
        .map_err(|e| format!("Failed to write cross-project-refs.json: {e}"))?;

    report.steps_completed.push(format!(
        "Found {} cross-project reference(s) across {} remote repo(s)",
        refs.len(),
        refs.iter()
            .map(|r| r["remote_repo"].as_str().unwrap_or(""))
            .collect::<std::collections::HashSet<_>>()
            .len()
    ));
    Ok(())
}

// ─── Step: stamp_version ────────────────────────────────────────────────────

fn check_stamp_version(ctx: &MigrationContext) -> StepStatus {
    let version_file = ctx.root.join(".specsync/version");
    if version_file.exists()
        && let Ok(v) = fs::read_to_string(&version_file)
        && v.trim() == V4_VERSION
    {
        return StepStatus::Done;
    }
    StepStatus::Pending
}

fn apply_stamp_version(ctx: &MigrationContext, report: &mut MigrationReport) -> Result<(), String> {
    let version_file = ctx.root.join(".specsync/version");
    fs::create_dir_all(ctx.root.join(".specsync"))
        .map_err(|e| format!("Failed to create .specsync/: {e}"))?;
    fs::write(&version_file, format!("{V4_VERSION}\n"))
        .map_err(|e| format!("Failed to write version file: {e}"))?;
    report
        .steps_completed
        .push(format!("Stamped version {V4_VERSION}"));
    Ok(())
}

// ─── Main command ───────────────────────────────────────────────────────────

pub fn cmd_migrate(root: &Path, format: OutputFormat, dry_run: bool, no_backup: bool) {
    // Discover spec files
    let spec_files = discover_specs(root);

    let ctx = MigrationContext {
        root: root.to_path_buf(),
        dry_run,
        no_backup,
        format,
        spec_files,
    };

    let migration_steps = steps();
    let mut report = MigrationReport::default();

    // Pre-flight: check if already migrated
    let version_file = root.join(".specsync/version");
    if version_file.exists()
        && let Ok(v) = fs::read_to_string(&version_file)
        && v.trim() == V4_VERSION
    {
        match format {
            OutputFormat::Json => {
                let output = serde_json::json!({
                    "status": "already_migrated",
                    "version": V4_VERSION,
                    "message": "Already at v4.0.0 — nothing to migrate",
                });
                println!("{}", serde_json::to_string_pretty(&output).unwrap());
            }
            _ => {
                println!(
                    "{} Already at v{V4_VERSION} — nothing to migrate.",
                    "✓".green()
                );
            }
        }
        return;
    }

    if dry_run {
        match format {
            OutputFormat::Json => {}
            _ => {
                println!(
                    "\n{} {} Migration dry run — no files will be modified\n",
                    "specsync migrate".bold(),
                    "(dry-run)".dimmed()
                );
            }
        }
    } else {
        match format {
            OutputFormat::Json => {}
            _ => {
                println!(
                    "\n{} Migrating to v{V4_VERSION}...\n",
                    "specsync migrate".bold()
                );
            }
        }
    }

    // Execute steps
    let mut had_error = false;
    for step in &migration_steps {
        let status = (step.check)(&ctx);

        match &status {
            StepStatus::Done => {
                report.steps_skipped.push(step.name.to_string());
                if !matches!(format, OutputFormat::Json) && !dry_run {
                    println!(
                        "  {} {} {}",
                        "✓".green(),
                        step.description,
                        "(already done)".dimmed()
                    );
                }
                if dry_run && !matches!(format, OutputFormat::Json) {
                    println!(
                        "  {} {} — {}",
                        "·".dimmed(),
                        step.description,
                        "already done".dimmed()
                    );
                }
                continue;
            }
            StepStatus::Partial(reason) => {
                if !matches!(format, OutputFormat::Json) {
                    println!(
                        "  {} {} — {} (will fix forward)",
                        "⚠".yellow(),
                        step.description,
                        reason
                    );
                }
            }
            StepStatus::Pending => {
                if dry_run && !matches!(format, OutputFormat::Json) {
                    println!("  {} {}", "→".cyan(), step.description);
                    continue;
                }
            }
        }

        if dry_run {
            continue;
        }

        // Apply step
        match (step.apply)(&ctx, &mut report) {
            Ok(()) => {
                if !matches!(format, OutputFormat::Json) {
                    println!("  {} {}", "✓".green(), step.description);
                }
            }
            Err(e) => {
                had_error = true;
                if !matches!(format, OutputFormat::Json) {
                    eprintln!("  {} {} — {}", "✗".red(), step.description, e.red());
                }
                report
                    .warnings
                    .push(format!("Step '{}' failed: {e}", step.name));
                // Don't break — try to complete remaining steps where possible
                // But some steps depend on prior ones, so they'll detect partial state
            }
        }
    }

    // Print summary
    print_summary(&report, &ctx, dry_run, had_error);

    if had_error && !dry_run {
        process::exit(1);
    }
}

fn print_summary(report: &MigrationReport, ctx: &MigrationContext, dry_run: bool, had_error: bool) {
    match ctx.format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "status": if dry_run { "dry_run" } else if had_error { "partial" } else { "completed" },
                "version": V4_VERSION,
                "dry_run": dry_run,
                "steps_completed": report.steps_completed,
                "steps_skipped": report.steps_skipped,
                "files_moved": report.files_moved.iter()
                    .map(|(from, to)| serde_json::json!({"from": from, "to": to}))
                    .collect::<Vec<_>>(),
                "dirs_created": report.dirs_created,
                "specs_updated": report.specs_updated,
                "lifecycle_files_created": report.lifecycle_files_created,
                "warnings": report.warnings,
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        _ => {
            println!();
            if dry_run {
                println!(
                    "{} Dry run complete. Run {} to apply.",
                    "Done:".bold(),
                    "specsync migrate".cyan()
                );
            } else if had_error {
                println!(
                    "{} Migration partially completed with errors. Check warnings above.",
                    "Warning:".yellow().bold()
                );
                println!(
                    "  Your original files are preserved in {}",
                    ".specsync/backup-3x/".cyan()
                );
            } else {
                println!(
                    "{} Successfully migrated to v{V4_VERSION}!",
                    "Done:".green().bold()
                );
            }

            // Stats line
            let mut stats = Vec::new();
            if !report.files_moved.is_empty() {
                stats.push(format!("{} file(s) moved", report.files_moved.len()));
            }
            if !report.dirs_created.is_empty() {
                stats.push(format!("{} dir(s) created", report.dirs_created.len()));
            }
            if !report.specs_updated.is_empty() {
                stats.push(format!("{} spec(s) cleaned", report.specs_updated.len()));
            }
            if !report.lifecycle_files_created.is_empty() {
                stats.push(format!(
                    "{} lifecycle file(s) extracted",
                    report.lifecycle_files_created.len()
                ));
            }
            if !stats.is_empty() {
                println!("  {}", stats.join(", "));
            }

            // Warnings
            for w in &report.warnings {
                println!("  {} {w}", "⚠".yellow());
            }
        }
    }
}

/// ISO 8601 timestamp without external dependencies.
fn iso_timestamp() -> String {
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();

    // Simple UTC date-time from unix timestamp
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    // Days since epoch to Y-M-D (simplified — accurate for 1970-2099)
    let mut y = 1970;
    let mut remaining_days = days as i64;
    loop {
        let days_in_year = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
            366
        } else {
            365
        };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        y += 1;
    }
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let month_days: [i64; 12] = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut m = 0;
    for (i, &md) in month_days.iter().enumerate() {
        if remaining_days < md {
            m = i + 1;
            break;
        }
        remaining_days -= md;
    }
    let d = remaining_days + 1;

    format!("{y:04}-{m:02}-{d:02}T{hours:02}:{minutes:02}:{seconds:02}Z")
}

/// Parse specs_dir from a legacy TOML config file.
fn parse_specs_dir_from_toml(content: &str) -> Option<String> {
    static SPECS_DIR_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"(?m)^\s*(?:specsDir|specs_dir)\s*=\s*"([^"]+)""#)
            .expect("valid specs dir regex")
    });
    SPECS_DIR_RE
        .captures(content)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}

/// Discover spec files without requiring a valid config (since config may be mid-migration).
fn discover_specs(root: &Path) -> Vec<PathBuf> {
    // Try loading config to find specs_dir (JSON then TOML), fall back to "specs"
    let specs_dir_name = fs::read_to_string(root.join("specsync.json"))
        .or_else(|_| fs::read_to_string(root.join(".specsync/config.json")))
        .ok()
        .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
        .and_then(|v| v.get("specsDir").and_then(|s| s.as_str()).map(String::from))
        .or_else(|| {
            fs::read_to_string(root.join(".specsync.toml"))
                .ok()
                .and_then(|content| parse_specs_dir_from_toml(&content))
        })
        .unwrap_or_else(|| "specs".to_string());

    let specs_dir = root.join(&specs_dir_name);
    if !specs_dir.exists() {
        return Vec::new();
    }

    crate::validator::find_spec_files(&specs_dir)
}
