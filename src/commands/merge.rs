use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process;

use crate::config::load_config;
use crate::git_utils::git_cmd;
use crate::merge;
use crate::merge_driver;
use crate::types;

pub fn cmd_merge(root: &Path, dry_run: bool, all: bool, format: types::OutputFormat) {
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

    // Exit non-zero if any conflicts need manual resolution, or if the scan
    // could not run at all — an unperformed scan must never read as a pass.
    if results.iter().any(|r| {
        matches!(
            r.status,
            merge::MergeStatus::Manual | merge::MergeStatus::Unknown
        )
    }) {
        process::exit(1);
    }
}

/// Register the git merge driver and the attribute rules for this clone.
///
/// The driver command is the absolute path of this executable, so a later
/// upgrade should run `--install` again. Attribute lines are committed;
/// `git config` is local to the clone. An undefined driver leaves git's
/// ordinary merge in place, so a clone that has not installed the driver
/// still merges the way it did before.
pub fn cmd_install_merge_driver(root: &Path, format: types::OutputFormat) {
    match install_merge_driver(root) {
        Ok(summary) => match format {
            types::OutputFormat::Json => {
                println!(
                    "{}",
                    serde_json::json!({
                        "installed": true,
                        "driver": summary.driver,
                        "attributes": summary.attributes,
                    })
                );
            }
            _ => {
                println!("Installed SpecSync merge driver");
                println!("  {}", summary.driver);
                for line in &summary.attributes {
                    println!("  {line}");
                }
            }
        },
        Err(error) => {
            eprintln!("{} {error}", "error:".red().bold());
            process::exit(1);
        }
    }
}

struct MergeDriverInstall {
    driver: String,
    attributes: Vec<String>,
}

fn install_merge_driver(root: &Path) -> Result<MergeDriverInstall, String> {
    let specs_dir = load_config(root).specs_dir;
    let attributes = attribute_lines(&specs_dir);
    write_attribute_lines(root, &attributes)?;
    let driver = merge_driver_command()?;
    git_config(
        root,
        "merge.specsync.name",
        "SpecSync module contract merge",
    )?;
    git_config(root, "merge.specsync.driver", &driver)?;
    Ok(MergeDriverInstall { driver, attributes })
}

fn attribute_lines(specs_dir: &str) -> Vec<String> {
    let specs_dir = specs_dir.trim_matches('/').replace('\\', "/");
    vec![
        format!("{specs_dir}/**/*.spec.md merge=specsync"),
        format!("{specs_dir}/**/requirements.md merge=specsync"),
    ]
}

fn write_attribute_lines(root: &Path, lines: &[String]) -> Result<(), String> {
    let path = root.join(".gitattributes");
    if let Ok(meta) = fs::symlink_metadata(&path)
        && meta.file_type().is_symlink()
    {
        return Err(
            "refusing to update symlinked .gitattributes; add the merge=specsync rules manually"
                .into(),
        );
    }
    let mut content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(format!("failed to read .gitattributes: {error}")),
    };
    let mut changed = false;
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
        changed = true;
    }
    for line in lines {
        if content.lines().any(|existing| existing == line) {
            continue;
        }
        content.push_str(line);
        content.push('\n');
        changed = true;
    }
    if changed {
        fs::write(&path, content)
            .map_err(|error| format!("failed to write .gitattributes: {error}"))?;
    }
    Ok(())
}

fn merge_driver_command() -> Result<String, String> {
    let exe = std::env::current_exe()
        .map_err(|error| format!("cannot locate specsync to register the merge driver: {error}"))?;
    let exe = exe.to_string_lossy();
    let quoted = if exe.contains(' ') || exe.contains('"') {
        format!("\"{}\"", exe.replace('"', "\\\""))
    } else {
        exe.to_string()
    };
    Ok(format!("{quoted} merge-file %O %A %B %P"))
}

fn git_config(root: &Path, key: &str, value: &str) -> Result<(), String> {
    let output = git_cmd(root)
        .args(["config", key, value])
        .output()
        .map_err(|error| format!("git config {key} failed to start: {error}"))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format!("git config {key} failed: {}", stderr.trim()))
}

/// Git merge driver entry. Writes the merge into `current` (`%A`).
///
/// Exit 0 when the contract merged. Exit 1 when a conflict remains in that
/// file or a side could not be read. Conflict markers are left in `current`
/// so the unmerged path still shows both sides.
pub fn cmd_merge_file(ancestor: &Path, current: &Path, other: &Path, pathname: Option<&str>) {
    let read = |path: &Path, label: &str| -> Result<String, String> {
        fs::read_to_string(path)
            .map_err(|error| format!("cannot read merge {label} {}: {error}", path.display()))
    };
    let merged = match (
        read(ancestor, "ancestor"),
        read(current, "current"),
        read(other, "other"),
    ) {
        (Ok(ancestor), Ok(current_text), Ok(other)) => {
            merge_driver::merge_contract(&ancestor, &current_text, &other, pathname.unwrap_or(""))
        }
        (Err(error), _, _) | (_, Err(error), _) | (_, _, Err(error)) => {
            eprintln!("{} {error}", "error:".red().bold());
            process::exit(1);
        }
    };
    if let Err(error) = fs::write(current, &merged.content) {
        eprintln!(
            "{} cannot write merge result {}: {error}",
            "error:".red().bold(),
            current.display()
        );
        process::exit(1);
    }
    if !merged.clean {
        process::exit(1);
    }
}
