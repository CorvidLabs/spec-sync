use colored::Colorize;
use std::path::Path;
use std::process;

use crate::generator;

pub fn cmd_wizard(root: &Path) {
    use dialoguer::{Confirm, Input, Select};

    let config = crate::config::load_config_allowing_unloadable(root);
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

    if let Err(e) = super::validate_scaffold_module_name(&module_name) {
        eprintln!("{} {e}", "Error:".red());
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
            // Only through the confined, budgeted walk: a source folder that leads out of the
            // project is never walked (REQ-cmd-wizard-004).
            crate::util::find_project_files(root, &full_src, "", 100_000, 256)
                .unwrap_or_default()
                .into_iter()
                .filter(|path| {
                    is_wizard_source_candidate(
                        path,
                        &module_name,
                        &config.source_extensions,
                        config.include_extensionless,
                    )
                })
                .map(|path| {
                    path.strip_prefix(root)
                        .unwrap_or(&path)
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
        _ => (
            "1. Define an invariant that must remain true for supported inputs.",
            "",
        ),
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

### Scenario: Core behavior

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
    if let Err(message) = write_wizard_spec(
        root,
        &spec_file,
        &module_name,
        &spec_content,
        config.companions.design,
    ) {
        eprintln!("{message}");
        process::exit(1);
    }
    println!(
        "\n{} Run {} to validate your new spec.",
        "Tip:".cyan().bold(),
        "specsync check".bold()
    );
}

/// The wizard's write step: the spec directory, the spec file, then its companions. All of it
/// goes through the project file functions, because the spec path and its directory can be
/// committed as symbolic links: one that leaves the project, or dangles, is an error that names
/// the path, and nothing is created outside the project (REQ-cmd-wizard-003).
fn write_wizard_spec(
    root: &Path,
    spec_file: &Path,
    module_name: &str,
    spec_content: &str,
    design_enabled: bool,
) -> Result<(), String> {
    let spec_dir = spec_file.parent().unwrap_or(root);
    crate::util::create_project_dir(root, spec_dir)
        .map_err(|e| format!("Failed to create {}: {e}", spec_dir.display()))?;
    generator::create_spec_file(root, spec_file, spec_content)
        .map_err(|e| format!("Failed to write {}: {e}", spec_file.display()))?;
    let rel = spec_file.strip_prefix(root).unwrap_or(spec_file).display();
    println!("\n  {} Created {rel}", "✓".green());
    generator::generate_companion_files_for_spec(root, spec_dir, module_name, design_enabled);
    Ok(())
}

fn is_wizard_source_candidate(
    path: &Path,
    module_name: &str,
    source_extensions: &[String],
    include_extensionless: bool,
) -> bool {
    if !path.is_file() {
        return false;
    }
    let name = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    let parent = path
        .parent()
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str())
        .unwrap_or("");
    (name == module_name || parent == module_name)
        && crate::exports::has_configured_extension(path, source_extensions, include_extensionless)
}

#[cfg(test)]
mod tests {
    use super::{is_wizard_source_candidate, write_wizard_spec};
    use std::fs;

    /// Verifies REQ-cmd-wizard-003. The wizard is interactive, so its write step is driven
    /// directly: a dangling spec link and a spec directory linked outside the project are refused
    /// and create nothing, and a plain path is written.
    #[cfg(unix)]
    #[test]
    fn wizard_write_refuses_a_dangling_spec_link_and_a_linked_spec_directory() {
        use std::os::unix::fs::symlink;
        let temp = tempfile::tempdir().expect("temp dir");
        let root = temp.path().join("project");
        let outside = temp.path().join("outside");
        fs::create_dir_all(root.join("specs/billing")).unwrap();
        fs::create_dir_all(outside.join("elsewhere")).unwrap();

        let spec_file = root.join("specs/billing/billing.spec.md");
        symlink("../../../outside/planted.md", &spec_file).unwrap();
        let error = write_wizard_spec(&root, &spec_file, "billing", "# Billing\n", false)
            .expect_err("a dangling spec link is refused");
        assert!(
            error.contains(
                "specs/billing/billing.spec.md is a symbolic link that leaves the project"
            ),
            "{error}"
        );
        assert!(!outside.join("planted.md").exists());

        symlink("../../outside/elsewhere", root.join("specs/payments")).unwrap();
        let spec_file = root.join("specs/payments/payments.spec.md");
        let error = write_wizard_spec(&root, &spec_file, "payments", "# Payments\n", false)
            .expect_err("a spec directory linked outside is refused");
        assert!(
            error.contains("specs/payments is a symbolic link that leaves the project"),
            "{error}"
        );
        assert_eq!(fs::read_dir(outside.join("elsewhere")).unwrap().count(), 0);

        let spec_file = root.join("specs/orders/orders.spec.md");
        write_wizard_spec(&root, &spec_file, "orders", "# Orders\n", false).unwrap();
        assert_eq!(fs::read_to_string(&spec_file).unwrap(), "# Orders\n");
        assert!(root.join("specs/orders/tasks.md").is_file());
    }

    #[test]
    fn wizard_source_candidates_exclude_matching_directories() {
        let temp = tempfile::tempdir().expect("temp dir");
        let directory = temp.path().join("tool");
        fs::create_dir(&directory).expect("matching directory");
        let source = directory.join("tool");
        fs::write(&source, "#!/bin/sh\n").expect("extensionless source");

        assert!(!is_wizard_source_candidate(&directory, "tool", &[], true));
        assert!(is_wizard_source_candidate(&source, "tool", &[], true));
    }
}
