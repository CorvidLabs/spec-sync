use std::fs;
use std::path::Path;

use crate::parser::parse_frontmatter;

/// Sections visible to each role.
fn sections_for_role(role: &str) -> Option<Vec<&'static str>> {
    match role {
        "dev" => Some(vec![
            "Purpose",
            "Public API",
            "Invariants",
            "Dependencies",
            "Change Log",
        ]),
        "qa" => Some(vec!["Behavioral Examples", "Error Cases", "Invariants"]),
        "product" => Some(vec!["Purpose", "Change Log"]),
        "agent" => Some(vec![
            "Purpose",
            "Public API",
            "Invariants",
            "Behavioral Examples",
            "Error Cases",
        ]),
        _ => None,
    }
}

/// All supported role names.
pub fn valid_roles() -> &'static [&'static str] {
    &["dev", "qa", "product", "agent"]
}

/// Filter a spec file to show only sections relevant to a given role.
/// Returns the filtered markdown content.
pub fn view_spec(spec_path: &Path, role: &str) -> Result<String, String> {
    let allowed = sections_for_role(role).ok_or_else(|| {
        format!(
            "Unknown role '{}' — valid roles: {}",
            role,
            valid_roles().join(", ")
        )
    })?;

    let content = fs::read_to_string(spec_path)
        .map_err(|e| format!("Cannot read {}: {e}", spec_path.display()))?;

    let parsed =
        parse_frontmatter(&content).ok_or_else(|| "Cannot parse frontmatter".to_string())?;

    let fm = &parsed.frontmatter;
    let body = &parsed.body;

    let mut output = String::new();

    // Header with module name and role context
    if let Some(module) = &fm.module {
        output.push_str(&format!("# {} (view: {role})\n\n", module));
    }

    // Show status and agent_policy for agent role
    if role == "agent" {
        if let Some(status) = &fm.status {
            output.push_str(&format!("**Status:** {status}\n"));
        }
        if let Some(policy) = &fm.agent_policy {
            output.push_str(&format!("**Agent Policy:** {policy}\n"));
        } else {
            output.push_str("**Agent Policy:** not set (default: full-access)\n");
        }
        output.push('\n');
    }

    // For product role, also show requirements companion if it exists
    if role == "product"
        && let Some(parent) = spec_path.parent() {
            let req_path = parent.join("requirements.md");
            if req_path.exists()
                && let Ok(req_content) = fs::read_to_string(&req_path) {
                    // Strip frontmatter from requirements.md
                    let req_body = strip_frontmatter(&req_content);
                    if !req_body.trim().is_empty() {
                        output.push_str("## Requirements\n\n");
                        output.push_str(req_body.trim());
                        output.push_str("\n\n");
                    }
                }
        }

    // Split body into sections and filter
    let sections = split_sections(body);
    for (heading, content) in &sections {
        if allowed.iter().any(|a| heading.contains(a)) {
            output.push_str(&format!("## {heading}\n"));
            output.push_str(content);
            output.push('\n');
        }
    }

    Ok(output)
}

/// Split markdown body into (heading_text, section_content) pairs.
/// Only splits on `## ` level headings.
fn split_sections(body: &str) -> Vec<(String, String)> {
    let mut sections = Vec::new();
    let mut current_heading: Option<String> = None;
    let mut current_content = String::new();

    for line in body.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            // Flush previous section
            if let Some(h) = current_heading.take() {
                sections.push((h, current_content.clone()));
                current_content.clear();
            }
            current_heading = Some(heading.trim().to_string());
        } else if current_heading.is_some() {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    // Flush last section
    if let Some(h) = current_heading {
        sections.push((h, current_content));
    }

    sections
}

/// Strip YAML frontmatter from a markdown file.
fn strip_frontmatter(content: &str) -> &str {
    if let Some(stripped) = content.strip_prefix("---\n")
        && let Some(end) = stripped.find("\n---\n") {
            return &stripped[end + 5..]; // skip past closing ---\n
        }
    content
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sections_for_role() {
        assert!(sections_for_role("dev").unwrap().contains(&"Public API"));
        assert!(sections_for_role("qa").unwrap().contains(&"Error Cases"));
        assert!(sections_for_role("product").unwrap().contains(&"Purpose"));
        assert!(sections_for_role("agent").unwrap().contains(&"Invariants"));
        assert!(sections_for_role("unknown").is_none());
    }

    #[test]
    fn test_split_sections() {
        let body = "## Purpose\n\nDoes things.\n\n## Public API\n\n| Fn | Desc |\n\n## Change Log\n\n| Date | Change |\n";
        let sections = split_sections(body);
        assert_eq!(sections.len(), 3);
        assert_eq!(sections[0].0, "Purpose");
        assert_eq!(sections[1].0, "Public API");
        assert_eq!(sections[2].0, "Change Log");
    }

    #[test]
    fn test_strip_frontmatter() {
        let content = "---\nmodule: test\n---\n\n## Purpose\n";
        let result = strip_frontmatter(content);
        assert!(result.contains("## Purpose"));
    }
}
