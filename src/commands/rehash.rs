use colored::Colorize;
use std::path::{Path, PathBuf};

use crate::ignore::IgnoreRules;
use crate::validator::get_schema_table_names;
use crate::{config, hash_cache, validator};

use super::{
    build_schema_columns, global_validation_inputs, run_validation_with_cache, spec_inventory,
};

pub fn cmd_rehash(root: &Path) {
    let (config, spec_files) = discover_spec_files(root);
    if spec_files.is_empty() {
        let abs_specs = root.join(&config.specs_dir);
        println!(
            "No spec files found in {}/. Run `specsync generate` to scaffold specs.",
            abs_specs.display()
        );
        std::process::exit(0);
    }

    let mut cache = hash_cache::HashCache::default();
    let global_inputs = global_validation_inputs(root, &config);
    let inventory = spec_inventory(root, &spec_files);
    let schema_tables = get_schema_table_names(root, &config);
    let schema_columns = build_schema_columns(root, &config);
    let ignore_rules = IgnoreRules::load(root);
    let (total_errors, _, _, _, _, _, _) = run_validation_with_cache(
        root,
        &spec_files,
        &spec_files,
        &schema_tables,
        &schema_columns,
        &config,
        true,
        false,
        &ignore_rules,
        Some(&mut cache),
        &global_inputs,
        &inventory,
    );
    if total_errors > 0 {
        // Rehash is not a validation gate, but it must never bless incomplete
        // or project-invalid results as replayable. The next check performs a
        // full validation and reports the errors through its normal channels.
        cache.snapshots.clear();
    }
    hash_cache::update_cache(root, &spec_files, &mut cache);
    for input in &global_inputs {
        cache.update(root, input);
    }

    if let Err(e) = cache.save(root) {
        eprintln!("{} Failed to save hash cache: {e}", "error:".red().bold());
        std::process::exit(1);
    }

    println!(
        "{} Regenerated hash cache for {} spec(s) → .specsync/hashes.json",
        "✓".green(),
        spec_files.len()
    );
}

fn discover_spec_files(root: &Path) -> (crate::types::SpecSyncConfig, Vec<PathBuf>) {
    let config = config::load_config(root);
    let spec_files = validator::find_spec_files(&root.join(&config.specs_dir))
        .into_iter()
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| !name.starts_with('_'))
                .unwrap_or(true)
        })
        .collect();
    (config, spec_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn discover_spec_files_honors_config_and_excludes_templates() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join(".specsync")).unwrap();
        fs::write(
            root.join(".specsync/config.toml"),
            "specs_dir = \"contracts\"\nsource_dirs = [\"src\"]\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("contracts/auth")).unwrap();
        fs::write(root.join("contracts/_template.spec.md"), "template\n").unwrap();
        fs::write(root.join("contracts/auth/auth.spec.md"), "auth\n").unwrap();

        assert_eq!(
            discover_spec_files(root).1,
            vec![root.join("contracts/auth/auth.spec.md")]
        );
    }

    #[test]
    fn cmd_rehash_rebuilds_a_fresh_cache() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join(".specsync")).unwrap();
        fs::write(
            root.join(".specsync/config.toml"),
            "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("specs/auth")).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/auth.rs"), "pub fn login() {}\n").unwrap();
        fs::write(
            root.join("specs/auth/auth.spec.md"),
            "---\nmodule: auth\nversion: 1\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n",
        )
        .unwrap();
        fs::write(
            root.join(".specsync/hashes.json"),
            r#"{"hashes":{"stale":"entry"}}"#,
        )
        .unwrap();

        cmd_rehash(root);

        let cache = hash_cache::HashCache::load(root);
        assert!(!cache.hashes.contains_key("stale"));
        assert!(cache.hashes.contains_key("specs/auth/auth.spec.md"));
        assert!(cache.hashes.contains_key("src/auth.rs"));
        // The config file `check` consults as a global validation input must be
        // cached too, or the first `check` after `rehash` re-validates everything.
        assert!(cache.hashes.contains_key(".specsync/config.toml"));
    }
}
