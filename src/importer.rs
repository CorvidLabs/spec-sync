//! External importers — generate spec files from GitHub Issues, Jira, and Confluence.
//!
//! Each importer fetches structured data from an external system and converts it
//! into a spec file with frontmatter, purpose, requirements, and starter sections.

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
    /// Content preserved from a local markdown source, when the source is
    /// itself a spec-like document. External importers leave this `None`.
    pub preserved: Option<PreservedSource>,
}

/// Content preserved from a local markdown source file.
#[derive(Debug, Clone)]
pub struct PreservedSource {
    /// Frontmatter `files` list — the source files the imported spec covers.
    pub files: Vec<String>,
    /// Frontmatter `depends_on` list, kept verbatim.
    pub depends_on: Vec<String>,
    /// Frontmatter `db_tables` list, kept verbatim.
    pub db_tables: Vec<String>,
    /// Frontmatter `implements` issue numbers, kept verbatim.
    pub implements: Vec<u64>,
    /// Frontmatter `version`, kept verbatim when declared.
    pub version: Option<String>,
    /// Frontmatter `status` when declared and not `draft` — recorded in the
    /// import Change Log row; rendered specs are always `status: draft`.
    pub source_status: Option<String>,
    /// Existing `# ` title, kept verbatim when present.
    pub title: Option<String>,
    /// Text between the H1 and the first `## ` heading, kept verbatim.
    pub lead_in: Option<String>,
    /// Existing `## ` sections as `(title, body)`, in source order.
    pub sections: Vec<(String, String)>,
}

/// Supported import sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportSource {
    GitHub,
    Jira,
    Confluence,
    /// A local markdown file imported with `--from-dir`.
    LocalDir,
}

impl std::fmt::Display for ImportSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportSource::GitHub => write!(f, "GitHub"),
            ImportSource::Jira => write!(f, "Jira"),
            ImportSource::Confluence => write!(f, "Confluence"),
            ImportSource::LocalDir => write!(f, "local directory"),
        }
    }
}

/// Render an `ImportedItem` into a spec markdown string.
pub fn render_spec(item: &ImportedItem) -> String {
    match &item.preserved {
        Some(source) => render_spec_preserving(item, source),
        None => render_spec_skeleton(item),
    }
}

/// The canonical `##` sections of a rendered spec, in order, excluding the
/// trailing Change Log.
const CANONICAL_SECTIONS: &[&str] = &[
    "Purpose",
    "Public API",
    "Invariants",
    "Behavioral Examples",
    "Error Cases",
    "Dependencies",
];

/// Render an item whose source file was itself a spec-like markdown document:
/// reuse every section the source already has and scaffold only what is missing.
fn render_spec_preserving(item: &ImportedItem, source: &PreservedSource) -> String {
    let implements_field = if !source.implements.is_empty() {
        format!(
            "[{}]",
            source
                .implements
                .iter()
                .map(u64::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        )
    } else {
        match item.issue_number {
            Some(n) => format!("[{n}]"),
            None => "[]".to_string(),
        }
    };

    let yaml_list = |items: &[String]| -> String {
        if items.is_empty() {
            " []".to_string()
        } else {
            format!(
                "\n{}",
                items
                    .iter()
                    .map(|f| format!("  - {f}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        }
    };
    let files_field = yaml_list(&source.files);
    let depends_field = yaml_list(&source.depends_on);
    let db_tables_field = yaml_list(&source.db_tables);

    let version = source.version.as_deref().unwrap_or("1");
    let title = source
        .title
        .clone()
        .unwrap_or_else(|| skeleton_title(&item.module_name));

    let find = |name: &str| {
        source
            .sections
            .iter()
            .find(|(t, _)| t.eq_ignore_ascii_case(name))
            .map(|(_, body)| body.as_str())
    };

    let mut out = format!(
        "---\nmodule: {module}\nversion: {version}\nstatus: draft\nfiles:{files}\ndb_tables:{db_tables}\ndepends_on:{depends}\nimplements: {implements}\n---\n\n# {title}\n",
        module = item.module_name,
        files = files_field,
        db_tables = db_tables_field,
        depends = depends_field,
        implements = implements_field,
    );
    if let Some(lead_in) = &source.lead_in {
        let lead_in = lead_in.trim_matches('\n');
        if !lead_in.is_empty() {
            out.push_str(&format!("\n{lead_in}\n"));
        }
    }

    for &name in CANONICAL_SECTIONS {
        match find(name) {
            Some(body) => {
                out.push_str(&format!("\n## {name}\n{body}"));
            }
            None => {
                out.push_str(&default_section(name, item));
            }
        }
    }

    // Source sections outside the canonical set are kept, in source order,
    // ahead of the Change Log.
    for (section_title, body) in &source.sections {
        let canonical = CANONICAL_SECTIONS
            .iter()
            .any(|name| section_title.eq_ignore_ascii_case(name))
            || section_title.eq_ignore_ascii_case("Change Log");
        if !canonical {
            out.push_str(&format!("\n## {section_title}\n{body}"));
        }
    }

    let import_row = match &source.source_status {
        Some(status) => format!(
            "| {} | Imported from {}: {} (source status: {status}, re-drafted) |",
            today(),
            item.source_type,
            item.source_url
        ),
        None => format!(
            "| {} | Imported from {}: {} |",
            today(),
            item.source_type,
            item.source_url
        ),
    };
    match find("Change Log") {
        Some(body) => {
            let body = body.trim_end();
            out.push_str(&format!("\n## Change Log\n\n{body}\n{import_row}\n"));
        }
        None => {
            out.push_str(&format!(
                "\n## Change Log\n\n| Date | Change |\n|------|--------|\n{import_row}\n"
            ));
        }
    }

    out
}

/// Render the fixed skeleton used for external sources that carry no
/// preservable spec body.
fn render_spec_skeleton(item: &ImportedItem) -> String {
    let implements_field = match item.issue_number {
        Some(n) => format!("[{n}]"),
        None => "[]".to_string(),
    };

    let title = skeleton_title(&item.module_name);

    let requirements_section = if item.requirements.is_empty() {
        "- Define requirements from the imported source before promoting this spec.".to_string()
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

### Scenario: Imported behavior

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

/// Title-case a slug-cased module name for the skeleton H1.
fn skeleton_title(module_name: &str) -> String {
    module_name
        .split('-')
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The scaffold body for one canonical section the source did not carry,
/// matching what `render_spec_skeleton` writes for the same section.
fn default_section(name: &str, item: &ImportedItem) -> String {
    match name {
        "Purpose" => format!("\n## Purpose\n\n{}\n", item.purpose),
        "Public API" => "\n## Public API\n\n### Exported Functions\n\n| Function | Parameters | Returns | Description |\n|----------|-----------|---------|-------------|\n\n### Exported Types\n\n| Type | Description |\n|------|-------------|\n".to_string(),
        "Invariants" => {
            let requirements = if item.requirements.is_empty() {
                "- Define requirements from the imported source before promoting this spec."
                    .to_string()
            } else {
                item.requirements
                    .iter()
                    .enumerate()
                    .map(|(i, r)| format!("{}. {r}", i + 1))
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            format!("\n## Invariants\n\n{requirements}\n")
        }
        "Behavioral Examples" => "\n## Behavioral Examples\n\n### Scenario: Imported behavior\n\n- **Given** precondition\n- **When** action\n- **Then** result\n".to_string(),
        "Error Cases" => "\n## Error Cases\n\n| Condition | Behavior |\n|-----------|----------|\n".to_string(),
        "Dependencies" => "\n## Dependencies\n\n### Consumes\n\n| Module | What is used |\n|--------|-------------|\n\n### Consumed By\n\n| Module | What is used |\n|--------|-------------|\n".to_string(),
        _ => format!("\n## {name}\n"),
    }
}

/// Parse a local markdown file into an `ImportedItem`, preserving its
/// frontmatter declarations and existing sections.
///
/// A file whose frontmatter declares `module` and a non-empty `files` list
/// imports with those values intact; frontmatter-less fields fall back to
/// derivation (title from the first `# ` heading, purpose from the first
/// non-empty paragraph, module name from the filename). A parseable file with
/// no `files` list imports as a draft the command layer warns about; only
/// unreadable sources fail.
pub fn import_local_markdown(filename: &str, content: &str) -> Result<ImportedItem, String> {
    let parsed = crate::parser::parse_frontmatter(content);
    let frontmatter = parsed.as_ref().map(|p| &p.frontmatter);

    let module_name = match frontmatter.and_then(|f| f.module.clone()) {
        Some(module) => module,
        None => slugify(filename),
    };
    crate::commands::validate_module_name(&module_name)
        .map_err(|error| format!("Unsafe module name for `{filename}`: {error}"))?;

    let files = frontmatter.map(|f| f.files.clone()).unwrap_or_default();

    let body = crate::parser::strip_frontmatter(content);
    let title = body
        .lines()
        .find(|l| l.starts_with("# "))
        .map(|l| l.trim_start_matches("# ").trim().to_string());

    let (lead_in, sections) = parse_body_sections(body);

    let purpose = sections
        .iter()
        .find(|(t, _)| t.eq_ignore_ascii_case("Purpose"))
        .map(|(_, b)| b.trim().to_string())
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| {
            lead_in
                .as_deref()
                .and_then(first_prose_line)
                .or_else(|| title.clone())
                .unwrap_or_else(|| module_name.clone())
        });

    Ok(ImportedItem {
        module_name,
        purpose,
        requirements: extract_requirements_pub(content),
        labels: Vec::new(),
        source_url: filename.to_string(),
        issue_number: None,
        source_type: ImportSource::LocalDir,
        preserved: Some(PreservedSource {
            files,
            depends_on: frontmatter
                .map(|f| f.depends_on.clone())
                .unwrap_or_default(),
            db_tables: frontmatter.map(|f| f.db_tables.clone()).unwrap_or_default(),
            implements: frontmatter
                .map(|f| f.implements.clone())
                .unwrap_or_default(),
            version: frontmatter.and_then(|f| f.version.clone()),
            source_status: frontmatter
                .and_then(|f| f.status.clone())
                .filter(|status| !status.eq_ignore_ascii_case("draft")),
            title,
            lead_in,
            sections,
        }),
    })
}

/// Split a markdown body into the lead-in (text between the H1 and the first
/// `## ` heading, when any) and its `## ` sections as `(title, body)` pairs,
/// in order. A `## ` line inside a fenced code block is content, never a
/// heading.
fn parse_body_sections(body: &str) -> (Option<String>, Vec<(String, String)>) {
    let mut lead_in: Option<String> = None;
    let mut sections = Vec::new();
    let mut current: Option<(String, String)> = None;
    // CommonMark: a fence is a run of 3+ backticks or tildes; the closing
    // fence uses the same character and is at least as long as the opening.
    let mut fence: Option<(char, usize)> = None;
    for line in body.lines() {
        let trimmed = line.trim_start();
        let marker = trimmed
            .chars()
            .next()
            .filter(|c| *c == '`' || *c == '~')
            .and_then(|c| {
                let len = trimmed.chars().take_while(|ch| *ch == c).count();
                (len >= 3).then_some((c, len))
            });
        if let Some((ch, len)) = marker {
            match fence {
                Some((open_ch, open_len)) if open_ch == ch && len >= open_len => fence = None,
                None => fence = Some((ch, len)),
                _ => {}
            }
        }
        if fence.is_none()
            && let Some(heading) = line.strip_prefix("## ")
        {
            if let Some(done) = current.take() {
                sections.push(done);
            }
            current = Some((heading.trim().to_string(), String::new()));
            continue;
        }
        match current.as_mut() {
            Some((_, body_text)) => {
                body_text.push_str(line);
                body_text.push('\n');
            }
            None => {
                // The H1 is the title, never lead-in prose; blank lines before
                // it are not lead-in either, so they must not make the H1
                // count as content.
                if line.starts_with("# ") && lead_in.as_deref().is_none_or(|l| l.trim().is_empty())
                {
                    continue;
                }
                let lead = lead_in.get_or_insert_with(String::new);
                lead.push_str(line);
                lead.push('\n');
            }
        }
    }
    if let Some(done) = current {
        sections.push(done);
    }
    (lead_in, sections)
}

/// The first line of actual prose in a markdown fragment: skips blank lines,
/// headings, table rows, list items (bulleted, numbered, checkbox), and
/// fenced code including its contents. Purpose prose is never structure.
fn first_prose_line(text: &str) -> Option<String> {
    let mut fence: Option<(char, usize)> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        let marker = trimmed
            .chars()
            .next()
            .filter(|c| *c == '`' || *c == '~')
            .and_then(|c| {
                let len = trimmed.chars().take_while(|ch| *ch == c).count();
                (len >= 3).then_some((c, len))
            });
        if let Some((ch, len)) = marker {
            match fence {
                Some((open_ch, open_len)) if open_ch == ch && len >= open_len => fence = None,
                None => fence = Some((ch, len)),
                _ => {}
            }
            continue;
        }
        if fence.is_some() {
            continue;
        }
        let numbered = trimmed
            .find(|c: char| !c.is_ascii_digit())
            .is_some_and(|i| trimmed[..i].len() > 0 && trimmed[i..].starts_with(". "));
        let structural = trimmed.is_empty()
            || trimmed.starts_with('#')
            || trimmed.starts_with('|')
            || trimmed.starts_with("- ")
            || trimmed.starts_with("* ")
            || trimmed.starts_with("- [")
            || numbered;
        if !structural {
            return Some(trimmed.to_string());
        }
    }
    None
}

fn today() -> String {
    let output = std::process::Command::new("date")
        .args(["+%Y-%m-%d"])
        .output();
    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => "YYYY-MM-DD".to_string(),
    }
}

/// Redact a secret from an error message before surfacing it to the user.
///
/// Auth tokens are sent in request headers, so well-behaved HTTP errors should
/// never echo them back — but a misbehaving proxy, redirect, or future client
/// change could. This strips any verbatim occurrence of the token as
/// defense-in-depth before an importer error reaches logs or terminal output.
fn redact_secret(message: String, secret: &str) -> String {
    if !secret.is_empty() && message.contains(secret) {
        message.replace(secret, "[REDACTED]")
    } else {
        message
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

fn validated_slug(source: &str, title: &str) -> Result<String, String> {
    let module_name = slugify(title);
    crate::commands::validate_module_name(&module_name)
        .map_err(|error| format!("{source} title cannot produce a safe module name: {error}"))?;
    Ok(module_name)
}

// ─── GitHub Issues Importer ────────────────────────────────────────────

/// Fetch a GitHub issue and convert it to an `ImportedItem`.
/// Uses the shared in-process REST path with an explicit `GITHUB_TOKEN`.
pub fn import_github_issue(repo: &str, number: u64) -> Result<ImportedItem, String> {
    import_github_issue_with(repo, number, crate::github::fetch_issue_details)
}

fn import_github_issue_with(
    repo: &str,
    number: u64,
    fetch: impl FnOnce(&str, u64) -> Result<crate::github::GitHubIssueDetails, String>,
) -> Result<ImportedItem, String> {
    let details = fetch(repo, number)?;
    import_github_issue_details(details)
}

fn import_github_issue_details(
    details: crate::github::GitHubIssueDetails,
) -> Result<ImportedItem, String> {
    let requirements = extract_requirements(&details.body);
    let module_name = validated_slug(
        &format!("GitHub issue #{}", details.issue.number),
        &details.issue.title,
    )?;

    Ok(ImportedItem {
        module_name,
        purpose: details.issue.title,
        requirements,
        labels: details.issue.labels,
        source_url: details.issue.url,
        issue_number: Some(details.issue.number),
        source_type: ImportSource::GitHub,
        preserved: None,
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
        .map_err(|e| redact_secret(format!("Jira API request failed: {e}"), &token))?;

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
    let module_name = validated_slug(&format!("Jira issue {issue_key}"), &summary)?;

    Ok(ImportedItem {
        module_name,
        purpose: summary,
        requirements,
        labels,
        source_url: browse_url,
        issue_number: None,
        source_type: ImportSource::Jira,
        preserved: None,
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
        .map_err(|e| redact_secret(format!("Confluence API request failed: {e}"), &token))?;

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
    let module_name = validated_slug(&format!("Confluence page {page_id}"), &title)?;

    Ok(ImportedItem {
        module_name,
        purpose,
        requirements,
        labels: Vec::new(),
        source_url: page_url,
        issue_number: None,
        source_type: ImportSource::Confluence,
        preserved: None,
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

pub fn extract_requirements_pub(body: &str) -> Vec<String> {
    extract_requirements(body)
}

pub(crate) fn extract_requirements(body: &str) -> Vec<String> {
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
            preserved: None,
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
            preserved: None,
        };
        let spec = render_spec(&item);
        assert!(spec.contains("module: data-pipeline"));
        assert!(spec.contains("implements: []"));
        assert!(spec.contains("Define requirements from the imported source"));
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

    // ─── GitHub issue conversion ───────────────────────────────────────

    #[test]
    fn test_import_github_issue_details_full() {
        let details = crate::github::GitHubIssueDetails {
            issue: crate::github::GitHubIssue {
                number: 99,
                title: "Add user registration".to_string(),
                state: "open".to_string(),
                labels: vec!["enhancement".to_string(), "auth".to_string()],
                url: "https://github.com/org/repo/issues/99".to_string(),
            },
            body: "## Summary\nUsers need to register.\n- [ ] Email validation\n- [ ] Password hashing"
                .to_string(),
        };
        let item = import_github_issue_details(details).unwrap();
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
    fn test_import_github_issue_details_empty_body() {
        let details = crate::github::GitHubIssueDetails {
            issue: crate::github::GitHubIssue {
                number: 1,
                title: "Fix bug".to_string(),
                state: "open".to_string(),
                labels: Vec::new(),
                url: "https://github.com/org/repo/issues/1".to_string(),
            },
            body: String::new(),
        };
        let item = import_github_issue_details(details).unwrap();
        assert_eq!(item.module_name, "fix-bug");
        assert!(item.requirements.is_empty());
    }

    #[test]
    fn test_import_github_issue_rejects_title_without_a_safe_module_name() {
        let error = import_github_issue_with("org/repo", 7, |repo, number| {
            Ok(crate::github::GitHubIssueDetails {
                issue: crate::github::GitHubIssue {
                    number,
                    title: "!!! 🐦".to_string(),
                    state: "open".to_string(),
                    labels: Vec::new(),
                    url: format!("https://github.com/{repo}/issues/{number}"),
                },
                body: String::new(),
            })
        })
        .expect_err("punctuation-only titles must not produce an empty output path");

        assert!(error.contains("safe module name"), "{error}");
    }

    #[test]
    fn test_external_imports_reject_nonportable_slugs() {
        let github_details = crate::github::GitHubIssueDetails {
            issue: crate::github::GitHubIssue {
                number: 8,
                title: "CON".to_string(),
                state: "open".to_string(),
                labels: Vec::new(),
                url: "https://github.com/org/repo/issues/8".to_string(),
            },
            body: String::new(),
        };
        assert!(import_github_issue_details(github_details).is_err());

        let overlong_github_details = crate::github::GitHubIssueDetails {
            issue: crate::github::GitHubIssue {
                number: 9,
                title: "a".repeat(248),
                state: "open".to_string(),
                labels: Vec::new(),
                url: "https://github.com/org/repo/issues/9".to_string(),
            },
            body: String::new(),
        };
        assert!(import_github_issue_details(overlong_github_details).is_err());

        let jira = serde_json::json!({
            "fields": {
                "summary": "LPT9",
                "description": null,
                "labels": []
            }
        });
        assert!(parse_jira_json(&jira, "PROJ-9", "https://jira.example.com").is_err());

        let confluence = serde_json::json!({
            "title": "NUL",
            "body": {"storage": {"value": "<p>Unsafe title</p>"}},
            "_links": {}
        });
        assert!(parse_confluence_json(&confluence, "10", "https://wiki.example.com/wiki").is_err());
    }

    #[test]
    fn test_import_github_issue_entry_path_converts_shared_typed_details() {
        let item = import_github_issue_with("org/repo", 99, |repo, number| {
            assert_eq!(repo, "org/repo");
            assert_eq!(number, 99);
            Ok(crate::github::GitHubIssueDetails {
                issue: crate::github::GitHubIssue {
                    number,
                    title: "Add user registration".to_string(),
                    state: "open".to_string(),
                    labels: vec!["enhancement".to_string()],
                    url: format!("https://github.com/{repo}/issues/{number}"),
                },
                body: "- [ ] Validate email".to_string(),
            })
        })
        .expect("typed GitHub details must flow through the importer entry path");

        assert_eq!(item.module_name, "add-user-registration");
        assert_eq!(item.requirements, vec!["Validate email"]);
        assert_eq!(item.issue_number, Some(99));
    }

    #[test]
    fn test_import_github_issue_entry_path_returns_no_item_on_provider_failure() {
        let error = import_github_issue_with("org/repo", 99, |_repo, _number| {
            Err("GitHub repository access is inconclusive".to_string())
        })
        .expect_err("provider failures must abort before an imported item exists");

        assert_eq!(error, "GitHub repository access is inconclusive");
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
        assert_eq!(format!("{}", ImportSource::LocalDir), "local directory");
    }

    // ─── Local markdown import ─────────────────────────────────────────

    #[test]
    fn test_import_local_markdown_preserves_frontmatter_and_sections() {
        let content = "---\nmodule: widget\nversion: 3\nstatus: draft\nfiles:\n  - src/widget.rs\n---\n\n# Widget Spec\n\n## Purpose\n\nThe original purpose text.\n\n## Invariants\n\n1. Widgets are immutable.\n";
        let item = import_local_markdown("widget.spec", content).unwrap();
        assert_eq!(item.module_name, "widget");
        assert_eq!(item.purpose, "The original purpose text.");
        assert_eq!(item.source_type, ImportSource::LocalDir);

        let preserved = item.preserved.as_ref().unwrap();
        assert_eq!(preserved.files, vec!["src/widget.rs".to_string()]);
        assert_eq!(preserved.version.as_deref(), Some("3"));
        assert_eq!(preserved.title.as_deref(), Some("Widget Spec"));

        let spec = render_spec(&item);
        assert!(spec.contains("module: widget"));
        assert!(spec.contains("version: 3"));
        assert!(spec.contains("  - src/widget.rs"));
        assert!(spec.contains("The original purpose text."));
        assert!(spec.contains("1. Widgets are immutable."));
        assert!(!spec.contains("Confluence"));
        assert!(spec.contains("Imported from local directory: widget.spec"));
        // Missing canonical sections are scaffolded.
        assert!(spec.contains("## Public API"));
        assert!(spec.contains("## Error Cases"));
    }

    #[test]
    fn test_import_local_markdown_appends_to_existing_change_log() {
        let content = "---\nmodule: widget\nfiles:\n  - src/widget.rs\n---\n\n# Widget\n\n## Purpose\n\nWidget purpose.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Hand-written entry |\n";
        let item = import_local_markdown("widget", content).unwrap();
        let spec = render_spec(&item);
        assert!(spec.contains("Hand-written entry"));
        assert!(spec.contains("Imported from local directory"));
        let hand_written = spec.find("Hand-written entry").unwrap();
        let import_row = spec.find("Imported from local directory").unwrap();
        assert!(hand_written < import_row);
    }

    #[test]
    fn test_import_local_markdown_without_files_imports_as_warned_draft() {
        let content = "# Design Doc\n\nSome prose about the design.\n";
        let item = import_local_markdown("design", content).unwrap();
        assert_eq!(item.module_name, "design");
        let preserved = item.preserved.as_ref().unwrap();
        assert!(preserved.files.is_empty());
        let spec = render_spec(&item);
        assert!(spec.contains("status: draft"));
        assert!(spec.contains("files: []"));
        assert!(spec.contains("Some prose about the design."));
    }

    #[test]
    fn test_import_local_markdown_empty_files_list_is_not_an_error() {
        let content =
            "---\nmodule: widget\nfiles: []\n---\n\n# Widget\n\n## Purpose\n\nWidget purpose.\n";
        let item = import_local_markdown("widget", content).unwrap();
        assert!(item.preserved.as_ref().unwrap().files.is_empty());
    }

    #[test]
    fn test_import_local_markdown_fence_headings_stay_content() {
        let content = "---\nmodule: widget\nfiles:\n  - src/widget.rs\n---\n\n# Widget\n\n## Invariants\n\n1. Widgets are immutable.\n\n```text\n## Not a heading, inside a fence\n```\n\n## Purpose\n\nWidget purpose.\n";
        let item = import_local_markdown("widget", content).unwrap();
        let preserved = item.preserved.as_ref().unwrap();
        assert_eq!(preserved.sections.len(), 2);
        assert_eq!(preserved.sections[0].0, "Invariants");
        assert!(
            preserved.sections[0]
                .1
                .contains("## Not a heading, inside a fence")
        );
        let spec = render_spec(&item);
        assert!(spec.contains("## Not a heading, inside a fence"));
        // The fence content stays inside Invariants; no phantom section follows it.
        assert_eq!(spec.matches("## Not a heading").count(), 1);
    }

    #[test]
    fn test_import_local_markdown_without_h1_keeps_sections() {
        let content = "---\nmodule: noh1\nfiles:\n  - src/a.rs\n---\n\n## Purpose\n\nNo H1 purpose text must survive.\n\n## Invariants\n\n1. Keep it.\n";
        let item = import_local_markdown("noh1", content).unwrap();
        let spec = render_spec(&item);
        assert!(spec.contains("No H1 purpose text must survive."));
        assert!(spec.contains("1. Keep it."));
        assert!(!spec.contains("## Purpose\n\n## Purpose"));
    }

    #[test]
    fn test_import_local_markdown_purpose_fallback_never_takes_a_heading() {
        let content = "---\nmodule: apifirst\nfiles:\n  - src/api.rs\n---\n\n# Api First\n\n## Public API\n\n| Function | Description |\n|----------|-------------|\n| `call` | Calls |\n";
        let item = import_local_markdown("apifirst", content).unwrap();
        assert_eq!(item.purpose, "Api First");
        let spec = render_spec(&item);
        assert_eq!(spec.matches("## Public API").count(), 1);
        assert!(!spec.contains("## Purpose\n\n## Public API"));
    }

    #[test]
    fn test_import_local_markdown_long_fence_stays_content() {
        let content = "---\nmodule: widget\nfiles:\n  - src/widget.rs\n---\n\n# Widget\n\n## Invariants\n\n1. Immutable.\n\n````text\n```\n## Still content\n```\n````\n\n## Purpose\n\nWidget purpose.\n";
        let item = import_local_markdown("widget", content).unwrap();
        let preserved = item.preserved.as_ref().unwrap();
        assert_eq!(preserved.sections.len(), 2);
        assert!(preserved.sections[0].1.contains("## Still content"));
    }

    #[test]
    fn test_import_local_markdown_h1_appears_once_with_blank_line_after_frontmatter() {
        // The usual layout — blank line between frontmatter and the title —
        // must not duplicate the H1 into the lead-in.
        let content = "---\nmodule: loose\nfiles:\n  - src/b.rs\n---\n\n# Loose\n\nLead para.\n\n## Purpose\n\nLoose purpose.\n";
        let item = import_local_markdown("loose", content).unwrap();
        let spec = render_spec(&item);
        assert_eq!(spec.matches("# Loose\n").count(), 1, "{spec}");
        assert!(spec.contains("Lead para."));
        assert!(spec.contains("Loose purpose."));
    }

    #[test]
    fn test_import_local_markdown_purpose_fallback_skips_numbered_and_fenced() {
        let numbered = "---\nmodule: listy\nfiles:\n  - src/l.rs\n---\n\n# Listy\n\n## Invariants\n\n1. Keep it sorted.\n";
        let item = import_local_markdown("listy", numbered).unwrap();
        assert_eq!(item.purpose, "Listy");
        let spec = render_spec(&item);
        assert_eq!(spec.matches("1. Keep it sorted.").count(), 1, "{spec}");

        let fenced = "---\nmodule: codey\nfiles:\n  - src/c.rs\n---\n\n# Codey\n\n```rust\nfn secret_internal()\n```\n";
        let item = import_local_markdown("codey", fenced).unwrap();
        assert_eq!(item.purpose, "Codey");
        let spec = render_spec(&item);
        assert!(spec.contains("```rust\nfn secret_internal()\n```"));
        assert!(!spec.contains("## Purpose\n\nfn secret_internal()"));
    }

    #[test]
    fn test_import_local_markdown_preserves_lists_and_lead_in() {
        let content = "---\nmodule: widget\nstatus: active\nfiles:\n  - src/widget.rs\ndepends_on:\n  - base\ndb_tables:\n  - widgets\nimplements: [41, 42]\n---\n\n# Widget\n\nA lead-in paragraph about the widget.\n\n## Purpose\n\nWidget purpose.\n";
        let item = import_local_markdown("widget", content).unwrap();
        let spec = render_spec(&item);
        assert!(spec.contains("depends_on:\n  - base"));
        assert!(spec.contains("db_tables:\n  - widgets"));
        assert!(spec.contains("implements: [41, 42]"));
        assert!(spec.contains("A lead-in paragraph about the widget."));
        // Imports are always re-drafted; the source status is recorded instead.
        assert!(spec.contains("status: draft"));
        assert!(!spec.contains("\nstatus: active\n"));
        assert!(spec.contains("source status: active, re-drafted"));
    }

    #[test]
    fn test_import_local_markdown_keeps_unknown_sections() {
        let content = "---\nmodule: widget\nfiles:\n  - src/widget.rs\n---\n\n# Widget\n\n## Purpose\n\nWidget purpose.\n\n## Architecture Notes\n\nUses a double buffer.\n";
        let item = import_local_markdown("widget", content).unwrap();
        let spec = render_spec(&item);
        assert!(spec.contains("## Architecture Notes"));
        assert!(spec.contains("Uses a double buffer."));
    }
}
