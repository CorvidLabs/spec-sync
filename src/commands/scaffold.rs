use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use crate::config::load_config;
use crate::generator;
use crate::registry;

use super::validate_module_name;

pub fn cmd_add_spec(root: &Path, module_name: &str) {
    if let Err(e) = validate_module_name(module_name) {
        eprintln!("{e}");
        process::exit(1);
    }
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

    let module_files = generator::find_files_for_module(root, module_name, &config);
    if module_files.is_empty() {
        eprintln!(
            "{} No source files matched module '{module_name}' — the draft uses an explicit empty `files: []` list.",
            "⚠".yellow()
        );
    }
    let spec_content = generator::generate_spec(module_name, &module_files, root, &specs_dir);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_spec_omits_module_javascript_test_sources() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let module = root.join("src/widget");
        fs::create_dir_all(&module).unwrap();
        fs::write(module.join("index.mjs"), "export const value = true;\n").unwrap();
        fs::write(module.join("index.test.cjs"), "exports.helper = true;\n").unwrap();
        fs::write(
            module.join("index.spec.mjs"),
            "export const fixture = true;\n",
        )
        .unwrap();

        cmd_add_spec(root, "widget");

        let spec = fs::read_to_string(root.join("specs/widget/widget.spec.md")).unwrap();
        assert!(spec.contains("src/widget/index.mjs"), "{spec}");
        assert!(spec.contains("| `value` |"), "{spec}");
        assert!(!spec.contains("index.test.cjs"), "{spec}");
        assert!(!spec.contains("index.spec.mjs"), "{spec}");
    }
}

pub fn cmd_scaffold(
    root: &Path,
    module_name: &str,
    dir: Option<PathBuf>,
    template: Option<PathBuf>,
) {
    if let Err(e) = validate_module_name(module_name) {
        eprintln!("{e}");
        process::exit(1);
    }
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

    // Auto-detect source files matching the module name; for single-source-file
    // projects (e.g. only src/lib.rs) fall back to that file.
    let mut module_files = generator::find_files_for_module(root, module_name, &config);
    if module_files.is_empty()
        && let Some(single) = generator::find_single_source_fallback(root, &config)
    {
        module_files.push(single);
    }

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
