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
/// - `pub` items nested inside `impl` blocks and `mod { ... }` bodies, but
///   only when the enclosing `mod` is itself genuinely `pub` (a `pub` item
///   inside a private `mod` is not part of the crate's public API surface)
/// - `pub use` re-exports: single (`pub use a::B;`), aliased (`... as C`),
///   grouped/brace-nested (`pub use a::{B, C as D, E::F}`), and a bare
///   module re-export (`pub use foo;`). Glob re-exports (`pub use a::*;`)
///   and a bare `self` inside a group are intentionally not expanded, since
///   neither has a single statically-enumerable name from this file alone.
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

    // Once tree-sitter's error recovery is genuinely confused (e.g. leftover
    // merge-conflict markers, or other severely malformed input) it can
    // flatten valid, `pub`-tagged declarations into unstructured ERROR-node
    // tokens with no recoverable `function_item`/`visibility_modifier`
    // structure at all -- silently under-counting exports *without* the
    // overall result being empty, so the caller's "empty result -> fall
    // back to the regex backend" policy never kicks in. Once any parse
    // error is present anywhere in the tree we can no longer trust the
    // traversal's completeness, so deliberately return empty here to hand
    // off to the regex backend (which has no such structural blind spot
    // since it scans raw text and never requires a balanced/valid parse).
    if root.has_error() {
        return Vec::new();
    }

    let src = content.as_bytes();
    let mut symbols = Vec::new();

    collect_pub_items(&root, src, &mut symbols);

    symbols
}

fn collect_pub_items(node: &tree_sitter::Node, src: &[u8], symbols: &mut Vec<String>) {
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        let child_is_pub = is_pub_item(&child, src);

        if child_is_pub {
            if child.kind() == "use_declaration" {
                // `pub use ...;` re-exports: the exported name(s) live inside
                // the `argument` use-tree (a bare path, an `as`-aliased path,
                // a brace-grouped/nested list, or a glob), not as a `name`
                // field on the declaration itself. Handled by a dedicated
                // recursive walk mirroring the regex backend's
                // PUB_USE_SINGLE/PUB_USE_GROUP semantics.
                if let Some(argument) = child.child_by_field_name("argument") {
                    collect_use_tree_names(&argument, src, symbols);
                }
            } else if let Some(name) = extract_item_name(&child, src) {
                symbols.push(name);
            }
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
            "impl_item" => {
                if let Some(body) = child.child_by_field_name("body") {
                    collect_pub_items(&body, src, symbols);
                }
            }
            "mod_item" => {
                // A `pub` item declared inside a *private* `mod { ... }` body
                // is NOT actually reachable from outside the module: Rust's
                // effective visibility of a path is capped by the
                // least-visible ancestor module, so nothing beneath a
                // non-`pub` `mod` is part of the crate's public API surface,
                // no matter what visibility its own members declare (e.g.
                // `mod private_helpers { pub fn leaked() {} }` does NOT
                // export `leaked`). Only descend when the module itself
                // carries its own bare `pub` keyword; the crate root (this
                // function's initial caller) has no visibility modifier of
                // its own and is always the public starting point, so it is
                // unaffected by this gate.
                if child_is_pub && let Some(body) = child.child_by_field_name("body") {
                    collect_pub_items(&body, src, symbols);
                }
            }
            _ => {}
        }
    }
}

/// Recursively collect the leaf name(s) introduced by a `use` tree (the
/// `argument` of a `use_declaration`), covering every shape
/// tree-sitter-rust produces: a bare path (`foo::Bar` -> `Bar`), an aliased
/// path (`foo::Bar as Baz` -> `Baz`), a brace-grouped list (`foo::{a, b as
/// c, d::e}`, arbitrarily nested), and a bare re-export of a single
/// identifier (`pub use foo;` -> `foo`). Glob re-exports (`foo::*`) and a
/// bare `self` inside a group (`foo::{self, Bar}`) are intentionally
/// skipped: neither introduces a single statically-enumerable name, mirroring
/// the regex backend's PUB_USE_SINGLE/PUB_USE_GROUP, which leave globs
/// unexpanded and drop bare `self` entries for the same reason.
fn collect_use_tree_names(node: &tree_sitter::Node, src: &[u8], symbols: &mut Vec<String>) {
    match node.kind() {
        "identifier" => {
            if let Ok(text) = node.utf8_text(src) {
                symbols.push(text.to_string());
            }
        }
        "scoped_identifier" => {
            if let Some(name) = get_field_text(node, "name", src) {
                symbols.push(name);
            }
        }
        "use_as_clause" => {
            if let Some(alias) = get_field_text(node, "alias", src) {
                symbols.push(alias);
            }
        }
        "use_list" => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                collect_use_tree_names(&child, src, symbols);
            }
        }
        "scoped_use_list" => {
            // Only the nested `{ ... }` list holds re-exported leaf names;
            // the leading path (e.g. `a` in `a::{b, c}`) is just a namespace
            // prefix, not itself an exported symbol.
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "use_list" {
                    collect_use_tree_names(&child, src, symbols);
                }
            }
        }
        // "use_wildcard" (`foo::*`) and "self" (bare `self` inside a group)
        // intentionally fall through to the no-op default: neither has a
        // single statically-enumerable name to contribute.
        _ => {}
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

    #[test]
    fn test_struct_and_tuple_struct_fields_not_captured() {
        // A struct's own `pub` fields (including a tuple struct's positional
        // fields) are not top-level items and must not be captured just
        // because they carry their own `pub` keyword; only the struct name
        // itself is exported. Verified empirically: without this guard the
        // bug would show up as extra entries like "base_url"/"timeout_ms".
        let src = r#"
pub struct AuthConfig {
    pub base_url: String,
    pub timeout_ms: u64,
    retries: u8,
}

pub struct Wrapper(pub i32, i64);
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["AuthConfig", "Wrapper"]);
    }

    #[test]
    fn test_deeply_nested_pub_mod_two_levels() {
        // A `pub mod` nested inside another `pub mod` (two levels deep, not
        // just the single level the existing test_pub_mod covers) must still
        // have its inner `pub fn` recovered via recursive descent.
        let src = r#"
pub mod a {
    pub mod b {
        pub fn c() {}
    }
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_multiline_generic_where_clause() {
        // A `pub fn` with generics and a `where` clause spanning multiple
        // lines before its body -- the AST must not lose track of the name
        // just because the signature is not on one line.
        let src = r#"
pub fn complex<T>(x: T) -> T
where
    T: Clone + std::fmt::Debug,
{
    x
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["complex"]);
    }

    #[test]
    fn test_const_fn_and_unsafe_extern_fn() {
        // Unusual modifier orderings/combinations: `pub const fn` and
        // `pub unsafe extern "C" fn` must both still resolve to a plain
        // `function_item` with the visibility_modifier as a direct child.
        let src = r#"
pub const fn const_fn_example() -> i32 { 1 }
pub unsafe extern "C" fn ffi_fn() {}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["const_fn_example", "ffi_fn"]);
    }

    #[test]
    fn test_raw_string_containing_pub_fn_text_ignored() {
        // A raw string literal whose contents look like a `pub fn`
        // declaration must not be read as code (AST-native guarantee).
        let src = r####"
pub fn real_fn() {}
fn other() {
    let s = r#"pub fn fake_in_raw_string() {}"#;
}
"####;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["real_fn"]);
    }

    #[test]
    fn test_doc_comment_with_code_like_text_ignored() {
        // A `///` doc comment containing text that looks exactly like a
        // `pub fn` declaration must not be parsed as one.
        let src = r#"
/// pub fn fake_doc_fn() {}
fn real_private() {}

pub fn real_pub() {}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["real_pub"]);
    }

    #[test]
    fn test_const_generics_struct_name_captured() {
        // `pub struct Foo<const N: usize>` -- a const generic parameter must
        // not confuse extraction of the struct's own name.
        let src = r#"
pub struct ArrayWrapper<const N: usize> {
    data: [i32; N],
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["ArrayWrapper"]);
    }

    #[test]
    fn test_assoc_const_and_type_captured_inside_impl() {
        // Associated `pub const` and `pub type` inside an inherent `impl`
        // block carry their own explicit `pub` keyword (mirrors the regex
        // backend, which matches them too since it scans linearly), so they
        // must be captured alongside the struct name.
        let src = r#"
pub struct Foo;

impl Foo {
    pub const BAR: i32 = 1;
    pub type Assoc = String;
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Foo", "BAR", "Assoc"]);
    }

    #[test]
    fn test_pub_in_multi_segment_path_excluded() {
        // `pub(in crate::a::b::c)` -- a deeper, multi-segment restricted
        // path (beyond the single-segment `pub(in crate::auth)` already
        // covered elsewhere) must still be excluded.
        let src = r#"
pub(in crate::a::b::c) fn deeply_scoped() {}
pub fn truly_public() {}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["truly_public"]);
    }

    #[test]
    fn test_macro_rules_never_captured() {
        // Macros are exported via `#[macro_export]`, not `pub` (`pub
        // macro_rules!` is not valid Rust syntax). Neither an exported nor a
        // private `macro_rules!` should ever appear in the results, since
        // "macro_rules_item" is not among the recognized item kinds.
        let src = r#"
#[macro_export]
macro_rules! my_macro {
    () => {};
}

macro_rules! private_macro {
    () => {};
}

pub fn real_fn() {}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["real_fn"]);
    }

    #[test]
    fn test_parse_error_anywhere_defers_to_regex_fallback() {
        // Bug found via adversarial testing: a syntax error earlier in the
        // file (here, a truncated/unbalanced `pub fn broken(`) can confuse
        // tree-sitter's error recovery badly enough that a subsequent,
        // genuinely well-formed top-level `pub fn` loses its
        // `visibility_modifier` entirely (tree-sitter mis-lexes the second
        // `pub` token as part of the broken statement before it), so no
        // traversal-level fix can recover it -- the information is gone
        // from the tree itself, not merely unreached.
        //
        // Before the `root.has_error()` guard in `extract_exports`, this
        // silently produced an empty result, which happens to coincide with
        // the caller's fallback trigger here -- but the fix is what makes
        // that outcome *intentional and guaranteed* rather than incidental,
        // which matters for the next (nonempty) case below.
        let src = r#"
pub fn broken( {
    let x =

pub fn also_public() {}
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.is_empty(),
            "a parse error anywhere must defer entirely to the regex fallback: {symbols:?}"
        );
        // The regex backend has no such blind spot and recovers everything.
        let regex_symbols = crate::exports::rust_lang::extract_exports(src);
        assert!(regex_symbols.contains(&"also_public".to_string()));
    }

    #[test]
    fn test_pub_items_inside_private_mod_are_not_exported() {
        // Regression: a `pub fn`/`pub struct` nested inside a *private*
        // `mod { ... }` was previously captured just because the item itself
        // carried a `pub` keyword. That is wrong: Rust's effective
        // visibility of a path is capped by the least-visible ancestor
        // module, so nothing beneath a non-`pub` `mod` is actually reachable
        // from outside the module -- `mod private_mod { pub fn leaked() {} }`
        // does NOT put `leaked` on the crate's public API surface. Verified
        // empirically before the fix: `extract_exports` returned
        // `["should_not_leak", "AlsoShouldNotLeak", "top_level_ok"]`, i.e. a
        // wrong-but-nonempty result that would have masked the bug from any
        // "empty result -> fall back to regex" safety net.
        let src = r#"
mod private_mod {
    pub fn should_not_leak() {}
    pub struct AlsoShouldNotLeak;
}
pub fn top_level_ok() {}
"#;
        let symbols = extract_exports(src);
        assert_eq!(
            symbols,
            vec!["top_level_ok"],
            "pub items inside a private mod must not be treated as exported: {symbols:?}"
        );
    }

    #[test]
    fn test_pub_items_inside_nested_private_mod_under_pub_mod_are_not_exported() {
        // Regression: the private-mod gate must apply at every nesting
        // level, not just the outermost one. `pub mod outer { mod inner {
        // pub fn leaked() {} } }` -- `inner` is private even though `outer`
        // is `pub`, so `leaked` is still not reachable from outside the
        // crate.
        let src = r#"
pub mod outer {
    mod inner {
        pub fn leaked() {}
    }
    pub fn visible() {}
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["outer", "visible"]);
    }

    #[test]
    fn test_pub_use_single_reexport_captured() {
        // Regression: `pub use` re-exports were not recognized at all by the
        // AST backend (`use_declaration` was entirely absent from
        // `extract_item_name`'s match), even though re-exporting a
        // submodule's type at the crate root (`pub use crate::foo::Bar;`) is
        // one of the most common Rust public-API patterns (idiomatic in
        // almost every `lib.rs`). This silently under-reported exports on
        // any file containing at least one other captured item, since the
        // result was nonempty and so never triggered the regex fallback.
        let src = r#"
pub use crate::foo::Bar;
pub use super::baz::Qux as Renamed;
pub fn also_here() {}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Bar", "Renamed", "also_here"]);
    }

    #[test]
    fn test_pub_use_bare_module_reexport_captured() {
        // `pub use foo;` (no path segments) re-exports the identifier itself.
        let src = "pub use foo;\n";
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["foo"]);
    }

    #[test]
    fn test_pub_use_grouped_and_nested_reexports_captured() {
        // Grouped re-exports (`{a, b as c, d::e}`), including arbitrarily
        // nested groups (`{a::{b, c}, d}`) and a bare `self` entry (which
        // re-exports the parent path segment itself, not a new name, and
        // must not appear as a spurious "self" symbol).
        let src = r#"
pub use crate::baz::{Qux, Quux as Renamed, mod_a::mod_b};
pub use crate::nested::{x::{y, z}, w};
pub use crate::withself::{self, Kept};
"#;
        let symbols = extract_exports(src);
        assert_eq!(
            symbols,
            vec!["Qux", "Renamed", "mod_b", "y", "z", "w", "Kept"]
        );
        assert!(
            !symbols.iter().any(|s| s == "self"),
            "bare `self` in a grouped use list must not appear as a symbol: {symbols:?}"
        );
    }

    #[test]
    fn test_pub_use_glob_not_expanded() {
        // A glob re-export (`pub use foo::*;`) has no single
        // statically-enumerable name and must not leak the module segment
        // ("glob") as a fake export, mirroring the regex backend's
        // intentionally-unexpanded glob handling.
        let src = r#"
pub use crate::glob::*;
pub fn real_fn() {}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["real_fn"]);
    }

    #[test]
    fn test_pub_crate_use_reexport_excluded() {
        // `pub(crate) use ...;` is a restricted-visibility re-export and must
        // be excluded just like any other `pub(crate)` item.
        let src = r#"
pub(crate) use crate::internal::Hidden;
pub use crate::public::Visible;
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Visible"]);
    }

    #[test]
    fn test_pub_use_reexport_inside_private_mod_excluded() {
        // The private-mod reachability gate applies to `pub use` re-exports
        // nested inside a private `mod` just as it does to any other `pub`
        // item: a `pub use` inside a non-`pub` module is not reachable from
        // outside the crate.
        let src = r#"
mod private_mod {
    pub use crate::foo::Bar;
}
pub fn top_level_ok() {}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["top_level_ok"]);
    }

    #[test]
    fn test_trait_body_members_never_captured_even_for_pub_trait() {
        // Rust forbids explicit visibility modifiers on trait body members
        // (`type Assoc;`, `const BAZ: i32;`, `fn bar(&self);`) -- they are
        // implicitly public whenever the trait itself is `pub`, but never
        // carry their own `pub` keyword, and `trait_item` is intentionally
        // absent from the recursion targets (only `impl_item`/`mod_item`
        // are descended into). Only the trait's own name should be captured.
        let src = r#"
pub trait Foo {
    type Assoc;
    const BAZ: i32;
    fn bar(&self);
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Foo"]);
    }

    #[test]
    fn test_doc_hidden_pub_item_still_captured() {
        // Verified-correct-but-previously-untested: `#[doc(hidden)]` merely
        // hides an item from generated documentation, it does not change its
        // actual visibility -- a `#[doc(hidden)] pub fn` is still a real,
        // callable part of the crate's public API surface (a very common
        // pattern for "public but not officially documented" internal-use
        // items), so it must still be captured just like any other `pub`
        // item preceded by an attribute (mirrors the existing `#[cfg(...)]`
        // handling).
        let src = r#"
#[doc(hidden)]
pub fn undocumented_but_public() {}

pub fn normal_fn() {}
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"undocumented_but_public".to_string()),
            "doc(hidden) pub items are still part of the public API: {symbols:?}"
        );
        assert!(symbols.contains(&"normal_fn".to_string()));
    }

    #[test]
    fn test_merge_conflict_markers_defer_to_regex_fallback() {
        // Bug found via adversarial testing: leftover merge-conflict markers
        // (a realistic real-world corruption) confuse tree-sitter badly
        // enough that only the first of four valid `pub fn` declarations
        // survives as a recognizable `function_item` -- the rest are
        // flattened into unstructured ERROR-node tokens with no
        // `visibility_modifier`/`function_item` structure left to recover.
        // Because the pre-fix result was *nonempty* (`["stable_fn"]`), the
        // caller's "empty result -> fall back to regex" policy never
        // triggered, so this silently under-reported exports with no safety
        // net. The `root.has_error()` guard fixes this by deliberately
        // returning empty whenever any parse error is present anywhere in
        // the tree, forcing the caller to use the regex backend instead,
        // which recovers all four names correctly.
        let src = r#"
pub fn stable_fn() {}

<<<<<<< HEAD
pub fn feature_a() {}
=======
pub fn feature_b() {}
>>>>>>> feature-branch

pub fn another_fn() {}
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.is_empty(),
            "a parse error anywhere must defer entirely to the regex fallback: {symbols:?}"
        );
        let regex_symbols = crate::exports::rust_lang::extract_exports(src);
        for name in ["stable_fn", "feature_a", "feature_b", "another_fn"] {
            assert!(
                regex_symbols.contains(&name.to_string()),
                "regex fallback should recover {name}: {regex_symbols:?}"
            );
        }
    }
}
