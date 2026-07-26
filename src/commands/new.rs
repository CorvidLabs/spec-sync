use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process;

use crate::config::load_config;
use crate::generator;

use super::validate_module_name;

/// Quick-create a minimal spec for a module with auto-detected source files.
pub fn cmd_new(root: &Path, module_name: &str, full: bool) {
    if let Err(e) = validate_module_name(module_name) {
        eprintln!("{e}");
        process::exit(1);
    }
    let config = load_config(root);
    let specs_dir = root.join(&config.specs_dir);
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

    // Use the same source discovery and renderer as generate/scaffold/add-spec.
    let mut source_files = generator::find_files_for_module(root, module_name, &config);
    if source_files.is_empty()
        && let Some(single) = generator::find_single_source_fallback(root, &config)
    {
        source_files.push(single);
    }
    if source_files.is_empty() {
        eprintln!(
            "{} No source files matched module '{module_name}' — the spec is created with an empty `files:` list.",
            "⚠".yellow()
        );
        eprintln!(
            "  Add the module's source path(s) to the `files:` list in the spec frontmatter,"
        );
        eprintln!("  or define the module in your config before promoting the draft.");
    }

    let all_exports = generator::collect_exports_for_files(root, &source_files);
    let spec_content = generator::generate_spec(module_name, &source_files, root, &specs_dir);

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
    if !all_exports.is_empty() {
        println!(
            "  {} Pre-populated {} export(s) in Public API",
            "→".cyan(),
            all_exports.len()
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
