use colored::Colorize;
use std::path::Path;

use crate::archive;
use crate::compact;
use crate::config::load_config;

pub(crate) fn cmd_compact(root: &Path, keep: usize, dry_run: bool) {
    let config = load_config(root);
    let specs_dir = root.join(&config.specs_dir);

    if dry_run {
        println!("{} Dry run — no files will be modified\n", "ℹ".cyan());
    }

    let results = compact::compact_changelogs(root, &specs_dir, keep, dry_run);

    if results.is_empty() {
        println!(
            "{}",
            "No changelogs need compaction (all within limit).".green()
        );
        return;
    }

    for r in &results {
        let verb = if dry_run {
            "would compact"
        } else {
            "compacted"
        };
        println!(
            "  {} {} — {verb} {} entries (kept {})",
            "✓".green(),
            r.spec_path,
            r.removed,
            r.compacted_entries,
        );
    }

    let total_removed: usize = results.iter().map(|r| r.removed).sum();
    println!(
        "\n{} {} entries across {} spec(s)",
        if dry_run {
            "Would compact".to_string()
        } else {
            "Compacted".to_string()
        },
        total_removed,
        results.len()
    );
}

pub(crate) fn cmd_archive_tasks(root: &Path, dry_run: bool) {
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
