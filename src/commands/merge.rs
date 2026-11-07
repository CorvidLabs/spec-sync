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

/// Installs the attributes and the local driver config, or changes nothing.
///
/// Everything that can fail before a write (the driver path, the repository,
/// reading `.gitattributes`) is checked first. If `git config` then fails, the
/// previous `.gitattributes` bytes and config values are put back.
fn install_merge_driver(root: &Path) -> Result<MergeDriverInstall, String> {
    let specs_dir = load_config(root).specs_dir;
    let attributes = attribute_lines(&specs_dir);
    let driver = merge_driver_command()?;
    let inside = git_cmd(root)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map_err(|error| format!("git failed to start: {error}"))?;
    if !inside.status.success() || String::from_utf8_lossy(&inside.stdout).trim() != "true" {
        return Err(format!(
            "{} is not inside a Git work tree; the merge driver was not installed",
            root.display()
        ));
    }
    let attributes_path = root.join(".gitattributes");
    let previous_attributes = match fs::read(&attributes_path) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(format!("failed to read .gitattributes: {error}")),
    };
    let keys = ["merge.specsync.name", "merge.specsync.driver"];
    let previous_config: Vec<Option<String>> =
        keys.iter().map(|key| git_config_get(root, key)).collect();

    write_attribute_lines(root, &attributes)?;
    let configured = git_config(root, keys[0], "SpecSync module contract merge")
        .and_then(|()| git_config(root, keys[1], &driver));
    if let Err(error) = configured {
        let restored = match &previous_attributes {
            Some(bytes) => fs::write(&attributes_path, bytes),
            None => fs::remove_file(&attributes_path),
        };
        for (key, value) in keys.iter().zip(&previous_config) {
            let _ = match value {
                Some(value) => git_config(root, key, value),
                None => git_config_unset(root, key),
            };
        }
        return Err(match restored {
            Ok(()) => error,
            Err(restore) => format!("{error}; restoring .gitattributes also failed: {restore}"),
        });
    }
    Ok(MergeDriverInstall { driver, attributes })
}

fn git_config_get(root: &Path, key: &str) -> Option<String> {
    let output = git_cmd(root)
        .args(["config", "--local", "--get", key])
        .output()
        .ok()?;
    output.status.success().then(|| {
        String::from_utf8_lossy(&output.stdout)
            .trim_end_matches('\n')
            .to_string()
    })
}

fn git_config_unset(root: &Path, key: &str) -> Result<(), String> {
    let output = git_cmd(root)
        .args(["config", "--local", "--unset", key])
        .output()
        .map_err(|error| format!("git config --unset {key} failed to start: {error}"))?;
    // Exit 5 means the key was already absent.
    if output.status.success() || output.status.code() == Some(5) {
        Ok(())
    } else {
        Err(format!("git config --unset {key} failed"))
    }
}

fn attribute_lines(specs_dir: &str) -> Vec<String> {
    let mut specs_dir = specs_dir.replace('\\', "/");
    // `./specs` and `specs` name the same directory, but a `.gitattributes`
    // pattern starting with `./` never matches.
    while let Some(rest) = specs_dir.strip_prefix("./") {
        specs_dir = rest.trim_start_matches('/').to_string();
    }
    let specs_dir = specs_dir.trim_matches('/');
    let specs_dir = if specs_dir == "." { "" } else { specs_dir };
    let literal = escape_wildmatch(specs_dir);
    ["**/*.spec.md", "**/requirements.md"]
        .iter()
        .map(|tail| {
            let pattern = if specs_dir.is_empty() {
                tail.to_string()
            } else {
                format!("{literal}/{tail}")
            };
            format!("{} merge=specsync", attribute_pattern(&pattern))
        })
        .collect()
}

/// Escapes wildmatch metacharacters so a configured directory matches itself.
fn escape_wildmatch(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for ch in text.chars() {
        if matches!(ch, '*' | '?' | '[' | ']' | '\\') {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped
}

/// A `.gitattributes` pattern. One with whitespace, a quote, a backslash or a
/// control character is written in Git's C-style quoted form, so it stays one
/// pattern instead of splitting into several tokens.
fn attribute_pattern(pattern: &str) -> String {
    let needs_quotes = pattern
        .chars()
        .any(|ch| ch.is_whitespace() || ch == '"' || ch == '\\' || ch.is_control());
    if !needs_quotes {
        return pattern.to_string();
    }
    let mut quoted = String::from("\"");
    for ch in pattern.chars() {
        match ch {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            '\t' => quoted.push_str("\\t"),
            '\n' => quoted.push_str("\\n"),
            other => quoted.push(other),
        }
    }
    quoted.push('"');
    quoted
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
    Ok(format!(
        "{} merge-file %O %A %B %P",
        shell_quote(&exe.to_string_lossy())
    ))
}

/// Single-quotes `text` for the POSIX shell Git runs the driver through. Inside
/// single quotes nothing is expanded; an embedded `'` closes, escapes and reopens.
fn shell_quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', "'\\''"))
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

#[cfg(test)]
mod install_tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn shell_quote_keeps_metacharacters_literal() {
        let path = "/Users/O'Brien/$HOME/`id`/specsync";
        let output = Command::new("sh")
            .arg("-c")
            .arg(format!("printf %s {}", shell_quote(path)))
            .output()
            .unwrap();
        assert_eq!(String::from_utf8_lossy(&output.stdout), path);
    }

    #[test]
    fn attribute_lines_match_a_specs_dir_with_spaces_and_brackets() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path();
        let run = |args: &[&str]| {
            Command::new("git")
                .args(args)
                .current_dir(root)
                .output()
                .unwrap()
        };
        assert!(run(&["init", "-q"]).status.success());
        let specs_dir = "docs/product specs [v2]";
        fs::write(
            root.join(".gitattributes"),
            attribute_lines(specs_dir).join("\n") + "\n",
        )
        .unwrap();
        for path in [
            "docs/product specs [v2]/cli/cli.spec.md",
            "docs/product specs [v2]/cli/requirements.md",
        ] {
            let output = run(&["check-attr", "merge", "--", path]);
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.ends_with("merge: specsync\n"), "{path}: {stdout}");
        }
        let other = run(&[
            "check-attr",
            "merge",
            "--",
            "docs/product specs v/cli/cli.spec.md",
        ]);
        assert!(
            String::from_utf8_lossy(&other.stdout).ends_with("merge: unspecified\n"),
            "brackets matched as a character class"
        );
    }

    #[test]
    fn dot_prefixed_specs_dir_matches_the_contracts() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path();
        let run = |args: &[&str]| {
            Command::new("git")
                .args(args)
                .current_dir(root)
                .output()
                .unwrap()
        };
        assert!(run(&["init", "-q"]).status.success());
        fs::write(
            root.join(".gitattributes"),
            attribute_lines("./specs/").join("\n") + "\n",
        )
        .unwrap();
        let output = run(&["check-attr", "merge", "--", "specs/x/x.spec.md"]);
        assert!(String::from_utf8_lossy(&output.stdout).ends_with("merge: specsync\n"));
    }

    #[test]
    fn root_specs_dir_matches_contracts() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path();
        let run = |args: &[&str]| {
            Command::new("git")
                .args(args)
                .current_dir(root)
                .output()
                .unwrap()
        };
        assert!(run(&["init", "-q"]).status.success());
        for specs_dir in [".", "./"] {
            fs::write(
                root.join(".gitattributes"),
                attribute_lines(specs_dir).join("\n") + "\n",
            )
            .unwrap();
            for path in [
                "root.spec.md",
                "nested/root.spec.md",
                "requirements.md",
                "nested/requirements.md",
            ] {
                let output = run(&["check-attr", "merge", "--", path]);
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(
                    stdout.ends_with("merge: specsync\n"),
                    "specs_dir={specs_dir} path={path}: {stdout}"
                );
            }
        }
    }

    #[test]
    fn failed_install_leaves_gitattributes_unchanged() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join("specs")).unwrap();
        assert!(
            install_merge_driver(root).is_err(),
            "install outside a Git repository succeeded"
        );
        assert!(
            !root.join(".gitattributes").exists(),
            ".gitattributes was written"
        );

        fs::write(root.join(".gitattributes"), "*.txt text\n").unwrap();
        assert!(install_merge_driver(root).is_err());
        assert_eq!(
            fs::read_to_string(root.join(".gitattributes")).unwrap(),
            "*.txt text\n"
        );
    }

    #[cfg(unix)]
    #[test]
    fn config_failure_restores_gitattributes() {
        use std::os::unix::fs::PermissionsExt;
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path();
        assert!(
            Command::new("git")
                .args(["init", "-q"])
                .current_dir(root)
                .status()
                .unwrap()
                .success()
        );
        fs::create_dir_all(root.join("specs")).unwrap();
        fs::write(root.join(".gitattributes"), "*.txt text\n").unwrap();
        let git_dir = root.join(".git");
        let original = fs::metadata(&git_dir).unwrap().permissions();
        fs::set_permissions(&git_dir, fs::Permissions::from_mode(0o555)).unwrap();
        if fs::write(git_dir.join("probe"), "").is_ok() {
            // Running as a user that ignores directory permissions (root): this
            // setup cannot make `git config` fail, so there is nothing to observe.
            let _ = fs::remove_file(git_dir.join("probe"));
            fs::set_permissions(&git_dir, original).unwrap();
            return;
        }
        let result = install_merge_driver(root);
        fs::set_permissions(&git_dir, original).unwrap();
        assert!(result.is_err(), "install succeeded with an unwritable .git");
        assert_eq!(
            fs::read_to_string(root.join(".gitattributes")).unwrap(),
            "*.txt text\n"
        );
    }
}
