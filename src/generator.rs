use crate::exports::{has_configured_extension, is_test_file};
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

- [ ] Add implementation, validation, or release tasks that belong to this spec.

## Gaps

Record concrete coverage gaps or edge cases that still need tests.

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

- As a maintainer, I want this module's contract captured clearly so changes can be reviewed against stable behavior.

## Acceptance Criteria

- Define acceptance criteria from the module's source behavior and user-facing responsibilities.

## Constraints

- Capture performance, compatibility, security, and compliance constraints that apply to this module.

## Out of Scope

- List behaviors or responsibilities intentionally handled by other modules.
"#;

const CONTEXT_TEMPLATE: &str = r#"---
spec: {module}.spec.md
---

## Key Decisions

- Record architectural or design decisions relevant to this spec.

## Files to Read First

- List the most important files an agent or new developer should read.

## Current Status

- Summarize implemented behavior, active work, and known blockers.

## Notes

- Capture useful links, investigation notes, and operational context.
"#;

const TESTING_TEMPLATE: &str = r#"---
spec: {module}.spec.md
---

## Automated Testing

List the automated tests and fixtures that protect this module.

| Test File | Type | What It Covers |
|-----------|------|----------------|

## Manual Testing

List manual QA flows, platform checks, and review notes for this module.

- [ ] Run the module's primary workflow and compare behavior against this spec.

## Edge Cases & Boundary Conditions

List boundary conditions, race risks, permission cases, and error paths.

| Scenario | Expected Behavior |
|----------|-------------------|
"#;

const DESIGN_TEMPLATE: &str = r#"---
spec: {module}.spec.md
sources: []
---

## Layout

- Document layout structure, responsive breakpoints, and positioning rules.

## Components

- Document component tree, inputs, outputs, and slots.

## Tokens

- Document color, spacing, typography, and state token overrides.

## Assets

- List icons, images, illustrations, and asset ownership.
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

Document this module's responsibility, inputs, outputs, and ownership boundaries.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|

### Exported Types

| Type | Description |
|------|-------------|

## Invariants

1. Define an invariant that must remain true for supported inputs.

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

Document this module's responsibility, inputs, outputs, and ownership boundaries.

## Public API

### Types

| Type | Kind | Description |
|------|------|-------------|

### Protocols

| Protocol | Description |
|----------|-------------|

## Invariants

1. Define an invariant that must remain true for supported inputs.

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

Document this module's responsibility, inputs, outputs, and ownership boundaries.

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

1. Define an invariant that must remain true for supported inputs.

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

Document this module's responsibility, inputs, outputs, and ownership boundaries.

## Public API

### Classes & Interfaces

| Type | Kind | Description |
|------|------|-------------|

### Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|

## Invariants

1. Define an invariant that must remain true for supported inputs.

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

Document this package's responsibility, inputs, outputs, and ownership boundaries.

## Public API

### Types

| Type | Kind | Description |
|------|------|-------------|

### Functions

| Function | Signature | Description |
|----------|-----------|-------------|

## Invariants

1. Define an invariant that must remain true for supported inputs.

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

Document this module's responsibility, inputs, outputs, and ownership boundaries.

## Public API

### Classes

| Class | Description |
|-------|-------------|

### Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|

## Invariants

1. Define an invariant that must remain true for supported inputs.

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
        }
        // TypeScript, C#, Dart, and fallback use the default template
        _ => DEFAULT_TEMPLATE,
    }
}

/// Find source files in a module directory.
fn find_module_source_files(dir: &Path, config: &SpecSyncConfig, root: &Path) -> Vec<String> {
    let mut results = Vec::new();
    if !dir.exists() {
        return results;
    }

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file()
            && has_configured_extension(
                path,
                &config.source_extensions,
                config.include_extensionless,
            )
            && !is_test_file(path, root)
        {
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
                module_files.extend(find_module_source_files(&full_path, config, root));
            }
        }
        if !module_files.is_empty() {
            return module_files;
        }
    }

    // Second: look for subdirectory-based modules (src/module_name/)
    for src_dir in &config.source_dirs {
        let module_dir = root.join(src_dir).join(module_name);
        let files = find_module_source_files(&module_dir, config, root);
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
                        || !has_configured_extension(
                            &path,
                            &config.source_extensions,
                            config.include_extensionless,
                        )
                        || is_test_file(&path, root)
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

/// Fallback source detection for single-source-file projects.
///
/// When a module name matches no source directory or file (e.g. `greeter` in a
/// cargo project whose only source is `src/lib.rs`), but the project contains
/// exactly one non-test source file across all configured source directories,
/// that file is unambiguously the module's source. Returns the root-relative
/// path, or `None` when there are zero or multiple candidates.
pub fn find_single_source_fallback(root: &Path, config: &SpecSyncConfig) -> Option<String> {
    let mut found: Option<String> = None;
    for src_dir in &config.source_dirs {
        let base = root.join(src_dir);
        if !base.is_dir() {
            continue;
        }
        for entry in WalkDir::new(&base).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file()
                || !has_configured_extension(
                    path,
                    &config.source_extensions,
                    config.include_extensionless,
                )
                || is_test_file(path, root)
            {
                continue;
            }
            let rel = path
                .strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");
            if found.is_some() {
                return None; // More than one source file — ambiguous, no fallback
            }
            found = Some(rel);
        }
    }
    found
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

    // Replace files list (handles both `files: []` and multi-line YAML list).
    // With no detected source files emit `files: []` — a bare `files:` parses
    // as YAML null and fails the tool's own frontmatter validation.
    let files_re = regex::Regex::new(r"(?m)^files:\s*\[\]|^files:\n(?:\s+-\s+.+\n?)*").unwrap();
    if source_files.is_empty() {
        spec = files_re.replace(&spec, "files: []\n").to_string();
    } else {
        spec = files_re
            .replace(&spec, format!("files:\n{files_yaml}\n"))
            .to_string();
    }

    // Replace title
    let title_re = regex::Regex::new(r"(?m)^# .+$").unwrap();
    spec = title_re.replace(&spec, format!("# {title}")).to_string();

    // Clear db_tables
    let db_re = regex::Regex::new(r"(?m)^db_tables:\n(?:\s+-\s+.+\n?)*").unwrap();
    spec = db_re.replace(&spec, "db_tables: []\n").to_string();

    // Pre-populate the Public API table from detected exports so generated
    // specs document their API surface like `specsync new` does.
    let exports = collect_exports_for_files(root, source_files);
    spec = populate_public_api_table(&spec, &exports);

    spec
}

/// Collect deduplicated exported symbols across a module's source files.
///
/// Paths may be absolute or root-relative; anything escaping the project root
/// is skipped (consistency with validate/score/diff).
pub fn collect_exports_for_files(root: &Path, source_files: &[String]) -> Vec<String> {
    let mut all_exports: Vec<String> = Vec::new();
    for file in source_files {
        let rel = Path::new(file)
            .strip_prefix(root)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| file.clone());
        if !crate::validator::source_within_root(root, &rel) {
            continue;
        }
        let full_path = root.join(&rel);
        all_exports.extend(crate::exports::get_exported_symbols(&full_path));
    }
    let mut seen = std::collections::HashSet::new();
    all_exports.retain(|s| seen.insert(s.clone()));
    all_exports
}

/// Insert a populated export table directly under the `## Public API` heading.
/// Leaves the spec untouched when no exports were detected.
pub fn populate_public_api_table(spec: &str, exports: &[String]) -> String {
    if exports.is_empty() {
        return spec.to_string();
    }
    let header = "| Export | Description |\n|--------|-------------|";
    let rows: String = exports
        .iter()
        .map(|e| {
            format!("| `{e}` | Document the export's responsibility and caller-visible behavior. |")
        })
        .collect::<Vec<_>>()
        .join("\n");
    let heading_re = regex::Regex::new(r"(?m)^## Public API[ \t]*$").unwrap();
    heading_re
        .replace(spec, format!("## Public API\n\n{header}\n{rows}\n"))
        .to_string()
}

/// Generate deterministic spec content for a module from the built-in template.
fn generate_module_spec(
    module_name: &str,
    module_files: &[String],
    root: &Path,
    specs_dir: &Path,
) -> String {
    generate_spec(module_name, module_files, root, specs_dir)
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

/// Outcome of a spec-generation run.
#[derive(Debug, Default)]
pub struct GenerationOutcome {
    /// Number of spec files written.
    pub generated: usize,
    /// Paths (relative to root) of the spec files written.
    pub generated_paths: Vec<String>,
}

/// Generate spec files for all unspecced modules.
/// Returns the deterministic generation outcome.
pub fn generate_specs_for_unspecced_modules(
    root: &Path,
    report: &CoverageReport,
    config: &SpecSyncConfig,
) -> GenerationOutcome {
    let specs_dir = root.join(&config.specs_dir);
    let mut outcome = GenerationOutcome::default();

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

        let spec_content = generate_module_spec(module_name, &module_files, root, &specs_dir);

        match fs::write(&spec_file, &spec_content) {
            Ok(_) => {
                let rel = spec_file.strip_prefix(root).unwrap_or(&spec_file);
                println!(
                    "  {} Generated {} ({} files)",
                    "✓".green(),
                    rel.display(),
                    module_files.len()
                );
                generate_companion_files(&spec_dir, module_name, config.companions.design);
                let _ = std::io::stdout().flush();
                outcome.generated += 1;
                outcome
                    .generated_paths
                    .push(rel.to_string_lossy().to_string());
            }
            Err(e) => {
                eprintln!("  Failed to write {}: {e}", spec_file.display());
            }
        }
    }

    outcome
}

/// Generate spec files for all unspecced modules without per-file progress
/// output (used by JSON and MCP callers). Returns the generation outcome,
/// including the paths of the spec files written.
pub fn generate_specs_for_unspecced_modules_paths(
    root: &Path,
    report: &CoverageReport,
    config: &SpecSyncConfig,
) -> GenerationOutcome {
    let specs_dir = root.join(&config.specs_dir);
    let mut outcome = GenerationOutcome::default();

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

        let spec_content = generate_module_spec(module_name, &module_files, root, &specs_dir);

        if fs::write(&spec_file, &spec_content).is_ok() {
            let rel = spec_file
                .strip_prefix(root)
                .unwrap_or(&spec_file)
                .to_string_lossy()
                .to_string();
            outcome.generated += 1;
            outcome.generated_paths.push(rel);
        }
    }

    outcome
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

        let config = SpecSyncConfig {
            source_extensions: vec!["ts".to_string()],
            ..SpecSyncConfig::default()
        };
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

        let config = SpecSyncConfig {
            source_extensions: vec!["ts".to_string()],
            ..SpecSyncConfig::default()
        };
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

    // ── find_single_source_fallback ────────────────────────────────

    #[test]
    fn single_source_fallback_returns_only_file() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let src_dir = root.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("lib.rs"), "pub fn greet() {}").unwrap();

        let config = SpecSyncConfig::default();
        let file = find_single_source_fallback(root, &config);
        assert_eq!(file.as_deref(), Some("src/lib.rs"));
    }

    #[test]
    fn single_source_fallback_ambiguous_returns_none() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let src_dir = root.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("lib.rs"), "pub fn greet() {}").unwrap();
        fs::write(src_dir.join("util.rs"), "pub fn helper() {}").unwrap();

        let config = SpecSyncConfig::default();
        assert_eq!(find_single_source_fallback(root, &config), None);
    }

    #[test]
    fn single_source_fallback_ignores_test_files() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let src_dir = root.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("lib.ts"), "export function greet() {}").unwrap();
        fs::write(src_dir.join("lib.test.ts"), "test('greet', () => {})").unwrap();

        let config = SpecSyncConfig {
            source_extensions: vec!["ts".to_string()],
            ..SpecSyncConfig::default()
        };
        let file = find_single_source_fallback(root, &config);
        assert_eq!(file.as_deref(), Some("src/lib.ts"));
    }

    #[test]
    fn single_source_fallback_empty_project_returns_none() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src")).unwrap();

        let config = SpecSyncConfig::default();
        assert_eq!(find_single_source_fallback(root, &config), None);
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

    // ── #421 regression: generators must emit self-valid specs ─────────

    #[test]
    fn generate_spec_empty_files_emits_valid_empty_list() {
        // A bare `files:` (YAML null) fails the tool's own frontmatter
        // validation; generators must emit `files: []` instead.
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let specs_dir = root.join("specs");
        fs::create_dir_all(&specs_dir).unwrap();

        let spec = generate_spec("ghost", &[], root, &specs_dir);
        assert!(spec.contains("files: []"), "{spec}");
        let parsed = crate::parser::parse_frontmatter(&spec).expect("frontmatter parses");
        assert!(parsed.frontmatter.files.is_empty());
    }

    #[test]
    fn generate_spec_prepopulates_public_api_from_exports() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let specs_dir = root.join("specs");
        fs::create_dir_all(&specs_dir).unwrap();
        let src_dir = root.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(
            src_dir.join("auth.rs"),
            "pub fn login() {}\npub fn logout() {}\n",
        )
        .unwrap();

        let files = vec!["src/auth.rs".to_string()];
        let spec = generate_spec("auth", &files, root, &specs_dir);
        assert!(spec.contains("| `login` |"), "{spec}");
        assert!(spec.contains("| `logout` |"), "{spec}");

        // The populated table is visible to the spec symbol scanner.
        let parsed = crate::parser::parse_frontmatter(&spec).unwrap();
        let symbols = crate::parser::get_spec_symbols(&parsed.body);
        assert!(symbols.iter().any(|s| s == "login"), "{symbols:?}");
    }

    #[test]
    fn populate_public_api_table_no_exports_is_noop() {
        let spec = "## Public API\n\nempty\n";
        assert_eq!(populate_public_api_table(spec, &[]), spec);
    }
}
