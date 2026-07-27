use std::collections::HashSet;
use std::fs::{self, Permissions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::validator::find_spec_files;

/// A task archival operation selected for one companion `tasks.md` file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveResult {
    pub tasks_path: PathBuf,
    pub archived_count: usize,
}

/// The filesystem phase that failed while preparing or applying an archive operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchiveOperation {
    Inspect,
    Read,
    Preflight,
    Stage,
    Publish,
    Rollback,
}

impl ArchiveOperation {
    /// Stable machine-readable name for structured command output.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Inspect => "inspect",
            Self::Read => "read",
            Self::Preflight => "preflight",
            Self::Stage => "stage",
            Self::Publish => "publish",
            Self::Rollback => "rollback",
        }
    }
}

/// A structured failure for one filesystem phase and companion path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveFailure {
    pub tasks_path: PathBuf,
    pub operation: ArchiveOperation,
    pub error: String,
}

/// Complete plan and execution outcome for an archive-tasks invocation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveReport {
    pub dry_run: bool,
    pub planned: Vec<ArchiveResult>,
    pub succeeded: Vec<ArchiveResult>,
    pub rolled_back: Vec<ArchiveResult>,
    pub failed: Vec<ArchiveFailure>,
}

impl ArchiveReport {
    /// Whether every selected operation was either previewed or applied successfully.
    pub fn is_complete(&self) -> bool {
        self.failed.is_empty() && (self.dry_run || self.succeeded.len() == self.planned.len())
    }

    /// Whether at least one destination remains changed after an incomplete apply.
    pub fn is_partial(&self) -> bool {
        !self.dry_run && !self.is_complete() && !self.succeeded.is_empty()
    }

    /// Whether a complete non-empty apply was published.
    pub fn applied(&self) -> bool {
        !self.dry_run && self.is_complete() && !self.succeeded.is_empty()
    }

    /// Total number of tasks selected by the plan.
    pub fn planned_tasks(&self) -> usize {
        self.planned
            .iter()
            .map(|result| result.archived_count)
            .sum()
    }

    /// Total number of tasks left archived after execution.
    pub fn succeeded_tasks(&self) -> usize {
        self.succeeded
            .iter()
            .map(|result| result.archived_count)
            .sum()
    }
}

#[derive(Clone)]
struct PlannedArchive {
    result: ArchiveResult,
    absolute_path: PathBuf,
    original_content: Vec<u8>,
    replacement_content: Vec<u8>,
    permissions: Permissions,
}

struct StagedArchive {
    plan: PlannedArchive,
    temporary: tempfile::NamedTempFile,
}

#[derive(Clone, Copy, Debug, Default)]
struct FailureInjection {
    stage_index: Option<usize>,
    publish_index: Option<usize>,
    rollback_index: Option<usize>,
}

/// Archive completed tasks across all companion tasks.md files.
///
/// The invocation first plans and validates every operation, then stages every replacement in
/// its destination directory before publishing any file. Planning or staging failures therefore
/// leave all destination files untouched. A late publication failure is reported and previously
/// published replacements are rolled back when that can be done safely.
pub fn archive_tasks(root: &Path, specs_dir: &Path, dry_run: bool) -> ArchiveReport {
    archive_tasks_with_failures(root, specs_dir, dry_run, FailureInjection::default())
}

fn archive_tasks_with_failures(
    root: &Path,
    specs_dir: &Path,
    dry_run: bool,
    failure_injection: FailureInjection,
) -> ArchiveReport {
    let (plans, failures) = plan_archive_operations(root, specs_dir);
    let mut report = ArchiveReport {
        dry_run,
        planned: plans.iter().map(|plan| plan.result.clone()).collect(),
        succeeded: Vec::new(),
        rolled_back: Vec::new(),
        failed: failures,
    };

    if dry_run || !report.failed.is_empty() || plans.is_empty() {
        return report;
    }

    let mut staged = Vec::with_capacity(plans.len());
    for (index, plan) in plans.into_iter().enumerate() {
        if failure_injection.stage_index == Some(index) {
            report.failed.push(ArchiveFailure {
                tasks_path: plan.result.tasks_path,
                operation: ArchiveOperation::Stage,
                error: "injected staging failure".to_string(),
            });
            return report;
        }

        match stage_archive(plan) {
            Ok(operation) => staged.push(operation),
            Err(failure) => {
                report.failed.push(failure);
                return report;
            }
        }
    }

    let mut published = Vec::new();
    for (index, operation) in staged.into_iter().enumerate() {
        if failure_injection.publish_index == Some(index) {
            report.failed.push(ArchiveFailure {
                tasks_path: operation.plan.result.tasks_path.clone(),
                operation: ArchiveOperation::Publish,
                error: "injected publication failure".to_string(),
            });
            rollback_published(&mut report, published, failure_injection.rollback_index);
            return report;
        }

        let StagedArchive { plan, temporary } = operation;
        match temporary.persist(&plan.absolute_path) {
            Ok(file) => {
                drop(file);
                report.succeeded.push(plan.result.clone());
                published.push(plan);
            }
            Err(error) => {
                report.failed.push(ArchiveFailure {
                    tasks_path: plan.result.tasks_path,
                    operation: ArchiveOperation::Publish,
                    error: error.error.to_string(),
                });
                rollback_published(&mut report, published, failure_injection.rollback_index);
                return report;
            }
        }
    }

    report
}

fn plan_archive_operations(
    root: &Path,
    specs_dir: &Path,
) -> (Vec<PlannedArchive>, Vec<ArchiveFailure>) {
    let mut plans = Vec::new();
    let mut failures = Vec::new();
    let mut seen_tasks_paths = HashSet::new();

    for spec_path in find_spec_files(specs_dir) {
        let Some(spec_dir) = spec_path.parent() else {
            continue;
        };
        let tasks_path = spec_dir.join("tasks.md");
        if !seen_tasks_paths.insert(tasks_path.clone()) {
            continue;
        }

        let report_path = relative_report_path(root, &tasks_path);
        let metadata = match fs::symlink_metadata(&tasks_path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => {
                failures.push(ArchiveFailure {
                    tasks_path: report_path,
                    operation: ArchiveOperation::Inspect,
                    error: error.to_string(),
                });
                continue;
            }
        };

        if metadata.file_type().is_symlink() {
            failures.push(ArchiveFailure {
                tasks_path: report_path,
                operation: ArchiveOperation::Preflight,
                error: "refusing to replace a symbolic-link tasks file".to_string(),
            });
            continue;
        }
        if !metadata.is_file() {
            failures.push(ArchiveFailure {
                tasks_path: report_path,
                operation: ArchiveOperation::Preflight,
                error: "tasks path is not a regular file".to_string(),
            });
            continue;
        }

        let original_content = match fs::read(&tasks_path) {
            Ok(content) => content,
            Err(error) => {
                failures.push(ArchiveFailure {
                    tasks_path: report_path,
                    operation: ArchiveOperation::Read,
                    error: error.to_string(),
                });
                continue;
            }
        };
        let content = match std::str::from_utf8(&original_content) {
            Ok(content) => content,
            Err(error) => {
                failures.push(ArchiveFailure {
                    tasks_path: report_path,
                    operation: ArchiveOperation::Read,
                    error: format!("tasks file is not valid UTF-8: {error}"),
                });
                continue;
            }
        };

        if let Some((replacement_content, archived_count)) = archive_completed_tasks(content)
            && archived_count > 0
        {
            plans.push(PlannedArchive {
                result: ArchiveResult {
                    tasks_path: report_path,
                    archived_count,
                },
                absolute_path: tasks_path,
                original_content,
                replacement_content: replacement_content.into_bytes(),
                permissions: metadata.permissions(),
            });
        }
    }

    (plans, failures)
}

fn relative_report_path(root: &Path, tasks_path: &Path) -> PathBuf {
    tasks_path
        .strip_prefix(root)
        .unwrap_or(tasks_path)
        .to_path_buf()
}

fn stage_archive(plan: PlannedArchive) -> Result<StagedArchive, ArchiveFailure> {
    let temporary = stage_replacement(
        &plan.absolute_path,
        &plan.replacement_content,
        &plan.permissions,
    )
    .map_err(|error| ArchiveFailure {
        tasks_path: plan.result.tasks_path.clone(),
        operation: ArchiveOperation::Stage,
        error: error.to_string(),
    })?;

    Ok(StagedArchive { plan, temporary })
}

fn stage_replacement(
    destination: &Path,
    content: &[u8],
    permissions: &Permissions,
) -> io::Result<tempfile::NamedTempFile> {
    let parent = destination.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "archive destination has no parent directory",
        )
    })?;
    let mut temporary = tempfile::Builder::new()
        .prefix(".specsync-archive-")
        .tempfile_in(parent)?;
    temporary.as_file_mut().write_all(content)?;
    temporary.as_file_mut().flush()?;
    fs::set_permissions(temporary.path(), permissions.clone())?;
    temporary.as_file_mut().sync_all()?;
    Ok(temporary)
}

fn rollback_published(
    report: &mut ArchiveReport,
    published: Vec<PlannedArchive>,
    injected_failure_index: Option<usize>,
) {
    let mut rollback_failed_paths = HashSet::new();

    for (index, plan) in published.into_iter().rev().enumerate() {
        let rollback_result = if injected_failure_index == Some(index) {
            Err(io::Error::other("injected rollback failure"))
        } else {
            stage_replacement(
                &plan.absolute_path,
                &plan.original_content,
                &plan.permissions,
            )
            .and_then(|temporary| {
                temporary
                    .persist(&plan.absolute_path)
                    .map(drop)
                    .map_err(|error| error.error)
            })
        };

        match rollback_result {
            Ok(()) => report.rolled_back.push(plan.result),
            Err(error) => {
                rollback_failed_paths.insert(plan.result.tasks_path.clone());
                report.failed.push(ArchiveFailure {
                    tasks_path: plan.result.tasks_path,
                    operation: ArchiveOperation::Rollback,
                    error: error.to_string(),
                });
            }
        }
    }

    report
        .succeeded
        .retain(|result| rollback_failed_paths.contains(&result.tasks_path));
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

    fn write_archive_fixture(root: &Path, module: &str, tasks: &[u8]) -> PathBuf {
        let module_dir = root.join("specs").join(module);
        fs::create_dir_all(&module_dir).unwrap();
        fs::write(module_dir.join(format!("{module}.spec.md")), "# fixture\n").unwrap();
        let tasks_path = module_dir.join("tasks.md");
        fs::write(&tasks_path, tasks).unwrap();
        tasks_path
    }

    // Requirement evidence: REQ-archive-001.
    #[test]
    fn planning_failure_prevents_all_destination_writes() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        let first = write_archive_fixture(root, "first", b"## Tasks\n\n- [x] done\n");
        let invalid = write_archive_fixture(root, "invalid", b"## Tasks\n\n- [x] \xFF\n");
        let first_before = fs::read(&first).unwrap();
        let invalid_before = fs::read(&invalid).unwrap();

        let report = archive_tasks(root, &root.join("specs"), false);

        assert!(!report.is_complete());
        assert!(!report.applied());
        assert!(report.succeeded.is_empty());
        assert_eq!(report.planned.len(), 1);
        assert_eq!(report.failed.len(), 1);
        assert_eq!(report.failed[0].operation, ArchiveOperation::Read);
        assert_eq!(fs::read(first).unwrap(), first_before);
        assert_eq!(fs::read(invalid).unwrap(), invalid_before);
    }

    #[test]
    fn all_failed_planning_is_incomplete_without_selected_work() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        let invalid = write_archive_fixture(root, "invalid", b"## Tasks\n\n- [x] \xFF\n");
        let invalid_before = fs::read(&invalid).unwrap();

        let report = archive_tasks(root, &root.join("specs"), false);

        assert!(!report.is_complete());
        assert!(!report.is_partial());
        assert!(!report.applied());
        assert!(report.planned.is_empty());
        assert!(report.succeeded.is_empty());
        assert_eq!(report.failed.len(), 1);
        assert_eq!(report.failed[0].operation, ArchiveOperation::Read);
        assert_eq!(fs::read(invalid).unwrap(), invalid_before);
    }

    #[test]
    fn middle_staging_failure_prevents_all_destination_writes() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        let first = write_archive_fixture(root, "first", b"## Tasks\n\n- [x] first\n");
        let second = write_archive_fixture(root, "second", b"## Tasks\n\n- [x] second\n");
        let first_before = fs::read(&first).unwrap();
        let second_before = fs::read(&second).unwrap();

        let report = archive_tasks_with_failures(
            root,
            &root.join("specs"),
            false,
            FailureInjection {
                stage_index: Some(1),
                ..FailureInjection::default()
            },
        );

        assert!(!report.is_complete());
        assert!(report.succeeded.is_empty());
        assert_eq!(report.failed[0].operation, ArchiveOperation::Stage);
        assert_eq!(fs::read(first).unwrap(), first_before);
        assert_eq!(fs::read(second).unwrap(), second_before);
    }

    #[test]
    fn middle_publish_failure_rolls_back_prior_replacements() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        let first = write_archive_fixture(root, "first", b"## Tasks\n\n- [x] first\n");
        let second = write_archive_fixture(root, "second", b"## Tasks\n\n- [x] second\n");
        let first_before = fs::read(&first).unwrap();
        let second_before = fs::read(&second).unwrap();

        let report = archive_tasks_with_failures(
            root,
            &root.join("specs"),
            false,
            FailureInjection {
                publish_index: Some(1),
                ..FailureInjection::default()
            },
        );

        assert!(!report.is_complete());
        assert!(!report.is_partial());
        assert!(!report.applied());
        assert!(report.succeeded.is_empty());
        assert_eq!(report.rolled_back.len(), 1);
        assert_eq!(report.failed[0].operation, ArchiveOperation::Publish);
        assert_eq!(fs::read(first).unwrap(), first_before);
        assert_eq!(fs::read(second).unwrap(), second_before);
    }

    #[test]
    fn rollback_failure_exposes_the_remaining_partial_apply() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        let first = write_archive_fixture(root, "first", b"## Tasks\n\n- [x] first\n");
        let second = write_archive_fixture(root, "second", b"## Tasks\n\n- [x] second\n");
        let first_before = fs::read(&first).unwrap();
        let second_before = fs::read(&second).unwrap();

        let report = archive_tasks_with_failures(
            root,
            &root.join("specs"),
            false,
            FailureInjection {
                publish_index: Some(1),
                rollback_index: Some(0),
                ..FailureInjection::default()
            },
        );

        assert!(!report.is_complete());
        assert!(report.is_partial());
        assert!(!report.applied());
        assert_eq!(report.succeeded.len(), 1);
        assert!(report.rolled_back.is_empty());
        assert_eq!(report.failed.len(), 2);
        assert_eq!(report.failed[0].operation, ArchiveOperation::Publish);
        assert_eq!(report.failed[1].operation, ArchiveOperation::Rollback);
        assert_ne!(fs::read(first).unwrap(), first_before);
        assert_eq!(fs::read(second).unwrap(), second_before);
    }

    #[test]
    fn dry_run_and_apply_share_the_same_plan() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        let tasks_path =
            write_archive_fixture(root, "work", b"## Tasks\n\n- [x] done\n- [ ] open\n");
        let before = fs::read(&tasks_path).unwrap();

        let preview = archive_tasks(root, &root.join("specs"), true);
        assert!(preview.is_complete());
        assert!(!preview.applied());
        assert_eq!(fs::read(&tasks_path).unwrap(), before);

        let applied = archive_tasks(root, &root.join("specs"), false);
        assert!(applied.is_complete());
        assert!(applied.applied());
        assert_eq!(preview.planned, applied.planned);
        assert_eq!(applied.succeeded, applied.planned);
    }

    #[cfg(unix)]
    #[test]
    fn apply_preserves_original_unix_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        let tasks_path = write_archive_fixture(root, "work", b"## Tasks\n\n- [x] done\n");
        fs::set_permissions(&tasks_path, Permissions::from_mode(0o640)).unwrap();

        let report = archive_tasks(root, &root.join("specs"), false);

        assert!(report.applied());
        assert_eq!(
            fs::metadata(tasks_path).unwrap().permissions().mode() & 0o777,
            0o640
        );
    }
}
