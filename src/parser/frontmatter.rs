//! Compatibility frontmatter parsing and the canonical borrowed delimiter scan.

use super::ParsedSpec;
use crate::types::Frontmatter;
use regex::Regex;
use std::borrow::Cow;
use std::collections::HashSet;
use std::sync::LazyLock;

/// `[ \t\r]*` after each delimiter is the SAME rule [`is_frontmatter_delimiter`] applies, spelled
/// in regex because this parser is regex-driven. The two must not drift: a delimiter shape one
/// reader accepts and another does not is how a document comes to mean two things at once
/// (#716), and `all_frontmatter_readers_agree_on_what_a_delimiter_is` fails if they ever do.
static FRONTMATTER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)^---[ \t\r]*\n(.*?)\n---[ \t\r]*\n(.*)$").unwrap());

/// Parse YAML frontmatter from a spec file.
/// Zero-dependency YAML: uses regex, no YAML parser needed.
///
/// CRLF input is accepted. `FRONTMATTER_RE` is LF-only, and there is no
/// "normalize before you call me" convention to lean on — measured, 21 of the 39
/// call sites outside this module normalize and 18 do not, so a Windows checkout
/// made `specsync view` fail with "Cannot parse frontmatter" on every spec (#696).
/// Normalizing here fixes every one of those callers without touching any of them,
/// where an obligation on 18 call sites would be unenforceable and fail silently.
/// The returned `body` is always LF-only, which is what every caller already
/// assumed it was — including a document whose frontmatter is LF and whose body
/// is CRLF, which returned CRLF before normalization landed here and returns LF
/// now. That was an undisclosed consequence of #696; every consumer is read-only
/// analysis that never maps back to raw file bytes, and it is pinned by test and
/// stated in the spec rather than left to be rediscovered (#716).
///
/// Delimiters follow [`is_frontmatter_delimiter`]: three dashes and trailing
/// whitespace, at both ends. A closer padded with a trailing space used to be
/// walked past, so the frontmatter capture ran on to the first horizontal rule in
/// the body — feeding prose to the YAML line parser and returning a body cut at
/// that rule (#716).
pub fn parse_frontmatter(content: &str) -> Option<ParsedSpec> {
    // A leading UTF-8 BOM (U+FEFF) is a non-semantic encoding marker that some
    // editors prepend; left in place it sits before the opening `---`, so the
    // `^---` anchor fails and a perfectly valid spec is reported as having
    // "malformed frontmatter" (the delimiters are right there — the user just
    // can't see the invisible byte). Strip leading BOM(s) only (a U+FEFF anywhere
    // else is real content and is left untouched); this is lossless.
    let content = content.trim_start_matches('\u{feff}');
    // Guarded so an LF document — every tracked spec in this repository — allocates
    // nothing and takes the borrowed path.
    let normalized: Cow<'_, str> = if content.contains('\r') {
        Cow::Owned(content.replace("\r\n", "\n"))
    } else {
        Cow::Borrowed(content)
    };
    let content: &str = normalized.as_ref();
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
            match unquote_yaml_scalar(&strip_yaml_comment(stripped.trim())) {
                Ok(item) => current_list.push(item),
                Err(error) => errors.push(error),
            }
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

            let raw_value = strip_yaml_comment(line[colon_pos + 1..].trim());
            // A flow-style list keeps its brackets for `parse_flow_string_list`,
            // which unquotes its own items.
            let value = if raw_value.starts_with('[') {
                raw_value
            } else {
                match unquote_yaml_scalar(&raw_value) {
                    Ok(unquoted) => unquoted,
                    Err(error) => {
                        errors.push(error);
                        continue;
                    }
                }
            };

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

/// The body of a Markdown document, with YAML frontmatter removed, borrowed from the input.
///
/// The single canonical stripper (#696). Frontmatter stripping had four implementations in this
/// repository and they disagreed; two deleted body content rather than merely leaving frontmatter
/// behind, and neither failure raised an error. A delimiter line is three dashes and nothing but
/// whitespace after them, at both ends of the block — see [`is_frontmatter_delimiter`] for what
/// that admits, what it refuses, and why the line stops where it does (#716). A document that
/// opens with anything else is returned whole, which for a caller counting prose means the YAML
/// reads as content; that residual is stated in `parser.spec.md` rather than guessed at.
///
/// Use this one — it is correct on all six axes that
/// separated them: LF, CRLF, a leading BOM, unterminated frontmatter, a closing delimiter at EOF,
/// and a horizontal rule in the body. Any one of those alone is survivable, which is why a partial
/// implementation looks correct until it meets the combination.
///
/// Unlike [`parse_frontmatter`] this does not normalize: the return borrows, so a CRLF body comes
/// back with its CRLFs intact. Callers that need LF should normalize their own input or go
/// through [`parse_frontmatter`].
///
/// Frontmatter ends at its CLOSING delimiter LINE, never at the next `---` anywhere in the
/// document. Splitting on the delimiter and taking the third field looks equivalent and is not:
/// a body with a horizontal rule loses everything after it, and lost prose reads exactly like
/// prose nobody ever wrote.
pub fn strip_frontmatter(text: &str) -> &str {
    // A leading UTF-8 BOM must not hide the opening delimiter.
    let text = text.trim_start_matches('\u{feff}');
    match split_frontmatter(text) {
        Some((_, body)) => body,
        // No opening delimiter, or frontmatter that never closes: keep the whole document rather
        // than guess where the block ended.
        None => text,
    }
}

/// True when `line` — one line including its terminator, if it has one — is a frontmatter
/// delimiter.
///
/// EXACTLY three dashes, then nothing but horizontal whitespace and the line ending. Both ends
/// of the block are held to this same rule, in either line encoding.
///
/// Trailing whitespace is tolerated (#716). An author cannot see a trailing space, every Markdown
/// frontmatter implementation ignores one, and refusing it was not a safety property but a
/// silent one: a `---  ` OPENER left the YAML in the body, where `change`'s completeness gate
/// counted it as prose and approved an artifact with nothing written in it; a `---  ` CLOSER made
/// the scan walk past the real end of the block and stop at the first horizontal rule in the
/// body, deleting the prose above it.
///
/// The tolerance stops exactly there, and that is the load-bearing half. `----` is a legal
/// Markdown thematic break, and a document that opens with one is a document, not frontmatter:
/// accepting it would make this scan run forward to the next rule and return a body cut at it —
/// and prose lost that way reads exactly like prose nobody ever wrote (#697, #699, #705). Leading
/// whitespace is refused for the same reason. Anything else that begins with three dashes —
/// `----`, `--- x`, `---change: x` — is content, and the document is returned whole.
fn is_frontmatter_delimiter(line: &str) -> bool {
    line.trim_end_matches([' ', '\t', '\r', '\n']) == "---"
}

/// The YAML frontmatter block and the Markdown body of `text`, or `None` when `text` does not
/// open with a delimiter line or never closes one.
///
/// The one delimiter scan in this repository. Frontmatter stripping had four implementations that
/// disagreed (#696); this keeps the reader that survived and the issue-reference reader from
/// becoming the next pair, because a delimiter rule written twice is a delimiter rule that will
/// eventually be loosened once.
///
/// The caller strips any leading BOM: byte offsets here are into `text` as given.
pub(super) fn split_frontmatter(text: &str) -> Option<(&str, &str)> {
    let mut lines = text.split_inclusive('\n');
    // The opener must be a delimiter line AND be terminated by a newline. A document that is
    // nothing but `---` opens no block, and there is no body to return.
    let open = lines.next().filter(|line| line.ends_with('\n'))?;
    if !is_frontmatter_delimiter(open) {
        return None;
    }
    // BOTH line endings, and they need not match each other: a Windows-authored companion
    // otherwise keeps its frontmatter and every untouched scaffold reports itself as recorded
    // knowledge.
    let after_open = &text[open.len()..];
    let mut offset = 0usize;
    for line in after_open.split_inclusive('\n') {
        if is_frontmatter_delimiter(line) {
            return Some((&after_open[..offset], &after_open[offset + line.len()..]));
        }
        offset += line.len();
    }
    None
}

/// Remove matched surrounding YAML quotes from a scalar, and any comment that
/// follows the closing quote.
///
/// Quoting is valid YAML and is the documented answer for a path containing a
/// space or a leading special character, but block scalars and block list items
/// used to keep their quotes: `- "src/a.rs"` was resolved as the literal path
/// `"src/a.rs"`, reported as a missing source file, and then cascaded into a
/// bogus "documents X but no matching export found" — while frontmatter was
/// still reported valid, so nothing pointed at the quotes. Flow-style lists
/// (`files: ["a", "b"]`) already unquoted via [`parse_flow_string_list`]; this
/// closes the asymmetry.
///
/// An opening quote with no match is an error rather than a silently retained
/// literal, matching the flow-style list behavior.
fn unquote_yaml_scalar(value: &str) -> Result<String, String> {
    let unterminated = || format!("Frontmatter has an unterminated quoted string: `{value}`");
    for quote in ['"', '\''] {
        let Some(rest) = value.strip_prefix(quote) else {
            continue;
        };
        let close = rest.find(quote).ok_or_else(unterminated)?;
        // Anything after the closing quote may only be a comment.
        let trailing = rest[close + quote.len_utf8()..].trim();
        if !trailing.is_empty() && !trailing.starts_with('#') {
            return Err(unterminated());
        }
        return Ok(rest[..close].to_string());
    }
    Ok(value.to_string())
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

#[cfg(test)]
#[path = "tests/frontmatter.rs"]
mod tests;
