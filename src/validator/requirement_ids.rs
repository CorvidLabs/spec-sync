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
    let mut fence: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if let Some(open) = &fence {
            if trimmed.starts_with(open) {
                fence = None;
            }
            continue;
        }
        if let Some(marker) = fence_marker(trimmed) {
            fence = Some(marker);
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
    for entry in WalkDir::new(&specs).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.file_name() != "requirements.md" {
            continue;
        }
        let path = entry.path();
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

fn requirement_heading_id(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("### ")?.trim();
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

fn fence_marker(trimmed: &str) -> Option<String> {
    for ch in ['`', '~'] {
        let run = trimmed.chars().take_while(|c| *c == ch).count();
        if run >= 3 {
            return Some(ch.to_string().repeat(run));
        }
    }
    None
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
}
