use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process;

use crate::config::load_config;
use crate::generator;

pub fn cmd_wizard(root: &Path) {
    use dialoguer::{Confirm, Input, Select};

    let config = load_config(root);
    let specs_dir = root.join(&config.specs_dir);

    println!(
        "\n{}",
        "═══════════════════════════════════════════════════".cyan()
    );
    println!("{}", "  SpecSync — New Spec Wizard".cyan().bold());
    println!(
        "{}\n",
        "═══════════════════════════════════════════════════".cyan()
    );

    // 1. Module name
    let module_name: String = Input::new()
        .with_prompt("Module name")
        .interact_text()
        .unwrap_or_else(|_| process::exit(0));
    let module_name = module_name.trim().to_string();

    if module_name.is_empty() {
        eprintln!("{} Module name cannot be empty", "Error:".red());
        process::exit(1);
    }

    let spec_dir = specs_dir.join(&module_name);
    let spec_file = spec_dir.join(format!("{module_name}.spec.md"));
    if spec_file.exists() {
        eprintln!(
            "{} Spec already exists: {}",
            "!".yellow(),
            spec_file.strip_prefix(root).unwrap_or(&spec_file).display()
        );
        process::exit(1);
    }

    // 2. Purpose
    let purpose: String = Input::new()
        .with_prompt("What does this module do? (one sentence)")
        .interact_text()
        .unwrap_or_else(|_| process::exit(0));

    // 3. Template type
    let templates = vec![
        "Generic module",
        "API endpoint / route handler",
        "Data model / database layer",
        "Utility / helper library",
        "UI component",
    ];
    let template_idx = Select::new()
        .with_prompt("Module type")
        .items(&templates)
        .default(0)
        .interact()
        .unwrap_or_else(|_| process::exit(0));

    // 4. Status
    let statuses = vec!["draft", "unstable", "stable", "locked"];
    let status_idx = Select::new()
        .with_prompt("Initial status")
        .items(&statuses)
        .default(0)
        .interact()
        .unwrap_or_else(|_| process::exit(0));
    let status = statuses[status_idx];

    // 5. Auto-detect source files
    let module_files: Vec<String> = config
        .source_dirs
        .iter()
        .flat_map(|src_dir| {
            let full_src = root.join(src_dir);
            if !full_src.is_dir() {
                return vec![];
            }
            walkdir::WalkDir::new(&full_src)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    if !e.path().is_file() {
                        return false;
                    }
                    let name = e.path().file_stem().and_then(|n| n.to_str()).unwrap_or("");
                    let parent = e
                        .path()
                        .parent()
                        .and_then(|p| p.file_name())
                        .and_then(|n| n.to_str())
                        .unwrap_or("");
                    (name == module_name || parent == module_name)
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
        })
        .collect();

    let files_yaml = if module_files.is_empty() {
        println!(
            "\n{} No source files auto-detected for '{module_name}'.",
            "i".blue()
        );
        let manual_file: String = Input::new()
            .with_prompt("Source file path (or leave empty to skip)")
            .allow_empty(true)
            .interact_text()
            .unwrap_or_else(|_| process::exit(0));
        if manual_file.is_empty() {
            "  # - path/to/source/file".to_string()
        } else {
            format!("  - {manual_file}")
        }
    } else {
        println!(
            "\n{} Found {} source file(s):",
            "✓".green(),
            module_files.len()
        );
        for f in &module_files {
            println!("    {f}");
        }
        module_files
            .iter()
            .map(|f| format!("  - {f}"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    // 6. Dependencies
    let deps: String = Input::new()
        .with_prompt("Dependencies (comma-separated module names, or empty)")
        .allow_empty(true)
        .interact_text()
        .unwrap_or_else(|_| process::exit(0));
    let depends_on: Vec<String> = deps
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // Build the title
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

    let depends_yaml = if depends_on.is_empty() {
        "[]".to_string()
    } else {
        format!(
            "\n{}",
            depends_on
                .iter()
                .map(|d| format!("  - {d}"))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };

    // Template-specific sections
    let (extra_invariants, extra_api_hint) = match template_idx {
        1 => (
            "1. All endpoints validate input before processing\n2. Authentication is required unless explicitly marked public",
            "### Endpoints\n\n| Method | Path | Description |\n|--------|------|-------------|\n",
        ),
        2 => (
            "1. All mutations go through a single write path\n2. Schema migrations are backward-compatible",
            "### Models\n\n| Model | Description |\n|-------|-------------|\n",
        ),
        3 => (
            "1. All functions are pure (no side effects) unless documented\n2. All inputs are validated",
            "",
        ),
        4 => (
            "1. Component renders without crashing given any valid props\n2. Accessibility requirements are met (ARIA labels, keyboard nav)",
            "### Props\n\n| Prop | Type | Default | Description |\n|------|------|---------|-------------|\n",
        ),
        _ => ("1. <!-- TODO -->", ""),
    };

    let spec_content = format!(
        r#"---
module: {module_name}
version: 1
status: {status}
files:
{files_yaml}
db_tables: []
depends_on: {depends_yaml}
---

# {title}

## Purpose

{purpose}

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|

### Exported Types

| Type | Description |
|------|-------------|

{extra_api_hint}
## Invariants

{extra_invariants}

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
    );

    // 7. Preview
    println!(
        "\n{}",
        "─── Preview ────────────────────────────────────────".cyan()
    );
    // Show first ~30 lines of the spec
    for (i, line) in spec_content.lines().enumerate() {
        if i > 30 {
            println!("  ...(truncated)");
            break;
        }
        println!("  {line}");
    }
    println!(
        "{}",
        "────────────────────────────────────────────────────".cyan()
    );

    let confirmed = Confirm::new()
        .with_prompt("Write this spec?")
        .default(true)
        .interact()
        .unwrap_or(false);

    if !confirmed {
        println!("{}", "Cancelled.".yellow());
        return;
    }

    // Write the spec
    if let Err(e) = fs::create_dir_all(&spec_dir) {
        eprintln!("Failed to create {}: {e}", spec_dir.display());
        process::exit(1);
    }

    match fs::write(&spec_file, &spec_content) {
        Ok(_) => {
            let rel = spec_file.strip_prefix(root).unwrap_or(&spec_file).display();
            println!("\n  {} Created {rel}", "✓".green());
            generator::generate_companion_files_for_spec(&spec_dir, &module_name, config.companions.design);
            println!(
                "\n{} Run {} to validate your new spec.",
                "Tip:".cyan().bold(),
                "specsync check".bold()
            );
        }
        Err(e) => {
            eprintln!("Failed to write {}: {e}", spec_file.display());
            process::exit(1);
        }
    }
}
