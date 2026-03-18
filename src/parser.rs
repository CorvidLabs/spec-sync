use crate::types::Frontmatter;
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

/// Parsed spec file: frontmatter + markdown body.
pub struct ParsedSpec {
    pub frontmatter: Frontmatter,
    pub body: String,
}

static FRONTMATTER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)^---\n(.*?)\n---\n(.*)$").unwrap());

/// Parse YAML frontmatter from a spec file.
/// Zero-dependency YAML: uses regex, no YAML parser needed.
pub fn parse_frontmatter(content: &str) -> Option<ParsedSpec> {
    let caps = FRONTMATTER_RE.captures(content)?;
    let yaml_block = caps.get(1)?.as_str();
    let body = caps.get(2)?.as_str().to_string();

    let mut fm = Frontmatter::default();
    let mut current_key: Option<String> = None;
    let mut current_list: Vec<String> = Vec::new();

    for line in yaml_block.lines() {
        // List item: "  - value"
        if let Some(stripped) = line.trim_start().strip_prefix("- ")
            && current_key.is_some() {
                current_list.push(stripped.trim().to_string());
                continue;
            }

        // Key-value: "key: value" or "key:"
        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim();
            if key.is_empty() || key.contains(' ') {
                continue;
            }

            // Flush previous list
            if let Some(prev_key) = current_key.take() {
                set_field(&mut fm, &prev_key, &current_list);
                current_list.clear();
            }

            let value = line[colon_pos + 1..].trim();

            if value.is_empty() || value == "[]" {
                current_key = Some(key.to_string());
                current_list.clear();
            } else {
                set_scalar(&mut fm, key, value);
            }
            continue;
        }

        // Blank or comment line: flush
        let trimmed = line.trim();
        if (trimmed.is_empty() || trimmed.starts_with('#'))
            && let Some(prev_key) = current_key.take() {
                set_field(&mut fm, &prev_key, &current_list);
                current_list.clear();
            }
    }

    // Flush trailing list
    if let Some(prev_key) = current_key.take() {
        set_field(&mut fm, &prev_key, &current_list);
    }

    Some(ParsedSpec {
        frontmatter: fm,
        body,
    })
}

fn set_scalar(fm: &mut Frontmatter, key: &str, value: &str) {
    match key {
        "module" => fm.module = Some(value.to_string()),
        "version" => fm.version = Some(value.to_string()),
        "status" => fm.status = Some(value.to_string()),
        _ => {}
    }
}

fn set_field(fm: &mut Frontmatter, key: &str, values: &[String]) {
    match key {
        "files" => fm.files = values.to_vec(),
        "db_tables" => fm.db_tables = values.to_vec(),
        "depends_on" => fm.depends_on = values.to_vec(),
        _ => {}
    }
}

static TABLE_ROW_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\|\s*`(\w+)`").unwrap());

static METHOD_HEADER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^####\s+.*(?:Methods|Constructor|Properties)").unwrap());

/// Extract symbol names from the spec's Public API section.
/// Only extracts the FIRST backtick-quoted word in each table row.
/// Skips class method sub-tables.
pub fn get_spec_symbols(body: &str) -> Vec<String> {
    let mut symbols = Vec::new();

    // Find the Public API section manually (no lookahead in Rust regex)
    let api_start = match body.find("## Public API") {
        Some(pos) => pos,
        None => return symbols,
    };
    // Skip the "## Public API" line itself
    let after_header = match body[api_start..].find('\n') {
        Some(pos) => api_start + pos + 1,
        None => return symbols,
    };
    // Find the next ## heading (but not ### or deeper)
    let api_section = {
        let rest = &body[after_header..];
        let heading_re = Regex::new(r"(?m)^## [^#]").unwrap();
        match heading_re.find(rest) {
            Some(m) => &rest[..m.start()],
            None => rest,
        }
    };

    let sub_re = Regex::new(r"(?m)(?:^|\n)(### )").unwrap();
    // Split by ### headers
    let sub_sections: Vec<&str> = {
        let mut sections = Vec::new();
        let mut last = 0;
        for m in sub_re.find_iter(api_section) {
            if m.start() > last {
                sections.push(&api_section[last..m.start()]);
            }
            last = m.start();
        }
        if last < api_section.len() {
            sections.push(&api_section[last..]);
        }
        sections
    };

    for sub in sub_sections {
        // Check header — skip leading blank lines from the split
        let header = sub
            .lines()
            .map(|l| l.trim())
            .find(|l| !l.is_empty())
            .unwrap_or("");
        if header.ends_with("Methods")
            || header.ends_with("Constructor")
            || header.ends_with("Properties")
        {
            continue;
        }

        let mut in_method_subsection = false;

        for line in sub.lines() {
            if METHOD_HEADER_RE.is_match(line) {
                in_method_subsection = true;
                continue;
            }
            if line.starts_with("### ") {
                in_method_subsection = false;
            }
            if in_method_subsection {
                continue;
            }

            if let Some(caps) = TABLE_ROW_RE.captures(line)
                && let Some(sym) = caps.get(1) {
                    symbols.push(sym.as_str().to_string());
                }
        }
    }

    // Deduplicate while preserving order
    let mut seen = HashSet::new();
    symbols.retain(|s| seen.insert(s.clone()));
    symbols
}

/// Check which required sections are missing from the spec body.
pub fn get_missing_sections(body: &str, required_sections: &[String]) -> Vec<String> {
    let mut missing = Vec::new();
    for section in required_sections {
        let escaped = regex::escape(section);
        let pattern = format!(r"(?m)^## {escaped}");
        let re = Regex::new(&pattern).unwrap();
        if !re.is_match(body) {
            missing.push(section.clone());
        }
    }
    missing
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter_basic() {
        let content = "---\nmodule: auth\nversion: 1\nstatus: active\nfiles:\n  - src/auth.ts\ndb_tables: []\ndepends_on: []\n---\n\n# Auth\n\n## Purpose\n";
        let parsed = parse_frontmatter(content).unwrap();
        assert_eq!(parsed.frontmatter.module.as_deref(), Some("auth"));
        assert_eq!(parsed.frontmatter.version.as_deref(), Some("1"));
        assert_eq!(parsed.frontmatter.status.as_deref(), Some("active"));
        assert_eq!(parsed.frontmatter.files, vec!["src/auth.ts"]);
        assert!(parsed.frontmatter.db_tables.is_empty());
    }

    #[test]
    fn test_parse_frontmatter_missing() {
        let content = "# No frontmatter here\n\nJust markdown.";
        assert!(parse_frontmatter(content).is_none());
    }

    #[test]
    fn test_get_missing_sections() {
        let body = "## Purpose\nSomething\n\n## Public API\nStuff\n";
        let required = vec![
            "Purpose".to_string(),
            "Public API".to_string(),
            "Invariants".to_string(),
        ];
        let missing = get_missing_sections(body, &required);
        assert_eq!(missing, vec!["Invariants"]);
    }

    #[test]
    fn test_get_spec_symbols() {
        let body = r#"## Purpose
Something

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `createAuth` | config: Config | Auth | Creates auth |
| `validateToken` | token: string | bool | Validates |

### Exported Types

| Type | Description |
|------|-------------|
| `AuthConfig` | Config type |

## Invariants
"#;
        let symbols = get_spec_symbols(body);
        assert_eq!(symbols, vec!["createAuth", "validateToken", "AuthConfig"]);
    }
}
