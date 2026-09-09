use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process;

use crate::config::load_config;
use crate::exports;
use crate::generator;

use super::{check_case_collision, validate_scaffold_module_name};

/// Quick-create a minimal spec for a module with auto-detected source files.
pub fn cmd_new(root: &Path, module_name: &str, full: bool) {
    if let Err(e) = validate_scaffold_module_name(module_name) {
        eprintln!("{e}");
        process::exit(1);
    }
    let config = load_config(root);
    let specs_dir = root.join(&config.specs_dir);
    if let Err(e) = check_case_collision(&specs_dir, module_name) {
        eprintln!("{e}");
        process::exit(1);
    }
    let spec_dir = specs_dir.join(module_name);
    let spec_file = spec_dir.join(format!("{module_name}.spec.md"));

    if spec_file.exists() {
        eprintln!(
            "{} Spec already exists: {}",
            "Error:".red(),
            spec_file.strip_prefix(root).unwrap_or(&spec_file).display()
        );
        process::exit(1);
    }

    if let Err(e) = fs::create_dir_all(&spec_dir) {
        eprintln!("{} Failed to create directory: {e}", "Error:".red());
        process::exit(1);
    }

    // Auto-detect source files for this module
    let source_files = detect_module_sources(root, module_name, &config);
    if source_files.is_empty() {
        eprintln!(
            "{} No source files matched module '{module_name}' — the spec is created with an empty `files:` list.",
            "⚠".yellow()
        );
        eprintln!(
            "  Add the module's source path(s) to the `files:` list in the spec frontmatter,"
        );
        eprintln!(
            "  or define the module in your config — `specsync check` fails on empty `files:`."
        );
    }

    // Same skeleton as `add-spec` / `generate`: every required_sections heading
    // `init` writes, plus a pre-populated Public API table from detected exports.
    let spec_content =
        generator::generate_spec(module_name, &source_files, root, &specs_dir, &config);

    if let Err(e) = fs::write(&spec_file, &spec_content) {
        eprintln!("{} Failed to write spec: {e}", "Error:".red());
        process::exit(1);
    }

    let rel = spec_file.strip_prefix(root).unwrap_or(&spec_file);
    println!("{} Created {}", "✓".green(), rel.display());

    if !source_files.is_empty() {
        println!(
            "  {} Auto-detected {} source file(s)",
            "→".cyan(),
            source_files.len()
        );
    }
    let export_count = generator::collect_exports_for_files(root, &source_files, &config).len();
    if export_count > 0 {
        println!(
            "  {} Pre-populated {export_count} export(s) in Public API",
            "→".cyan(),
        );
    }

    if full {
        generator::generate_companion_files_for_spec(
            &spec_dir,
            module_name,
            config.companions.design,
        );
        let design_note = if config.companions.design {
            ", design.md"
        } else {
            ""
        };
        println!(
            "  {} Created companion files (tasks.md, context.md, requirements.md, testing.md{})",
            "→".cyan(),
            design_note,
        );
    }
}

/// Detect source files that belong to this module by scanning source directories.
fn detect_module_sources(
    root: &Path,
    module_name: &str,
    config: &crate::types::SpecSyncConfig,
) -> Vec<String> {
    let mut files: Vec<String> = Vec::new();

    for src_dir in &config.source_dirs {
        let base = root.join(src_dir);

        // Check for directory matching module name (e.g., src/auth/)
        let module_dir = base.join(module_name);
        if module_dir.is_dir() {
            for entry in walkdir::WalkDir::new(&module_dir)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.path().is_file()
                    && exports::has_configured_extension(
                        entry.path(),
                        &config.source_extensions,
                        config.include_extensionless,
                    )
                    && !exports::is_test_file(entry.path(), root)
                {
                    let rel = entry
                        .path()
                        .strip_prefix(root)
                        .unwrap_or(entry.path())
                        .to_string_lossy()
                        .replace('\\', "/");
                    files.push(rel);
                }
            }
        }

        // Check for single file matching module name (e.g., src/auth.ts, src/auth.rs)
        if base.is_dir() {
            for entry in fs::read_dir(&base).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.is_file() {
                    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                    if stem == module_name
                        && exports::has_configured_extension(
                            &path,
                            &config.source_extensions,
                            config.include_extensionless,
                        )
                        && !exports::is_test_file(&path, root)
                    {
                        let rel = path
                            .strip_prefix(root)
                            .unwrap_or(&path)
                            .to_string_lossy()
                            .replace('\\', "/");
                        if !files.contains(&rel) {
                            files.push(rel);
                        }
                    }
                }
            }
        }
    }

    // Fallback: a single-source-file project (e.g. only src/lib.rs) has exactly
    // one possible source — use it even though the name doesn't match.
    if files.is_empty()
        && let Some(single) = generator::find_single_source_fallback(root, config)
    {
        files.push(single);
    }

    files.sort();
    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detected_sources_omit_module_javascript_tests() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let module = root.join("src/widget");
        fs::create_dir_all(&module).unwrap();
        fs::write(module.join("index.cjs"), "exports.value = true;\n").unwrap();
        fs::write(module.join("index.test.cjs"), "exports.helper = true;\n").unwrap();
        fs::write(
            module.join("index.spec.mjs"),
            "export const fixture = true;\n",
        )
        .unwrap();
        let config = crate::types::SpecSyncConfig::default();

        let files = detect_module_sources(root, "widget", &config);

        assert_eq!(files, vec!["src/widget/index.cjs"]);
    }
}
