use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process;

use crate::config::load_config;
use crate::registry;

pub fn cmd_init_registry(root: &Path, name: Option<String>, format: crate::types::OutputFormat) {
    let json = matches!(format, crate::types::OutputFormat::Json);
    // Respect the project layout: v4 projects get .specsync/registry.toml,
    // un-migrated 3.x projects keep the legacy root-level file.
    let registry_path = registry::local_registry_path(root);
    // Normalize to forward slashes so output matches the tool's path style on every platform.
    let rel_display = registry_path
        .strip_prefix(root)
        .unwrap_or(&registry_path)
        .display()
        .to_string()
        .replace('\\', "/");
    if registry_path.exists() {
        if json {
            println!(
                "{}",
                serde_json::json!({"created": false, "registry": rel_display})
            );
        } else {
            println!("{rel_display} already exists — nothing to do");
            println!("  Delete it first (or edit it by hand) to regenerate.");
        }
        return;
    }

    let config = load_config(root);
    let project_name = name.unwrap_or_else(|| {
        root.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project")
            .to_string()
    });
    if project_name.trim().is_empty() {
        eprintln!(
            "{} Registry name must not be empty — pass --name with a non-empty project name.",
            "Error:".red()
        );
        process::exit(1);
    }

    let content = registry::generate_registry(root, &project_name, &config.specs_dir);
    if let Some(parent) = registry_path.parent()
        && let Err(e) = fs::create_dir_all(parent)
    {
        eprintln!("Failed to create {}: {e}", parent.display());
        process::exit(1);
    }
    match fs::write(&registry_path, &content) {
        Ok(_) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({"created": true, "registry": rel_display, "name": project_name})
                );
            } else {
                println!("{} Created {rel_display}", "✓".green());
            }
        }
        Err(e) => {
            eprintln!("Failed to write {rel_display}: {e}");
            process::exit(1);
        }
    }
}
