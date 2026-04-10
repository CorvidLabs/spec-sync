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
/// - `pub(crate)` items
/// - `pub async fn`, `pub unsafe fn`
/// - Feature-gated exports (`#[cfg(feature = "...")]`)
/// - Correctly ignores `pub` inside string literals and comments (AST-native)
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
        // Only look at top-level items (source_file children)
        if is_pub_item(&child, src) {
            if let Some(name) = extract_item_name(&child, src) {
                symbols.push(name);
            }
        }
    }
}

/// Check if a node has `pub` or `pub(crate)` visibility.
fn is_pub_item(node: &tree_sitter::Node, src: &[u8]) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "visibility_modifier" {
            let text = child.utf8_text(src).unwrap_or_default();
            return text.starts_with("pub");
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
    fn test_pub_crate() {
        let src = r#"
pub(crate) fn internal_fn() {}
pub(crate) struct InternalStruct {}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["internal_fn", "InternalStruct"]);
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
        // inner is not top-level
        assert!(!symbols.contains(&"inner".to_string()));
    }
}
