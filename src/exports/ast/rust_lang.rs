use tree_sitter::{Parser, Tree};

/// Parse Rust source into a tree-sitter AST.
fn parse_rust(content: &str) -> Option<Tree> {
    let mut parser = Parser::new();
    let language = tree_sitter_rust::LANGUAGE.into();
    parser.set_language(&language).ok()?;
    parser.parse(content, None)
}

/// Extract public symbols from Rust source using tree-sitter AST.
///
/// Handles:
/// - `pub fn/struct/enum/trait/type/const/static/mod`
/// - `pub async fn`, `pub unsafe fn`
/// - Feature-gated exports (`#[cfg(feature = "...")]`)
/// - `pub` items nested inside `impl` blocks and inline `mod { ... }` bodies
/// - Correctly ignores `pub` inside string literals and comments (AST-native)
///
/// Excludes restricted visibility forms such as `pub(crate)`, `pub(super)`,
/// `pub(self)`, and `pub(in path::to::mod)` because they are not exported outside
/// the crate or module.
pub fn extract_exports(content: &str) -> Vec<String> {
    let tree = match parse_rust(content) {
        Some(t) => t,
        None => return Vec::new(),
    };

    let root = tree.root_node();
    let src = content.as_bytes();
    let mut symbols = Vec::new();

    collect_pub_items(&root, src, &mut symbols);

    symbols
}

fn collect_pub_items(node: &tree_sitter::Node, src: &[u8], symbols: &mut Vec<String>) {
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        if is_pub_item(&child, src)
            && let Some(name) = extract_item_name(&child, src)
        {
            symbols.push(name);
        }

        // Recurse into containers whose members carry their own explicit `pub`
        // keyword — inherent `impl` blocks (constructors/builder methods/
        // accessors on a `pub struct`, the most common shape of a real crate's
        // public API) and inline `mod { ... }` bodies — so their `pub fn` /
        // `pub const` / etc. members are not silently dropped. This mirrors the
        // regex backend, which finds these regardless of nesting since it scans
        // the source linearly. (`impl Trait for Type` bodies are visited too,
        // but Rust forbids visibility modifiers on trait-impl items, so
        // `is_pub_item` never matches there — no risk of false positives.)
        match child.kind() {
            "impl_item" | "mod_item" => {
                if let Some(body) = child.child_by_field_name("body") {
                    collect_pub_items(&body, src, symbols);
                }
            }
            _ => {}
        }
    }
}

/// Check if a node has plain `pub` visibility.
/// Restricted forms such as `pub(crate)`, `pub(super)`, `pub(self)`, and
/// `pub(in path)` are treated as non-exported.
fn is_pub_item(node: &tree_sitter::Node, src: &[u8]) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "visibility_modifier" {
            let text = child.utf8_text(src).unwrap_or_default();
            return text == "pub";
        }
    }
    false
}

/// Extract the name from a Rust item declaration.
fn extract_item_name(node: &tree_sitter::Node, src: &[u8]) -> Option<String> {
    match node.kind() {
        "function_item" => get_field_text(node, "name", src),
        "struct_item" => get_field_text(node, "name", src),
        "enum_item" => get_field_text(node, "name", src),
        "trait_item" => get_field_text(node, "name", src),
        "type_item" => get_field_text(node, "name", src),
        "const_item" => get_field_text(node, "name", src),
        "static_item" => get_field_text(node, "name", src),
        "mod_item" => get_field_text(node, "name", src),
        // Attribute items (e.g. #[cfg(feature = "...")] pub fn ...)
        // The attribute is a sibling, tree-sitter still captures the item
        _ => None,
    }
}

fn get_field_text(node: &tree_sitter::Node, field: &str, src: &[u8]) -> Option<String> {
    node.child_by_field_name(field)
        .map(|n| n.utf8_text(src).unwrap_or_default().to_string())
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
    fn test_async_unsafe() {
        let src = r#"
pub async fn async_fn() {}
pub unsafe fn unsafe_fn() {}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["async_fn", "unsafe_fn"]);
    }

    #[test]
    fn test_ignores_pub_in_strings() {
        // AST inherently doesn't parse string contents as code
        let src = "pub fn real_fn() {}\nfn other() { let s = \"pub fn fake() {}\"; }\n";
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["real_fn"]);
    }

    #[test]
    fn test_feature_gated() {
        let src = r#"
#[cfg(feature = "optional")]
pub fn optional_fn() {}

pub fn always_fn() {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"optional_fn".to_string()));
        assert!(symbols.contains(&"always_fn".to_string()));
    }

    #[test]
    fn test_pub_mod() {
        let src = r#"
pub mod submodule;
pub mod inline_mod {
    pub fn inner() {}
}
mod private_mod;
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"submodule".to_string()));
        assert!(symbols.contains(&"inline_mod".to_string()));
        // `inner` has its own explicit `pub` keyword inside an inline `mod`
        // body, so it must be captured (matches the regex backend, which
        // finds it too since it scans linearly regardless of nesting).
        assert!(symbols.contains(&"inner".to_string()));
    }

    #[test]
    fn test_pub_items_inside_impl_block_are_captured() {
        // The most common shape of a real crate's public API: constructors,
        // builder methods, and accessors declared `pub` inside an `impl`
        // block on a `pub struct`. Each carries its own explicit `pub`
        // keyword, so it must not be dropped just because it's nested.
        let src = r#"
pub struct AuthConfig {
    pub base_url: String,
    pub timeout_ms: u64,
    retries: u8,
}

impl AuthConfig {
    pub fn new(base_url: impl Into<String>, timeout_ms: u64) -> Self {
        Self { base_url: base_url.into(), timeout_ms, retries: 0 }
    }

    pub fn with_retries(mut self, retries: u8) -> Self {
        self.retries = retries;
        self
    }

    pub fn retries(&self) -> u8 {
        self.retries
    }

    fn validate_internal(&self) -> bool {
        !self.base_url.is_empty()
    }
}

pub mod util {
    pub fn generate_token() -> String {
        "token".to_string()
    }
}
"#;
        let symbols = extract_exports(src);
        for name in [
            "AuthConfig",
            "new",
            "with_retries",
            "retries",
            "util",
            "generate_token",
        ] {
            assert!(
                symbols.contains(&name.to_string()),
                "missing {name}: {symbols:?}"
            );
        }
        assert!(
            !symbols.contains(&"validate_internal".to_string()),
            "validate_internal has no pub keyword and must not be captured: {symbols:?}"
        );
    }

    #[test]
    fn test_learnxinyminutes_preamble_no_false_positives() {
        // Real excerpt (verbatim, truncated mid-function) from
        // learnxinyminutes.com/rust.md: nested `/* /* */ */` block comments, a
        // `///` doc comment containing a markdown code fence, and stacked
        // `#[allow(...)]` attributes, none of which precede a `pub` item.
        // Also exercises the AST backend's tolerance of a truncated/unclosed
        // `fn main() {` body (must not panic or hang on a parse error node).
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
        // `fn`, and a `type` alias -- none carrying a `pub` keyword. Rust
        // forbids visibility modifiers on trait-impl items, so `frobnicate`
        // inside `impl<T> Frobnicate<T> for Foo<T>` must not be captured even
        // though the backend recurses into `impl_item` bodies.
        let src = r#"
    trait Frobnicate<T> {
        fn frobnicate(self) -> Option<T>;
    }

    impl<T> Frobnicate<T> for Foo<T> {
        fn frobnicate(self) -> Option<T> {
            Some(self.bar)
        }
    }

    fn fibonacci(n: u32) -> u32 {
        match n {
            0 => 1,
            1 => 1,
            _ => fibonacci(n - 1) + fibonacci(n - 2),
        }
    }

    type FunctionPointer = fn(u32) -> u32;
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.is_empty(),
            "no pub items in this excerpt, but got: {symbols:?}"
        );
    }

    #[test]
    fn test_pub_super_and_pub_in_path_items_excluded() {
        let src = r#"
pub(super) fn helper_for_parent() {}
pub(in crate::auth) struct ScopedConfig {}
pub(crate) fn crate_visible() {}
pub fn truly_public() {}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["truly_public"]);
    }
}
