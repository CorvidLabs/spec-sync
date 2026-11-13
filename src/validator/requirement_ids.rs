// Duplicate requirement IDs in a living `requirements.md`.
//
// Two branches that each append the next free ID can both choose the same one.
// A merge that keeps both blocks is allowed to finish; `check` and `change audit`
// then refuse and name the ID. Fenced examples are not IDs.

use std::collections::BTreeMap;

/// Requirement IDs that occur more than once in `text`, in ascending order.
///
/// Only a `### REQ-<module>-<number>` heading counts. The same ID inside a
/// fenced code block is an example, not a second requirement.
pub(crate) fn duplicate_requirement_ids(text: &str) -> Vec<(String, usize)> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut fence: Option<(char, usize)> = None;
    for line in text.lines() {
        if update_fence(line, &mut fence) || fence.is_some() {
            continue;
        }
        let Some(id) = requirement_heading_id(line.trim_end()) else {
            continue;
        };
        *counts.entry(id.to_string()).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .collect()
}

/// One error per duplicated ID. The message names the ID and the file.
pub(crate) fn duplicate_requirement_errors_in(text: &str, relative_path: &str) -> Vec<String> {
    duplicate_requirement_ids(text)
        .into_iter()
        .map(|(id, count)| {
            format!("requirement ID `{id}` appears {count} times in {relative_path}")
        })
        .collect()
}

/// Every duplicated requirement ID under the configured specs directory.
pub(crate) fn duplicate_requirement_id_errors(root: &Path) -> Vec<String> {
    let config = crate::config::load_config(root);
    let specs = root.join(&config.specs_dir);
    if !specs.is_dir() {
        return Vec::new();
    }
    let mut errors = Vec::new();
    for entry in WalkDir::new(&specs) {
        // A directory that cannot be traversed hides every requirements.md below it;
        // skip silently and an audit passes over work it never inspected (#873).
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                let path = error
                    .path()
                    .map(|path| path.strip_prefix(root).unwrap_or(path).to_path_buf());
                // t384's nit on #1046: walkdir's Display repeats the absolute path;
                // the OS error carries the real cause.
                let detail = error
                    .io_error()
                    .map(|e| e.to_string())
                    .unwrap_or_else(|| error.to_string());
                errors.push(format!(
                    "cannot traverse {} to check requirement IDs: {detail}",
                    path.map(|path| path.display().to_string().replace('\\', "/"))
                        .unwrap_or_else(|| specs.display().to_string())
                ));
                continue;
            }
        };
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.file_name() != "requirements.md" {
            continue;
        }
        let path = entry.path();
        // `check` validates no companion of an archived spec. A requirements file
        // whose directory holds specs that are all archived is history, not living
        // contract, so the audit skips it too. A file with no spec beside it is
        // still checked.
        if path.parent().is_some_and(only_archived_specs_in) {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) => {
                errors.push(format!(
                    "cannot read {relative} to check requirement IDs: {error}"
                ));
                continue;
            }
        };
        errors.extend(duplicate_requirement_errors_in(&text, &relative));
    }
    errors
}

/// Whether `dir` holds at least one `*.spec.md` and every one of them parses
/// with `status: archived`. A spec that cannot be read or parsed counts as live.
fn only_archived_specs_in(dir: &Path) -> bool {
    let Ok(entries) = fs::read_dir(dir) else {
        return false;
    };
    let mut seen = false;
    for entry in entries.filter_map(Result::ok) {
        let name = entry.file_name();
        if !name.to_string_lossy().ends_with(".spec.md") {
            continue;
        }
        let Ok(content) = fs::read_to_string(entry.path()) else {
            return false;
        };
        let archived = crate::parser::parse_frontmatter(&content).is_some_and(|parsed| {
            parsed.frontmatter.parsed_status() == Some(crate::types::SpecStatus::Archived)
        });
        if !archived {
            return false;
        }
        seen = true;
    }
    seen
}

pub(crate) fn requirement_heading_id(line: &str) -> Option<&str> {
    // An ATX heading may be indented by up to three spaces; four is indented code.
    let indent = line.chars().take_while(|ch| *ch == ' ').count();
    if indent > 3 {
        return None;
    }
    let rest = line[indent..].strip_prefix("### ")?.trim();
    let token = rest.split_whitespace().next().unwrap_or(rest);
    let token = token.trim_matches(|ch: char| {
        matches!(ch, ':' | ',' | '.' | ';' | ')' | '(' | ']' | '[')
    });
    is_requirement_id(token).then_some(token)
}

fn is_requirement_id(token: &str) -> bool {
    let mut parts = token.split('-');
    if parts.next() != Some("REQ") {
        return false;
    }
    match parts.next() {
        Some(module) if !module.is_empty() && module.chars().all(|ch| ch.is_ascii_alphanumeric()) => {}
        _ => return false,
    }
    let rest: Vec<&str> = parts.collect();
    let Some(last) = rest.last() else {
        return false;
    };
    rest.iter().all(|part| {
        !part.is_empty() && part.chars().all(|ch| ch.is_ascii_alphanumeric())
    }) && last.chars().all(|ch| ch.is_ascii_digit())
}

/// Updates the open-fence state for `line` and reports whether `line` is a
/// fence delimiter. CommonMark rules: at most three leading spaces (four is
/// indented code, not a fence), three or more backticks or tildes, and a
/// closing run of the same character at least as long with only spaces or
/// tabs after it. A backtick fence's info string holds no backtick.
fn update_fence(line: &str, fence: &mut Option<(char, usize)>) -> bool {
    let indent = line.chars().take_while(|ch| *ch == ' ').count();
    if indent > 3 {
        return false;
    }
    let rest = &line[indent..];
    let Some(marker) = rest.chars().next().filter(|ch| matches!(ch, '`' | '~')) else {
        return false;
    };
    let length = rest.chars().take_while(|ch| *ch == marker).count();
    if length < 3 {
        return false;
    }
    let after = &rest[length..];
    match *fence {
        Some((open, open_length)) => {
            if marker == open && length >= open_length && after.trim().is_empty() {
                *fence = None;
                true
            } else {
                false
            }
        }
        None if marker == '`' && after.contains('`') => false,
        None => {
            *fence = Some((marker, length));
            true
        }
    }
}

#[cfg(test)]
mod requirement_id_tests {
    use super::*;

    #[test]
    fn names_a_repeated_heading_and_ignores_a_fenced_example() {
        let text = "\
### REQ-auth-002

The system SHALL keep one session.

```
### REQ-auth-002
```

### REQ-auth-002

The system SHALL keep another session.
";
        let errors = duplicate_requirement_errors_in(text, "specs/auth/requirements.md");
        assert_eq!(
            errors,
            vec!["requirement ID `REQ-auth-002` appears 2 times in specs/auth/requirements.md"]
        );
    }

    #[test]
    fn a_single_heading_is_not_a_duplicate() {
        let text = "### REQ-auth-001\n\nThe system SHALL start.\n";
        assert!(duplicate_requirement_ids(text).is_empty());
    }

    #[test]
    fn an_indented_code_line_does_not_open_a_fence() {
        let text = "    ```\n\n### REQ-auth-002\n\nOne.\n\n### REQ-auth-002\n\nTwo.\n";
        let errors = duplicate_requirement_errors_in(text, "specs/auth/requirements.md");
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].contains("REQ-auth-002"));
    }

    #[test]
    fn a_fence_closes_only_on_a_bare_closing_run() {
        let text = "\
### REQ-auth-002

One.

```
```not-a-close
### REQ-auth-002
```
";
        let errors = duplicate_requirement_errors_in(text, "specs/auth/requirements.md");
        assert!(errors.is_empty(), "{errors:?}");
    }

    #[test]
    fn archived_module_duplicates_do_not_block_the_project_audit() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path();
        let duplicate = "### REQ-old-001\n\nOne.\n\n### REQ-old-001\n\nTwo.\n";
        for (module, status) in [("old", "archived"), ("live", "stable")] {
            let dir = root.join("specs").join(module);
            fs::create_dir_all(&dir).unwrap();
            fs::write(
                dir.join(format!("{module}.spec.md")),
                format!("---\nmodule: {module}\nversion: 1\nstatus: {status}\nfiles: []\n---\n\n# {module}\n"),
            )
            .unwrap();
            fs::write(dir.join("requirements.md"), duplicate.replace("old", module)).unwrap();
        }
        let errors = duplicate_requirement_id_errors(root);
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].contains("REQ-live-001"), "{errors:?}");
    }

    /// #877 (REQ-change-125): a legacy `{module}.req.md` companion is a requirements file too,
    /// as the hash cache and the merge driver already read it, so the project audit names an
    /// ID it repeats.
    #[test]
    fn a_legacy_req_md_companion_is_scanned_for_duplicate_ids() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path();
        let dir = root.join("specs/auth");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("auth.spec.md"),
            "---\nmodule: auth\nversion: 1\nstatus: stable\nfiles: []\n---\n\n# auth\n",
        )
        .unwrap();
        fs::write(
            dir.join("auth.req.md"),
            "### REQ-auth-001\n\nOne.\n\n### REQ-auth-001\n\nTwo.\n",
        )
        .unwrap();
        let errors = duplicate_requirement_id_errors(root);
        assert_eq!(
            errors,
            vec!["requirement ID `REQ-auth-001` appears 2 times in specs/auth/auth.req.md"]
        );
    }

    /// #873: an unreadable subdirectory under the specs tree is an audit error naming the
    /// path, never a silent skip.
    #[cfg(unix)]
    #[test]
    fn an_unreadable_specs_subdirectory_is_an_error_naming_the_path() {
        use std::os::unix::fs::PermissionsExt;
        // chmod 000 does not block root; skip where the bit can't bite (t384's nit on #1046).
        if unsafe { libc::geteuid() } == 0 {
            return;
        }
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path();
        let sealed = root.join("specs/locked");
        fs::create_dir_all(&sealed).unwrap();
        fs::write(
            sealed.join("requirements.md"),
            "### REQ-locked-001\n\nHidden.\n",
        )
        .unwrap();
        fs::set_permissions(&sealed, fs::Permissions::from_mode(0o000)).unwrap();
        let errors = duplicate_requirement_id_errors(root);
        fs::set_permissions(&sealed, fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(
            errors[0].contains("specs/locked"),
            "the error names the path: {errors:?}"
        );
    }

    #[test]
    fn indented_atx_requirement_headings_are_counted() {
        let text = "  ### REQ-auth-002\n\nOne.\n\n   ### REQ-auth-002\n\nTwo.\n";
        let errors = duplicate_requirement_errors_in(text, "specs/auth/requirements.md");
        assert_eq!(errors.len(), 1, "{errors:?}");
        let code = "    ### REQ-auth-002\n\n    ### REQ-auth-002\n";
        assert!(duplicate_requirement_errors_in(code, "x").is_empty());
    }
}
