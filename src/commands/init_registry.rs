use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process;

use crate::config::load_config;
use crate::registry;

pub fn cmd_init_registry(root: &Path, name: Option<String>) {
    let registry_path = root.join("specsync-registry.toml");
    if registry_path.exists() {
        println!("specsync-registry.toml already exists");
        return;
    }

    let config = load_config(root);
    let project_name = name.unwrap_or_else(|| {
        root.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project")
            .to_string()
    });

    let content = registry::generate_registry(root, &project_name, &config.specs_dir);
    match fs::write(&registry_path, &content) {
        Ok(_) => {
            println!("{} Created specsync-registry.toml", "✓".green());
        }
        Err(e) => {
            eprintln!("Failed to write specsync-registry.toml: {e}");
            process::exit(1);
        }
    }
}
