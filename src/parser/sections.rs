//! Required headings, near misses, and substantive section content.

use crate::util::levenshtein;
use regex::Regex;

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

/// For each required section that is missing an exact heading, check whether
/// a near-miss `## Heading` exists (edit distance ≤ 2, case-insensitive).
/// Returns `(required_name, actual_heading_in_body)` pairs.
pub fn get_near_miss_sections(body: &str, required_sections: &[String]) -> Vec<(String, String)> {
    let heading_re = Regex::new(r"(?m)^## (.+?)\s*$").unwrap();
    let headings: Vec<String> = heading_re
        .captures_iter(body)
        .map(|cap| cap.get(1).unwrap().as_str().to_string())
        .collect();

    let missing = get_missing_sections(body, required_sections);
    let mut near_misses = Vec::new();
    for section in &missing {
        let section_lower = section.to_ascii_lowercase();
        if let Some(nearest) = headings
            .iter()
            .map(|h| (h, levenshtein(&h.to_ascii_lowercase(), &section_lower)))
            .filter(|(_, d)| *d > 0 && *d <= 2)
            .min_by_key(|(_, d)| *d)
            .map(|(h, _)| h.clone())
        {
            near_misses.push((section.clone(), nearest));
        }
    }
    near_misses
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
    // Generator scaffold placeholder text (emitted by `new`/`generate`/`scaffold`/
    // `add-spec`/`import`). A section containing only these lines is unfinished
    // draft content, not real documentation — treat it as a stub so scoring and
    // validation don't grade the tool's own placeholders as complete.
    "document this module's responsibility, inputs, outputs, and ownership boundaries.",
    "document this package's responsibility, inputs, outputs, and ownership boundaries.",
    "define an invariant that must remain true for supported inputs.",
    "1. define an invariant that must remain true for supported inputs.",
    "### scenario: core behavior",
    "### scenario: imported behavior",
    "**given** precondition",
    "**when** action",
    "**then** result",
    "list runtime dependencies and the specific symbols, services, or data they provide.",
];

/// Stock sentences emitted verbatim by `specsync new` / `generate` scaffolds.
/// They read like prose but are pure template — a spec consisting of them is
/// an untouched scaffold, not documentation (#441).
const SCAFFOLD_BOILERPLATE_PREFIXES: &[&str] = &[
    "document this module's responsibility",
    "list runtime dependencies and the specific symbols",
    "define an invariant that must remain true",
    "**given** precondition",
    "**when** action",
    "**then** result",
];

/// Strip list markers (`- `, `* `, `> `, `1. `) from the start of a line.
fn strip_list_marker(mut s: &str) -> &str {
    s = s.trim();
    for marker in ["- ", "* ", "> "] {
        if let Some(rest) = s.strip_prefix(marker) {
            s = rest.trim_start();
        }
    }
    let digits = s.len() - s.trim_start_matches(|c: char| c.is_ascii_digit()).len();
    if digits > 0 && s[digits..].starts_with(". ") {
        s = s[digits + 2..].trim_start();
    }
    s
}

/// Check if a line is verbatim scaffold boilerplate (case-insensitive).
pub(crate) fn is_boilerplate_line(line: &str) -> bool {
    let lower = strip_list_marker(line).to_ascii_lowercase();
    SCAFFOLD_BOILERPLATE_PREFIXES
        .iter()
        .any(|p| lower.starts_with(p))
}

/// Check if a line is a stub/placeholder (case-insensitive).
fn is_stub_line(line: &str) -> bool {
    let t = strip_list_marker(line);
    let lower = t.to_ascii_lowercase();
    STUB_PHRASES.contains(&lower.as_str()) || is_boilerplate_line(line)
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
#[path = "tests/sections.rs"]
mod tests;
