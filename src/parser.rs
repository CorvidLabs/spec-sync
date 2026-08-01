use crate::types::Frontmatter;
use crate::util::levenshtein;
use regex::Regex;
use serde::Deserialize;
use serde::de::{IgnoredAny, MapAccess, SeqAccess, Visitor};
use std::collections::HashSet;
use std::fmt;
use std::sync::LazyLock;

/// Parsed spec file: frontmatter + markdown body.
pub struct ParsedSpec {
    pub frontmatter: Frontmatter,
    pub body: String,
    /// Hard frontmatter problems (duplicate keys, wrong shapes, unclosed
    /// brackets/quotes). The offending text is included in each message.
    pub errors: Vec<String>,
    /// Soft frontmatter problems (non-numeric versions, ignored garbage lines).
    pub warnings: Vec<String>,
}

static FRONTMATTER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)^---\n(.*?)\n---\n(.*)$").unwrap());

const INVALID_YAML_FRONTMATTER_ERROR: &str = "invalid YAML frontmatter";
const DUPLICATE_YAML_KEY_ERROR: &str = "duplicate YAML frontmatter key";
const DUPLICATE_IMPLEMENTS_ERROR: &str = "duplicate `implements` issue-reference field";
const DUPLICATE_TRACKS_ERROR: &str = "duplicate `tracks` issue-reference field";
const INVALID_IMPLEMENTS_SHAPE_ERROR: &str =
    "`implements` must be a list of unsigned issue numbers";
const INVALID_TRACKS_SHAPE_ERROR: &str = "`tracks` must be a list of unsigned issue numbers";
const INVALID_IMPLEMENTS_NUMBER_ERROR: &str =
    "`implements` contains an invalid unsigned issue number";
const INVALID_TRACKS_NUMBER_ERROR: &str = "`tracks` contains an invalid unsigned issue number";

#[derive(Default)]
struct CheckedIssueReferences {
    implements: Vec<u64>,
    tracks: Vec<u64>,
}

struct ImplementsIssueNumbers(Vec<u64>);
struct TracksIssueNumbers(Vec<u64>);

impl<'de> Deserialize<'de> for CheckedIssueReferences {
    fn deserialize<Deserializer>(deserializer: Deserializer) -> Result<Self, Deserializer::Error>
    where
        Deserializer: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(CheckedIssueReferencesVisitor)
    }
}

struct CheckedIssueReferencesVisitor;

impl<'de> Visitor<'de> for CheckedIssueReferencesVisitor {
    type Value = CheckedIssueReferences;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("YAML frontmatter mapping")
    }

    fn visit_map<Mapping>(self, mut mapping: Mapping) -> Result<Self::Value, Mapping::Error>
    where
        Mapping: MapAccess<'de>,
    {
        let mut references = CheckedIssueReferences::default();
        let mut implements_seen = false;
        let mut tracks_seen = false;

        while let Some(key) = mapping.next_key::<String>()? {
            match key.as_str() {
                "implements" => {
                    if implements_seen {
                        return Err(serde::de::Error::custom(DUPLICATE_IMPLEMENTS_ERROR));
                    }
                    implements_seen = true;
                    references.implements = mapping.next_value::<ImplementsIssueNumbers>()?.0;
                }
                "tracks" => {
                    if tracks_seen {
                        return Err(serde::de::Error::custom(DUPLICATE_TRACKS_ERROR));
                    }
                    tracks_seen = true;
                    references.tracks = mapping.next_value::<TracksIssueNumbers>()?.0;
                }
                _ => {
                    mapping.next_value::<IgnoredAny>()?;
                }
            }
        }

        Ok(references)
    }
}

impl<'de> Deserialize<'de> for ImplementsIssueNumbers {
    fn deserialize<Deserializer>(deserializer: Deserializer) -> Result<Self, Deserializer::Error>
    where
        Deserializer: serde::Deserializer<'de>,
    {
        deserialize_issue_number_list(
            deserializer,
            INVALID_IMPLEMENTS_SHAPE_ERROR,
            INVALID_IMPLEMENTS_NUMBER_ERROR,
        )
        .map(Self)
    }
}

impl<'de> Deserialize<'de> for TracksIssueNumbers {
    fn deserialize<Deserializer>(deserializer: Deserializer) -> Result<Self, Deserializer::Error>
    where
        Deserializer: serde::Deserializer<'de>,
    {
        deserialize_issue_number_list(
            deserializer,
            INVALID_TRACKS_SHAPE_ERROR,
            INVALID_TRACKS_NUMBER_ERROR,
        )
        .map(Self)
    }
}

fn deserialize_issue_number_list<'de, Deserializer>(
    deserializer: Deserializer,
    shape_error: &'static str,
    number_error: &'static str,
) -> Result<Vec<u64>, Deserializer::Error>
where
    Deserializer: serde::Deserializer<'de>,
{
    deserializer.deserialize_any(IssueNumberListVisitor {
        shape_error,
        number_error,
    })
}

struct IssueNumberListVisitor {
    shape_error: &'static str,
    number_error: &'static str,
}

impl<'de> Visitor<'de> for IssueNumberListVisitor {
    type Value = Vec<u64>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("list of positive unsigned issue numbers")
    }

    fn visit_bool<Error>(self, _value: bool) -> Result<Self::Value, Error>
    where
        Error: serde::de::Error,
    {
        Err(Error::custom(self.shape_error))
    }

    fn visit_i64<Error>(self, _value: i64) -> Result<Self::Value, Error>
    where
        Error: serde::de::Error,
    {
        Err(Error::custom(self.shape_error))
    }

    fn visit_u64<Error>(self, _value: u64) -> Result<Self::Value, Error>
    where
        Error: serde::de::Error,
    {
        Err(Error::custom(self.shape_error))
    }

    fn visit_f64<Error>(self, _value: f64) -> Result<Self::Value, Error>
    where
        Error: serde::de::Error,
    {
        Err(Error::custom(self.shape_error))
    }

    fn visit_str<Error>(self, _value: &str) -> Result<Self::Value, Error>
    where
        Error: serde::de::Error,
    {
        Err(Error::custom(self.shape_error))
    }

    fn visit_string<Error>(self, _value: String) -> Result<Self::Value, Error>
    where
        Error: serde::de::Error,
    {
        Err(Error::custom(self.shape_error))
    }

    fn visit_none<Error>(self) -> Result<Self::Value, Error>
    where
        Error: serde::de::Error,
    {
        Err(Error::custom(self.shape_error))
    }

    fn visit_unit<Error>(self) -> Result<Self::Value, Error>
    where
        Error: serde::de::Error,
    {
        Err(Error::custom(self.shape_error))
    }

    fn visit_map<Mapping>(self, _mapping: Mapping) -> Result<Self::Value, Mapping::Error>
    where
        Mapping: MapAccess<'de>,
    {
        Err(serde::de::Error::custom(self.shape_error))
    }

    fn visit_seq<Sequence>(self, mut sequence: Sequence) -> Result<Self::Value, Sequence::Error>
    where
        Sequence: SeqAccess<'de>,
    {
        let mut numbers = Vec::new();
        while let Some(number) = sequence
            .next_element::<u64>()
            .map_err(|_| serde::de::Error::custom(self.number_error))?
        {
            if number == 0 {
                return Err(serde::de::Error::custom(self.number_error));
            }
            numbers.push(number);
        }
        Ok(numbers)
    }
}

/// Parse and validate top-level GitHub issue references from YAML frontmatter.
///
/// Unknown extension fields remain valid YAML but are otherwise ignored. Errors are
/// intentionally stable and content-free so callers can safely expose them.
pub fn parse_checked_issue_references(content: &str) -> Result<(Vec<u64>, Vec<u64>), String> {
    let content = content.trim_start_matches('\u{feff}');
    let frontmatter = content
        .strip_prefix("---\n")
        .and_then(|remainder| remainder.split_once("\n---\n"))
        .or_else(|| {
            content
                .strip_prefix("---\r\n")
                .and_then(|remainder| remainder.split_once("\r\n---\r\n"))
        })
        .map(|(frontmatter, _)| frontmatter);
    let Some(frontmatter) = frontmatter else {
        return Err("missing or malformed YAML frontmatter".to_string());
    };

    let options = serde_saphyr::options! {
        duplicate_keys: serde_saphyr::DuplicateKeyPolicy::Error,
        with_snippet: false,
    };
    let frontmatter = format!("{frontmatter}\n");
    let references =
        serde_saphyr::from_str_with_options::<CheckedIssueReferences>(&frontmatter, options)
            .map_err(checked_issue_reference_error)?;

    Ok((references.implements, references.tracks))
}

fn checked_issue_reference_error(error: serde_saphyr::Error) -> String {
    match error.without_snippet() {
        serde_saphyr::Error::Message { msg, .. }
            if [
                DUPLICATE_IMPLEMENTS_ERROR,
                DUPLICATE_TRACKS_ERROR,
                INVALID_IMPLEMENTS_SHAPE_ERROR,
                INVALID_TRACKS_SHAPE_ERROR,
                INVALID_IMPLEMENTS_NUMBER_ERROR,
                INVALID_TRACKS_NUMBER_ERROR,
            ]
            .contains(&msg.as_str()) =>
        {
            msg.clone()
        }
        serde_saphyr::Error::DuplicateMappingKey { key, .. } => match key.as_deref() {
            Some("implements") => DUPLICATE_IMPLEMENTS_ERROR.to_string(),
            Some("tracks") => DUPLICATE_TRACKS_ERROR.to_string(),
            _ => DUPLICATE_YAML_KEY_ERROR.to_string(),
        },
        _ => INVALID_YAML_FRONTMATTER_ERROR.to_string(),
    }
}

/// Parse YAML frontmatter from a spec file.
/// Zero-dependency YAML: uses regex, no YAML parser needed.
pub fn parse_frontmatter(content: &str) -> Option<ParsedSpec> {
    // A leading UTF-8 BOM (U+FEFF) is a non-semantic encoding marker that some
    // editors prepend; left in place it sits before the opening `---`, so the
    // `^---` anchor fails and a perfectly valid spec is reported as having
    // "malformed frontmatter" (the delimiters are right there — the user just
    // can't see the invisible byte). Strip leading BOM(s) only (a U+FEFF anywhere
    // else is real content and is left untouched); this is lossless.
    let content = content.trim_start_matches('\u{feff}');
    let caps = FRONTMATTER_RE.captures(content)?;
    let yaml_block = caps.get(1)?.as_str();
    let body = caps.get(2)?.as_str().to_string();

    let mut fm = Frontmatter::default();
    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut current_key: Option<String> = None;
    let mut current_list: Vec<String> = Vec::new();
    let mut seen_keys: HashSet<String> = HashSet::new();

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

            // Duplicate keys are a validation bypass (a hidden second
            // `status: draft` silently disables all section/export checks).
            // Reject loudly with the offending line.
            if !seen_keys.insert(key.to_string()) {
                errors.push(format!(
                    "Frontmatter duplicate key `{key}` (offending line: `{}`) — remove the duplicate; the last value would silently win",
                    line.trim()
                ));
            }

            let value = strip_yaml_comment(line[colon_pos + 1..].trim());

            if value.is_empty() || value == "[]" {
                current_key = Some(key.to_string());
                current_list.clear();
            } else {
                set_scalar(
                    &mut fm,
                    key,
                    &value,
                    line.trim(),
                    &mut errors,
                    &mut warnings,
                );
            }
            continue;
        }

        // Blank or comment line: flush
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            if let Some(prev_key) = current_key.take() {
                set_field(&mut fm, &prev_key, &current_list);
                current_list.clear();
            }
        } else {
            // A non-empty, non-comment line that is neither `key: value` nor a
            // list item under an active key cannot be parsed — surface it
            // instead of silently dropping it.
            warnings.push(format!(
                "Ignoring malformed frontmatter line (expected `key: value`): `{trimmed}`"
            ));
        }
    }

    // Flush trailing list
    if let Some(prev_key) = current_key.take() {
        set_field(&mut fm, &prev_key, &current_list);
    }

    Some(ParsedSpec {
        frontmatter: fm,
        body,
        errors,
        warnings,
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

/// Fields that must hold a YAML list of strings.
const LIST_FIELDS: &[&str] = &["files", "db_tables", "depends_on"];

fn set_scalar(
    fm: &mut Frontmatter,
    key: &str,
    value: &str,
    offending_line: &str,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    // List fields given a non-list shape: a mapping (`depends_on: {a: b}`)
    // would otherwise parse to an empty list and silently validate nothing.
    if LIST_FIELDS.contains(&key) {
        if value.starts_with('{') {
            errors.push(format!(
                "Frontmatter field `{key}` must be a YAML list, got a mapping (offending line: `{offending_line}`)"
            ));
            return;
        }
        if value.starts_with('[') {
            // Flow-style list (`depends_on: [a, b]`) — the form `specsync new`
            // scaffolds. Parse it; an unclosed bracket is a hard error.
            match parse_flow_string_list(value) {
                Ok(items) => set_field(fm, key, &items),
                Err(message) => errors.push(message),
            }
            return;
        }
        // A bare scalar (`depends_on: auth`) is treated as a one-item list,
        // matching YAML's leniency, but flagged so typos are visible.
        warnings.push(format!(
            "Frontmatter field `{key}` should be a YAML list, got scalar `{value}` — treating it as a one-item list"
        ));
        set_field(fm, key, std::slice::from_ref(&value.to_string()));
        return;
    }
    match key {
        "module" => fm.module = Some(value.to_string()),
        "version" => {
            if !is_version_shaped(value) {
                warnings.push(format!(
                    "Frontmatter `version` should be a plain number, got `{value}`"
                ));
            }
            fm.version = Some(value.to_string());
        }
        "status" => fm.status = Some(value.to_string()),
        "agent_policy" => fm.agent_policy = Some(value.to_string()),
        // Handle inline bracket arrays like `implements: [42, 57]`
        "implements" => fm.implements = parse_inline_issue_numbers(value),
        "tracks" => fm.tracks = parse_inline_issue_numbers(value),
        // A scalar `depends_on: alpha` (or inline `depends_on: [a, b]`) used to
        // be silently DROPPED — the dependency edge vanished from validation
        // and graphing with no diagnostic. Normalize it into the list so the
        // edge is enforced, and warn so the typo is visible.
        "depends_on" => {
            fm.depends_on = parse_inline_string_list(value);
            if !value.trim_start().starts_with('[') {
                eprintln!(
                    "warning: scalar `depends_on: {value}` should be a YAML list (`depends_on: [{value}]`); treating it as a single dependency"
                );
            }
        }
        _ => {}
    }
}

fn is_version_shaped(value: &str) -> bool {
    let unquoted = value
        .strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .or_else(|| value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
        .unwrap_or(value);
    !unquoted.is_empty()
        && unquoted
            .split('.')
            .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
}

/// Parse a flow-style YAML string list: `[a, b, "c"]` → `vec![a, b, c]`.
/// Rejects unclosed brackets and unterminated quotes loudly.
fn parse_flow_string_list(value: &str) -> Result<Vec<String>, String> {
    let inner = value
        .strip_prefix('[')
        .and_then(|v| v.strip_suffix(']'))
        .ok_or_else(|| {
            format!("Frontmatter flow-style list is missing a closing `]`: `{value}`")
        })?;
    let mut items = Vec::new();
    for raw in inner.split(',') {
        let item = raw.trim();
        if item.is_empty() {
            continue;
        }
        let unquoted = if let Some(rest) = item.strip_prefix('"') {
            rest.strip_suffix('"').ok_or_else(|| {
                format!("Frontmatter list has an unterminated quoted string: `{item}`")
            })?
        } else if let Some(rest) = item.strip_prefix('\'') {
            rest.strip_suffix('\'').ok_or_else(|| {
                format!("Frontmatter list has an unterminated quoted string: `{item}`")
            })?
        } else {
            item
        };
        items.push(unquoted.to_string());
    }
    Ok(items)
}

fn parse_inline_string_list(value: &str) -> Vec<String> {
    let s = value.trim();
    let inner = if s.starts_with('[') && s.ends_with(']') {
        &s[1..s.len() - 1]
    } else {
        s
    };
    inner
        .split(',')
        .map(|v| v.trim().trim_matches('"').trim_matches('\'').to_string())
        .filter(|v| !v.is_empty())
        .collect()
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
        // Dedupe at parse time (order-preserving): duplicate `depends_on`
        // entries used to inflate edge counts and emit doubled mermaid edges.
        "depends_on" => {
            let mut seen = std::collections::HashSet::new();
            fm.depends_on = values
                .iter()
                .filter(|v| seen.insert((*v).clone()))
                .cloned()
                .collect();
        }
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

/// Captures one complete, nonempty inline-code span occupying the first table
/// cell. Extractors own symbol spelling, so this intentionally does not impose
/// an identifier character allowlist. Requiring the closing backtick and cell
/// delimiter prevents prose and later-column code spans from becoming exports.
static TABLE_ROW_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[ \t]{0,3}\|\s*`([^`\r\n]+)`\s*\|").unwrap());

static METHOD_HEADER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^####\s+.*(?:Methods|Constructor|Properties)").unwrap());

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

    let api_start = match find_section_offset(body, "Public API") {
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
    fn test_scalar_depends_on_normalized_to_list() {
        // Regression (#419): a scalar `depends_on: alpha` was silently dropped,
        // disabling the dependency edge with no diagnostic.
        let content = "---\nmodule: beta\nversion: 1\nstatus: active\nfiles: []\ndepends_on: alpha\n---\n\n# Beta\n";
        let parsed = parse_frontmatter(content).unwrap();
        assert_eq!(parsed.frontmatter.depends_on, vec!["alpha"]);
    }

    #[test]
    fn test_inline_bracket_depends_on_parsed() {
        let content = "---\nmodule: beta\ndepends_on: [alpha, gamma]\n---\n\n# Beta\n";
        let parsed = parse_frontmatter(content).unwrap();
        assert_eq!(parsed.frontmatter.depends_on, vec!["alpha", "gamma"]);
    }

    #[test]
    fn test_duplicate_depends_on_deduped() {
        // Regression (#419): duplicate entries inflated edge counts and doubled
        // mermaid edges.
        let content = "---\nmodule: beta\ndepends_on:\n  - alpha\n  - alpha\n  - gamma\n  - alpha\n---\n\n# Beta\n";
        let parsed = parse_frontmatter(content).unwrap();
        assert_eq!(parsed.frontmatter.depends_on, vec!["alpha", "gamma"]);
    }

    #[test]
    fn test_scaffold_boilerplate_detected() {
        // Regression (#441): an untouched `specsync new` scaffold must not
        // count as placeholder-free, meaningful content.
        assert!(is_boilerplate_line(
            "Document this module's responsibility, inputs, outputs, and ownership boundaries."
        ));
        assert!(is_boilerplate_line("- **Given** precondition"));
        assert!(is_boilerplate_line(
            "1. Define an invariant that must remain true for supported inputs."
        ));
        assert!(!is_boilerplate_line("Handles authentication tokens."));
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
    fn test_parse_frontmatter_leading_bom() {
        // A leading UTF-8 BOM must not break frontmatter parsing: the spec is valid,
        // the invisible byte just precedes the opening `---`.
        let content = "\u{feff}---\nmodule: auth\nversion: 1\nstatus: active\nfiles:\n  - src/auth.ts\ndb_tables: []\ndepends_on: []\n---\n\n# Auth\n\n## Purpose\n";
        let parsed = parse_frontmatter(content).expect("BOM-prefixed frontmatter should parse");
        assert_eq!(parsed.frontmatter.module.as_deref(), Some("auth"));
        assert_eq!(parsed.frontmatter.files, vec!["src/auth.ts"]);
        // The body is BOM-free (the strip happens before the split).
        assert!(
            parsed.body.starts_with("\n# Auth"),
            "body: {:?}",
            parsed.body
        );
        assert!(!parsed.body.contains('\u{feff}'));

        // Repeated leading BOMs are also tolerated (trim_start_matches).
        let doubled = format!("\u{feff}{content}");
        let parsed2 = parse_frontmatter(&doubled).expect("repeated-BOM frontmatter should parse");
        assert_eq!(parsed2.frontmatter.module.as_deref(), Some("auth"));
    }

    #[test]
    fn test_parse_frontmatter_bom_only_leading() {
        // Only a *leading* BOM is stripped; a U+FEFF elsewhere in the content is
        // real (zero-width no-break space) and must be preserved.
        let content =
            "---\nmodule: a\nfiles:\n  - src/a.ts\n---\n\n# A\n\nZero\u{feff}width in body.\n";
        let parsed = parse_frontmatter(content).unwrap();
        assert_eq!(parsed.frontmatter.module.as_deref(), Some("a"));
        assert!(
            parsed.body.contains('\u{feff}'),
            "a non-leading BOM must be preserved"
        );
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
    fn test_get_spec_symbols_preserves_complete_punctuated_symbols() {
        let body = r#"## Public API

### Exported Symbols

| Symbol | Description |
|--------|-------------|
| `inputs.config` | Dotted YAML input |
| `inputs.working-directory` | Dotted and hyphenated YAML input |
| `outputs.atlas-enabled` | Dotted and hyphenated YAML output |
| `permissions.id-token` | Dotted and hyphenated permission |
| `jobs.deploy-atlas` | Dotted and hyphenated job |
| `login` | Ordinary identifier |
| `active?` | Predicate-style identifier |
| `save!` | Mutating identifier |
| `name_=` | Setter identifier |
| `setName:age:` | Objective-C selector |
| `~Widget` | C++ destructor |
| `map'` | Apostrophe identifier |
| `*global-var*` | Lisp identifier |
| `%+%` | R operator |
| `>>=` | Haskell or OCaml operator |
| `operator==` | C++ operator |
| `operator[]` | C++ subscript operator |
| `operator new` | C++ operator containing a space |
| `<|>` | Elixir or Scala operator containing a pipe |
| `!!!` | Punctuation-only operator |
| `MyApp.Auth` | Dotted module path |
| `módulo` | Unicode identifier |
| `with space` | Quoted atom or string export name |

## Invariants
"#;

        let expected = vec![
            "inputs.config",
            "inputs.working-directory",
            "outputs.atlas-enabled",
            "permissions.id-token",
            "jobs.deploy-atlas",
            "login",
            "active?",
            "save!",
            "name_=",
            "setName:age:",
            "~Widget",
            "map'",
            "*global-var*",
            "%+%",
            ">>=",
            "operator==",
            "operator[]",
            "operator new",
            "<|>",
            "!!!",
            "MyApp.Auth",
            "módulo",
            "with space",
        ];

        assert_eq!(get_spec_symbols(body), expected);
        assert_eq!(get_all_api_table_symbols(body), expected);
    }

    #[test]
    fn test_api_table_symbol_parser_rejects_empty_or_malformed_rows() {
        let body = r#"## Public API

### Exported Symbols

| Symbol | Description |
|--------|-------------|
| `` | Empty code span |
| `   ` | Whitespace-only code span |
| `unterminated | Missing closing delimiter |
| plain | `laterColumn` |
| `trailing` text | Closing span does not occupy the cell |
This prose contains `proseSymbol` but is not a table row.
| `login` | Valid control row |

## Invariants
"#;

        assert_eq!(get_spec_symbols(body), vec!["login"]);
        assert_eq!(get_all_api_table_symbols(body), vec!["login"]);
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
    fn test_get_all_api_table_symbols_includes_informational_tables() {
        let body = r#"# Auth

## Public API

### Functions

| Function | Description |
|----------|-------------|
| `greet` | Says hello |

### API Endpoints

| Endpoint | Description |
|----------|-------------|
| `login` | Login route |

## Error Cases

| Condition | Behavior |
|-----------|----------|
| `OutsideApiSection` | Not collected |
"#;
        let symbols = get_all_api_table_symbols(body);
        // Tables under ANY ### heading within ## Public API count, but tables
        // in other ## sections do not.
        assert_eq!(symbols, vec!["greet", "login"]);
    }

    #[test]
    fn test_get_all_api_table_symbols_no_api_section() {
        assert!(get_all_api_table_symbols("# Title\n\n## Purpose\n\nText.\n").is_empty());
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
    fn checked_issue_references_accept_normal_lists_and_inline_comments() {
        let content = "\
---
module: auth
implements:
  - 41
  - 42 # inline item comment
tracks: [43, 44] # inline list comment
---

# Auth
";

        assert_eq!(
            parse_checked_issue_references(content).unwrap(),
            (vec![41, 42], vec![43, 44])
        );

        let max = "---\nimplements: [18446744073709551615]\n---\n\n# Auth\n";
        assert_eq!(
            parse_checked_issue_references(max).unwrap(),
            (vec![u64::MAX], Vec::new())
        );
    }

    #[test]
    fn checked_issue_references_accept_crlf_frontmatter() {
        let content = "---\r\nmodule: auth\r\nimplements:\r\n  - 41\r\n  - 42 # inline item comment\r\ntracks: [43, 44]\r\n---\r\n\r\n# Auth\r\n";

        assert_eq!(
            parse_checked_issue_references(content).unwrap(),
            (vec![41, 42], vec![43, 44])
        );
    }

    #[test]
    fn checked_issue_references_keep_crlf_validation_strict() {
        for (content, expected) in [
            (
                "---\r\nmodule: auth\r\nimplements: [41]\r\nimplements: [42]\r\n---\r\n\r\n# Auth\r\n",
                DUPLICATE_IMPLEMENTS_ERROR,
            ),
            (
                "---\r\nmodule: auth\r\nextensions:\r\n  nested: [unterminated\r\n---\r\n\r\n# Auth\r\n",
                INVALID_YAML_FRONTMATTER_ERROR,
            ),
            (
                "---\r\nmodule: auth\r\ntracks: 42\r\n---\r\n\r\n# Auth\r\n",
                INVALID_TRACKS_SHAPE_ERROR,
            ),
        ] {
            assert_eq!(
                parse_checked_issue_references(content).unwrap_err(),
                expected
            );
        }
    }

    #[test]
    fn checked_issue_references_accept_yaml_trailing_commas() {
        let content = "---\nmodule: auth\nimplements: [42,]\ntracks: []\n---\n\n# Auth\n";

        assert_eq!(
            parse_checked_issue_references(content).unwrap(),
            (vec![42], Vec::new())
        );
    }

    #[test]
    fn checked_issue_references_reject_duplicate_keys() {
        for (content, expected) in [
            (
                "---\nmodule: auth\nimplements: [41]\nimplements: [42]\n---\n\n# Auth\n",
                DUPLICATE_IMPLEMENTS_ERROR,
            ),
            (
                "---\nmodule: auth\nextensions:\n  mode: one\n  mode: two\n---\n\n# Auth\n",
                DUPLICATE_YAML_KEY_ERROR,
            ),
        ] {
            assert_eq!(
                parse_checked_issue_references(content).unwrap_err(),
                expected
            );
        }
    }

    #[test]
    fn checked_issue_references_reject_invalid_known_values() {
        let cases = [
            ("implements:", INVALID_IMPLEMENTS_SHAPE_ERROR),
            ("implements: null", INVALID_IMPLEMENTS_SHAPE_ERROR),
            ("implements: 42", INVALID_IMPLEMENTS_SHAPE_ERROR),
            ("implements: [0]", INVALID_IMPLEMENTS_NUMBER_ERROR),
            ("implements: [-1]", INVALID_IMPLEMENTS_NUMBER_ERROR),
            (
                "implements: [18446744073709551616]",
                INVALID_IMPLEMENTS_NUMBER_ERROR,
            ),
            ("implements: [41, nope]", INVALID_IMPLEMENTS_NUMBER_ERROR),
            ("tracks:", INVALID_TRACKS_SHAPE_ERROR),
            ("tracks: null", INVALID_TRACKS_SHAPE_ERROR),
            ("tracks: [41, 0]", INVALID_TRACKS_NUMBER_ERROR),
        ];

        for (field, expected) in cases {
            let content = format!("---\nmodule: auth\n{field}\n---\n\n# Auth\n");
            let result = parse_checked_issue_references(&content);
            assert!(
                result.is_err(),
                "expected rejection for {field}: {result:?}"
            );
            assert_eq!(
                result.unwrap_err(),
                expected,
                "unexpected result for {field}"
            );
        }
    }

    #[test]
    fn checked_issue_references_ignore_nested_extensions_and_block_scalars() {
        let content = "\
---
module: auth
extensions:
  implements: [900]
  nested:
    tracks: invalid
extension_sequence:
  - implements: [901]
    tracks: [902]
notes: |
  implements: invalid
  tracks:
    - 903
folded: >
  tracks: [904]
implements: [41]
tracks: [42]
---

# Auth
";

        assert_eq!(
            parse_checked_issue_references(content).unwrap(),
            (vec![41], vec![42])
        );
    }

    #[test]
    fn checked_issue_references_reject_malformed_unknown_extensions() {
        let content = "---\nmodule: auth\nextensions:\n  nested: [unterminated\n---\n\n# Auth\n";

        assert_eq!(
            parse_checked_issue_references(content).unwrap_err(),
            INVALID_YAML_FRONTMATTER_ERROR
        );
    }

    #[test]
    fn checked_issue_references_reject_reviewer_reproducer_without_leaking_content() {
        let content = "\
---
module: auth
implements:
private_extension:
  secret: [reviewer-reproducer
---

# Auth
";

        let error = parse_checked_issue_references(content).unwrap_err();

        assert!(matches!(
            error.as_str(),
            INVALID_IMPLEMENTS_SHAPE_ERROR | INVALID_YAML_FRONTMATTER_ERROR
        ));
        assert!(!error.contains("reviewer-reproducer"));
        assert!(!error.contains("private_extension"));
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

    #[test]
    fn generator_placeholder_text_counts_as_stub() {
        // #421: the generators' own scaffold sentences are draft markers, not
        // real content — a section filled only with them is a stub.
        let body = "## Purpose\n\nDocument this module's responsibility, inputs, outputs, and ownership boundaries.\n\n## Invariants\n\n1. Define an invariant that must remain true for supported inputs.\n";
        assert!(!section_has_content(body, "Purpose"));
        assert!(!section_has_content(body, "Invariants"));

        let behavioral = "## Behavioral Examples\n\n### Scenario: Core behavior\n\n- **Given** precondition\n- **When** action\n- **Then** result\n";
        assert!(!section_has_content(behavioral, "Behavioral Examples"));

        // ...but real content is still real content.
        let real = "## Purpose\n\nAuthenticates users via OAuth2 and session tokens.\n";
        assert!(section_has_content(real, "Purpose"));
    }
}
