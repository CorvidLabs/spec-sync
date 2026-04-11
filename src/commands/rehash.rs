use colored::Colorize;
use std::path::Path;

use crate::hash_cache;

pub fn cmd_rehash(root: &Path) {
    let (_config, spec_files) = super::load_and_discover(root, false);

    let mut cache = hash_cache::HashCache::default();
    hash_cache::update_cache(root, &spec_files, &mut cache);

    if let Err(e) = cache.save(root) {
        eprintln!(
            "{} Failed to save hash cache: {e}",
            "error:".red().bold()
        );
        std::process::exit(1);
    }

    println!(
        "{} Regenerated hash cache for {} spec(s) → .specsync/hashes.json",
        "✓".green(),
        spec_files.len()
    );
}
