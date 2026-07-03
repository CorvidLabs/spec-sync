use regex::Regex;
use std::sync::LazyLock;

/// Raw string literals: r###"..."###, r##"..."##, r#"..."#, r"..." and their
/// byte/C-string variants (br"...", cr"..."). The optional `(?:b|c)?` before `r`
/// is essential: without it `br#"..."#` is never blanked, and the linear scanner
/// then reads its interior quotes as real delimiters and can run a string to EOF,
/// hiding later declarations. Processed most-hashes-first so inner patterns don't
/// match prematurely.
static RAW_STR_3: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?s)\b(?:b|c)?r\#\#\#".*?"\#\#\#"#).unwrap());
static RAW_STR_2: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?s)\b(?:b|c)?r\#\#".*?"\#\#"#).unwrap());
static RAW_STR_1: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?s)\b(?:b|c)?r\#".*?"\#"#).unwrap());
static RAW_STR_0: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?s)\b(?:b|c)?r"[^"]*""#).unwrap());

/// Char literals that contain a double quote: '"' or '\"'
static CHAR_DQUOTE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"'(?:\\.|")'"#).unwrap());

/// pub fn, pub struct, pub enum, pub trait, pub type, pub const, pub static, pub mod
static PUB_DECL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"pub(?:\(crate\))?\s+(?:async\s+)?(?:unsafe\s+)?(?:fn|struct|enum|trait|type|const|static|mod)\s+(\w+)",
    )
    .unwrap()
});

/// Extract public symbols from Rust source code.
/// Looks for `pub fn`, `pub struct`, `pub enum`, `pub trait`, `pub type`,
/// `pub const`, `pub static`, and `pub mod` declarations.
/// Also matches `pub(crate)` items.
pub fn extract_exports(content: &str) -> Vec<String> {
    // Blank out raw strings and double-quote char literals up front (their hash-
    // balanced / quoted forms are awkward for a linear scan); their inner `"`,
    // `//`, `/*` then can't be misread as delimiters.
    let stripped = RAW_STR_3.replace_all(content, r#""""#);
    let stripped = RAW_STR_2.replace_all(&stripped, r#""""#);
    let stripped = RAW_STR_1.replace_all(&stripped, r#""""#);
    let stripped = RAW_STR_0.replace_all(&stripped, r#""""#);
    let stripped = CHAR_DQUOTE.replace_all(&stripped, "' '");

    // Strip regular strings AND comments in a SINGLE pass so neither is misread as
    // the other. A regex that strips strings first treats a `"` inside a `//` doc
    // comment as a string opener and swallows code up to the next real `"` —
    // deleting `pub fn` declarations between them (a `///` line with an odd number
    // of `"` silently hid exports). Recognizing `//` / `/*` before `"` fixes it,
    // and a `//` inside a string is correctly kept as string content.
    let stripped = strip_strings_and_comments(&stripped);

    let mut symbols = Vec::new();

    for caps in PUB_DECL.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            symbols.push(name.as_str().to_string());
        }
    }

    symbols
}

/// Replace regular double-quoted strings, `//` line comments, and (nesting) `/* */`
/// block comments with blanks, in one linear pass so a `"` in a comment is not
/// treated as a string and a `//` in a string is not treated as a comment. Newlines
/// are preserved. Assumes raw strings and double-quote char literals are already
/// blanked (see `extract_exports`).
fn strip_strings_and_comments(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < n {
        let c = chars[i];
        // Line comment: skip to end of line (keep the newline).
        if c == '/' && i + 1 < n && chars[i + 1] == '/' {
            i += 2;
            while i < n && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        // Block comment (Rust block comments nest).
        if c == '/' && i + 1 < n && chars[i + 1] == '*' {
            i += 2;
            let mut depth = 1;
            while i < n && depth > 0 {
                if chars[i] == '/' && i + 1 < n && chars[i + 1] == '*' {
                    depth += 1;
                    i += 2;
                } else if chars[i] == '*' && i + 1 < n && chars[i + 1] == '/' {
                    depth -= 1;
                    i += 2;
                } else {
                    if chars[i] == '\n' {
                        out.push('\n');
                    }
                    i += 1;
                }
            }
            continue;
        }
        // Regular string literal: skip to the closing quote, honoring `\` escapes.
        if c == '"' {
            i += 1;
            while i < n {
                if chars[i] == '\\' {
                    i += 2;
                    continue;
                }
                if chars[i] == '"' {
                    i += 1;
                    break;
                }
                if chars[i] == '\n' {
                    out.push('\n');
                }
                i += 1;
            }
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_exports() {
        let src = r#"
pub fn create_auth(config: Config) -> Auth {}
pub struct AuthService {}
pub enum AuthStatus { Active, Expired }
pub trait Authenticator {}
pub type Token = String;
pub const DEFAULT_TTL: u64 = 3600;
pub static INSTANCE: Lazy<Auth> = Lazy::new(|| Auth::new());
fn private_fn() {}
struct PrivateStruct {}
"#;
        let symbols = extract_exports(src);
        assert_eq!(
            symbols,
            vec![
                "create_auth",
                "AuthService",
                "AuthStatus",
                "Authenticator",
                "Token",
                "DEFAULT_TTL",
                "INSTANCE"
            ]
        );
    }

    #[test]
    fn test_pub_crate() {
        let src = r#"
pub(crate) fn internal_fn() {}
pub(crate) struct InternalStruct {}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["internal_fn", "InternalStruct"]);
    }

    #[test]
    fn test_ignores_pub_inside_string_literals() {
        // Raw string with pub declarations inside — should be ignored
        let src = r###"
pub fn real_fn() {}

let test_data = r#"
pub fn create_auth(config: Config) -> Auth {}
pub struct AuthService {}
"#;

let regular_str = "pub fn fake_fn() {}";
"###;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["real_fn"]);
    }

    #[test]
    fn test_pub_after_raw_string_with_hash_in_content() {
        // Simulates ai.rs: a large r#"..."# raw string followed by pub fn declarations
        let src = r###"
pub fn before_string() {}

let prompt = r#"some template with "quotes" and stuff
pub fn fake_in_template() {}
more template"#;

pub fn after_string() {}
"###;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["before_string", "after_string"]);
    }

    #[test]
    fn test_real_ai_rs() {
        let content = std::fs::read_to_string("src/ai.rs").unwrap();
        let symbols = extract_exports(&content);
        assert!(
            symbols.contains(&"resolve_ai_command".to_string()),
            "resolve_ai_command not found in: {:?}",
            symbols
        );
    }

    #[test]
    fn test_real_registry_rs() {
        let content = std::fs::read_to_string("src/registry.rs").unwrap();
        let symbols = extract_exports(&content);
        assert!(
            symbols.contains(&"generate_registry".to_string()),
            "generate_registry not found in: {:?}",
            symbols
        );
    }

    #[test]
    fn test_char_literal_with_quote() {
        let src = r#"
let x = value.trim_matches('"');
pub fn after_char_lit() {}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["after_char_lit"]);
    }

    #[test]
    fn test_identifier_trailing_r_not_raw_string() {
        let src = r#"
pub fn setup_hooks() {
    hooks.push(("pre_pr", c.as_str()));
}

pub struct PluginEntry {
    name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pinned_ref: Option<String>,
}
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"PluginEntry".to_string()),
            "PluginEntry should not be eaten by false raw-string match: {:?}",
            symbols
        );
        assert_eq!(symbols, vec!["setup_hooks", "PluginEntry"]);
    }

    #[test]
    fn test_async_unsafe() {
        let src = r#"
pub async fn async_fn() {}
pub unsafe fn unsafe_fn() {}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["async_fn", "unsafe_fn"]);
    }

    #[test]
    fn test_quotes_in_comments_do_not_hide_exports() {
        // A doc/line/block comment containing an odd number of `"` used to be read
        // as a string opener, swallowing the following `pub fn` up to the next real
        // quote. Comments must be recognized before strings.
        let src = r#"
/// Returns the escape for `\"` (a lone quote in a doc comment).
pub fn alpha() {}

/// A "quoted phrase" in a doc comment.
pub fn bravo() {}

pub const URL: &str = "http://example.com/a//b";
pub fn charlie() {}

/* block "comment" with a quote and // slashes */
pub fn delta() {}
"#;
        let symbols = extract_exports(src);
        for name in ["alpha", "bravo", "URL", "charlie", "delta"] {
            assert!(
                symbols.contains(&name.to_string()),
                "missing {name}: {symbols:?}"
            );
        }
    }

    #[test]
    fn test_pub_fn_inside_string_not_extracted() {
        // A `pub fn` that appears only inside a string literal must not be captured.
        let src = r#"pub fn real() {} let s = "pub fn fake() {}";"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"real".to_string()));
        assert!(
            !symbols.contains(&"fake".to_string()),
            "fake is inside a string literal"
        );
    }

    #[test]
    fn test_byte_and_c_raw_strings_do_not_hide_exports() {
        // A byte raw string `br#"..."#` (or `cr#"..."#`) with an odd number of
        // interior quotes must be blanked; otherwise the scanner opens a string on
        // the trailing quote and swallows the following pub fn to EOF.
        let src = r####"
let x = br#"a "b"#;
pub fn real() {}
"####;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"real".to_string()), "got {symbols:?}");
    }
}
