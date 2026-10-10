//! Public API table symbols and byte-preserving fenced-example masking.

use super::find_section_offset;
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

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

/// Captures one complete, nonempty inline-code span occupying the first table
/// cell. Extractors own symbol spelling, so this intentionally does not impose
/// an identifier character allowlist. Requiring the closing backtick and cell
/// delimiter prevents prose and later-column code spans from becoming exports.
static TABLE_ROW_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[ \t]{0,3}\|\s*`([^`\r\n]+)`\s*\|").unwrap());

static METHOD_HEADER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^####\s+.*(?:Methods|Constructor|Properties)").unwrap());

/// Replace every line inside a fenced code block (and the fence lines) with
/// spaces of the same byte length, preserving newlines so both line numbers
/// and byte offsets still line up with the original.
///
/// Public API tables, `###` subsection splits, and `--fix` heading scans all
/// walk Markdown. A fenced example that happens to look like a table row or a
/// `### Exported Functions` heading is quoted text, not the contract —
/// counting those backticks as documented exports let a spec pass `--strict`
/// at 100% documented with an empty real table. `--fix` uses the blanked
/// offsets as indexes into the original, so the blanking must not shrink.
pub(crate) fn blank_fenced_code(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut open_fence: Option<(char, usize)> = None;
    for line in body.split_inclusive('\n') {
        let content = line.strip_suffix('\n').unwrap_or(line);
        let had_newline = line.ends_with('\n');
        let trimmed = content.trim_start();
        let marker = ['`', '~'].into_iter().find_map(|c| {
            let run = trimmed.chars().take_while(|ch| *ch == c).count();
            (run >= 3).then_some((c, run))
        });
        let blank_line = |out: &mut String| {
            out.extend(std::iter::repeat_n(' ', content.len()));
            if had_newline {
                out.push('\n');
            }
        };
        match (open_fence, marker) {
            (Some((open_char, open_len)), Some((c, len)))
                if c == open_char && len >= open_len && trimmed[len..].trim().is_empty() =>
            {
                open_fence = None;
                blank_line(&mut out);
                continue;
            }
            (Some(_), _) => {
                blank_line(&mut out);
                continue;
            }
            (None, Some(open)) => {
                open_fence = Some(open);
                blank_line(&mut out);
                continue;
            }
            (None, None) => {}
        }
        out.push_str(content);
        if had_newline {
            out.push('\n');
        }
    }
    debug_assert_eq!(out.len(), body.len());
    out
}

/// Extract symbol names from the spec's Public API section.
/// Only extracts the FIRST nonempty backtick-quoted symbol in each table row.
/// Skips class method sub-tables.
pub fn get_spec_symbols(body: &str) -> Vec<String> {
    let mut symbols = collect_spec_symbols(body);
    // Deduplicate while preserving order
    let mut seen = HashSet::new();
    symbols.retain(|s| seen.insert(s.clone()));
    symbols
}

/// Symbols listed more than once in the Public API tables — duplicate rows
/// are almost always a paste error and were previously deduplicated silently.
pub(crate) fn get_duplicate_spec_symbols(body: &str) -> Vec<String> {
    let symbols = collect_spec_symbols(body);
    let mut seen = HashSet::new();
    let mut reported = HashSet::new();
    let mut duplicates = Vec::new();
    for symbol in symbols {
        if !seen.insert(symbol.clone()) && reported.insert(symbol.clone()) {
            duplicates.push(symbol);
        }
    }
    duplicates
}

fn collect_spec_symbols(body: &str) -> Vec<String> {
    let mut symbols = Vec::new();
    let body = blank_fenced_code(body);

    // Find the Public API section manually (no lookahead in Rust regex).
    // Use regex for exact line match — avoids false positives like "## Public API Overview".
    let api_start = match find_section_offset(&body, "Public API") {
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

            if let Some(symbol) = api_table_row_symbol(line) {
                symbols.push(symbol.to_string());
            }
        }
    }

    symbols
}

/// Extract the first backtick-quoted symbol from EVERY table row in the
/// Public API section, including informational subsections that
/// `get_spec_symbols` skips (e.g. "### API Endpoints", "### Functions").
/// `check --fix` uses this to avoid appending a duplicate row for a symbol
/// that a human already documented under a non-export heading.
pub fn get_all_api_table_symbols(body: &str) -> Vec<String> {
    let mut symbols = Vec::new();
    let body = blank_fenced_code(body);

    let api_start = match find_section_offset(&body, "Public API") {
        Some(pos) => pos,
        None => return symbols,
    };
    let after_header = match body[api_start..].find('\n') {
        Some(pos) => api_start + pos + 1,
        None => return symbols,
    };
    let rest = &body[after_header..];
    let heading_re = Regex::new(r"(?m)^## [^#]").unwrap();
    let api_section = match heading_re.find(rest) {
        Some(m) => &rest[..m.start()],
        None => rest,
    };

    for line in api_section.lines() {
        if let Some(symbol) = api_table_row_symbol(line) {
            symbols.push(symbol.to_string());
        }
    }

    let mut seen = HashSet::new();
    symbols.retain(|s| seen.insert(s.clone()));
    symbols
}

/// Return the complete inline-code symbol from the first cell of a Markdown
/// table row. Leading or trailing whitespace inside the code span is rejected
/// because extractor symbols never contain it; internal spaces remain valid.
fn api_table_row_symbol(line: &str) -> Option<&str> {
    let symbol = TABLE_ROW_RE.captures(line)?.get(1)?.as_str();
    (!symbol.trim().is_empty() && symbol == symbol.trim()).then_some(symbol)
}

#[cfg(test)]
#[path = "tests/api_symbols.rs"]
mod tests;
