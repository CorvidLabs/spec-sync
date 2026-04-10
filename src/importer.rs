//! External importers — generate spec files from GitHub Issues, Jira, and Confluence.
//!
//! Each importer fetches structured data from an external system and converts it
//! into a spec file with frontmatter, purpose, requirements, and placeholder sections.

use std::time::Duration;

/// An imported item from an external source, ready to become a spec.
#[derive(Debug, Clone)]
pub struct ImportedItem {
    /// Suggested module name (slug-cased).
    pub module_name: String,
    /// One-line purpose / summary.
    pub purpose: String,
    /// Requirements extracted from the source (acceptance criteria, description bullets).
    pub requirements: Vec<String>,
    /// Labels / tags from the source system.
    #[allow(dead_code)]
    pub labels: Vec<String>,
    /// Original URL for traceability.
    pub source_url: String,
    /// Issue number (for GitHub `implements` field).
    pub issue_number: Option<u64>,
    /// Source system identifier.
    pub source_type: ImportSource,
}

/// Supported import sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportSource {
    GitHub,
    Jira,
    Confluence,
}

impl std::fmt::Display for ImportSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportSource::GitHub => write!(f, "GitHub"),
            ImportSource::Jira => write!(f, "Jira"),
            ImportSource::Confluence => write!(f, "Confluence"),
        }
    }
}

/// Render an `ImportedItem` into a spec markdown string.
pub fn render_spec(item: &ImportedItem) -> String {
    let implements_field = match item.issue_number {
        Some(n) => format!("[{n}]"),
        None => "[]".to_string(),
    };

    let title = item
        .module_name
        .split('-')
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let requirements_section = if item.requirements.is_empty() {
        "- <!-- TODO: define requirements -->".to_string()
    } else {
        item.requirements
            .iter()
            .enumerate()
            .map(|(i, r)| format!("{}. {r}", i + 1))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        r#"---
module: {module}
version: 1
status: draft
files: []
db_tables: []
depends_on: []
implements: {implements}
---

# {title}

## Purpose

{purpose}

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|

### Exported Types

| Type | Description |
|------|-------------|

## Invariants

{requirements}

## Behavioral Examples

### Scenario: TODO

- **Given** precondition
- **When** action
- **Then** result

## Error Cases

| Condition | Behavior |
|-----------|----------|

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|

### Consumed By

| Module | What is used |
|--------|-------------|

## Change Log

| Date | Change |
|------|--------|
| {date} | Imported from {source}: {url} |
"#,
        module = item.module_name,
        implements = implements_field,
        title = title,
        purpose = item.purpose,
        requirements = requirements_section,
        date = today(),
        source = item.source_type,
        url = item.source_url,
    )
}

fn today() -> String {
    // Use a simple approach — read from system
    let output = std::process::Command::new("date")
        .args(["+%Y-%m-%d"])
        .output();
    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => "YYYY-MM-DD".to_string(),
    }
}

/// Slugify a title into a valid module name.
pub fn slugify(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

// ─── GitHub Issues Importer ────────────────────────────────────────────

/// Fetch a GitHub issue and convert it to an `ImportedItem`.
/// Uses `gh` CLI first, falls back to REST API with `GITHUB_TOKEN`.
pub fn import_github_issue(repo: &str, number: u64) -> Result<ImportedItem, String> {
    if crate::github::gh_is_available() {
        import_github_issue_gh(repo, number)
    } else {
        import_github_issue_api(repo, number)
    }
}

fn import_github_issue_gh(repo: &str, number: u64) -> Result<ImportedItem, String> {
    let output = std::process::Command::new("gh")
        .args([
            "issue",
            "view",
            &number.to_string(),
            "--repo",
            repo,
            "--json",
            "number,title,body,labels,url",
        ])
        .output()
        .map_err(|e| format!("Failed to run gh: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("gh error for #{number}: {}", stderr.trim()));
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse gh output: {e}"))?;

    parse_github_json(&json, number)
}

fn import_github_issue_api(repo: &str, number: u64) -> Result<ImportedItem, String> {
    let token = std::env::var("GITHUB_TOKEN")
        .map_err(|_| "GITHUB_TOKEN not set and gh CLI not available".to_string())?;

    let url = format!("https://api.github.com/repos/{repo}/issues/{number}");

    let agent = ureq::Agent::new_with_config(
        ureq::config::Config::builder()
            .timeout_global(Some(Duration::from_secs(10)))
            .build(),
    );

    let mut response = agent
        .get(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "specsync")
        .call()
        .map_err(|e| format!("GitHub API request failed: {e}"))?;

    if response.status() == 404 {
        return Err(format!("Issue #{number} not found in {repo}"));
    }
    if response.status() != 200 {
        return Err(format!("GitHub API returned HTTP {}", response.status()));
    }

    let json: serde_json::Value = response
        .body_mut()
        .read_json()
        .map_err(|e| format!("Failed to parse GitHub API response: {e}"))?;

    parse_github_json(&json, number)
}

fn parse_github_json(json: &serde_json::Value, number: u64) -> Result<ImportedItem, String> {
    let title = json["title"].as_str().unwrap_or("").to_string();
    let body = json["body"].as_str().unwrap_or("");
    let url = json["url"]
        .as_str()
        .or_else(|| json["html_url"].as_str())
        .unwrap_or("")
        .to_string();

    let labels: Vec<String> = json["labels"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|l| l["name"].as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let requirements = extract_requirements(body);

    Ok(ImportedItem {
        module_name: slugify(&title),
        purpose: title.clone(),
        requirements,
        labels,
        source_url: url,
        issue_number: Some(number),
        source_type: ImportSource::GitHub,
    })
}

// ─── Jira Importer ─────────────────────────────────────────────────────

/// Fetch a Jira issue and convert it to an `ImportedItem`.
///
/// Requires:
/// - `JIRA_URL` env var (e.g., `https://mycompany.atlassian.net`)
/// - `JIRA_TOKEN` env var (API token or PAT)
/// - `JIRA_EMAIL` env var (for Atlassian Cloud basic auth)
pub fn import_jira_issue(issue_key: &str) -> Result<ImportedItem, String> {
    let base_url = std::env::var("JIRA_URL")
        .map_err(|_| "JIRA_URL environment variable not set".to_string())?;
    let token = std::env::var("JIRA_TOKEN")
        .map_err(|_| "JIRA_TOKEN environment variable not set".to_string())?;
    let email = std::env::var("JIRA_EMAIL").unwrap_or_default();

    let url = format!(
        "{}/rest/api/3/issue/{}",
        base_url.trim_end_matches('/'),
        issue_key
    );

    let agent = ureq::Agent::new_with_config(
        ureq::config::Config::builder()
            .timeout_global(Some(Duration::from_secs(15)))
            .build(),
    );

    let mut req = agent
        .get(&url)
        .header("Accept", "application/json")
        .header("User-Agent", "specsync");

    // Atlassian Cloud uses email:token basic auth; Server/DC uses Bearer token
    if !email.is_empty() {
        let credentials = format!("{email}:{token}");
        let encoded = base64_encode(&credentials);
        req = req.header("Authorization", &format!("Basic {encoded}"));
    } else {
        req = req.header("Authorization", &format!("Bearer {token}"));
    }

    let mut response = req
        .call()
        .map_err(|e| format!("Jira API request failed: {e}"))?;

    if response.status() == 404 {
        return Err(format!("Jira issue {issue_key} not found"));
    }
    if response.status() != 200 {
        return Err(format!("Jira API returned HTTP {}", response.status()));
    }

    let json: serde_json::Value = response
        .body_mut()
        .read_json()
        .map_err(|e| format!("Failed to parse Jira response: {e}"))?;

    parse_jira_json(&json, issue_key, &base_url)
}

fn parse_jira_json(
    json: &serde_json::Value,
    issue_key: &str,
    base_url: &str,
) -> Result<ImportedItem, String> {
    let fields = &json["fields"];
    let summary = fields["summary"].as_str().unwrap_or(issue_key).to_string();

    // Extract description — Jira v3 uses ADF (Atlassian Document Format)
    let description = extract_jira_description(fields);

    let labels: Vec<String> = fields["labels"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|l| l.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let requirements = extract_requirements(&description);

    let browse_url = format!("{}/browse/{issue_key}", base_url.trim_end_matches('/'));

    Ok(ImportedItem {
        module_name: slugify(&summary),
        purpose: summary,
        requirements,
        labels,
        source_url: browse_url,
        issue_number: None,
        source_type: ImportSource::Jira,
    })
}

/// Extract plain text from Jira's ADF (Atlassian Document Format) or plain string.
fn extract_jira_description(fields: &serde_json::Value) -> String {
    let desc = &fields["description"];

    // Plain text (Jira Server / older API)
    if let Some(s) = desc.as_str() {
        return s.to_string();
    }

    // ADF format (Jira Cloud v3) — walk the content tree for text nodes
    if desc.is_object() {
        let mut texts = Vec::new();
        extract_adf_text(desc, &mut texts);
        return texts.join("\n");
    }

    String::new()
}

fn extract_adf_text(node: &serde_json::Value, out: &mut Vec<String>) {
    if let Some(text) = node["text"].as_str() {
        out.push(text.to_string());
    }
    if let Some(content) = node["content"].as_array() {
        for child in content {
            extract_adf_text(child, out);
        }
    }
}

// ─── Confluence Importer ───────────────────────────────────────────────

/// Fetch a Confluence page and convert it to an `ImportedItem`.
///
/// Requires:
/// - `CONFLUENCE_URL` env var (e.g., `https://mycompany.atlassian.net/wiki`)
/// - `CONFLUENCE_TOKEN` env var (API token or PAT)
/// - `CONFLUENCE_EMAIL` env var (for Atlassian Cloud basic auth)
pub fn import_confluence_page(page_id: &str) -> Result<ImportedItem, String> {
    let base_url = std::env::var("CONFLUENCE_URL")
        .map_err(|_| "CONFLUENCE_URL environment variable not set".to_string())?;
    let token = std::env::var("CONFLUENCE_TOKEN")
        .map_err(|_| "CONFLUENCE_TOKEN environment variable not set".to_string())?;
    let email = std::env::var("CONFLUENCE_EMAIL").unwrap_or_default();

    let url = format!(
        "{}/rest/api/content/{}?expand=body.storage",
        base_url.trim_end_matches('/'),
        page_id
    );

    let agent = ureq::Agent::new_with_config(
        ureq::config::Config::builder()
            .timeout_global(Some(Duration::from_secs(15)))
            .build(),
    );

    let mut req = agent
        .get(&url)
        .header("Accept", "application/json")
        .header("User-Agent", "specsync");

    if !email.is_empty() {
        let credentials = format!("{email}:{token}");
        let encoded = base64_encode(&credentials);
        req = req.header("Authorization", &format!("Basic {encoded}"));
    } else {
        req = req.header("Authorization", &format!("Bearer {token}"));
    }

    let mut response = req
        .call()
        .map_err(|e| format!("Confluence API request failed: {e}"))?;

    if response.status() == 404 {
        return Err(format!("Confluence page {page_id} not found"));
    }
    if response.status() != 200 {
        return Err(format!(
            "Confluence API returned HTTP {}",
            response.status()
        ));
    }

    let json: serde_json::Value = response
        .body_mut()
        .read_json()
        .map_err(|e| format!("Failed to parse Confluence response: {e}"))?;

    parse_confluence_json(&json, page_id, &base_url)
}

fn parse_confluence_json(
    json: &serde_json::Value,
    page_id: &str,
    base_url: &str,
) -> Result<ImportedItem, String> {
    let title = json["title"].as_str().unwrap_or(page_id).to_string();

    // Extract body from storage format (HTML-like)
    let body_html = json["body"]["storage"]["value"].as_str().unwrap_or("");

    let plain_text = strip_html_tags(body_html);
    let requirements = extract_requirements(&plain_text);

    // Purpose: first non-empty line of body, or the title
    let purpose = plain_text
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or(&title)
        .trim()
        .to_string();

    let page_url = json["_links"]["base"]
        .as_str()
        .map(|base| {
            let webui = json["_links"]["webui"].as_str().unwrap_or("");
            format!("{base}{webui}")
        })
        .unwrap_or_else(|| format!("{}/pages/{page_id}", base_url.trim_end_matches('/')));

    Ok(ImportedItem {
        module_name: slugify(&title),
        purpose,
        requirements,
        labels: Vec::new(),
        source_url: page_url,
        issue_number: None,
        source_type: ImportSource::Confluence,
    })
}

/// Strip HTML tags to extract plain text (simple, no external dep).
fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                result.push(' ');
            }
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    // Collapse whitespace
    result
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

// ─── Shared Helpers ────────────────────────────────────────────────────

/// Extract requirement-like bullets from a body of text.
/// Looks for markdown checkboxes, numbered lists, "Acceptance Criteria" sections, etc.
pub fn extract_requirements_pub(body: &str) -> Vec<String> {
    extract_requirements(body)
}

fn extract_requirements(body: &str) -> Vec<String> {
    let mut reqs = Vec::new();
    let mut in_criteria_section = false;

    for line in body.lines() {
        let trimmed = line.trim();

        // Detect acceptance criteria section headers
        if trimmed.to_lowercase().contains("acceptance criteria")
            || trimmed.to_lowercase().contains("requirements")
            || trimmed.to_lowercase().contains("definition of done")
        {
            in_criteria_section = true;
            continue;
        }

        // Stop section on next header
        if in_criteria_section && trimmed.starts_with('#') {
            in_criteria_section = false;
            continue;
        }

        // Capture checkboxes anywhere
        if trimmed.starts_with("- [") || trimmed.starts_with("* [") {
            let cleaned = trimmed
                .trim_start_matches("- [ ] ")
                .trim_start_matches("- [x] ")
                .trim_start_matches("- [X] ")
                .trim_start_matches("* [ ] ")
                .trim_start_matches("* [x] ")
                .trim_start_matches("* [X] ")
                .to_string();
            if !cleaned.is_empty() {
                reqs.push(cleaned);
            }
            continue;
        }

        // Capture bullets in criteria sections
        if in_criteria_section && (trimmed.starts_with("- ") || trimmed.starts_with("* ")) {
            let cleaned = trimmed
                .trim_start_matches("- ")
                .trim_start_matches("* ")
                .to_string();
            if !cleaned.is_empty() {
                reqs.push(cleaned);
            }
        }
    }

    reqs
}

/// Simple base64 encoding (no padding issues for auth headers).
fn base64_encode(input: &str) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = input.as_bytes();
    let mut result = String::new();
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── slugify ───────────────────────────────────────────────────────

    #[test]
    fn test_slugify_simple() {
        assert_eq!(slugify("Hello World"), "hello-world");
    }

    #[test]
    fn test_slugify_special_chars() {
        assert_eq!(slugify("feat: add user auth!"), "feat-add-user-auth");
        assert_eq!(slugify("Hello!!!World"), "hello-world");
    }

    #[test]
    fn test_slugify_already_slug() {
        assert_eq!(slugify("my-module"), "my-module");
    }

    #[test]
    fn test_slugify_mixed_case_spaces() {
        assert_eq!(slugify("  Multiple   Spaces  "), "multiple-spaces");
    }

    #[test]
    fn test_slugify_empty() {
        assert_eq!(slugify(""), "");
    }

    // ─── extract_requirements ──────────────────────────────────────────

    #[test]
    fn test_extract_requirements_checkboxes() {
        let body = "## Summary\nSome text\n- [ ] First task\n- [x] Done task\n- [ ] Third task";
        let reqs = extract_requirements(body);
        assert_eq!(reqs, vec!["First task", "Done task", "Third task"]);
    }

    #[test]
    fn test_extract_requirements_criteria_section() {
        let body = "## Description\nBlah\n## Acceptance Criteria\n- Must do X\n- Must do Y\n## Other\n- Not this";
        let reqs = extract_requirements(body);
        assert_eq!(reqs, vec!["Must do X", "Must do Y"]);
    }

    #[test]
    fn test_extract_requirements_empty_body() {
        assert!(extract_requirements("").is_empty());
        assert!(extract_requirements("Just some text\nNo bullets here").is_empty());
    }

    #[test]
    fn test_extract_requirements_definition_of_done() {
        let body = "## Definition of Done\n- Tests pass\n- Docs updated\n## Notes\n- Ignore me";
        let reqs = extract_requirements(body);
        assert_eq!(reqs, vec!["Tests pass", "Docs updated"]);
    }

    // ─── strip_html_tags ───────────────────────────────────────────────

    #[test]
    fn test_strip_html_simple() {
        assert_eq!(strip_html_tags("<p>Hello</p>"), "Hello");
    }

    #[test]
    fn test_strip_html_nested() {
        assert_eq!(
            strip_html_tags("<div><p>Line <strong>one</strong></p><p>Line two</p></div>"),
            "Line  one   Line two"
        );
    }

    #[test]
    fn test_strip_html_empty() {
        assert_eq!(strip_html_tags(""), "");
    }

    #[test]
    fn test_strip_html_no_tags() {
        assert_eq!(strip_html_tags("Just plain text"), "Just plain text");
    }

    // ─── render_spec ───────────────────────────────────────────────────

    #[test]
    fn test_render_spec_with_issue_number() {
        let item = ImportedItem {
            module_name: "user-auth".to_string(),
            purpose: "Add user authentication".to_string(),
            requirements: vec![
                "Must support OAuth".to_string(),
                "Must support email/password".to_string(),
            ],
            labels: vec!["enhancement".to_string()],
            source_url: "https://github.com/org/repo/issues/42".to_string(),
            issue_number: Some(42),
            source_type: ImportSource::GitHub,
        };
        let spec = render_spec(&item);
        assert!(spec.contains("module: user-auth"));
        assert!(spec.contains("implements: [42]"));
        assert!(spec.contains("# User Auth"));
        assert!(spec.contains("Add user authentication"));
        assert!(spec.contains("1. Must support OAuth"));
        assert!(spec.contains("2. Must support email/password"));
        assert!(spec.contains("Imported from GitHub"));
    }

    #[test]
    fn test_render_spec_without_issue_number() {
        let item = ImportedItem {
            module_name: "data-pipeline".to_string(),
            purpose: "Data pipeline overview".to_string(),
            requirements: Vec::new(),
            labels: Vec::new(),
            source_url: "https://company.atlassian.net/wiki/pages/123".to_string(),
            issue_number: None,
            source_type: ImportSource::Confluence,
        };
        let spec = render_spec(&item);
        assert!(spec.contains("module: data-pipeline"));
        assert!(spec.contains("implements: []"));
        assert!(spec.contains("<!-- TODO: define requirements -->"));
        assert!(spec.contains("Imported from Confluence"));
    }

    // ─── base64_encode ─────────────────────────────────────────────────

    #[test]
    fn test_base64_encode() {
        assert_eq!(base64_encode("user:token"), "dXNlcjp0b2tlbg==");
        assert_eq!(base64_encode("a"), "YQ==");
        assert_eq!(base64_encode("ab"), "YWI=");
        assert_eq!(base64_encode("abc"), "YWJj");
    }

    // ─── parse_github_json ─────────────────────────────────────────────

    #[test]
    fn test_parse_github_json_full() {
        let json = serde_json::json!({
            "title": "Add user registration",
            "body": "## Summary\nUsers need to register.\n- [ ] Email validation\n- [ ] Password hashing",
            "labels": [{"name": "enhancement"}, {"name": "auth"}],
            "html_url": "https://github.com/org/repo/issues/99"
        });
        let item = parse_github_json(&json, 99).unwrap();
        assert_eq!(item.module_name, "add-user-registration");
        assert_eq!(item.purpose, "Add user registration");
        assert_eq!(
            item.requirements,
            vec!["Email validation", "Password hashing"]
        );
        assert_eq!(item.labels, vec!["enhancement", "auth"]);
        assert_eq!(item.issue_number, Some(99));
    }

    #[test]
    fn test_parse_github_json_minimal() {
        let json = serde_json::json!({
            "title": "Fix bug",
            "body": "",
            "labels": [],
            "html_url": "https://github.com/org/repo/issues/1"
        });
        let item = parse_github_json(&json, 1).unwrap();
        assert_eq!(item.module_name, "fix-bug");
        assert!(item.requirements.is_empty());
    }

    // ─── parse_jira_json ───────────────────────────────────────────────

    #[test]
    fn test_parse_jira_json_plain_description() {
        let json = serde_json::json!({
            "fields": {
                "summary": "Implement SSO",
                "description": "## Acceptance Criteria\n- SAML support\n- OIDC support\n## Notes\n- Talk to security team",
                "labels": ["security", "auth"]
            }
        });
        let item = parse_jira_json(&json, "PROJ-123", "https://jira.example.com").unwrap();
        assert_eq!(item.module_name, "implement-sso");
        assert_eq!(item.purpose, "Implement SSO");
        assert_eq!(item.requirements, vec!["SAML support", "OIDC support"]);
        assert_eq!(item.labels, vec!["security", "auth"]);
        assert_eq!(item.source_url, "https://jira.example.com/browse/PROJ-123");
        assert_eq!(item.source_type, ImportSource::Jira);
    }

    #[test]
    fn test_parse_jira_json_adf_description() {
        let json = serde_json::json!({
            "fields": {
                "summary": "ADF Issue",
                "description": {
                    "type": "doc",
                    "content": [
                        {
                            "type": "paragraph",
                            "content": [
                                {"type": "text", "text": "First paragraph"}
                            ]
                        },
                        {
                            "type": "paragraph",
                            "content": [
                                {"type": "text", "text": "Second paragraph"}
                            ]
                        }
                    ]
                },
                "labels": []
            }
        });
        let item = parse_jira_json(&json, "PROJ-456", "https://jira.example.com").unwrap();
        assert_eq!(item.module_name, "adf-issue");
    }

    #[test]
    fn test_parse_jira_json_no_description() {
        let json = serde_json::json!({
            "fields": {
                "summary": "Quick fix",
                "description": null,
                "labels": []
            }
        });
        let item = parse_jira_json(&json, "BUG-1", "https://jira.example.com").unwrap();
        assert_eq!(item.module_name, "quick-fix");
        assert!(item.requirements.is_empty());
    }

    // ─── parse_confluence_json ─────────────────────────────────────────

    #[test]
    fn test_parse_confluence_json() {
        let json = serde_json::json!({
            "title": "API Design Document",
            "body": {
                "storage": {
                    "value": "<h1>Overview</h1><p>This is the API design.</p><h2>Requirements</h2><ul><li>RESTful</li><li>Versioned</li></ul>"
                }
            },
            "_links": {
                "base": "https://wiki.example.com",
                "webui": "/pages/viewpage.action?pageId=12345"
            }
        });
        let item = parse_confluence_json(&json, "12345", "https://wiki.example.com/wiki").unwrap();
        assert_eq!(item.module_name, "api-design-document");
        assert_eq!(item.source_type, ImportSource::Confluence);
        assert_eq!(
            item.source_url,
            "https://wiki.example.com/pages/viewpage.action?pageId=12345"
        );
    }

    #[test]
    fn test_parse_confluence_json_no_links() {
        let json = serde_json::json!({
            "title": "My Page",
            "body": {
                "storage": {
                    "value": "<p>Hello world</p>"
                }
            },
            "_links": {}
        });
        let item = parse_confluence_json(&json, "999", "https://wiki.example.com/wiki").unwrap();
        assert_eq!(item.source_url, "https://wiki.example.com/wiki/pages/999");
    }

    // ─── ImportSource display ──────────────────────────────────────────

    #[test]
    fn test_import_source_display() {
        assert_eq!(format!("{}", ImportSource::GitHub), "GitHub");
        assert_eq!(format!("{}", ImportSource::Jira), "Jira");
        assert_eq!(format!("{}", ImportSource::Confluence), "Confluence");
    }
}
