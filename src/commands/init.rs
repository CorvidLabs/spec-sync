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
}
