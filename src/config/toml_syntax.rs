//! String, comment, and array syntax shared by TOML parsing and serialization.

/// Escape a string value for safe embedding in a TOML quoted string.
pub(super) fn toml_escape(s: &str) -> String {
    let mut escaped = String::with_capacity(s.len());
    for character in s.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\u{08}' => escaped.push_str("\\b"),
            '\t' => escaped.push_str("\\t"),
            '\n' => escaped.push_str("\\n"),
            '\u{0C}' => escaped.push_str("\\f"),
            '\r' => escaped.push_str("\\r"),
            control if control.is_control() => {
                escaped.push_str(&format!("\\u{:04X}", control as u32));
            }
            value => escaped.push(value),
        }
    }
    escaped
}

/// Format a `name = ["a", "b"]` TOML array line (an empty slice → `name = []`).
pub(super) fn toml_array_line(name: &str, items: &[String]) -> String {
    let body = items
        .iter()
        .map(|s| format!("\"{}\"", toml_escape(s)))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{name} = [{body}]")
}

/// Inverse of `toml_escape`: decodes the backslash, double-quote, newline,
/// carriage-return, and tab escape sequences it emits. Without this,
/// `config_to_toml` writes a correctly-escaped basic string that the reader would
/// load with the escapes still literal — silently changing any value containing a
/// backslash or quote on migrate (e.g. a `schema_pattern` regex, or a Windows
/// module path). An unrecognized escape is left byte-for-byte so a hand-written
/// config with a stray backslash is not mangled.
pub(super) fn toml_unescape(s: &str) -> String {
    if !s.contains('\\') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('\\') => out.push('\\'),
            Some('"') => out.push('"'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            // Unknown escape — preserve both characters literally.
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

/// Parse a TOML string value: `"value"` -> `value`
/// Strip an inline TOML comment from a value string.
/// Respects quoted strings — `#` inside quotes is not a comment.
/// Examples:
///   `"ollama" # local`  →  `"ollama"`
///   `120 # seconds`     →  `120`
///   `"has # inside"`    →  `"has # inside"`
pub(super) fn strip_inline_comment(s: &str) -> String {
    // Track both TOML string kinds: basic `"..."` (backslash-escaped) and literal
    // `'...'` (no escapes). A `#` inside either is content, not a comment.
    let mut in_basic = false;
    let mut in_literal = false;
    let mut prev_backslash = false;
    for (i, c) in s.char_indices() {
        if in_basic {
            if c == '"' && !prev_backslash {
                in_basic = false;
            }
            prev_backslash = c == '\\' && !prev_backslash;
            continue;
        }
        if in_literal {
            // Literal strings have no escapes, so the next `'` always closes.
            if c == '\'' {
                in_literal = false;
            }
            continue;
        }
        prev_backslash = false;
        match c {
            '"' => in_basic = true,
            '\'' => in_literal = true,
            // A `#` outside any string starts a comment.
            '#' => return s[..i].trim().to_string(),
            _ => {}
        }
    }
    s.to_string()
}

pub(super) fn parse_toml_string(s: &str) -> String {
    let s = s.trim();
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        // Quoted basic string: decode the escapes `toml_escape` emits so the value
        // round-trips (an unquoted bare value is left as-is).
        toml_unescape(&s[1..s.len() - 1])
    } else if s.len() >= 2 && s.starts_with('\'') && s.ends_with('\'') {
        // Literal string: content is taken verbatim (TOML literal strings perform no
        // escape processing), so a Windows path or regex keeps its backslashes.
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

/// Return the byte index of the `]` that closes the first `[` in `s`, scanning
/// with quote-, escape-, and bracket-depth awareness so a `[`/`]` inside a quoted
/// string (e.g. a glob char-class `"**/[abc]/**"` or its literal-string form
/// `'**/[abc]/**'`) or a nested array does not count. Both TOML string kinds are
/// tracked: basic `"..."` (backslash-escaped) and literal `'...'` (no escapes).
/// Returns `None` if the array is not closed within `s`.
pub(super) fn find_toml_array_close(s: &str) -> Option<usize> {
    let mut in_basic = false;
    let mut in_literal = false;
    let mut prev_backslash = false;
    let mut depth: i32 = 0;
    let mut opened = false;
    for (i, ch) in s.char_indices() {
        if in_basic {
            if ch == '"' && !prev_backslash {
                in_basic = false;
            }
            prev_backslash = ch == '\\' && !prev_backslash;
            continue;
        }
        if in_literal {
            if ch == '\'' {
                in_literal = false;
            }
            continue;
        }
        prev_backslash = false;
        match ch {
            '"' => in_basic = true,
            '\'' => in_literal = true,
            '[' => {
                depth += 1;
                opened = true;
            }
            ']' => {
                depth -= 1;
                if opened && depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Parse a TOML array of strings: `["a", "b"]` -> vec!["a", "b"]
pub(super) fn parse_toml_string_array(s: &str) -> Vec<String> {
    let s = s.trim();
    // Extract the span between the first `[` and its matching `]` (quote/depth
    // aware — see `find_toml_array_close`), so a `]` inside a quoted item and any
    // trailing content are tolerated. A value with no array brackets is treated as
    // a single-element array (unchanged behavior).
    let Some(start) = s.find('[') else {
        return vec![parse_toml_string(s)];
    };
    let Some(end) = find_toml_array_close(&s[start..]).map(|rel| start + rel) else {
        return vec![parse_toml_string(s)];
    };
    // Split on commas OUTSIDE quoted strings so a quoted item containing a comma
    // (e.g. a path or glob) is not torn into bogus entries. A bare `split(',')`
    // would mangle `["a, b", "c"]`.
    split_toml_array_items(&s[start + 1..end])
        .into_iter()
        .map(|item| parse_toml_string(item.trim()))
        .filter(|item| !item.is_empty())
        .collect()
}

/// Split a TOML array body on top-level commas, skipping commas inside quoted
/// strings — both basic `"..."` (escape-aware) and literal `'...'` (no escapes).
/// Each returned segment still carries its quotes/whitespace for `parse_toml_string`
/// to strip.
fn split_toml_array_items(body: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut cur = String::new();
    let mut in_basic = false;
    let mut in_literal = false;
    let mut prev_backslash = false;
    for ch in body.chars() {
        if in_basic {
            cur.push(ch);
            if ch == '"' && !prev_backslash {
                in_basic = false;
            }
            prev_backslash = ch == '\\' && !prev_backslash;
            continue;
        }
        if in_literal {
            cur.push(ch);
            if ch == '\'' {
                in_literal = false;
            }
            continue;
        }
        prev_backslash = false;
        match ch {
            '"' => {
                in_basic = true;
                cur.push(ch);
            }
            '\'' => {
                in_literal = true;
                cur.push(ch);
            }
            ',' => items.push(std::mem::take(&mut cur)),
            _ => cur.push(ch),
        }
    }
    items.push(cur);
    items
}

/// Parse a TOML boolean value.
pub(super) fn parse_toml_bool(s: &str) -> bool {
    matches!(s.trim(), "true" | "yes" | "1")
}

#[cfg(test)]
mod tests;
