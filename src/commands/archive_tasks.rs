use colored::Colorize;
use std::path::Path;

use crate::archive;
use crate::config::load_config;

pub fn cmd_archive_tasks(root: &Path, dry_run: bool) {
    let config = load_config(root);
    let specs_dir = root.join(&config.specs_dir);

    if dry_run {
        println!("{} Dry run — no files will be modified\n", "ℹ".cyan());
    }

    let results = archive::archive_tasks(root, &specs_dir, dry_run);

    if results.is_empty() {
        println!("{}", "No completed tasks to archive.".green());
        return;
    }

    for r in &results {
        let verb = if dry_run { "would archive" } else { "archived" };
        println!(
            "  {} {} — {verb} {} task(s)",
            "✓".green(),
            r.tasks_path,
            r.archived_count,
        );
    }

    let total: usize = results.iter().map(|r| r.archived_count).sum();
    println!(
        "\n{} {} task(s) across {} file(s)",
        if dry_run {
            "Would archive".to_string()
        } else {
            "Archived".to_string()
        },
        total,
        results.len()
    );
}
