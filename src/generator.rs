use crate::ai;
use crate::exports::{has_extension, is_test_file};
use crate::types::{CoverageReport, SpecSyncConfig};
use colored::Colorize;
use std::fs;
use std::io::Write;
use std::path::Path;
use walkdir::WalkDir;

const DEFAULT_TEMPLATE: &str = r#"---
module: module-name
version: 1
status: draft
files: []
db_tables: []
depends_on: []
---

# Module Name

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
"#;

/// Find source files in a module directory.
fn find_module_source_files(dir: &Path, config: &SpecSyncConfig) -> Vec<String> {
    let mut results = Vec::new();
    if !dir.exists() {
        return results;
    }

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && has_extension(path, &config.source_extensions) && !is_test_file(path) {
            results.push(path.to_string_lossy().to_string());
        }
    }

    results
        .into_iter()
        .map(|p| {
            // Get path relative to root (two levels up from module dir)
            p.replace('\\', "/")
        })
        .collect()
}

/// Find source files for a module, checking subdirectories first, then flat files.
fn find_files_for_module(root: &Path, module_name: &str, config: &SpecSyncConfig) -> Vec<String> {
    let mut module_files = Vec::new();

    // First: look for subdirectory-based modules (src/module_name/)
    for src_dir in &config.source_dirs {
        let module_dir = root.join(src_dir).join(module_name);
        let files = find_module_source_files(&module_dir, config);
        module_files.extend(files);
    }

    // Fallback: look for flat files matching the module name (src/module_name.rs, etc.)
    if module_files.is_empty() {
        for src_dir in &config.source_dirs {
            let src_path = root.join(src_dir);
            if let Ok(entries) = std::fs::read_dir(&src_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_file()
                        || !has_extension(&path, &config.source_extensions)
                        || is_test_file(&path)
                    {
                        continue;
                    }
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                        && stem == module_name
                    {
                        module_files.push(path.to_string_lossy().replace('\\', "/"));
                    }
                }
            }
        }
    }

    module_files
}

/// Generate a spec from a template.
fn generate_spec(
    module_name: &str,
    source_files: &[String],
    root: &Path,
    specs_dir: &Path,
) -> String {
    let template_path = specs_dir.join("_template.spec.md");
    let template = if template_path.exists() {
        fs::read_to_string(&template_path).unwrap_or_else(|_| DEFAULT_TEMPLATE.to_string())
    } else {
        DEFAULT_TEMPLATE.to_string()
    };

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

    // Make paths relative to root
    let files_yaml: String = source_files
        .iter()
        .map(|f| {
            let rel = Path::new(f)
                .strip_prefix(root.to_string_lossy().as_ref())
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| f.clone());
            format!("  - {rel}")
        })
        .collect::<Vec<_>>()
        .join("\n");

    let mut spec = template;

    // Replace module name
    let module_re = regex::Regex::new(r"(?m)^module:\s*.+$").unwrap();
    spec = module_re
        .replace(&spec, format!("module: {module_name}"))
        .to_string();

    // Replace status
    let status_re = regex::Regex::new(r"(?m)^status:\s*.+$").unwrap();
    spec = status_re.replace(&spec, "status: draft").to_string();

    // Replace version
    let version_re = regex::Regex::new(r"(?m)^version:\s*.+$").unwrap();
    spec = version_re.replace(&spec, "version: 1").to_string();

    // Replace files list (handles both `files: []` and multi-line YAML list)
    let files_re = regex::Regex::new(r"(?m)^files:\s*\[\]|^files:\n(?:\s+-\s+.+\n?)*").unwrap();
    spec = files_re
        .replace(&spec, format!("files:\n{files_yaml}\n"))
        .to_string();

    // Replace title
    let title_re = regex::Regex::new(r"(?m)^# .+$").unwrap();
    spec = title_re.replace(&spec, format!("# {title}")).to_string();

    // Clear db_tables
    let db_re = regex::Regex::new(r"(?m)^db_tables:\n(?:\s+-\s+.+\n?)*").unwrap();
    spec = db_re.replace(&spec, "db_tables: []\n").to_string();

    spec
}

/// Generate spec content for a module, using AI if an API key is provided.
fn generate_module_spec(
    module_name: &str,
    module_files: &[String],
    root: &Path,
    specs_dir: &Path,
    config: &SpecSyncConfig,
    ai_command: Option<&str>,
) -> String {
    if let Some(cmd) = ai_command {
        // Make paths relative to root for the AI prompt
        let rel_files: Vec<String> = module_files
            .iter()
            .map(|f| {
                Path::new(f)
                    .strip_prefix(root.to_string_lossy().as_ref())
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| f.clone())
            })
            .collect();

        match ai::generate_spec_with_ai(module_name, &rel_files, root, config, cmd) {
            Ok(spec) => return spec,
            Err(e) => {
                eprintln!(
                    "  {} AI generation failed for {module_name}: {e} — falling back to template",
                    "⚠".yellow()
                );
            }
        }
    }

    generate_spec(module_name, module_files, root, specs_dir)
}

/// Generate spec files for all unspecced modules.
/// Returns the number of specs generated.
pub fn generate_specs_for_unspecced_modules(
    root: &Path,
    report: &CoverageReport,
    config: &SpecSyncConfig,
    ai_command: Option<&str>,
) -> usize {
    let specs_dir = root.join(&config.specs_dir);
    let mut generated = 0;

    for module_name in &report.unspecced_modules {
        let spec_dir = specs_dir.join(module_name);
        let spec_file = spec_dir.join(format!("{module_name}.spec.md"));

        if spec_file.exists() {
            continue;
        }

        let module_files = find_files_for_module(root, module_name, config);

        if module_files.is_empty() {
            continue;
        }

        if let Err(e) = fs::create_dir_all(&spec_dir) {
            eprintln!("  Failed to create {}: {e}", spec_dir.display());
            continue;
        }

        if ai_command.is_some() {
            let rel = spec_file.strip_prefix(root).unwrap_or(&spec_file).display();
            eprintln!("  Generating {rel} with AI...");
        }

        let spec_content =
            generate_module_spec(module_name, &module_files, root, &specs_dir, config, ai_command);

        match fs::write(&spec_file, &spec_content) {
            Ok(_) => {
                let rel = spec_file.strip_prefix(root).unwrap_or(&spec_file).display();
                if ai_command.is_some() {
                    // AI generation complete
                }
                println!("  {} Generated {rel} ({} files)", "✓".green(), module_files.len());
                let _ = std::io::stdout().flush();
                generated += 1;
            }
            Err(e) => {
                eprintln!("  Failed to write {}: {e}", spec_file.display());
            }
        }
    }

    generated
}

/// Generate spec files for all unspecced modules, returning the paths of generated files.
pub fn generate_specs_for_unspecced_modules_paths(
    root: &Path,
    report: &CoverageReport,
    config: &SpecSyncConfig,
    ai_command: Option<&str>,
) -> Vec<String> {
    let specs_dir = root.join(&config.specs_dir);
    let mut generated_paths = Vec::new();

    for module_name in &report.unspecced_modules {
        let spec_dir = specs_dir.join(module_name);
        let spec_file = spec_dir.join(format!("{module_name}.spec.md"));

        if spec_file.exists() {
            continue;
        }

        let module_files = find_files_for_module(root, module_name, config);

        if module_files.is_empty() {
            continue;
        }

        if fs::create_dir_all(&spec_dir).is_err() {
            continue;
        }

        let spec_content =
            generate_module_spec(module_name, &module_files, root, &specs_dir, config, ai_command);

        if fs::write(&spec_file, &spec_content).is_ok() {
            let rel = spec_file
                .strip_prefix(root)
                .unwrap_or(&spec_file)
                .to_string_lossy()
                .to_string();
            generated_paths.push(rel);
        }
    }

    generated_paths
}
