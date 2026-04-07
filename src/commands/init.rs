use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process;

use crate::config::{detect_source_dirs, load_config};
use crate::generator;
use crate::registry;

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

pub fn cmd_add_spec(root: &Path, module_name: &str) {
    let config = load_config(root);
    let specs_dir = root.join(&config.specs_dir);
    let spec_dir = specs_dir.join(module_name);
    let spec_file = spec_dir.join(format!("{module_name}.spec.md"));

    if spec_file.exists() {
        println!(
            "{} Spec already exists: {}",
            "!".yellow(),
            spec_file.strip_prefix(root).unwrap_or(&spec_file).display()
        );
        // Still generate companion files if missing
        generator::generate_companion_files_for_spec(&spec_dir, module_name);
        return;
    }

    if let Err(e) = fs::create_dir_all(&spec_dir) {
        eprintln!("Failed to create {}: {e}", spec_dir.display());
        process::exit(1);
    }

    // Find any matching source files
    let module_files: Vec<String> = config
        .source_dirs
        .iter()
        .flat_map(|src_dir| {
            let module_dir = root.join(src_dir).join(module_name);
            if module_dir.exists() {
                walkdir::WalkDir::new(&module_dir)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.path().is_file()
                            && crate::exports::has_extension(e.path(), &config.source_extensions)
                    })
                    .map(|e| {
                        e.path()
                            .strip_prefix(root)
                            .unwrap_or(e.path())
                            .to_string_lossy()
                            .replace('\\', "/")
                    })
                    .collect::<Vec<_>>()
            } else {
                vec![]
            }
        })
        .collect();

    let spec_content = build_spec_template(module_name, &module_files);

    match fs::write(&spec_file, &spec_content) {
        Ok(_) => {
            let rel = spec_file.strip_prefix(root).unwrap_or(&spec_file).display();
            println!("  {} Created {rel}", "✓".green());
            generator::generate_companion_files_for_spec(&spec_dir, module_name);
        }
        Err(e) => {
            eprintln!("Failed to write {}: {e}", spec_file.display());
            process::exit(1);
        }
    }
}

fn build_spec_template(module_name: &str, module_files: &[String]) -> String {
    let title = module_name
        .split('-')
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let files_yaml = if module_files.is_empty() {
        "  # - path/to/source/file".to_string()
    } else {
        module_files
            .iter()
            .map(|f| format!("  - {f}"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        r#"---
module: {module_name}
version: 1
status: draft
files:
{files_yaml}
db_tables: []
depends_on: []
---

# {title}

## Purpose

<!-- TODO: describe what this module does -->

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|

### Exported Types

| Type | Description |
|------|-------------|

## Invariants

1. <!-- TODO -->

## Behavioral Examples

### Scenario: TODO

- **Given** precondition
- **When** action
- **Then** result

## Error Cases

| Condition | Behavior |
|-----------|----------|

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|

### Consumed By

| Module | What is used |
|--------|-------------|

## Change Log

| Date | Author | Change |
|------|--------|--------|
"#
    )
}

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
