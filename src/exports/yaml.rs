use regex::Regex;
use std::sync::LazyLock;

/// Top-level key: a line starting at column 0 with `key:` followed by space, newline, or EOF
static TOP_LEVEL_KEY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^([a-zA-Z_][a-zA-Z0-9_-]*):(?:\s|$)").unwrap());

/// YAML anchor: `&anchor-name`
static ANCHOR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"&([a-zA-Z_][a-zA-Z0-9_.-]*)").unwrap());

/// Well-known YAML keys whose children represent named entries worth extracting.
/// e.g., `jobs:` in GitHub Actions, `services:` in Docker Compose.
const NESTED_SYMBOL_PARENTS: &[&str] = &[
    "jobs",
    "services",
    "volumes",
    "networks",
    "secrets",
    "stages",
    "steps",
    "targets",
    "outputs",
    "inputs",
    "permissions",
    "deployments",
];

/// Extract symbols from YAML source content.
///
/// Symbols extracted:
/// - All top-level keys
/// - Named entries under well-known parent keys (e.g., `jobs.test`, `services.web`)
/// - YAML anchors (`&anchor-name`)
pub fn extract_exports(content: &str) -> Vec<String> {
    let mut symbols = Vec::new();

    // Collect top-level keys
    let top_level_keys: Vec<String> = TOP_LEVEL_KEY
        .captures_iter(content)
        .filter_map(|caps| caps.get(1).map(|m| m.as_str().to_string()))
        .collect();

    for key in &top_level_keys {
        symbols.push(key.clone());
    }

    // For well-known parent keys, extract second-level children as `parent.child`
    // We scan line-by-line, detecting the indent of the first child to only match
    // direct children (not deeper nested keys).
    let top_level_line = Regex::new(r"^([a-zA-Z_][a-zA-Z0-9_-]*):").unwrap();

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        // Check if this is a top-level key that's a well-known parent
        if let Some(caps) = top_level_line.captures(line)
            && let Some(key_match) = caps.get(1)
        {
            let parent = key_match.as_str();
            if NESTED_SYMBOL_PARENTS.contains(&parent) {
                // Scan subsequent lines for second-level keys under this parent
                let mut j = i + 1;
                let mut child_indent: Option<usize> = None;
                while j < lines.len() {
                    let child_line = lines[j];
                    // Stop if we hit another top-level key or end of file
                    if !child_line.is_empty()
                        && !child_line.starts_with(' ')
                        && !child_line.starts_with('\t')
                        && !child_line.starts_with('#')
                    {
                        break;
                    }
                    // Measure leading whitespace
                    let indent = child_line.len() - child_line.trim_start().len();
                    let trimmed = child_line.trim_start();
                    if indent > 0 && !trimmed.is_empty() && !trimmed.starts_with('#') {
                        // Detect indent of first child
                        if child_indent.is_none() {
                            child_indent = Some(indent);
                        }
                        // Only match lines at the exact child indent level
                        if Some(indent) == child_indent
                            && let Some(child_caps) = top_level_line.captures(trimmed)
                            && let Some(child_match) = child_caps.get(1)
                        {
                            symbols.push(format!("{}.{}", parent, child_match.as_str()));
                        }
                    }
                    j += 1;
                }
            }
        }
        i += 1;
    }

    // Extract anchors
    for caps in ANCHOR.captures_iter(content) {
        if let Some(anchor) = caps.get(1) {
            let name = anchor.as_str().to_string();
            if !symbols.contains(&name) {
                symbols.push(name);
            }
        }
    }

    symbols
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_actions_workflow() {
        let content = r#"name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps: []
  lint:
    runs-on: ubuntu-latest
    steps: []
  build:
    runs-on: ubuntu-latest
    steps: []
"#;
        let symbols = extract_exports(content);
        assert!(symbols.contains(&"name".to_string()));
        assert!(symbols.contains(&"on".to_string()));
        assert!(symbols.contains(&"jobs".to_string()));
        assert!(symbols.contains(&"jobs.test".to_string()));
        assert!(symbols.contains(&"jobs.lint".to_string()));
        assert!(symbols.contains(&"jobs.build".to_string()));
    }

    #[test]
    fn test_docker_compose() {
        let content = r#"version: "3.8"
services:
  web:
    image: nginx
    ports: ["80:80"]
  db:
    image: postgres
    environment: {}
volumes:
  pgdata:
    driver: local
"#;
        let symbols = extract_exports(content);
        assert!(symbols.contains(&"version".to_string()));
        assert!(symbols.contains(&"services".to_string()));
        assert!(symbols.contains(&"services.web".to_string()));
        assert!(symbols.contains(&"services.db".to_string()));
        assert!(symbols.contains(&"volumes".to_string()));
        assert!(symbols.contains(&"volumes.pgdata".to_string()));
    }

    #[test]
    fn test_anchors() {
        let content = r#"defaults: &defaults
  timeout: 30
  retries: 3
jobs:
  test:
    <<: *defaults
    command: test
"#;
        let symbols = extract_exports(content);
        assert!(symbols.contains(&"defaults".to_string()));
        assert!(symbols.contains(&"jobs".to_string()));
        assert!(symbols.contains(&"jobs.test".to_string()));
        assert!(symbols.contains(&"defaults".to_string())); // anchor name matches key
    }

    #[test]
    fn test_top_level_only() {
        let content = r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-app
spec:
  replicas: 3
"#;
        let symbols = extract_exports(content);
        assert!(symbols.contains(&"apiVersion".to_string()));
        assert!(symbols.contains(&"kind".to_string()));
        assert!(symbols.contains(&"metadata".to_string()));
        assert!(symbols.contains(&"spec".to_string()));
        // metadata.name should NOT be extracted since metadata is not a well-known parent
        assert!(!symbols.contains(&"metadata.name".to_string()));
    }

    #[test]
    fn test_four_space_indentation() {
        let content = r#"name: CI
on: push
jobs:
    test:
        runs-on: ubuntu-latest
    build:
        runs-on: ubuntu-latest
services:
    web:
        image: nginx
"#;
        let symbols = extract_exports(content);
        assert!(symbols.contains(&"jobs.test".to_string()));
        assert!(symbols.contains(&"jobs.build".to_string()));
        assert!(symbols.contains(&"services.web".to_string()));
    }

    #[test]
    fn test_four_space_nested_not_extracted() {
        let content = r#"jobs:
    test:
        runs-on: ubuntu-latest
        steps:
            - name: checkout
"#;
        let symbols = extract_exports(content);
        assert!(symbols.contains(&"jobs.test".to_string()));
        // `runs-on` is deeper nesting, not a direct child of `jobs`
        assert!(!symbols.contains(&"jobs.runs-on".to_string()));
    }

    #[test]
    fn test_tab_indentation() {
        let content = "services:\n\tweb:\n\t\timage: nginx\n\tdb:\n\t\timage: postgres\n";
        let symbols = extract_exports(content);
        assert!(symbols.contains(&"services.web".to_string()));
        assert!(symbols.contains(&"services.db".to_string()));
    }

    #[test]
    fn test_comments_and_empty_lines() {
        let content = r#"# This is a comment
name: test

# Another comment
on:
  push:
    branches: [main]
"#;
        let symbols = extract_exports(content);
        assert!(symbols.contains(&"name".to_string()));
        assert!(symbols.contains(&"on".to_string()));
        assert_eq!(symbols.len(), 2);
    }
}
