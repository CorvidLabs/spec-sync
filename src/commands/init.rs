use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process;

use crate::config::detect_source_dirs;

pub fn cmd_init(root: &Path) {
    let config_path = root.join("specsync.json");
    let toml_path = root.join(".specsync.toml");
    if config_path.exists() {
        println!("specsync.json already exists");
        return;
    }
    if toml_path.exists() {
        println!(".specsync.toml already exists");
        return;
    }

    let detected_dirs = detect_source_dirs(root);
    let dirs_display = detected_dirs.join(", ");

    let default = serde_json::json!({
        "specsDir": "specs",
        "sourceDirs": detected_dirs,
        "requiredSections": [
            "Purpose",
            "Public API",
            "Invariants",
            "Behavioral Examples",
            "Error Cases",
            "Dependencies",
            "Change Log"
        ],
        "excludeDirs": ["__tests__"],
        "excludePatterns": ["**/__tests__/**", "**/*.test.ts", "**/*.spec.ts"]
    });

    let content = serde_json::to_string_pretty(&default).unwrap() + "\n";
    if let Err(e) = fs::write(&config_path, content) {
        eprintln!(
            "{} Failed to write specsync.json: {e}",
            "error:".red().bold()
        );
        process::exit(1);
    }
    println!("{} Created specsync.json", "✓".green());
    println!("  Detected source directories: {dirs_display}");

    // Ensure .specsync/hashes.json is gitignored (hash cache is local-only)
    match ensure_hashes_gitignored(root) {
        Ok(true) => println!("{} Added .specsync/hashes.json to .gitignore", "✓".green()),
        Ok(false) => {}
        Err(e) => eprintln!("{} {e}", "warning:".yellow().bold()),
    }
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
