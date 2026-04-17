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
        // List item: "  - value" (supports spaces or tabs for indentation)
        if let Some(stripped) = line.trim_start().strip_prefix("- ")
            && current_key.is_some()
        {
            current_list.push(strip_yaml_comment(stripped.trim()));
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

            let value = strip_yaml_comment(line[colon_pos + 1..].trim());

            if value.is_empty() || value == "[]" {
                current_key = Some(key.to_string());
                current_list.clear();
            } else {
                set_scalar(&mut fm, key, &value);
            }
            continue;
        }

        // Blank or comment line: flush
        let trimmed = line.trim();
        if (trimmed.is_empty() || trimmed.starts_with('#'))
            && let Some(prev_key) = current_key.take()
        {
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

/// Strip inline YAML comments from a value.
/// Handles: `value # comment` → `value`
/// Preserves: `value` (no comment), quoted strings with `#` inside.
fn strip_yaml_comment(value: &str) -> String {
    // Don't strip from quoted strings or bracket arrays
    if value.starts_with('"') || value.starts_with('\'') || value.starts_with('[') {
        return value.to_string();
    }
    // Find ` # ` pattern (space-hash-space) which is a YAML comment
    if let Some(pos) = value.find(" #") {
        // Verify the # is followed by a space or is at end of string (YAML comment convention)
        let after = &value[pos + 2..];
        if after.is_empty() || after.starts_with(' ') {
            return value[..pos].trim_end().to_string();
        }
    }
    value.to_string()
}

fn set_scalar(fm: &mut Frontmatter, key: &str, value: &str) {
    match key {
        "module" => fm.module = Some(value.to_string()),
        "version" => fm.version = Some(value.to_string()),
        "status" => fm.status = Some(value.to_string()),
        "agent_policy" => fm.agent_policy = Some(value.to_string()),
        // Handle inline bracket arrays like `implements: [42, 57]`
        "implements" => fm.implements = parse_inline_issue_numbers(value),
        "tracks" => fm.tracks = parse_inline_issue_numbers(value),
        _ => {}
    }
}

/// Parse an inline bracket array of issue numbers: `[42, 57]` → vec![42, 57].
fn parse_inline_issue_numbers(value: &str) -> Vec<u64> {
    let s = value.trim();
    let inner = if s.starts_with('[') && s.ends_with(']') {
        &s[1..s.len() - 1]
    } else {
        s
    };
    inner
        .split(',')
        .filter_map(|v| v.trim().parse::<u64>().ok())
        .collect()
}

/// Parse a list of strings as u64 issue numbers, ignoring invalid entries.
fn parse_issue_numbers(values: &[String]) -> Vec<u64> {
    values
        .iter()
        .filter_map(|v| v.trim().parse::<u64>().ok())
        .collect()
}

fn set_field(fm: &mut Frontmatter, key: &str, values: &[String]) {
    match key {
        "files" => fm.files = values.to_vec(),
        "db_tables" => fm.db_tables = values.to_vec(),
        "depends_on" => fm.depends_on = values.to_vec(),
        "implements" => fm.implements = parse_issue_numbers(values),
        "tracks" => fm.tracks = parse_issue_numbers(values),
        "lifecycle_log" => fm.lifecycle_log = values.to_vec(),
        _ => {}
    }
}

/// Check if a ### header describes exported symbols (case-insensitive).
/// Matches headers containing "Exported", "Exports", "Export", or "Public" as keywords.
/// Examples that match:
///   "### Exported Functions", "### TypeScript Exports", "### Exports",
///   "### Public Types", "### Export Functions", "### Exported Symbols"
/// Examples that do NOT match:
///   "### API Endpoints", "### Component API", "### Configuration",
///   "### Internal Functions", "### Route Handlers"
pub fn is_export_header(header: &str) -> bool {
    let lower = header.to_ascii_lowercase();
    EXPORT_HEADER_RE.is_match(&lower)
}

/// Matches export-describing headers using word boundaries.
/// Catches "Exported", "Exports", "Export", "Public" but NOT "Unexported".
static EXPORT_HEADER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bexport(?:ed|s)?\b|\bpublic\b").unwrap());

static TABLE_ROW_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\|\s*`(\w+)`").unwrap());

static METHOD_HEADER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^####\s+.*(?:Methods|Constructor|Properties)").unwrap());

/// Extract symbol names from the spec's Public API section.
/// Only extracts the FIRST backtick-quoted word in each table row.
/// Skips class method sub-tables.
pub fn get_spec_symbols(body: &str) -> Vec<String> {
    let mut symbols = Vec::new();

    // Find the Public API section manually (no lookahead in Rust regex).
    // Use regex for exact line match — avoids false positives like "## Public API Overview".
    let api_start = match find_section_offset(body, "Public API") {
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

        // Allowlist: only validate tables under ### headers that describe exports.
        // Accepted patterns (case-insensitive):
        //   - "### Exported Functions", "### Exported Types" (contains "Exported")
        //   - "### TypeScript Exports", "### Exports" (contains "Exports")
        //   - "### Public Functions", "### Public Types" (contains "Public")
        //   - "### Exported Symbols", "### Export Types" (contains "Export")
        // Tables directly under ## Public API (no ### header) are also validated.
        // Everything else (### API Endpoints, ### Component API, ### Route Handlers,
        // ### Configuration, ### Internal Functions, etc.) is informational only.
        if header.starts_with("### ") && !is_export_header(header) {
            continue;
        }

        let mut in_method_subsection = false;

        for line in sub.lines() {
            // Skip #### sub-tables for class methods/constructors/properties
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
                && let Some(sym) = caps.get(1)
            {
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
        let pattern = format!(r"(?m)^## {escaped}\s*$");
        let re = Regex::new(&pattern).unwrap();
        if !re.is_match(body) {
            missing.push(section.clone());
        }
    }
    missing
}

// ─── Stub/Placeholder Detection ─────────────────────────────────────────

/// Common stub phrases that indicate a section has no real content.
const STUB_PHRASES: &[&str] = &[
    "tbd",
    "tbd.",
    "to be determined",
    "to be defined",
    "to be documented",
    "coming soon",
    "n/a",
    "n/a.",
    "not applicable",
    "todo",
    "todo.",
    "placeholder",
    "fill in",
    "add content",
    "describe here",
    "write here",
    "...",
    "\u{2026}", // ellipsis character
];

/// Check if a line is a stub/placeholder (case-insensitive).
fn is_stub_line(line: &str) -> bool {
    let t = line
        .trim()
        .trim_start_matches("- ")
        .trim_start_matches("* ")
        .trim_start_matches("> ");
    let lower = t.to_ascii_lowercase();
    STUB_PHRASES.contains(&lower.as_str())
}

/// Find the byte offset of an exact `## Section` heading line.
/// Anchored to start-of-line, tolerates trailing whitespace.
/// Returns `None` if not found.
pub fn find_section_offset(body: &str, section: &str) -> Option<usize> {
    let escaped = regex::escape(section);
    let pattern = format!(r"(?m)^## {escaped}\s*$");
    let re = Regex::new(&pattern).unwrap();
    re.find(body).map(|m| m.start())
}

/// Return true iff `body` contains an exact `## Section` heading line.
pub fn body_has_section(body: &str, section: &str) -> bool {
    find_section_offset(body, section).is_some()
}

/// Check if a specific section has meaningful (non-stub) content.
pub fn section_has_content(body: &str, section: &str) -> bool {
    let header = format!("## {section}");
    let start = match find_section_offset(body, section) {
        Some(s) => s,
        None => return false,
    };
    let after = start + header.len();
    let rest = &body[after..];
    let end = rest.find("\n## ").unwrap_or(rest.len());
    let section_body = rest[..end].trim();

    // Filter to meaningful lines (not empty, not HTML comments, not table separators)
    let content_lines: Vec<&str> = section_body
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty()
                && !t.starts_with("<!--")
                && !t.ends_with("-->")
                && !t.starts_with("|--")
                && !t.starts_with("| -")
                && !t.contains("<!-- TODO")
        })
        .collect();

    if content_lines.is_empty() {
        return false;
    }

    // If ALL content lines are stubs, section is not meaningful
    let non_stub_count = content_lines.iter().filter(|l| !is_stub_line(l)).count();

    // A table header + separator with no data rows is not meaningful content
    // Header rows have column names, separator rows have dashes (|---|---|)
    let table_lines: Vec<&&str> = content_lines
        .iter()
        .filter(|l| l.trim().starts_with('|'))
        .collect();
    let non_table_lines = content_lines.len() - table_lines.len();
    if non_table_lines == 0 && !table_lines.is_empty() {
        // All content is table rows — check if there are any data rows
        // (rows that aren't header separators like |---|---|)
        let data_rows = table_lines
            .iter()
            .filter(|l| {
                let t = l.trim().trim_start_matches('|').trim_end_matches('|');
                // A separator row contains only dashes, spaces, pipes, and colons
                !t.chars()
                    .all(|c| c == '-' || c == ' ' || c == '|' || c == ':')
            })
            .count();
        // Need at least a header row AND a data row (so > 1 non-separator rows)
        if data_rows <= 1 {
            return false;
        }
    }

    non_stub_count > 0
}

/// Find sections that exist but contain only stub/placeholder content.
pub fn find_stub_sections(body: &str, required_sections: &[String]) -> Vec<String> {
    let mut stubs = Vec::new();
    for section in required_sections {
        if body_has_section(body, section) && !section_has_content(body, section) {
            stubs.push(section.clone());
        }
    }
    stubs
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
    fn test_strip_yaml_comment() {
        assert_eq!(strip_yaml_comment("active"), "active");
        assert_eq!(strip_yaml_comment("active # this is the status"), "active");
        assert_eq!(
            strip_yaml_comment("value #no-space-means-not-comment"),
            "value #no-space-means-not-comment"
        );
        assert_eq!(
            strip_yaml_comment("[42, 57] # issue list"),
            "[42, 57] # issue list"
        ); // brackets preserved
        assert_eq!(
            strip_yaml_comment("\"quoted # value\""),
            "\"quoted # value\""
        ); // quotes preserved
        assert_eq!(strip_yaml_comment("value #"), "value");
    }

    #[test]
    fn test_parse_frontmatter_inline_comments() {
        let content = "---\nmodule: auth # the auth module\nversion: 1 # initial\nstatus: active # current status\nfiles:\n  - src/auth.ts # main file\n---\n\n# Auth\n";
        let parsed = parse_frontmatter(content).unwrap();
        assert_eq!(parsed.frontmatter.module.as_deref(), Some("auth"));
        assert_eq!(parsed.frontmatter.version.as_deref(), Some("1"));
        assert_eq!(parsed.frontmatter.status.as_deref(), Some("active"));
        assert_eq!(parsed.frontmatter.files, vec!["src/auth.ts"]);
    }

    #[test]
    fn test_parse_frontmatter_tabs_and_whitespace() {
        // Tabs used for indentation instead of spaces
        let content = "---\nmodule: auth\nversion: 1\nstatus: active\nfiles:\n\t- src/auth.ts\n\t- src/auth.utils.ts\n---\n\n# Auth\n";
        let parsed = parse_frontmatter(content).unwrap();
        assert_eq!(
            parsed.frontmatter.files,
            vec!["src/auth.ts", "src/auth.utils.ts"]
        );
    }

    #[test]
    fn test_parse_frontmatter_trailing_spaces() {
        let content = "---\nmodule: auth   \nversion: 1  \nstatus: active  \nfiles:\n  - src/auth.ts   \n---\n\n# Auth\n";
        let parsed = parse_frontmatter(content).unwrap();
        assert_eq!(parsed.frontmatter.module.as_deref(), Some("auth"));
        assert_eq!(parsed.frontmatter.files, vec!["src/auth.ts"]);
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

    #[test]
    fn test_get_spec_symbols_skips_non_exported_subsections() {
        let body = r#"## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `authenticate` | token: string | User | Validates token |

### API Endpoints

| Endpoint | Method | Handler | Description |
|----------|--------|---------|-------------|
| `/login` | POST | `login` | Login route |
| `/logout` | POST | `logout` | Logout route |

### Component API

| Signal | Type | Description |
|--------|------|-------------|
| `activeTab` | string | Current tab |

### Route Handlers

| Handler | Description |
|---------|-------------|
| `registration_status` | Check registration |

### Exported Types

| Type | Description |
|------|-------------|
| `AuthConfig` | Config type |

### Configuration

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `timeout` | number | 30 | Request timeout |

### Internal Functions

| Function | Description |
|----------|-------------|
| `hashPassword` | Internal hashing |

## Invariants
"#;
        let symbols = get_spec_symbols(body);
        // Only symbols under "### Exported ..." subsections should be extracted
        assert_eq!(symbols, vec!["authenticate", "AuthConfig"]);
    }

    #[test]
    fn test_parse_frontmatter_implements_list() {
        let content = "---\nmodule: auth\nversion: 1\nstatus: active\nfiles:\n  - src/auth.ts\nimplements:\n  - 42\n  - 57\ntracks:\n  - 10\n---\n\n# Auth\n";
        let parsed = parse_frontmatter(content).unwrap();
        assert_eq!(parsed.frontmatter.implements, vec![42, 57]);
        assert_eq!(parsed.frontmatter.tracks, vec![10]);
    }

    #[test]
    fn test_parse_frontmatter_implements_inline() {
        let content = "---\nmodule: auth\nversion: 1\nstatus: active\nfiles:\n  - src/auth.ts\nimplements: [42, 57]\ntracks: [10]\n---\n\n# Auth\n";
        let parsed = parse_frontmatter(content).unwrap();
        assert_eq!(parsed.frontmatter.implements, vec![42, 57]);
        assert_eq!(parsed.frontmatter.tracks, vec![10]);
    }

    #[test]
    fn test_parse_frontmatter_empty_implements() {
        let content = "---\nmodule: auth\nversion: 1\nstatus: active\nfiles:\n  - src/auth.ts\nimplements: []\n---\n\n# Auth\n";
        let parsed = parse_frontmatter(content).unwrap();
        assert!(parsed.frontmatter.implements.is_empty());
        assert!(parsed.frontmatter.tracks.is_empty());
    }

    #[test]
    fn test_is_export_header() {
        // Should match
        assert!(is_export_header("### Exported Functions"));
        assert!(is_export_header("### Exported Types"));
        assert!(is_export_header("### TypeScript Exports"));
        assert!(is_export_header("### Exports"));
        assert!(is_export_header("### Public Functions"));
        assert!(is_export_header("### Public Types"));
        assert!(is_export_header("### Export Types"));
        assert!(is_export_header("### Exported Symbols"));
        assert!(is_export_header("### exported functions")); // case-insensitive

        // Should NOT match
        assert!(!is_export_header("### API Endpoints"));
        assert!(!is_export_header("### Component API"));
        assert!(!is_export_header("### Route Handlers"));
        assert!(!is_export_header("### Configuration"));
        assert!(!is_export_header("### Internal Functions"));
        // Should NOT match — "unexported" contains "exported" as a substring
        assert!(!is_export_header("### Unexported Functions"));
        assert!(!is_export_header("### Unexported Types"));
    }

    #[test]
    fn test_body_has_section_exact_match() {
        let body = "## Purpose\nSome text\n\n## PurposeFoo\nOther text\n";
        assert!(body_has_section(body, "Purpose"));
        assert!(body_has_section(body, "PurposeFoo"));
        // "Purpose" should NOT match "PurposeFoo" and vice versa
        assert!(!body_has_section(body, "PurposeF"));
    }

    #[test]
    fn test_body_has_section_trailing_whitespace() {
        // Header with trailing whitespace in body should still match
        let body = "## Purpose   \nSome text\n";
        assert!(body_has_section(body, "Purpose"));
    }

    #[test]
    fn test_get_missing_sections_no_false_positives() {
        // "## Purpose" should not satisfy a requirement for "## PurposeFoo"
        let body = "## Purpose\nContent\n\n## Design\nContent\n";
        let missing = get_missing_sections(body, &["PurposeFoo".to_string()]);
        assert_eq!(missing, vec!["PurposeFoo"]);
        // But "Purpose" itself is present
        let missing = get_missing_sections(body, &["Purpose".to_string()]);
        assert!(missing.is_empty());
    }

    #[test]
    fn test_section_has_content_no_false_positives() {
        // "## Purpose" should not be used to satisfy "## PurposeFoo"
        let body = "## Purpose\nReal content here.\n\n## PurposeFoo\n\n";
        assert!(section_has_content(body, "Purpose"));
        assert!(!section_has_content(body, "PurposeFoo"));
    }

    #[test]
    fn test_get_spec_symbols_accepts_header_variations() {
        let body = r#"## Public API

### TypeScript Exports

| Function | Description |
|----------|-------------|
| `createAuth` | Creates auth |
| `validateToken` | Validates |

### Public Types

| Type | Description |
|------|-------------|
| `AuthConfig` | Config type |

### API Endpoints

| Endpoint | Method |
|----------|--------|
| `/login` | POST |

## Invariants
"#;
        let symbols = get_spec_symbols(body);
        // Should extract from "TypeScript Exports" and "Public Types" but not "API Endpoints"
        assert_eq!(symbols, vec!["createAuth", "validateToken", "AuthConfig"]);
    }

    #[test]
    fn test_get_spec_symbols_top_level_table() {
        // Tables directly under ## Public API (no ### header) should be validated
        let body = r#"## Public API

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `helper` | input: string | string | Helps |

## Invariants
"#;
        let symbols = get_spec_symbols(body);
        assert_eq!(symbols, vec!["helper"]);
    }

    #[test]
    fn test_section_has_content_real() {
        let body = "## Purpose\nThis module handles authentication.\n\n## Invariants\n1. Tokens must be valid\n";
        assert!(section_has_content(body, "Purpose"));
        assert!(section_has_content(body, "Invariants"));
    }

    #[test]
    fn test_section_has_content_empty() {
        let body = "## Purpose\n\n## Invariants\n";
        assert!(!section_has_content(body, "Purpose"));
    }

    #[test]
    fn test_section_has_content_stub_tbd() {
        let body = "## Purpose\nTBD\n\n## Invariants\n- N/A\n";
        assert!(!section_has_content(body, "Purpose"));
        assert!(!section_has_content(body, "Invariants"));
    }

    #[test]
    fn test_section_has_content_stub_phrases() {
        let body =
            "## Purpose\nTo be determined\n\n## Error Cases\nComing soon\n\n## Dependencies\nTBD\n";
        assert!(!section_has_content(body, "Purpose"));
        assert!(!section_has_content(body, "Error Cases"));
        assert!(!section_has_content(body, "Dependencies"));
    }

    #[test]
    fn test_section_has_content_none_is_valid() {
        // "None." is legitimate content (e.g. "no dependencies")
        let body = "## Dependencies\nNone.\n";
        assert!(section_has_content(body, "Dependencies"));
    }

    #[test]
    fn test_section_has_content_table_header_only() {
        let body = "## Public API\n\n| Export | Description |\n|--------|-------------|\n\n## Invariants\n";
        assert!(!section_has_content(body, "Public API"));
    }

    #[test]
    fn test_section_has_content_table_with_data() {
        let body = "## Public API\n\n| Export | Description |\n|--------|-------------|\n| `foo` | Does things |\n\n## Invariants\n";
        assert!(section_has_content(body, "Public API"));
    }

    #[test]
    fn test_find_stub_sections() {
        let body = "## Purpose\nReal content here\n\n## Public API\nTBD\n\n## Invariants\nN/A\n\n## Error Cases\n| Condition | Behavior |\n|-----------|----------|\n| Bad input | Returns error |\n";
        let required = vec![
            "Purpose".to_string(),
            "Public API".to_string(),
            "Invariants".to_string(),
            "Error Cases".to_string(),
        ];
        let stubs = find_stub_sections(body, &required);
        assert_eq!(stubs, vec!["Public API", "Invariants"]);
    }

    #[test]
    fn test_find_stub_sections_none() {
        let body = "## Purpose\nReal content\n\n## Public API\n| Export | Desc |\n|--------|------|\n| `foo` | Bar |\n";
        let required = vec!["Purpose".to_string(), "Public API".to_string()];
        let stubs = find_stub_sections(body, &required);
        assert!(stubs.is_empty());
    }
}
