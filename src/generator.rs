use crate::exports::{has_extension, is_test_file};
use crate::types::{CoverageReport, SpecSyncConfig};
use std::fs;
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

    // Replace files list
    let files_re = regex::Regex::new(r"(?m)^files:\n(?:\s+-\s+.+\n?)*").unwrap();
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

/// Generate spec files for all unspecced modules.
/// Returns the number of specs generated.
pub fn generate_specs_for_unspecced_modules(
    root: &Path,
    report: &CoverageReport,
    config: &SpecSyncConfig,
) -> usize {
    let specs_dir = root.join(&config.specs_dir);
    let mut generated = 0;

    for module_name in &report.unspecced_modules {
        let spec_dir = specs_dir.join(module_name);
        let spec_file = spec_dir.join(format!("{module_name}.spec.md"));

        if spec_file.exists() {
            continue;
        }

        // Find source files for this module across all source dirs
        let mut module_files = Vec::new();
        for src_dir in &config.source_dirs {
            let module_dir = root.join(src_dir).join(module_name);
            let files = find_module_source_files(&module_dir, config);
            module_files.extend(files);
        }

        if module_files.is_empty() {
            continue;
        }

        if let Err(e) = fs::create_dir_all(&spec_dir) {
            eprintln!("  Failed to create {}: {e}", spec_dir.display());
            continue;
        }

        let spec_content = generate_spec(module_name, &module_files, root, &specs_dir);

        match fs::write(&spec_file, &spec_content) {
            Ok(_) => {
                let rel = spec_file.strip_prefix(root).unwrap_or(&spec_file).display();
                println!("  \u{2713} Generated {rel} ({} files)", module_files.len());
                generated += 1;
            }
            Err(e) => {
                eprintln!("  Failed to write {}: {e}", spec_file.display());
            }
        }
    }

    generated
}
