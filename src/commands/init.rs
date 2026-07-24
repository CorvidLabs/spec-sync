use colored::Colorize;
use dialoguer::{Confirm, Input};
use std::fs;
use std::io::IsTerminal;
use std::path::Path;
use std::process;

use crate::config::{config_to_toml, detect_source_dirs_with_confidence};
use crate::types::SpecSyncConfig;

/// Version stamp written to `.specsync/version` for fresh projects.
const PROJECT_VERSION: &str = crate::change::SDD_VERSION;

/// Subdirectories created inside `.specsync/` for a fresh 5.0 project.
const V4_DIRS: &[&str] = &[
    ".specsync/lifecycle",
    ".specsync/changes",
    ".specsync/archive",
    ".specsync/archive/changes",
];

pub fn cmd_init(root: &Path, repair: bool, format: crate::types::OutputFormat) {
    let json = matches!(format, crate::types::OutputFormat::Json);
    // Refuse to clobber any existing config — current or legacy.
    let v4_toml = root.join(".specsync/config.toml");
    let v4_json = root.join(".specsync/config.json");
    let legacy_json = root.join("specsync.json");
    let legacy_toml = root.join(".specsync.toml");
    if v4_toml.exists() || v4_json.exists() {
        let existing = if v4_toml.exists() {
            ".specsync/config.toml"
        } else {
            ".specsync/config.json"
        };
        if repair {
            repair_layout(root, existing, json);
            return;
        }
        let missing = missing_support_files(root);
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "created": false,
                    "config": existing,
                    "missing": missing,
                    "repair_hint": "specsync init --repair",
                })
            );
        } else {
            println!("{existing} already exists");
            if !missing.is_empty() {
                println!(
                    "  {} missing support file(s): {}",
                    "!".yellow(),
                    missing.join(", ")
                );
                println!(
                    "  Run `specsync init --repair` to restore them (your config is left untouched)."
                );
            }
        }
        return;
    }
    if legacy_json.exists() {
        println!("specsync.json already exists (legacy 3.x layout — run `specsync migrate`)");
        return;
    }
    if legacy_toml.exists() {
        println!(".specsync.toml already exists (legacy 3.x layout — run `specsync migrate`)");
        return;
    }

    // Subdir footgun: running `init` inside a subdirectory of an initialized
    // project must not create a nested .specsync — point at the parent instead.
    if let Some(parent) = find_initialized_ancestor(root) {
        eprintln!(
            "{} A spec-sync project is already initialized at {} — no nested .specsync was created.",
            "error:".red().bold(),
            parent.display()
        );
        eprintln!(
            "  Run from that root or pass `--root {}`.",
            parent.display()
        );
        process::exit(1);
    }

    let (detected_dirs, actually_detected) = detect_source_dirs_with_confidence(root);
    let dirs_display = detected_dirs.join(", ");

    if let Err(e) = write_current_layout(root, &detected_dirs) {
        eprintln!("{} {e}", "error:".red().bold());
        process::exit(1);
    }

    if let Err(error) =
        crate::change::write_default_policy(root, crate::change::detect_verification_commands(root))
    {
        eprintln!("{} {error}", "error:".red().bold());
        process::exit(1);
    }

    if json {
        println!(
            "{}",
            serde_json::json!({
                "created": true,
                "config": ".specsync/config.toml",
                "source_dirs": detected_dirs,
                "detected": actually_detected,
            })
        );
    } else {
        println!("{} Created .specsync/config.toml (5.0 layout)", "✓".green());
        if actually_detected {
            println!("  Detected source directories: {dirs_display}");
        } else {
            println!(
                "  {} No source directories detected — defaulted to source_dirs = [\"src\"].",
                "!".yellow()
            );
            println!("  Edit .specsync/config.toml if your sources live elsewhere.");
        }
    }

    // Ensure .specsync/hashes.json is gitignored (hash cache is local-only)
    match ensure_hashes_gitignored(root) {
        Ok(true) if !json => println!("{} Added .specsync/hashes.json to .gitignore", "✓".green()),
        Ok(_) => {}
        Err(e) => eprintln!("{} {e}", "warning:".yellow().bold()),
    }

    guided_sdd_bootstrap(root);
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
    missing
}

/// Restore missing `.specsync/` support files without touching the existing
/// config. Also sanity-checks that the config is at least plausible.
fn repair_layout(root: &Path, existing_config: &str, json: bool) {
    let mut restored: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // Sanity-check the existing config so a corrupt one is diagnosed, not ignored.
    if let Ok(content) = fs::read_to_string(root.join(existing_config))
        && !content.contains("specs_dir")
    {
        warnings.push(format!(
            "{existing_config} has no `specs_dir` key — it may be corrupt; fix or regenerate it manually"
        ));
    }

    for dir in V4_DIRS {
        let path = root.join(dir);
        if !path.is_dir() {
            match fs::create_dir_all(&path) {
                Ok(_) => restored.push(format!("{dir}/")),
                Err(e) => {
                    eprintln!("{} Failed to create {dir}: {e}", "error:".red().bold());
                    process::exit(1);
                }
            }
        }
    }

    let version_path = root.join(".specsync/version");
    if !version_path.is_file() {
        match fs::write(&version_path, format!("{PROJECT_VERSION}\n")) {
            Ok(_) => restored.push(".specsync/version".to_string()),
            Err(e) => {
                eprintln!("{} Failed to write .specsync/version: {e}", "error:".red().bold());
                process::exit(1);
            }
        }
    }

    let gitignore_path = root.join(".specsync/.gitignore");
    if !gitignore_path.is_file() {
        match fs::write(&gitignore_path, specsync_gitignore_content()) {
            Ok(_) => restored.push(".specsync/.gitignore".to_string()),
            Err(e) => {
                eprintln!(
                    "{} Failed to write .specsync/.gitignore: {e}",
                    "error:".red().bold()
                );
                process::exit(1);
            }
        }
    }

    // sdd.json — write_default_policy is a no-op when the file exists.
    let had_policy = root.join(".specsync/sdd.json").is_file();
    if let Err(error) =
        crate::change::write_default_policy(root, crate::change::detect_verification_commands(root))
    {
        eprintln!("{} {error}", "error:".red().bold());
        process::exit(1);
    }
    if !had_policy && root.join(".specsync/sdd.json").is_file() {
        restored.push(".specsync/sdd.json".to_string());
    }

    match ensure_hashes_gitignored(root) {
        Ok(true) => restored.push(".gitignore entry (.specsync/hashes.json)".to_string()),
        Ok(false) => {}
        Err(e) => warnings.push(e),
    }

    if json {
        println!(
            "{}",
            serde_json::json!({
                "repaired": restored,
                "warnings": warnings,
            })
        );
    } else {
        if restored.is_empty() {
            println!("{} Nothing to repair — .specsync layout is complete", "✓".green());
        } else {
            println!("{} Repaired: {}", "✓".green(), restored.join(", "));
        }
        for warning in &warnings {
            eprintln!("{} {warning}", "warning:".yellow().bold());
        }
    }
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
fn guided_sdd_bootstrap(root: &Path) {
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return;
    }
    let install_agents = Confirm::new()
        .with_prompt("Install native spec-sync SDD skills for supported AI agents?")
        .default(true)
        .interact()
        .unwrap_or(false);
    if install_agents {
        crate::agents::cmd_install(root, &[]);
    }
    let create_first = Confirm::new()
        .with_prompt("Create the project's first verified SDD change now?")
        .default(true)
        .interact()
        .unwrap_or(false);
    if !create_first {
        return;
    }
    let description: String = match Input::new()
        .with_prompt("What do you want to change?")
        .interact_text()
    {
        Ok(value) => value,
        Err(error) => {
            eprintln!(
                "{} Could not start the interview: {error}",
                "warning:".yellow()
            );
            return;
        }
    };
    let request = crate::change::CreateChangeRequest {
        description,
        kind: crate::change::ChangeKind::Feature,
        affected_specs: Vec::new(),
        affected_paths: vec!["src/".into()],
        requested_artifacts: Vec::new(),
        no_spec_change: false,
        rationale: None,
    };
    match crate::change::create_change(root, request) {
        Ok(record) => {
            println!("{} Created {}", "✓".green(), record.id);
            println!("  Continue with: specsync change show {}", record.id);
        }
        Err(error) => eprintln!("{} {error}", "warning:".yellow().bold()),
    }
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
    fs::write(&config_path, config_to_toml(&config))
        .map_err(|e| format!("Failed to write .specsync/config.toml: {e}"))?;

    let version_path = root.join(".specsync/version");
    fs::write(&version_path, format!("{PROJECT_VERSION}\n"))
        .map_err(|e| format!("Failed to write .specsync/version: {e}"))?;

    let gitignore_path = root.join(".specsync/.gitignore");
    if !gitignore_path.exists() {
        fs::write(&gitignore_path, specsync_gitignore_content())
            .map_err(|e| format!("Failed to write .specsync/.gitignore: {e}"))?;
    }

    Ok(())
}

/// Append `.specsync/hashes.json` to the root `.gitignore` if not already present.
/// Returns `Ok(true)` if added, `Ok(false)` if already present, `Err` on write failure.
pub fn ensure_hashes_gitignored(root: &Path) -> Result<bool, String> {
    let gitignore_path = root.join(".gitignore");
    let entry = ".specsync/hashes.json";

    let existing = fs::read_to_string(&gitignore_path).unwrap_or_default();
    if existing.lines().any(|line| line.trim() == entry) {
        return Ok(false);
    }

    let mut content = existing;
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(&format!(
        "\n# spec-sync hash cache (regenerated locally)\n{entry}\n"
    ));

    fs::write(&gitignore_path, content).map_err(|e| format!("Failed to update .gitignore: {e}"))?;
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
        assert!(result.unwrap_err().contains("Failed to update .gitignore"));
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
