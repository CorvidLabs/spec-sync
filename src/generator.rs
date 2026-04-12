use crate::ai::{self, ResolvedProvider};
use crate::exports::{has_extension, is_test_file};
use crate::types::{CoverageReport, Language, SpecSyncConfig};
use colored::Colorize;
use std::fs;
use std::io::Write;
use std::path::Path;
use walkdir::WalkDir;

const TASKS_TEMPLATE: &str = r#"---
spec: {module}.spec.md
---

## Tasks

- [ ] <!-- Add tasks for this spec -->

## Gaps

<!-- Uncovered areas, missing edge cases, or incomplete coverage -->

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
"#;

const REQUIREMENTS_TEMPLATE: &str = r#"---
spec: {module}.spec.md
---

## User Stories

- As a [role], I want [feature] so that [benefit]

## Acceptance Criteria

- <!-- TODO: define acceptance criteria -->

## Constraints

<!-- Non-functional requirements, performance targets, compliance needs -->

## Out of Scope

<!-- Explicitly excluded from this module's requirements -->
"#;

const CONTEXT_TEMPLATE: &str = r#"---
spec: {module}.spec.md
---

## Key Decisions

<!-- Record architectural or design decisions relevant to this spec -->

## Files to Read First

<!-- List the most important files an agent or new developer should read -->

## Current Status

<!-- What's done, what's in progress, what's blocked -->

## Notes

<!-- Free-form notes, links, or context -->
"#;

const TESTING_TEMPLATE: &str = r#"---
spec: {module}.spec.md
---

## Automated Testing

<!-- Expected test file locations, coverage targets, fixture descriptions -->

| Test File | Type | What It Covers |
|-----------|------|----------------|

## Manual Testing

<!-- Step-by-step QA checklists, device/browser matrices, user flow walkthroughs -->

- [ ] <!-- Add manual test steps -->

## Edge Cases & Boundary Conditions

<!-- Boundary values, race conditions, permission matrices, error paths -->

| Scenario | Expected Behavior |
|----------|-------------------|
"#;

const DESIGN_TEMPLATE: &str = r#"---
spec: {module}.spec.md
sources: []
---

## Layout

<!-- Page/component layout, responsive breakpoints, positioning -->

## Components

<!-- Component tree, props, slots -->

## Tokens

<!-- Design token overrides from global design system -->

## Assets

<!-- Icons, images, illustrations needed -->
"#;

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

/// Detect the primary language of a set of source files.
fn detect_primary_language(files: &[String]) -> Option<Language> {
    let mut counts = std::collections::HashMap::new();
    for file in files {
        let ext = Path::new(file)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if let Some(lang) = Language::from_extension(ext) {
            *counts.entry(lang).or_insert(0usize) += 1;
        }
    }
    counts.into_iter().max_by_key(|(_, c)| *c).map(|(l, _)| l)
}

/// Get a language-specific spec template.
fn language_template(lang: Language) -> &'static str {
    match lang {
        Language::Swift => {
            r#"---
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

### Types

| Type | Kind | Description |
|------|------|-------------|

### Protocols

| Protocol | Description |
|----------|-------------|

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
        }
        Language::Rust => {
            r#"---
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

### Structs & Enums

| Type | Description |
|------|-------------|

### Traits

| Trait | Description |
|-------|-------------|

### Functions

| Function | Signature | Description |
|----------|-----------|-------------|

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

| Crate/Module | What is used |
|-------------|-------------|

### Consumed By

| Module | What is used |
|--------|-------------|

## Change Log

| Date | Author | Change |
|------|--------|--------|
"#
        }
        Language::Kotlin | Language::Java => {
            r#"---
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

### Classes & Interfaces

| Type | Kind | Description |
|------|------|-------------|

### Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|

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
        }
        Language::Go => {
            r#"---
module: module-name
version: 1
status: draft
files: []
db_tables: []
depends_on: []
---

# Module Name

## Purpose

<!-- TODO: describe what this package does -->

## Public API

### Types

| Type | Kind | Description |
|------|------|-------------|

### Functions

| Function | Signature | Description |
|----------|-----------|-------------|

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

| Package | What is used |
|---------|-------------|

### Consumed By

| Package | What is used |
|---------|-------------|

## Change Log

| Date | Author | Change |
|------|--------|--------|
"#
        }
        Language::Python => {
            r#"---
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

### Classes

| Class | Description |
|-------|-------------|

### Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|

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
        }
        // TypeScript, C#, Dart, and fallback use the default template
        _ => DEFAULT_TEMPLATE,
    }
}

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

/// Find source files for a module, checking config module definitions first,
/// then subdirectories, then flat files.
pub fn find_files_for_module(
    root: &Path,
    module_name: &str,
    config: &SpecSyncConfig,
) -> Vec<String> {
    let mut module_files = Vec::new();

    // First: check user-defined module definitions in specsync.json
    if let Some(module_def) = config.modules.get(module_name) {
        for file in &module_def.files {
            let full_path = root.join(file);
            if full_path.exists() {
                module_files.push(full_path.to_string_lossy().replace('\\', "/"));
            } else if full_path.is_dir() {
                module_files.extend(find_module_source_files(&full_path, config));
            }
        }
        if !module_files.is_empty() {
            return module_files;
        }
    }

    // Second: look for subdirectory-based modules (src/module_name/)
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

/// Generate a spec from a template, using language-aware defaults.
pub fn generate_spec(
    module_name: &str,
    source_files: &[String],
    root: &Path,
    specs_dir: &Path,
) -> String {
    let template_path = specs_dir.join("_template.spec.md");
    let template = if template_path.exists() {
        // User-provided template takes priority
        fs::read_to_string(&template_path).unwrap_or_else(|_| DEFAULT_TEMPLATE.to_string())
    } else {
        // Use language-specific template
        match detect_primary_language(source_files) {
            Some(lang) => language_template(lang).to_string(),
            None => DEFAULT_TEMPLATE.to_string(),
        }
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

/// Generate spec content for a module, using AI if a provider is configured.
/// Returns `(spec_content, ai_was_used)`.
fn generate_module_spec(
    module_name: &str,
    module_files: &[String],
    root: &Path,
    specs_dir: &Path,
    config: &SpecSyncConfig,
    provider: Option<&ResolvedProvider>,
) -> (String, bool) {
    if let Some(provider) = provider {
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

        match ai::generate_spec_with_ai(module_name, &rel_files, root, config, provider) {
            Ok(spec) => return (spec, true),
            Err(e) => {
                eprintln!(
                    "  {} AI generation failed for {module_name}: {e} — falling back to template",
                    "⚠".yellow()
                );
            }
        }
    }

    (
        generate_spec(module_name, module_files, root, specs_dir),
        false,
    )
}

/// Generate companion files (tasks.md, context.md, requirements.md, testing.md,
/// and optionally design.md) alongside a spec file.
fn generate_companion_files(spec_dir: &Path, module_name: &str, design_enabled: bool) {
    let tasks_path = spec_dir.join("tasks.md");
    let context_path = spec_dir.join("context.md");
    let requirements_path = spec_dir.join("requirements.md");
    let testing_path = spec_dir.join("testing.md");

    if !tasks_path.exists() {
        let content = TASKS_TEMPLATE.replace("{module}", module_name);
        if fs::write(&tasks_path, &content).is_ok() {
            println!("    {} Generated tasks.md", "✓".green());
        }
    }

    if !context_path.exists() {
        let content = CONTEXT_TEMPLATE.replace("{module}", module_name);
        if fs::write(&context_path, &content).is_ok() {
            println!("    {} Generated context.md", "✓".green());
        }
    }

    if !requirements_path.exists() {
        let content = REQUIREMENTS_TEMPLATE.replace("{module}", module_name);
        if fs::write(&requirements_path, &content).is_ok() {
            println!("    {} Generated requirements.md", "✓".green());
        }
    }

    if !testing_path.exists() {
        let content = TESTING_TEMPLATE.replace("{module}", module_name);
        if fs::write(&testing_path, &content).is_ok() {
            println!("    {} Generated testing.md", "✓".green());
        }
    }

    if design_enabled {
        let design_path = spec_dir.join("design.md");
        if !design_path.exists() {
            let content = DESIGN_TEMPLATE.replace("{module}", module_name);
            if fs::write(&design_path, &content).is_ok() {
                println!("    {} Generated design.md", "✓".green());
            }
        }
    }
}

/// Generate companion files for a given spec.
///
/// When `design_enabled` is true, a `design.md` companion is also generated.
pub fn generate_companion_files_for_spec(spec_dir: &Path, module_name: &str, design_enabled: bool) {
    generate_companion_files(spec_dir, module_name, design_enabled);
}

/// Generate a spec using templates from a custom template directory.
/// Looks for `spec.md`, `tasks.md`, `context.md`, `requirements.md`, `testing.md` in the template dir.
/// Falls back to built-in templates for any missing template files.
pub fn generate_spec_from_custom_template(
    template_dir: &Path,
    module_name: &str,
    source_files: &[String],
    root: &Path,
) -> String {
    let template_file = template_dir.join("spec.md");
    let template = if template_file.exists() {
        fs::read_to_string(&template_file).unwrap_or_else(|_| DEFAULT_TEMPLATE.to_string())
    } else {
        // No custom spec template — use language-aware default
        match detect_primary_language(source_files) {
            Some(lang) => language_template(lang).to_string(),
            None => DEFAULT_TEMPLATE.to_string(),
        }
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

    let module_re = regex::Regex::new(r"(?m)^module:\s*.+$").unwrap();
    spec = module_re
        .replace(&spec, format!("module: {module_name}"))
        .to_string();

    let status_re = regex::Regex::new(r"(?m)^status:\s*.+$").unwrap();
    spec = status_re.replace(&spec, "status: draft").to_string();

    let version_re = regex::Regex::new(r"(?m)^version:\s*.+$").unwrap();
    spec = version_re.replace(&spec, "version: 1").to_string();

    let files_re = regex::Regex::new(r"(?m)^files:\s*\[\]|^files:\n(?:\s+-\s+.+\n?)*").unwrap();
    if source_files.is_empty() {
        spec = files_re.replace(&spec, "files: []\n").to_string();
    } else {
        spec = files_re
            .replace(&spec, format!("files:\n{files_yaml}\n"))
            .to_string();
    }

    let title_re = regex::Regex::new(r"(?m)^# .+$").unwrap();
    spec = title_re.replace(&spec, format!("# {title}")).to_string();

    let db_re = regex::Regex::new(r"(?m)^db_tables:\n(?:\s+-\s+.+\n?)*").unwrap();
    spec = db_re.replace(&spec, "db_tables: []\n").to_string();

    spec
}

/// Generate companion files from a custom template directory.
/// Falls back to built-in templates for any missing files.
pub fn generate_companion_files_from_template(
    spec_dir: &Path,
    module_name: &str,
    template_dir: &Path,
    design_enabled: bool,
) {
    let tasks_path = spec_dir.join("tasks.md");
    let context_path = spec_dir.join("context.md");
    let requirements_path = spec_dir.join("requirements.md");
    let testing_path = spec_dir.join("testing.md");

    if !tasks_path.exists() {
        let template_file = template_dir.join("tasks.md");
        let content = if template_file.exists() {
            fs::read_to_string(&template_file)
                .unwrap_or_else(|_| TASKS_TEMPLATE.to_string())
                .replace("{module}", module_name)
        } else {
            TASKS_TEMPLATE.replace("{module}", module_name)
        };
        if fs::write(&tasks_path, &content).is_ok() {
            println!("    {} Generated tasks.md", "✓".green());
        }
    }

    if !context_path.exists() {
        let template_file = template_dir.join("context.md");
        let content = if template_file.exists() {
            fs::read_to_string(&template_file)
                .unwrap_or_else(|_| CONTEXT_TEMPLATE.to_string())
                .replace("{module}", module_name)
        } else {
            CONTEXT_TEMPLATE.replace("{module}", module_name)
        };
        if fs::write(&context_path, &content).is_ok() {
            println!("    {} Generated context.md", "✓".green());
        }
    }

    if !requirements_path.exists() {
        let template_file = template_dir.join("requirements.md");
        let content = if template_file.exists() {
            fs::read_to_string(&template_file)
                .unwrap_or_else(|_| REQUIREMENTS_TEMPLATE.to_string())
                .replace("{module}", module_name)
        } else {
            REQUIREMENTS_TEMPLATE.replace("{module}", module_name)
        };
        if fs::write(&requirements_path, &content).is_ok() {
            println!("    {} Generated requirements.md", "✓".green());
        }
    }

    if !testing_path.exists() {
        let template_file = template_dir.join("testing.md");
        let content = if template_file.exists() {
            fs::read_to_string(&template_file)
                .unwrap_or_else(|_| TESTING_TEMPLATE.to_string())
                .replace("{module}", module_name)
        } else {
            TESTING_TEMPLATE.replace("{module}", module_name)
        };
        if fs::write(&testing_path, &content).is_ok() {
            println!("    {} Generated testing.md", "✓".green());
        }
    }

    if design_enabled {
        let design_path = spec_dir.join("design.md");
        if !design_path.exists() {
            let template_file = template_dir.join("design.md");
            let content = if template_file.exists() {
                fs::read_to_string(&template_file)
                    .unwrap_or_else(|_| DESIGN_TEMPLATE.to_string())
                    .replace("{module}", module_name)
            } else {
                DESIGN_TEMPLATE.replace("{module}", module_name)
            };
            if fs::write(&design_path, &content).is_ok() {
                println!("    {} Generated design.md", "✓".green());
            }
        }
    }
}

/// Generate spec files for all unspecced modules.
/// Returns the number of specs generated.
pub fn generate_specs_for_unspecced_modules(
    root: &Path,
    report: &CoverageReport,
    config: &SpecSyncConfig,
    provider: Option<&ResolvedProvider>,
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

        if provider.is_some() {
            let rel = spec_file.strip_prefix(root).unwrap_or(&spec_file).display();
            eprintln!("  Generating {rel} with AI...");
        }

        let (spec_content, ai_used) = generate_module_spec(
            module_name,
            &module_files,
            root,
            &specs_dir,
            config,
            provider,
        );

        match fs::write(&spec_file, &spec_content) {
            Ok(_) => {
                let rel = spec_file.strip_prefix(root).unwrap_or(&spec_file).display();
                let from = if provider.is_some() && !ai_used {
                    " from template"
                } else {
                    ""
                };
                println!(
                    "  {} Generated {rel}{from} ({} files)",
                    "✓".green(),
                    module_files.len()
                );
                generate_companion_files(&spec_dir, module_name, config.companions.design);
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
    provider: Option<&ResolvedProvider>,
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

        let (spec_content, _ai_used) = generate_module_spec(
            module_name,
            &module_files,
            root,
            &specs_dir,
            config,
            provider,
        );

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ── detect_primary_language ─────────────────────────────────────

    #[test]
    fn detect_language_rust() {
        let files = vec!["src/main.rs".to_string(), "src/lib.rs".to_string()];
        assert_eq!(detect_primary_language(&files), Some(Language::Rust));
    }

    #[test]
    fn detect_language_typescript() {
        let files = vec![
            "src/app.ts".to_string(),
            "src/util.ts".to_string(),
            "src/types.tsx".to_string(),
        ];
        assert_eq!(detect_primary_language(&files), Some(Language::TypeScript));
    }

    #[test]
    fn detect_language_python() {
        let files = vec!["app.py".to_string(), "models.py".to_string()];
        assert_eq!(detect_primary_language(&files), Some(Language::Python));
    }

    #[test]
    fn detect_language_go() {
        let files = vec!["main.go".to_string()];
        assert_eq!(detect_primary_language(&files), Some(Language::Go));
    }

    #[test]
    fn detect_language_mixed_majority_wins() {
        let files = vec![
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
            "src/utils.rs".to_string(),
            "build.py".to_string(),
        ];
        assert_eq!(detect_primary_language(&files), Some(Language::Rust));
    }

    #[test]
    fn detect_language_empty() {
        let files: Vec<String> = vec![];
        assert_eq!(detect_primary_language(&files), None);
    }

    #[test]
    fn detect_language_unknown_extensions() {
        let files = vec!["data.csv".to_string(), "readme.md".to_string()];
        assert_eq!(detect_primary_language(&files), None);
    }

    // ── language_template ──────────────────────────────────────────

    #[test]
    fn template_rust_has_structs_enums_section() {
        let t = language_template(Language::Rust);
        assert!(t.contains("### Structs & Enums"));
        assert!(t.contains("### Traits"));
        assert!(t.contains("Crate/Module"));
    }

    #[test]
    fn template_swift_has_protocols_section() {
        let t = language_template(Language::Swift);
        assert!(t.contains("### Protocols"));
        assert!(t.contains("### Types"));
    }

    #[test]
    fn template_go_has_package_terminology() {
        let t = language_template(Language::Go);
        assert!(t.contains("package"));
    }

    #[test]
    fn template_kotlin_has_classes_interfaces() {
        let t = language_template(Language::Kotlin);
        assert!(t.contains("### Classes & Interfaces"));
    }

    #[test]
    fn template_python_has_classes() {
        let t = language_template(Language::Python);
        assert!(t.contains("### Classes"));
    }

    #[test]
    fn template_typescript_uses_default() {
        let t = language_template(Language::TypeScript);
        // TypeScript falls through to DEFAULT_TEMPLATE
        assert!(t.contains("### Exported Functions"));
        assert!(t.contains("### Exported Types"));
    }

    // ── generate_spec (template-based) ─────────────────────────────

    #[test]
    fn generate_spec_fills_module_name() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let specs_dir = root.join("specs");
        fs::create_dir_all(&specs_dir).unwrap();

        let src_dir = root.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("auth.rs"), "pub fn login() {}").unwrap();

        let files = vec![src_dir.join("auth.rs").to_string_lossy().to_string()];
        let spec = generate_spec("auth", &files, root, &specs_dir);

        assert!(spec.contains("module: auth"));
        assert!(spec.contains("# Auth"));
        assert!(spec.contains("version: 1"));
        assert!(spec.contains("status: draft"));
    }

    #[test]
    fn generate_spec_hyphenated_name_title_case() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let specs_dir = root.join("specs");
        fs::create_dir_all(&specs_dir).unwrap();

        let spec = generate_spec("api-gateway", &[], root, &specs_dir);
        assert!(spec.contains("# Api Gateway"));
        assert!(spec.contains("module: api-gateway"));
    }

    #[test]
    fn generate_spec_uses_custom_template() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let specs_dir = root.join("specs");
        fs::create_dir_all(&specs_dir).unwrap();

        let custom_template = "---\nmodule: module-name\nversion: 1\nstatus: draft\nfiles: []\ndb_tables: []\ndepends_on: []\n---\n\n# Module Name\n\n## Purpose\n\nCustom template marker\n";
        fs::write(specs_dir.join("_template.spec.md"), custom_template).unwrap();

        let spec = generate_spec("my-mod", &[], root, &specs_dir);
        assert!(spec.contains("Custom template marker"));
        assert!(spec.contains("module: my-mod"));
    }

    #[test]
    fn generate_spec_rust_files_use_rust_template() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let specs_dir = root.join("specs");
        fs::create_dir_all(&specs_dir).unwrap();

        let files = vec!["src/parser.rs".to_string()];
        let spec = generate_spec("parser", &files, root, &specs_dir);
        // Should use Rust template (no custom template file exists)
        assert!(spec.contains("### Structs & Enums"));
    }

    // ── companion file templates ───────────────────────────────────

    #[test]
    fn tasks_template_has_required_sections() {
        assert!(TASKS_TEMPLATE.contains("## Tasks"));
        assert!(TASKS_TEMPLATE.contains("## Gaps"));
        assert!(TASKS_TEMPLATE.contains("## Review Sign-offs"));
        assert!(TASKS_TEMPLATE.contains("{module}"));
    }

    #[test]
    fn requirements_template_has_required_sections() {
        assert!(REQUIREMENTS_TEMPLATE.contains("## User Stories"));
        assert!(REQUIREMENTS_TEMPLATE.contains("## Acceptance Criteria"));
        assert!(REQUIREMENTS_TEMPLATE.contains("## Constraints"));
        assert!(REQUIREMENTS_TEMPLATE.contains("## Out of Scope"));
    }

    #[test]
    fn context_template_has_required_sections() {
        assert!(CONTEXT_TEMPLATE.contains("## Key Decisions"));
        assert!(CONTEXT_TEMPLATE.contains("## Files to Read First"));
        assert!(CONTEXT_TEMPLATE.contains("## Current Status"));
        assert!(CONTEXT_TEMPLATE.contains("## Notes"));
    }

    #[test]
    fn testing_template_has_required_sections() {
        assert!(TESTING_TEMPLATE.contains("## Automated Testing"));
        assert!(TESTING_TEMPLATE.contains("## Manual Testing"));
        assert!(TESTING_TEMPLATE.contains("## Edge Cases & Boundary Conditions"));
        assert!(TESTING_TEMPLATE.contains("{module}"));
    }

    #[test]
    fn design_template_has_required_sections() {
        assert!(DESIGN_TEMPLATE.contains("## Layout"));
        assert!(DESIGN_TEMPLATE.contains("## Components"));
        assert!(DESIGN_TEMPLATE.contains("## Tokens"));
        assert!(DESIGN_TEMPLATE.contains("## Assets"));
        assert!(DESIGN_TEMPLATE.contains("{module}"));
        assert!(DESIGN_TEMPLATE.contains("sources:"));
    }

    #[test]
    fn default_template_has_all_required_sections() {
        assert!(DEFAULT_TEMPLATE.contains("## Purpose"));
        assert!(DEFAULT_TEMPLATE.contains("## Public API"));
        assert!(DEFAULT_TEMPLATE.contains("## Invariants"));
        assert!(DEFAULT_TEMPLATE.contains("## Behavioral Examples"));
        assert!(DEFAULT_TEMPLATE.contains("## Error Cases"));
        assert!(DEFAULT_TEMPLATE.contains("## Dependencies"));
        assert!(DEFAULT_TEMPLATE.contains("## Change Log"));
    }

    // ── generate_companion_files ───────────────────────────────────

    #[test]
    fn companion_files_created_when_absent() {
        let tmp = TempDir::new().unwrap();
        let spec_dir = tmp.path();

        generate_companion_files(spec_dir, "auth", false);

        assert!(spec_dir.join("tasks.md").exists());
        assert!(spec_dir.join("context.md").exists());
        assert!(spec_dir.join("requirements.md").exists());
        assert!(spec_dir.join("testing.md").exists());
        // design.md should NOT be created when design_enabled is false
        assert!(!spec_dir.join("design.md").exists());

        let tasks = fs::read_to_string(spec_dir.join("tasks.md")).unwrap();
        assert!(tasks.contains("spec: auth.spec.md"));

        let reqs = fs::read_to_string(spec_dir.join("requirements.md")).unwrap();
        assert!(reqs.contains("spec: auth.spec.md"));

        let testing = fs::read_to_string(spec_dir.join("testing.md")).unwrap();
        assert!(testing.contains("spec: auth.spec.md"));
        assert!(testing.contains("## Automated Testing"));
    }

    #[test]
    fn companion_files_created_with_design_enabled() {
        let tmp = TempDir::new().unwrap();
        let spec_dir = tmp.path();

        generate_companion_files(spec_dir, "auth", true);

        assert!(spec_dir.join("tasks.md").exists());
        assert!(spec_dir.join("context.md").exists());
        assert!(spec_dir.join("requirements.md").exists());
        assert!(spec_dir.join("testing.md").exists());
        assert!(spec_dir.join("design.md").exists());

        let design = fs::read_to_string(spec_dir.join("design.md")).unwrap();
        assert!(design.contains("spec: auth.spec.md"));
        assert!(design.contains("## Layout"));
        assert!(design.contains("## Components"));
        assert!(design.contains("## Tokens"));
        assert!(design.contains("## Assets"));
    }

    #[test]
    fn companion_files_not_overwritten() {
        let tmp = TempDir::new().unwrap();
        let spec_dir = tmp.path();

        fs::write(spec_dir.join("tasks.md"), "existing content").unwrap();
        fs::write(spec_dir.join("testing.md"), "existing tests").unwrap();
        fs::write(spec_dir.join("design.md"), "existing design").unwrap();
        generate_companion_files(spec_dir, "auth", true);

        let tasks = fs::read_to_string(spec_dir.join("tasks.md")).unwrap();
        assert_eq!(tasks, "existing content");
        let testing = fs::read_to_string(spec_dir.join("testing.md")).unwrap();
        assert_eq!(testing, "existing tests");
        let design = fs::read_to_string(spec_dir.join("design.md")).unwrap();
        assert_eq!(design, "existing design");
    }

    #[test]
    fn companion_files_from_template_uses_custom_testing() {
        let tmp = TempDir::new().unwrap();
        let spec_dir = tmp.path();
        let template_dir = tmp.path().join("templates");
        fs::create_dir_all(&template_dir).unwrap();

        let custom =
            "---\nspec: {module}.spec.md\n---\n\n## Custom Tests\n\nCustom testing template\n";
        fs::write(template_dir.join("testing.md"), custom).unwrap();

        generate_companion_files_from_template(spec_dir, "auth", &template_dir, false);

        let testing = fs::read_to_string(spec_dir.join("testing.md")).unwrap();
        assert!(testing.contains("Custom testing template"));
        assert!(testing.contains("spec: auth.spec.md"));
    }

    #[test]
    fn companion_files_from_template_falls_back_for_testing() {
        let tmp = TempDir::new().unwrap();
        let spec_dir = tmp.path();
        let template_dir = tmp.path().join("templates");
        fs::create_dir_all(&template_dir).unwrap();
        // No testing.md in template dir — should fall back to built-in

        generate_companion_files_from_template(spec_dir, "auth", &template_dir, false);

        let testing = fs::read_to_string(spec_dir.join("testing.md")).unwrap();
        assert!(testing.contains("## Automated Testing"));
        assert!(testing.contains("spec: auth.spec.md"));
    }

    #[test]
    fn companion_files_from_template_uses_custom_design() {
        let tmp = TempDir::new().unwrap();
        let spec_dir = tmp.path();
        let template_dir = tmp.path().join("templates");
        fs::create_dir_all(&template_dir).unwrap();

        let custom =
            "---\nspec: {module}.spec.md\nsources: []\n---\n\n## Custom Design\n\nCustom layout\n";
        fs::write(template_dir.join("design.md"), custom).unwrap();

        generate_companion_files_from_template(spec_dir, "auth", &template_dir, true);

        let design = fs::read_to_string(spec_dir.join("design.md")).unwrap();
        assert!(design.contains("Custom layout"));
        assert!(design.contains("spec: auth.spec.md"));
    }

    #[test]
    fn companion_files_from_template_falls_back_for_design() {
        let tmp = TempDir::new().unwrap();
        let spec_dir = tmp.path();
        let template_dir = tmp.path().join("templates");
        fs::create_dir_all(&template_dir).unwrap();

        generate_companion_files_from_template(spec_dir, "auth", &template_dir, true);

        let design = fs::read_to_string(spec_dir.join("design.md")).unwrap();
        assert!(design.contains("## Layout"));
        assert!(design.contains("spec: auth.spec.md"));
    }

    // ── find_files_for_module ──────────────────────────────────────

    #[test]
    fn find_files_flat_module() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let src_dir = root.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("auth.rs"), "pub fn login() {}").unwrap();
        fs::write(src_dir.join("other.rs"), "pub fn other() {}").unwrap();

        let config = SpecSyncConfig::default();
        let files = find_files_for_module(root, "auth", &config);
        assert_eq!(files.len(), 1);
        assert!(files[0].contains("auth.rs"));
    }

    #[test]
    fn find_files_subdir_module() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let mod_dir = root.join("src").join("auth");
        fs::create_dir_all(&mod_dir).unwrap();
        fs::write(mod_dir.join("service.ts"), "export function login() {}").unwrap();
        fs::write(mod_dir.join("types.ts"), "export interface User {}").unwrap();

        let mut config = SpecSyncConfig::default();
        config.source_extensions = vec!["ts".to_string()];
        let files = find_files_for_module(root, "auth", &config);
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn find_files_excludes_test_files() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let src_dir = root.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("auth.ts"), "export function login() {}").unwrap();
        fs::write(src_dir.join("auth.test.ts"), "test('login', () => {})").unwrap();

        let mut config = SpecSyncConfig::default();
        config.source_extensions = vec!["ts".to_string()];
        let files = find_files_for_module(root, "auth", &config);
        assert_eq!(files.len(), 1);
        assert!(!files[0].contains("test"));
    }

    #[test]
    fn find_files_no_match() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let src_dir = root.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("other.rs"), "fn other() {}").unwrap();

        let config = SpecSyncConfig::default();
        let files = find_files_for_module(root, "nonexistent", &config);
        assert!(files.is_empty());
    }

    #[test]
    fn find_files_user_defined_module() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let src_dir = root.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("foo.rs"), "pub fn foo() {}").unwrap();
        fs::write(src_dir.join("bar.rs"), "pub fn bar() {}").unwrap();

        let mut config = SpecSyncConfig::default();
        config.modules.insert(
            "my-module".to_string(),
            crate::types::ModuleDefinition {
                files: vec!["src/foo.rs".to_string(), "src/bar.rs".to_string()],
                depends_on: vec![],
            },
        );
        let files = find_files_for_module(root, "my-module", &config);
        assert_eq!(files.len(), 2);
    }
}
