use colored::Colorize;
use dialoguer::{Confirm, Input};
use std::fs;
use std::io::IsTerminal;
use std::path::Path;
use std::process;

use crate::config::{config_to_toml, detect_source_dirs};
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

pub fn cmd_init(root: &Path) {
    // Refuse to clobber any existing config — current or legacy.
    let v4_toml = root.join(".specsync/config.toml");
    let v4_json = root.join(".specsync/config.json");
    let legacy_json = root.join("specsync.json");
    let legacy_toml = root.join(".specsync.toml");
    if v4_toml.exists() {
        println!(".specsync/config.toml already exists");
        return;
    }
    if v4_json.exists() {
        println!(".specsync/config.json already exists");
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

    let detected_dirs = detect_source_dirs(root);
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

    println!("{} Created .specsync/config.toml (5.0 layout)", "✓".green());
    println!("  Detected source directories: {dirs_display}");

    // Ensure .specsync/hashes.json is gitignored (hash cache is local-only)
    match ensure_hashes_gitignored(root) {
        Ok(true) => println!("{} Added .specsync/hashes.json to .gitignore", "✓".green()),
        Ok(false) => {}
        Err(e) => eprintln!("{} {e}", "warning:".yellow().bold()),
    }

    guided_sdd_bootstrap(root);
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
        let content = [
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
        .join("\n");
        fs::write(&gitignore_path, content)
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
        cmd_init(tmp.path());
        assert!(
            !crate::config::is_legacy_layout(tmp.path()),
            "fresh init must produce a 5.0 layout that does not trigger the migration nag"
        );
        assert!(tmp.path().join(".specsync/sdd.json").is_file());
    }
}
