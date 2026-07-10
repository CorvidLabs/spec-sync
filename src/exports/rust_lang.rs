use regex::Regex;
use std::sync::LazyLock;

/// pub fn, pub struct, pub enum, pub trait, pub type, pub const, pub static, pub mod
/// Only plain `pub` is treated as exported; restricted visibility forms such as
/// `pub(crate)`, `pub(super)`, `pub(self)`, and `pub(in path::to::mod)` are excluded.
static PUB_DECL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"pub(\([^)]*\))?\s+(?:async\s+)?(?:unsafe\s+)?(?:fn|struct|enum|trait|type|const|static|mod)\s+(\w+)",
    )
    .unwrap()
});

/// `pub use path::to::{A, B as C};` — a grouped re-export. Real bug found this
/// session: `pub use` re-exports were not recognized AT ALL, even though they
/// are one of the most common ways a Rust crate re-exports a submodule's type
/// at the crate root (e.g. `pub use crate::foo::Bar;` in `lib.rs`), making them
/// squarely part of the crate's public API surface. `[^}]*` spans newlines (no
/// `(?s)` needed: a character class excluding only `}` already matches `\n`),
/// so a rustfmt-wrapped multi-line brace list is still captured whole.
static PUB_USE_GROUP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"pub(\([^)]*\))?\s+use\s+[\w:]+::\{([^}]*)\}").unwrap());

/// `pub use path::to::Name;` or `pub use path::to::Name as Alias;` — a
/// single-item re-export. The glob form `pub use path::*;` is intentionally
/// NOT matched (no resolver to expand it here, mirroring how `export * from`
/// is only expanded in typescript.rs when a resolver is supplied) — left for
/// future work.
static PUB_USE_SINGLE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"pub(\([^)]*\))?\s+use\s+(?:\w+::)*(\w+)(?:\s+as\s+(\w+))?\s*;").unwrap()
});

/// Extract public symbols from Rust source code.
/// Looks for `pub fn`, `pub struct`, `pub enum`, `pub trait`, `pub type`,
/// `pub const`, `pub static`, and `pub mod` declarations.
/// Restricted visibility forms `pub(crate)`, `pub(super)`, `pub(self)`, and
/// `pub(in path::to::mod)` are excluded because they are not exported outside the
/// crate or module.
pub fn extract_exports(content: &str) -> Vec<String> {
    // Blank every string literal (regular, byte/C, and raw with ANY hash count) and
    // comment in one linear pass, so nothing inside them — a `"`, `//`, `/*`, or a
    // `pub fn`-shaped token — is misread as code or as a delimiter of the wrong kind.
    let stripped = strip_strings_and_comments(content);

    let mut symbols = Vec::new();

    for caps in PUB_DECL.captures_iter(&stripped) {
        // Skip restricted visibility: pub(crate), pub(super), pub(self),
        // pub(in path::to::mod), or any other parenthesized form.
        if caps.get(1).is_some() {
            continue;
        }
        if let Some(name) = caps.get(2) {
            symbols.push(name.as_str().to_string());
        }
    }

    // Grouped re-export: pub use path::{A, B as C};
    for caps in PUB_USE_GROUP.captures_iter(&stripped) {
        if caps.get(1).is_some() {
            continue; // restricted visibility, e.g. pub(crate) use
        }
        if let Some(names) = caps.get(2) {
            for item in names.as_str().split(',') {
                let item = item.trim();
                if item.is_empty() || item == "self" {
                    continue;
                }
                let final_name = item.split(" as ").last().unwrap_or(item).trim();
                if !final_name.is_empty() && final_name != "*" {
                    symbols.push(final_name.to_string());
                }
            }
        }
    }

    // Single re-export: pub use path::Name; or pub use path::Name as Alias;
    for caps in PUB_USE_SINGLE.captures_iter(&stripped) {
        if caps.get(1).is_some() {
            continue; // restricted visibility, e.g. pub(crate) use
        }
        let name = caps
            .get(3)
            .map(|m| m.as_str())
            .or_else(|| caps.get(2).map(|m| m.as_str()));
        if let Some(name) = name {
            symbols.push(name.to_string());
        }
    }

    symbols
}

fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Blank string literals and comments in one linear pass so neither is misread as
/// the other and nothing inside them is extracted. Handles, at each position:
/// `//` line comments; nesting `/* */` block comments; raw strings `r#"…"#` with
/// ANY hash count (and the byte/C prefixes `br`/`cr`); `'"'`/`'\"'` char literals
/// (whose `"` would otherwise open a string); and regular/byte/C `"…"` strings with
/// `\` escapes. Newlines are preserved so line-based callers stay aligned.
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
        // Raw string literal `r"…"`, `r#"…"#`, … with ANY number of hashes, plus the
        // byte/C prefixes `br`/`cr`. Only at an identifier boundary, so the `r` in
        // `for`/`error` (or an identifier ending in `r`) is not misread as a prefix.
        if (c == 'r' || ((c == 'b' || c == 'c') && i + 1 < n && chars[i + 1] == 'r'))
            && (i == 0 || !is_ident_char(chars[i - 1]))
        {
            let after_r = if c == 'r' { i + 1 } else { i + 2 };
            let mut hash_end = after_r;
            while hash_end < n && chars[hash_end] == '#' {
                hash_end += 1;
            }
            if hash_end < n && chars[hash_end] == '"' {
                let hashes = hash_end - after_r;
                // Scan for a closing `"` followed by exactly `hashes` `#`.
                let mut j = hash_end + 1;
                while j < n {
                    if chars[j] == '"' {
                        let mut k = 0;
                        while k < hashes && j + 1 + k < n && chars[j + 1 + k] == '#' {
                            k += 1;
                        }
                        if k == hashes {
                            j = j + 1 + hashes;
                            break;
                        }
                    }
                    if chars[j] == '\n' {
                        out.push('\n');
                    }
                    j += 1;
                }
                i = j;
                continue;
            }
        }
        // Char literal containing a double quote — `'"'` or `'\"'`. Blank it so its
        // `"` is not read as a string opener. Other char literals and lifetimes hold
        // no `"` and are harmless, so they are left as-is.
        if c == '\'' {
            if i + 2 < n && chars[i + 1] == '"' && chars[i + 2] == '\'' {
                i += 3;
                continue;
            }
            if i + 3 < n && chars[i + 1] == '\\' && chars[i + 2] == '"' && chars[i + 3] == '\'' {
                i += 4;
                continue;
            }
        }
        // Regular / byte / C (non-raw) string literal: skip to the closing quote,
        // honoring `\` escapes.
        if c == '"' {
            i += 1;
            while i < n {
                if chars[i] == '\\' {
                    if i + 1 < n && chars[i + 1] == '\n' {
                        out.push('\n');
                    }
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
    fn test_pub_crate_excluded() {
        let src = r#"
pub(crate) fn internal_fn() {}
pub(crate) struct InternalStruct {}
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.is_empty(),
            "pub(crate) items must not be exported: {symbols:?}"
        );
    }

    #[test]
    fn test_pub_super_and_pub_in_path_excluded() {
        // `pub(super)` and `pub(in path::to::mod)` are restricted visibility
        // forms used to share an item with a parent module without making it
        // part of the crate's public API. Only plain `pub` is exported.
        let src = r#"
pub(super) fn helper_for_parent() {}
pub(in crate::auth) struct ScopedConfig {}
pub(crate) fn crate_visible() {}
pub fn truly_public() {}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["truly_public"]);
    }

    #[test]
    fn test_pub_self_excluded() {
        let src = r#"
pub(self) fn private_self() {}
pub(self) struct PrivateStruct {}
pub fn truly_public() {}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["truly_public"]);
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

    #[test]
    fn test_learnxinyminutes_preamble_no_false_positives() {
        // Real excerpt (verbatim) from learnxinyminutes.com/rust.md: the file's
        // opening comment block plus its first two functions. Exercises nested
        // `/* /* */ */` block comments, a `///` doc comment containing a
        // markdown code fence (which itself contains Rust-looking source), and
        // stacked `#[allow(...)]` attributes -- none of which precede a `pub`
        // item here, so nothing should be extracted. Locks in that comment/attr
        // handling on real, syntax-rich source doesn't produce garbage.
        let src = r#"
// This is a comment. Line comments look like this...
// and extend multiple lines like this.

/* Block comments
  /* can be nested. */ */

/// Documentation comments look like this and support markdown notation.
/// # Examples
///
/// ```
/// let five = 5
/// ```

///////////////
// 1. Basics //
///////////////

#[allow(dead_code)]
// Functions
// `i32` is the type for 32-bit signed integers
fn add2(x: i32, y: i32) -> i32 {
    // Implicit return (no semicolon)
    x + y
}

#[allow(unused_variables)]
#[allow(unused_assignments)]
#[allow(dead_code)]
// Main function
fn main() {
    // Numbers //
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.is_empty(),
            "no pub items in this excerpt, but got: {symbols:?}"
        );
    }

    #[test]
    fn test_learnxinyminutes_traits_and_generics_no_false_positives() {
        // Real excerpt (verbatim) from learnxinyminutes.com/rust.md: trait
        // definitions, a generic `impl<T> Trait<T> for Type<T>`, a recursive
        // `fn`, a `type` alias, and pattern matching with Unicode smart-quote
        // string literals (`it’s an i32`). None of these carry a `pub`
        // keyword in the tutorial's source, so this is a strong real-world
        // negative test: lots of `trait`/`impl`/`fn`/`type`/`struct` keywords
        // that must NOT be misread as exports just because they resemble
        // declaration syntax.
        let src = r#"
    trait Frobnicate<T> {
        fn frobnicate(self) -> Option<T>;
    }

    impl<T> Frobnicate<T> for Foo<T> {
        fn frobnicate(self) -> Option<T> {
            Some(self.bar)
        }
    }

    let another_foo = Foo { bar: 1 };
    println!("{:?}", another_foo.frobnicate()); // Some(1)

    // Function pointer types //

    fn fibonacci(n: u32) -> u32 {
        match n {
            0 => 1,
            1 => 1,
            _ => fibonacci(n - 1) + fibonacci(n - 2),
        }
    }

    type FunctionPointer = fn(u32) -> u32;

    let fib : FunctionPointer = fibonacci;
    println!("Fib: {}", fib(4)); // 5

    let foo = OptionalI32::AnI32(1);
    match foo {
        OptionalI32::AnI32(n) => println!("it’s an i32: {}", n),
        OptionalI32::Nothing  => println!("it’s nothing!"),
    }
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.is_empty(),
            "no pub items in this excerpt, but got: {symbols:?}"
        );
    }

    #[test]
    fn test_pub_use_reexport() {
        // Real bug found this session: `pub use` re-exports were not
        // recognized at all. Re-exporting a submodule's type at the crate
        // root (`pub use crate::foo::Bar;`) is one of the most common Rust
        // public-API patterns (idiomatic in every `lib.rs`), and missing it
        // meant a removed/renamed re-export produced zero diff. Covers a
        // single-item re-export, a grouped brace list with an `as` alias,
        // `pub(crate) use` (restricted visibility, must be excluded like
        // other `pub(...)` forms), and a glob re-export (intentionally not
        // expanded here, mirroring typescript.rs's resolver-less `export *`).
        let src = r#"
pub use crate::foo::Bar;
pub use crate::baz::{Qux, Quux as Renamed};
pub(crate) use crate::internal::Hidden;
pub use crate::glob::*;
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Bar".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"Qux".to_string()), "{symbols:?}");
        assert!(
            symbols.contains(&"Renamed".to_string()),
            "aliased grouped item: {symbols:?}"
        );
        assert!(
            !symbols.contains(&"Hidden".to_string()),
            "pub(crate) use must be excluded: {symbols:?}"
        );
        assert!(
            !symbols.contains(&"glob".to_string()),
            "glob re-export must not leak the module segment: {symbols:?}"
        );
    }

    #[test]
    fn test_pub_use_single_with_alias_and_multiline_group() {
        // A single-item re-export aliased with `as`, and a grouped brace list
        // wrapped across multiple lines (common rustfmt output for long
        // lists) -- `[^}]*` must span the newlines to capture the whole list.
        let src = "
pub use crate::config::Settings as Config;
pub use crate::models::{\n    User,\n    Account as Acct,\n    self,\n};
";
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Config".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"User".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"Acct".to_string()), "{symbols:?}");
        assert!(
            !symbols.contains(&"self".to_string()),
            "bare `self` in a grouped use list is not a named export"
        );
    }

    #[test]
    fn test_doc_comment_code_example_not_captured() {
        // Verified-correct behavior: a rustdoc code-fence example containing
        // sample `pub fn` text is just `///`-prefixed lines, i.e. ordinary
        // line comments to strip_strings_and_comments (handled character by
        // character, not via a `$`-anchored regex, so it is immune to the
        // "comment regex missing (?m)" bug class fixed elsewhere this
        // session). Only the real declaration after the doc block is captured.
        let src = r#"
/// ```
/// pub fn documented_example() {}
/// ```
pub fn real_after_doc_example() {}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["real_after_doc_example"]);
    }

    #[test]
    fn test_raw_strings_with_many_hashes_do_not_hide_exports() {
        // Raw strings with 4+ hashes must be handled (the scanner counts hashes
        // rather than relying on fixed 0..3-hash regexes). An interior quote must
        // not leak and open an unterminated string that swallows later decls.
        let src = r######"
pub fn before_raw() {}
pub const P: &str = r####"a "### b"####;
pub fn after_raw() {}
pub struct AfterStruct {}
let q = r#####"x "#### y"#####;
pub fn tail() {}
"######;
        let symbols = extract_exports(src);
        for name in ["before_raw", "P", "after_raw", "AfterStruct", "tail"] {
            assert!(
                symbols.contains(&name.to_string()),
                "missing {name}: {symbols:?}"
            );
        }
    }
}
