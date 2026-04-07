use colored::Colorize;
use std::path::Path;
use std::process;

use crate::config::load_config;
use crate::merge;
use crate::types;

pub(crate) fn cmd_merge(root: &Path, dry_run: bool, all: bool, format: types::OutputFormat) {
    let config = load_config(root);
    let specs_dir = root.join(&config.specs_dir);

    if dry_run {
        println!("{} Dry run — no files will be modified\n", "ℹ".cyan());
    }

    let results = merge::merge_specs(root, &specs_dir, dry_run, all);

    match format {
        types::OutputFormat::Json => {
            println!("{}", merge::results_to_json(&results));
        }
        _ => {
            merge::print_results(&results, dry_run);
        }
    }

    // Exit non-zero if any conflicts need manual resolution
    if results
        .iter()
        .any(|r| matches!(r.status, merge::MergeStatus::Manual))
    {
        process::exit(1);
    }
}
