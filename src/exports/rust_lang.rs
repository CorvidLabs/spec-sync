use regex::Regex;
use std::sync::LazyLock;

static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)//.*$").unwrap());

static COMMENT_MULTI: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/").unwrap());

/// Raw string literals: r###"..."###, r##"..."##, r#"..."#, r"..."
/// Processed from most hashes to fewest so inner patterns don't match prematurely.
static RAW_STR_3: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?s)\br\#\#\#".*?"\#\#\#"#).unwrap());
static RAW_STR_2: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?s)\br\#\#".*?"\#\#"#).unwrap());
static RAW_STR_1: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?s)\br\#".*?"\#"#).unwrap());
static RAW_STR_0: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?s)\br"[^"]*""#).unwrap());

/// Char literals that contain a double quote: '"' or '\"'
static CHAR_DQUOTE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"'(?:\\.|")'"#).unwrap());

/// Regular string literals (handling escaped quotes and line continuations).
static REG_STR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"(?s)"(?:[^"\\]|\\.)*""#).unwrap());

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
    // Strip string literals first (before comments, since strings can contain //)
    let stripped = RAW_STR_3.replace_all(content, r#""""#);
    let stripped = RAW_STR_2.replace_all(&stripped, r#""""#);
    let stripped = RAW_STR_1.replace_all(&stripped, r#""""#);
    let stripped = RAW_STR_0.replace_all(&stripped, r#""""#);
    let stripped = CHAR_DQUOTE.replace_all(&stripped, "' '");
    let stripped = REG_STR.replace_all(&stripped, r#""""#);

    // Then strip comments
    let stripped = COMMENT_SINGLE.replace_all(&stripped, "");
    let stripped = COMMENT_MULTI.replace_all(&stripped, "");

    let mut symbols = Vec::new();

    for caps in PUB_DECL.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            symbols.push(name.as_str().to_string());
        }
    }

    symbols
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
}
