use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use crate::config::load_config;
use crate::generator;
use crate::registry;

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
        generator::generate_companion_files_for_spec(
            &spec_dir,
            module_name,
            config.companions.design,
        );
        return;
    }

    if let Err(e) = fs::create_dir_all(&spec_dir) {
        eprintln!("Failed to create {}: {e}", spec_dir.display());
        process::exit(1);
    }

    // Use the template-based generator (no AI for add-spec)
    let template_path = specs_dir.join("_template.spec.md");
    let template = if template_path.exists() {
        fs::read_to_string(&template_path).unwrap_or_default()
    } else {
        String::new()
    };

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

    let _ = template; // Template handling is done by generate_spec internal

    // Generate spec content using the internal generate function
    let spec_content = {
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
    };

    match fs::write(&spec_file, &spec_content) {
        Ok(_) => {
            let rel = spec_file.strip_prefix(root).unwrap_or(&spec_file).display();
            println!("  {} Created {rel}", "✓".green());
            generator::generate_companion_files_for_spec(
                &spec_dir,
                module_name,
                config.companions.design,
            );
        }
        Err(e) => {
            eprintln!("Failed to write {}: {e}", spec_file.display());
            process::exit(1);
        }
    }
}

pub fn cmd_scaffold(
    root: &Path,
    module_name: &str,
    dir: Option<PathBuf>,
    template: Option<PathBuf>,
) {
    let config = load_config(root);
    let specs_dir = dir.unwrap_or_else(|| root.join(&config.specs_dir));
    let spec_dir = specs_dir.join(module_name);
    let spec_file = spec_dir.join(format!("{module_name}.spec.md"));

    if spec_file.exists() {
        println!(
            "{} Spec already exists: {}",
            "!".yellow(),
            spec_file.strip_prefix(root).unwrap_or(&spec_file).display()
        );
        // Still generate companion files if missing
        if let Some(ref tpl_dir) = template {
            generator::generate_companion_files_from_template(
                &spec_dir,
                module_name,
                tpl_dir,
                config.companions.design,
            );
        } else {
            generator::generate_companion_files_for_spec(
                &spec_dir,
                module_name,
                config.companions.design,
            );
        }
        return;
    }

    if let Err(e) = fs::create_dir_all(&spec_dir) {
        eprintln!("Failed to create {}: {e}", spec_dir.display());
        process::exit(1);
    }

    // Auto-detect source files matching the module name
    let module_files = generator::find_files_for_module(root, module_name, &config);

    // Generate spec content
    let spec_content = if let Some(ref tpl_dir) = template {
        generator::generate_spec_from_custom_template(tpl_dir, module_name, &module_files, root)
    } else {
        generator::generate_spec(module_name, &module_files, root, &specs_dir)
    };

    match fs::write(&spec_file, &spec_content) {
        Ok(_) => {
            let rel = spec_file.strip_prefix(root).unwrap_or(&spec_file).display();
            println!("  {} Created {rel}", "✓".green());
            if !module_files.is_empty() {
                println!(
                    "    {} Auto-detected {} source file(s)",
                    "ℹ".cyan(),
                    module_files.len()
                );
            }
        }
        Err(e) => {
            eprintln!("Failed to write {}: {e}", spec_file.display());
            process::exit(1);
        }
    }

    // Generate companion files
    if let Some(ref tpl_dir) = template {
        generator::generate_companion_files_from_template(
            &spec_dir,
            module_name,
            tpl_dir,
            config.companions.design,
        );
    } else {
        generator::generate_companion_files_for_spec(
            &spec_dir,
            module_name,
            config.companions.design,
        );
    }

    // Auto-register in specsync-registry.toml if one exists
    let registry_path = root.join("specsync-registry.toml");
    if registry_path.exists() {
        let spec_rel = spec_file
            .strip_prefix(root)
            .unwrap_or(&spec_file)
            .to_string_lossy()
            .replace('\\', "/");
        if registry::register_module(root, module_name, &spec_rel) {
            println!("    {} Registered in specsync-registry.toml", "✓".green());
        }
    }
}
