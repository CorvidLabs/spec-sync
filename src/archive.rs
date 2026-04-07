use colored::Colorize;
use std::fs;
use std::path::Path;

use crate::validator::find_spec_files;

/// Result of archiving tasks in a single tasks.md file.
pub struct ArchiveResult {
    pub tasks_path: String,
    pub archived_count: usize,
}

/// Archive completed tasks across all companion tasks.md files.
/// Moves `- [x]` items to an `## Archive` section at the bottom.
pub fn archive_tasks(root: &Path, specs_dir: &Path, dry_run: bool) -> Vec<ArchiveResult> {
    let spec_files = find_spec_files(specs_dir);
    let mut results = Vec::new();

    for spec_path in &spec_files {
        // Find the companion tasks.md in the same directory
        let spec_dir = match spec_path.parent() {
            Some(d) => d,
            None => continue,
        };
        let tasks_path = spec_dir.join("tasks.md");
        if !tasks_path.exists() {
            continue;
        }

        let content = match fs::read_to_string(&tasks_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let rel_path = tasks_path
            .strip_prefix(root)
            .unwrap_or(&tasks_path)
            .to_string_lossy()
            .to_string();

        if let Some((new_content, count)) = archive_completed_tasks(&content) {
            if count > 0 {
                if !dry_run {
                    if let Err(e) = fs::write(&tasks_path, &new_content) {
                        eprintln!(
                            "{} Failed to write {}: {e}",
                            "error:".red().bold(),
                            rel_path
                        );
                        continue;
                    }
                }
                results.push(ArchiveResult {
                    tasks_path: rel_path,
                    archived_count: count,
                });
            }
        }
    }

    results
}

/// Archive completed tasks in a tasks.md file.
/// Returns (new_content, archived_count) if any tasks were archived.
fn archive_completed_tasks(content: &str) -> Option<(String, usize)> {
    let mut completed_tasks: Vec<String> = Vec::new();
    let mut remaining_lines: Vec<String> = Vec::new();
    let mut in_archive = false;
    let mut existing_archive: Vec<String> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Track if we're in the archive section
        if trimmed == "## Archive" {
            in_archive = true;
            continue;
        }
        if in_archive {
            if trimmed.starts_with("## ") {
                // Exited archive section into next section
                in_archive = false;
                remaining_lines.push(line.to_string());
            } else {
                existing_archive.push(line.to_string());
            }
            continue;
        }

        // Check for completed tasks outside the archive section
        if trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
            completed_tasks.push(line.to_string());
        } else {
            remaining_lines.push(line.to_string());
        }
    }

    if completed_tasks.is_empty() {
        return None;
    }

    let count = completed_tasks.len();

    // Build new content: remaining lines + archive section
    let mut new_content = remaining_lines.join("\n");

    // Ensure trailing newline before archive section
    if !new_content.ends_with('\n') {
        new_content.push('\n');
    }
    new_content.push('\n');
    new_content.push_str("## Archive\n\n");

    // Add existing archive entries first
    for line in &existing_archive {
        if !line.trim().is_empty() {
            new_content.push_str(line);
            new_content.push('\n');
        }
    }

    // Add newly archived tasks
    for task in &completed_tasks {
        new_content.push_str(task);
        new_content.push('\n');
    }

    Some((new_content, count))
}

/// Count completed tasks across all tasks.md files (for warnings in check command).
#[allow(dead_code)]
pub fn count_completed_tasks(specs_dir: &Path) -> usize {
    let spec_files = find_spec_files(specs_dir);
    let mut total = 0;

    for spec_path in &spec_files {
        let spec_dir = match spec_path.parent() {
            Some(d) => d,
            None => continue,
        };
        let tasks_path = spec_dir.join("tasks.md");
        if !tasks_path.exists() {
            continue;
        }
        if let Ok(content) = fs::read_to_string(&tasks_path) {
            total += content
                .lines()
                .filter(|l| {
                    let t = l.trim();
                    t.starts_with("- [x]") || t.starts_with("- [X]")
                })
                .count();
        }
    }

    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archive_completed_tasks() {
        let content = r#"---
spec: test.spec.md
---

## Tasks

- [ ] Uncompleted task
- [x] Done task 1
- [ ] Another open task
- [x] Done task 2

## Gaps

Nothing here.
"#;

        let (new_content, count) = archive_completed_tasks(content).unwrap();
        assert_eq!(count, 2);
        assert!(new_content.contains("## Archive"));
        assert!(new_content.contains("- [x] Done task 1"));
        assert!(new_content.contains("- [x] Done task 2"));
        assert!(new_content.contains("- [ ] Uncompleted task"));
        // Archived tasks should not appear in the Tasks section
        assert!(!new_content[..new_content.find("## Archive").unwrap()].contains("- [x]"));
    }

    #[test]
    fn test_archive_no_completed() {
        let content = r#"## Tasks

- [ ] Open task
"#;

        assert!(archive_completed_tasks(content).is_none());
    }

    #[test]
    fn test_archive_preserves_existing() {
        let content = r#"## Tasks

- [x] New done task

## Archive

- [x] Previously archived
"#;

        let (new_content, count) = archive_completed_tasks(content).unwrap();
        assert_eq!(count, 1);
        assert!(new_content.contains("- [x] Previously archived"));
        assert!(new_content.contains("- [x] New done task"));
    }
}
