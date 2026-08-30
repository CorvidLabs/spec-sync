use colored::Colorize;
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;

use crate::config::{config_to_toml, detect_source_dirs_with_confidence, validate_config_file};
use crate::output::csv_field;
use crate::types::{OutputFormat, SpecSyncConfig};

/// Version stamp written to `.specsync/version` for fresh projects.
const PROJECT_VERSION: &str = crate::change::SDD_VERSION;

/// Subdirectories created inside `.specsync/` for a fresh 5.0 project.
const V4_DIRS: &[&str] = &[
    ".specsync/lifecycle",
    ".specsync/changes",
    ".specsync/archive",
    ".specsync/archive/changes",
];

#[derive(Debug, Serialize)]
struct InitReport {
    command: &'static str,
    success: bool,
    created: bool,
    repaired: bool,
    unchanged: bool,
    config: Option<String>,
    source_dirs: Vec<String>,
    detected: Option<bool>,
    source_dirs_detected: Option<bool>,
    missing: Vec<String>,
    restored: Vec<String>,
    warnings: Vec<String>,
    repair_hint: Option<&'static str>,
    migration_hint: Option<&'static str>,
    initialized_ancestor: Option<String>,
    error: Option<String>,
}

impl InitReport {
    fn failure(error: String, config: Option<String>, ancestor: Option<PathBuf>) -> Self {
        Self {
            command: "init",
            success: false,
            created: false,
            repaired: false,
            unchanged: true,
            config,
            source_dirs: Vec::new(),
            detected: None,
            source_dirs_detected: None,
            missing: Vec::new(),
            restored: Vec::new(),
            warnings: Vec::new(),
            repair_hint: None,
            migration_hint: None,
            initialized_ancestor: ancestor.map(|path| path.display().to_string()),
            error: Some(error),
        }
    }
}

pub fn cmd_init(root: &Path, repair: bool, format: OutputFormat) {
    match execute_init(root, repair) {
        Ok(report) => {
            let should_hint_sdd = report.created && matches!(format, OutputFormat::Text);
            render_init_report(root, &report, format);
            if should_hint_sdd {
                println!(
                    "  SDD change workflow is off. Enable with `specsync change adopt` if you want it."
                );
            }
        }
        Err(report) => {
            render_init_report(root, &report, format);
            process::exit(1);
        }
    }
}

fn execute_init(root: &Path, repair: bool) -> Result<InitReport, Box<InitReport>> {
    // Refuse to clobber any existing config — current or legacy.
    let v4_toml = root.join(".specsync/config.toml");
    let v4_json = root.join(".specsync/config.json");
    let legacy_json = root.join("specsync.json");
    let legacy_toml = root.join(".specsync.toml");
    if v4_toml.exists() || v4_json.exists() {
        let (existing, path) = if v4_toml.exists() {
            (".specsync/config.toml", v4_toml)
        } else {
            (".specsync/config.json", v4_json)
        };
        if let Err(error) = validate_config_file(&path) {
            return Err(Box::new(InitReport::failure(
                error,
                Some(existing.to_string()),
                None,
            )));
        }
        if repair {
            return repair_layout(root, existing, &path).map_err(|error| {
                Box::new(InitReport::failure(error, Some(existing.to_string()), None))
            });
        }
        let missing = missing_support_files(root);
        return Ok(InitReport {
            command: "init",
            success: true,
            created: false,
            repaired: false,
            unchanged: true,
            config: Some(existing.to_string()),
            source_dirs: Vec::new(),
            detected: None,
            source_dirs_detected: None,
            repair_hint: (!missing.is_empty()).then_some("specsync init --repair"),
            missing,
            restored: Vec::new(),
            warnings: Vec::new(),
            migration_hint: None,
            initialized_ancestor: None,
            error: None,
        });
    }
    if legacy_json.exists() {
        if repair {
            return Err(Box::new(InitReport::failure(
                "cannot repair legacy config specsync.json; run `specsync migrate` first"
                    .to_string(),
                Some("specsync.json".to_string()),
                None,
            )));
        }
        return Ok(legacy_init_report("specsync.json"));
    }
    if legacy_toml.exists() {
        if repair {
            return Err(Box::new(InitReport::failure(
                "cannot repair legacy config .specsync.toml; run `specsync migrate` first"
                    .to_string(),
                Some(".specsync.toml".to_string()),
                None,
            )));
        }
        return Ok(legacy_init_report(".specsync.toml"));
    }

    // Subdir footgun: running `init` inside a subdirectory of an initialized
    // project must not create a nested .specsync — point at the parent instead.
    if let Some(parent) = find_initialized_ancestor(root) {
        return Err(Box::new(InitReport::failure(
            format!(
                "a spec-sync project is already initialized at {}; no nested .specsync was created",
                parent.display()
            ),
            None,
            Some(parent),
        )));
    }

    let (detected_dirs, actually_detected) = detect_source_dirs_with_confidence(root);
    preflight_layout(root).map_err(|error| Box::new(InitReport::failure(error, None, None)))?;
    for config in [
        ".specsync/config.toml",
        ".specsync/config.json",
        "specsync.json",
        ".specsync.toml",
    ] {
        preflight_absent_path(root, config)
            .map_err(|error| Box::new(InitReport::failure(error, None, None)))?;
    }

    if let Err(e) = write_current_layout(root, &detected_dirs) {
        return Err(Box::new(InitReport::failure(e, None, None)));
    }

    if let Err(error) = crate::change::write_default_policy(root, Vec::new()) {
        return Err(Box::new(InitReport::failure(error, None, None)));
    }

    let mut restored = Vec::new();
    let mut warnings = Vec::new();
    // Every file init just wrote is a protected SDD path, so without this record
    // the very next `specsync check` reports init's own output as uncovered
    // meaningful delivery — a gate no change workspace could have satisfied,
    // because none existed when the files were written. Non-fatal: a project
    // without Git evidence has no coverage gate to satisfy in the first place.
    if let Err(error) = crate::change::record_bootstrap_paths(root) {
        warnings.push(error);
    }
    // Ensure .specsync/hashes.json is gitignored (hash cache is local-only)
    match ensure_hashes_gitignored(root) {
        Ok(true) => restored.push(".gitignore entry (.specsync/hashes.json)".to_string()),
        Ok(false) => {}
        Err(error) => warnings.push(error),
    }

    Ok(InitReport {
        command: "init",
        success: true,
        created: true,
        repaired: false,
        unchanged: false,
        config: Some(".specsync/config.toml".to_string()),
        source_dirs: detected_dirs,
        detected: Some(actually_detected),
        source_dirs_detected: Some(actually_detected),
        missing: Vec::new(),
        restored,
        warnings,
        repair_hint: None,
        migration_hint: None,
        initialized_ancestor: None,
        error: None,
    })
}

fn legacy_init_report(config: &str) -> InitReport {
    InitReport {
        command: "init",
        success: true,
        created: false,
        repaired: false,
        unchanged: true,
        config: Some(config.to_string()),
        source_dirs: Vec::new(),
        detected: None,
        source_dirs_detected: None,
        missing: Vec::new(),
        restored: Vec::new(),
        warnings: Vec::new(),
        repair_hint: None,
        migration_hint: Some("specsync migrate"),
        initialized_ancestor: None,
        error: None,
    }
}

fn render_init_report(root: &Path, report: &InitReport, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string(report).expect("init report must serialize")
            );
        }
        OutputFormat::Markdown | OutputFormat::Github => {
            let status = if report.success { "success" } else { "failure" };
            println!("## SpecSync init\n");
            println!("- **Status:** {status}");
            println!("- **Created:** {}", report.created);
            println!("- **Repaired:** {}", report.repaired);
            println!("- **Unchanged:** {}", report.unchanged);
            if let Some(config) = &report.config {
                println!("- **Config:** `{config}`");
            }
            if let Some(error) = &report.error {
                println!("- **Error:** {error}");
            }
        }
        OutputFormat::Table => {
            println!("FIELD\tVALUE");
            println!(
                "status\t{}",
                if report.success { "success" } else { "failure" }
            );
            println!("created\t{}", report.created);
            println!("repaired\t{}", report.repaired);
            println!("unchanged\t{}", report.unchanged);
            if let Some(config) = &report.config {
                println!("config\t{config}");
            }
            if let Some(error) = &report.error {
                println!("error\t{error}");
            }
        }
        OutputFormat::Csv => {
            println!("command,success,created,repaired,unchanged,config,error");
            println!(
                "init,{},{},{},{},{},{}",
                report.success,
                report.created,
                report.repaired,
                report.unchanged,
                csv_field(report.config.as_deref().unwrap_or_default()),
                csv_field(report.error.as_deref().unwrap_or_default())
            );
        }
        OutputFormat::Text => render_init_text(root, report),
    }
}

fn render_init_text(root: &Path, report: &InitReport) {
    if !report.success {
        eprintln!(
            "{} {}",
            "error:".red().bold(),
            report.error.as_deref().unwrap_or("initialization failed")
        );
        if let Some(parent) = &report.initialized_ancestor {
            eprintln!("  Run from that root or pass `--root {parent}`.");
        }
        return;
    }

    if report.created {
        println!("{} Created .specsync/config.toml (5.0 layout)", "✓".green());
        println!(
            "  Detected source directories: {}",
            report.source_dirs.join(", ")
        );
        if report.source_dirs_detected != Some(true) {
            println!(
                "  {} No source directories detected — defaulted to source_dirs = [\"src\"].",
                "!".yellow()
            );
            println!("  Edit .specsync/config.toml if your sources live elsewhere.");
        }
        if report
            .restored
            .iter()
            .any(|path| path == ".gitignore entry (.specsync/hashes.json)")
        {
            println!("{} Added .specsync/hashes.json to .gitignore", "✓".green());
        }
    } else if report.repaired {
        if report.restored.is_empty() {
            println!(
                "{} Nothing to repair — .specsync layout is complete",
                "✓".green()
            );
        } else {
            println!("{} Repaired: {}", "✓".green(), report.restored.join(", "));
        }
    } else if let Some(config) = &report.config {
        if report.migration_hint.is_some() {
            println!("{config} already exists (legacy 3.x layout — run `specsync migrate`)");
        } else {
            println!("{config} already exists");
            print_legacy_policy_notice(root);
            if !report.missing.is_empty() {
                println!(
                    "  {} missing support file(s): {}",
                    "!".yellow(),
                    report.missing.join(", ")
                );
                println!(
                    "  Run `specsync init --repair` to restore them (your config is left untouched)."
                );
            }
        }
    }
    for warning in &report.warnings {
        eprintln!("{} {warning}", "warning:".yellow().bold());
    }
}

fn root_gitignore_has_hash_entry(root: &Path) -> bool {
    fs::read(root.join(".gitignore"))
        .ok()
        .is_some_and(|content| gitignore_has_hash_entry(&content))
}

fn gitignore_has_hash_entry(content: &[u8]) -> bool {
    content.split(|byte| *byte == b'\n').any(|line| {
        let start = line
            .iter()
            .position(|byte| !byte.is_ascii_whitespace())
            .unwrap_or(line.len());
        let end = line
            .iter()
            .rposition(|byte| !byte.is_ascii_whitespace())
            .map_or(start, |index| index + 1);
        &line[start..end] == b".specsync/hashes.json"
    })
}

fn preflight_layout(root: &Path) -> Result<(), String> {
    preflight_directory(root, ".specsync")?;
    for directory in V4_DIRS {
        preflight_directory(root, directory)?;
    }
    for file in [
        ".specsync/version",
        ".specsync/.gitignore",
        ".specsync/sdd.json",
    ] {
        preflight_regular_file(root, file)?;
    }
    Ok(())
}

fn preflight_directory(root: &Path, relative: &str) -> Result<(), String> {
    let path = root.join(relative);
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err(format!("{relative} is a symlink; expected a directory"))
        }
        Ok(metadata) if !metadata.is_dir() => Err(format!(
            "{relative} blocks initialization; expected a directory"
        )),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("failed to inspect {relative}: {error}")),
    }
}

fn preflight_regular_file(root: &Path, relative: &str) -> Result<(), String> {
    let path = root.join(relative);
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err(format!("{relative} is a symlink; expected a regular file"))
        }
        Ok(metadata) if !metadata.is_file() => Err(format!(
            "{relative} blocks initialization; expected a regular file"
        )),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("failed to inspect {relative}: {error}")),
    }
}

fn preflight_absent_path(root: &Path, relative: &str) -> Result<(), String> {
    match fs::symlink_metadata(root.join(relative)) {
        Ok(_) => Err(format!(
            "{relative} already occupies a configuration path; initialization will not overwrite it"
        )),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("failed to inspect {relative}: {error}")),
    }
}

/// Support files a healthy `.specsync/` layout must have; missing ones are
/// reported by re-init and restored by `init --repair`.
fn missing_support_files(root: &Path) -> Vec<String> {
    let mut missing = Vec::new();
    for file in [
        ".specsync/version",
        ".specsync/.gitignore",
        ".specsync/sdd.json",
    ] {
        if !root.join(file).is_file() {
            missing.push(file.to_string());
        }
    }
    for dir in V4_DIRS {
        if !root.join(dir).is_dir() {
            missing.push(format!("{dir}/"));
        }
    }
    if !root_gitignore_has_hash_entry(root) {
        missing.push(".gitignore entry (.specsync/hashes.json)".to_string());
    }
    missing
}

/// Restore missing `.specsync/` support files without touching the existing
/// config. Also sanity-checks that the config is at least plausible.
fn repair_layout(
    root: &Path,
    existing_config: &str,
    config_path: &Path,
) -> Result<InitReport, String> {
    validate_config_file(config_path)?;
    preflight_layout(root)?;

    let mut restored: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    for dir in V4_DIRS {
        let path = root.join(dir);
        if !path.is_dir() {
            fs::create_dir_all(&path)
                .map_err(|error| format!("failed to create {dir}: {error}"))?;
            restored.push(format!("{dir}/"));
        }
    }

    let version_path = root.join(".specsync/version");
    if !version_path.is_file() {
        write_new_file(
            &version_path,
            format!("{PROJECT_VERSION}\n").as_bytes(),
            ".specsync/version",
        )?;
        restored.push(".specsync/version".to_string());
    }

    let gitignore_path = root.join(".specsync/.gitignore");
    if !gitignore_path.is_file() {
        write_new_file(
            &gitignore_path,
            specsync_gitignore_content().as_bytes(),
            ".specsync/.gitignore",
        )?;
        restored.push(".specsync/.gitignore".to_string());
    }

    // sdd.json — write_default_policy is a no-op when the file exists.
    let had_policy = root.join(".specsync/sdd.json").is_file();
    crate::change::write_default_policy(root, Vec::new())?;
    if !had_policy && root.join(".specsync/sdd.json").is_file() {
        restored.push(".specsync/sdd.json".to_string());
    }

    match ensure_hashes_gitignored(root) {
        Ok(true) => restored.push(".gitignore entry (.specsync/hashes.json)".to_string()),
        Ok(false) => {}
        Err(e) => warnings.push(e),
    }

    Ok(InitReport {
        command: "init",
        success: true,
        created: false,
        repaired: true,
        unchanged: restored.is_empty(),
        config: Some(existing_config.to_string()),
        source_dirs: Vec::new(),
        detected: None,
        source_dirs_detected: None,
        missing: Vec::new(),
        restored,
        warnings,
        repair_hint: None,
        migration_hint: None,
        initialized_ancestor: None,
        error: None,
    })
}

/// Walk up from `root` to find an ancestor directory that already contains a
/// spec-sync project (any layout). Returns None when `root` is the topmost
/// initialized directory.
fn find_initialized_ancestor(root: &Path) -> Option<std::path::PathBuf> {
    let abs = root.canonicalize().ok()?;
    for ancestor in abs.ancestors().skip(1) {
        if ancestor.join(".specsync/config.toml").is_file()
            || ancestor.join(".specsync/config.json").is_file()
            || ancestor.join("specsync.json").is_file()
            || ancestor.join(".specsync.toml").is_file()
        {
            return Some(ancestor.to_path_buf());
        }
    }
    None
}

/// Contents of the generated `.specsync/.gitignore`.
fn specsync_gitignore_content() -> String {
    [
        "# spec-sync 5.0 — generated by `specsync init`",
        "# Committed: config.toml, registry.toml, lifecycle/, changes/, archive/",
        "# Ignored: backups, local config, hash cache (regenerated on each run)",
        "",
        "backup-3x/",
        "config.local.toml",
        "hashes.json",
        "change.lock",
        "change-transaction.json",
        "",
    ]
    .join("\n")
}

/// Create the 5.0 `.specsync/` directory structure: directories, `config.toml`,
/// `version` stamp, and `.specsync/.gitignore`. Mirrors what `specsync migrate`
/// produces so a fresh `init` never triggers the legacy-layout migration nag.
fn write_current_layout(root: &Path, source_dirs: &[String]) -> Result<(), String> {
    for dir in V4_DIRS {
        fs::create_dir_all(root.join(dir)).map_err(|e| format!("Failed to create {dir}: {e}"))?;
    }

    let config = SpecSyncConfig {
        source_dirs: source_dirs.to_vec(),
        ..Default::default()
    };
    let config_path = root.join(".specsync/config.toml");
    write_new_file(
        &config_path,
        config_to_toml(&config).as_bytes(),
        ".specsync/config.toml",
    )?;

    let version_path = root.join(".specsync/version");
    if !version_path.exists() {
        write_new_file(
            &version_path,
            format!("{PROJECT_VERSION}\n").as_bytes(),
            ".specsync/version",
        )?;
    }

    let gitignore_path = root.join(".specsync/.gitignore");
    if !gitignore_path.exists() {
        write_new_file(
            &gitignore_path,
            specsync_gitignore_content().as_bytes(),
            ".specsync/.gitignore",
        )?;
    }

    Ok(())
}

fn write_new_file(path: &Path, bytes: &[u8], display: &str) -> Result<(), String> {
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .and_then(|mut file| file.write_all(bytes))
        .map_err(|error| format!("Failed to write {display}: {error}"))
}

/// Append `.specsync/hashes.json` to the root `.gitignore` if not already present.
/// Returns `Ok(true)` if added, `Ok(false)` if already present, `Err` on write failure.
pub fn ensure_hashes_gitignored(root: &Path) -> Result<bool, String> {
    let gitignore_path = root.join(".gitignore");
    let existing = match fs::symlink_metadata(&gitignore_path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(
                "Refusing to update symlinked .gitignore; add .specsync/hashes.json manually"
                    .to_string(),
            );
        }
        Ok(metadata) if !metadata.is_file() => {
            return Err(
                "Refusing to update .gitignore because it is not a regular file".to_string(),
            );
        }
        Ok(_) => fs::read(&gitignore_path)
            .map_err(|error| format!("Failed to read .gitignore: {error}"))?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => return Err(format!("Failed to inspect .gitignore: {error}")),
    };
    if gitignore_has_hash_entry(&existing) {
        return Ok(false);
    }

    let mut addition = Vec::new();
    if !existing.is_empty() {
        if !existing.ends_with(b"\n") {
            addition.push(b'\n');
        }
        addition.push(b'\n');
    }
    addition.extend_from_slice(
        b"# spec-sync hash cache (regenerated locally)\n.specsync/hashes.json\n",
    );

    let mut options = fs::OpenOptions::new();
    options.write(true);
    if existing.is_empty() && !gitignore_path.exists() {
        options.create_new(true);
    } else {
        options.append(true);
    }
    options
        .open(&gitignore_path)
        .and_then(|mut file| file.write_all(&addition))
        .map_err(|error| format!("Failed to update .gitignore: {error}"))?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn adds_entry_to_missing_gitignore() {
        let tmp = TempDir::new().unwrap();
        let changed = ensure_hashes_gitignored(tmp.path()).unwrap();
        assert!(changed, "should report it wrote the entry");

        let content = fs::read_to_string(tmp.path().join(".gitignore")).unwrap();
        assert!(content.contains(".specsync/hashes.json"));
    }

    #[test]
    fn is_idempotent_when_entry_already_present() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join(".gitignore"),
            "target/\n.specsync/hashes.json\n",
        )
        .unwrap();

        let changed = ensure_hashes_gitignored(tmp.path()).unwrap();
        assert!(!changed, "entry already present — nothing to add");

        // Existing content is untouched (no duplicate entry appended).
        let content = fs::read_to_string(tmp.path().join(".gitignore")).unwrap();
        assert_eq!(content.matches(".specsync/hashes.json").count(), 1);
    }

    #[test]
    fn errors_when_gitignore_path_is_unwritable() {
        let tmp = TempDir::new().unwrap();
        // Make `.gitignore` a directory so the write fails — exercises the
        // error path that maps the io::Error into a String.
        fs::create_dir(tmp.path().join(".gitignore")).unwrap();

        let result = ensure_hashes_gitignored(tmp.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not a regular file"));
    }

    #[cfg(unix)]
    #[test]
    fn preserves_non_utf8_gitignore_bytes_when_appending() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join(".gitignore");
        let original = b"target/\n\xffbinary\n";
        fs::write(&path, original).unwrap();

        assert!(ensure_hashes_gitignored(tmp.path()).unwrap());

        let updated = fs::read(path).unwrap();
        assert!(updated.starts_with(original));
        assert!(gitignore_has_hash_entry(&updated));
    }

    #[cfg(unix)]
    #[test]
    fn refuses_symlinked_root_gitignore_without_touching_target() {
        use std::os::unix::fs::symlink;

        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("outside-ignore");
        fs::write(&target, b"keep-me\n").unwrap();
        symlink(&target, tmp.path().join(".gitignore")).unwrap();

        let error = ensure_hashes_gitignored(tmp.path()).unwrap_err();
        assert!(error.contains("symlinked .gitignore"));
        assert_eq!(fs::read(target).unwrap(), b"keep-me\n");
    }

    #[test]
    fn write_current_layout_creates_full_structure() {
        let tmp = TempDir::new().unwrap();
        write_current_layout(tmp.path(), &["src".to_string()]).unwrap();

        assert!(tmp.path().join(".specsync/config.toml").exists());
        assert!(tmp.path().join(".specsync/version").exists());
        assert!(tmp.path().join(".specsync/.gitignore").exists());
        assert!(tmp.path().join(".specsync/lifecycle").is_dir());
        assert!(tmp.path().join(".specsync/changes").is_dir());
        assert!(tmp.path().join(".specsync/archive").is_dir());

        let version = fs::read_to_string(tmp.path().join(".specsync/version")).unwrap();
        assert_eq!(version.trim(), PROJECT_VERSION);
        assert!(tmp.path().join(".specsync/sdd.json").exists() == false);

        let config = fs::read_to_string(tmp.path().join(".specsync/config.toml")).unwrap();
        assert!(config.contains("specs_dir = \"specs\""));
        assert!(config.contains("source_dirs = [\"src\"]"));
        assert!(config.contains("required_sections"));
    }

    #[test]
    fn fresh_init_is_not_legacy_layout() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("src")).unwrap();
        cmd_init(tmp.path(), false, crate::types::OutputFormat::Text);
        assert!(
            !crate::config::is_legacy_layout(tmp.path()),
            "fresh init must produce a 5.0 layout that does not trigger the migration nag"
        );
        assert!(tmp.path().join(".specsync/sdd.json").is_file());
    }

    #[test]
    fn generated_policy_includes_detected_source_directories() {
        for source_dir in ["lib", "."] {
            let tmp = TempDir::new().unwrap();
            write_current_layout(tmp.path(), &[source_dir.to_string()]).unwrap();
            crate::change::write_default_policy(tmp.path(), Vec::new()).unwrap();
            let policy = crate::change::load_policy(tmp.path()).unwrap();
            let expected = if source_dir == "." { "." } else { "lib/" };
            assert!(
                policy.meaningful_paths.iter().any(|path| path == expected),
                "missing source policy scope {expected}"
            );
        }
    }

    #[test]
    fn empty_dir_init_reports_fallback_not_detection() {
        // #440: init must not claim it "detected" src in an empty directory.
        let tmp = TempDir::new().unwrap();
        let (dirs, detected) = crate::config::detect_source_dirs_with_confidence(tmp.path());
        assert_eq!(dirs, vec!["src".to_string()]);
        assert!(!detected, "empty dir must report fallback, not detection");
    }

    #[test]
    fn repair_restores_deleted_support_files() {
        // #440: re-init short-circuits; --repair restores version/.gitignore/sdd.json.
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        cmd_init(root, false, crate::types::OutputFormat::Text);
        fs::remove_file(root.join(".specsync/version")).unwrap();
        fs::remove_file(root.join(".specsync/.gitignore")).unwrap();
        fs::remove_file(root.join(".specsync/sdd.json")).unwrap();
        assert_eq!(missing_support_files(root).len(), 3);

        cmd_init(root, true, crate::types::OutputFormat::Text);

        assert!(root.join(".specsync/version").is_file());
        assert!(root.join(".specsync/.gitignore").is_file());
        assert!(root.join(".specsync/sdd.json").is_file());
        assert!(missing_support_files(root).is_empty());
    }

    #[test]
    fn fresh_init_leaves_sdd_off_so_the_first_check_is_just_drift() {
        // Honest label: DISCRIMINATOR. On the unfixed binary init enabled SDD
        // and `check_project` required an active change covering `.specsync/`.
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let git = |args: &[&str]| {
            assert!(
                std::process::Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-q", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();
        git(&["add", "-A"]);
        git(&["commit", "-qm", "init"]);

        cmd_init(root, false, crate::types::OutputFormat::Text);
        let policy = crate::change::load_policy(root).unwrap();
        assert!(
            !policy.enabled,
            "fresh init must leave SDD off so `check` is the product"
        );
        assert!(
            !policy.require_change_for_meaningful_files,
            "fresh init must not require an active change for dirty paths"
        );
        let report = crate::change::check_project(root);
        assert!(
            report.errors.is_empty(),
            "SDD-off check_project must not fail: {:?}",
            report.errors
        );
        assert!(
            !report.enabled,
            "disabled policy must not report SDD as enabled"
        );
    }

    #[test]
    fn find_initialized_ancestor_detects_parent_project() {
        // #440: init in a subdir must find the parent project, not nest.
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        cmd_init(root, false, crate::types::OutputFormat::Text);
        let sub = root.join("src/deep");
        fs::create_dir_all(&sub).unwrap();
        let found = find_initialized_ancestor(&sub).expect("parent project detected");
        assert_eq!(found, root.canonicalize().unwrap());
        assert!(find_initialized_ancestor(root).is_none());
    }
}

/// Say so when re-running `init` on a project that is still on the legacy lifecycle.
///
/// `init` short-circuits on an existing project, and "already exists" reads as "nothing to do
/// here". For a repository upgraded from 5.x it is not: the policy still carries `version: 1`, so
/// every change created is workflow-v1, and nothing reports that until `ship` refuses. This is the
/// one moment `init` has the reader's attention with the fact in hand.
fn print_legacy_policy_notice(root: &Path) {
    if root.join(".specsync/workflow-v2-baseline.json").is_file() {
        return;
    }
    let Ok(text) = fs::read_to_string(root.join(".specsync/sdd.json")) else {
        return;
    };
    let Ok(policy) = serde_json::from_str::<serde_json::Value>(&text) else {
        return;
    };
    if policy
        .get("version")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(2)
        >= 2
    {
        return;
    }
    println!(
        "  {} this project is on workflow v1 (legacy) — new changes will use `change accept`/`change archive`, not `change finalize`",
        "!".yellow()
    );
    println!(
        "  Run `specsync change adopt` to adopt the current lifecycle for new changes (existing v1 evidence is preserved)."
    );
}
