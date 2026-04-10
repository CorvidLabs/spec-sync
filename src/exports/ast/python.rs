use tree_sitter::{Parser, Tree};

/// Parse Python source into a tree-sitter AST.
fn parse_py(content: &str) -> Option<Tree> {
    let mut parser = Parser::new();
    let language = tree_sitter_python::LANGUAGE.into();
    parser.set_language(&language).ok()?;
    parser.parse(content, None)
}

/// Extract exported symbols from Python source using tree-sitter AST.
///
/// Handles:
/// - `__all__` list (takes precedence if present)
/// - Top-level `def`, `async def`, `class` (excluding `_`-prefixed)
/// - Conditional imports in `__init__.py` patterns
pub fn extract_exports(content: &str) -> Vec<String> {
    let tree = match parse_py(content) {
        Some(t) => t,
        None => return Vec::new(),
    };

    let root = tree.root_node();
    let src = content.as_bytes();

    // First, look for __all__ assignment
    if let Some(all_symbols) = find_dunder_all(&root, src) {
        return all_symbols;
    }

    // Fallback: top-level def/class not starting with _
    let mut symbols = Vec::new();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor) {
        match child.kind() {
            "function_definition" => {
                if let Some(name) = child.child_by_field_name("name") {
                    let n = name.utf8_text(src).unwrap_or_default();
                    if !n.starts_with('_') {
                        symbols.push(n.to_string());
                    }
                }
            }
            "class_definition" => {
                if let Some(name) = child.child_by_field_name("name") {
                    let n = name.utf8_text(src).unwrap_or_default();
                    if !n.starts_with('_') {
                        symbols.push(n.to_string());
                    }
                }
            }
            // `@decorator\nasync def ...` — decorated definitions
            "decorated_definition" => {
                let mut inner_cursor = child.walk();
                for inner in child.children(&mut inner_cursor) {
                    if inner.kind() == "function_definition" || inner.kind() == "class_definition" {
                        if let Some(name) = inner.child_by_field_name("name") {
                            let n = name.utf8_text(src).unwrap_or_default();
                            if !n.starts_with('_') {
                                symbols.push(n.to_string());
                            }
                        }
                    }
                }
            }
            // Conditional imports: `if TYPE_CHECKING:` blocks at top level
            // We still don't export from these — they're type-only
            _ => {}
        }
    }

    symbols
}

/// Look for `__all__ = [...]` at the top level and extract symbol names.
fn find_dunder_all(root: &tree_sitter::Node, src: &[u8]) -> Option<Vec<String>> {
    let mut cursor = root.walk();

    for child in root.children(&mut cursor) {
        if child.kind() == "expression_statement" {
            let mut stmt_cursor = child.walk();
            for stmt_child in child.children(&mut stmt_cursor) {
                if stmt_child.kind() == "assignment" {
                    let left = stmt_child.child_by_field_name("left")?;
                    let left_text = left.utf8_text(src).unwrap_or_default();

                    if left_text == "__all__" {
                        let right = stmt_child.child_by_field_name("right")?;
                        if right.kind() == "list" {
                            return Some(extract_string_list(&right, src));
                        }
                    }
                }
            }
        }
    }

    None
}

/// Extract string values from a list literal: `["foo", "bar"]`.
fn extract_string_list(node: &tree_sitter::Node, src: &[u8]) -> Vec<String> {
    let mut names = Vec::new();
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        if child.kind() == "string" {
            // String node contains string_content child(ren)
            let text = child.utf8_text(src).unwrap_or_default();
            // Strip surrounding quotes
            let trimmed = text
                .trim_start_matches(|c| c == '\'' || c == '"')
                .trim_end_matches(|c| c == '\'' || c == '"');
            if !trimmed.is_empty() {
                names.push(trimmed.to_string());
            }
        }
    }

    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_all() {
        let src = r#"
__all__ = ["create_auth", "AuthService", "DEFAULT_TTL"]

def create_auth(config):
    pass

class AuthService:
    pass

def _internal():
    pass
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["create_auth", "AuthService", "DEFAULT_TTL"]);
    }

    #[test]
    fn test_python_no_all() {
        let src = r#"
def create_auth(config):
    pass

class AuthService:
    pass

def _internal():
    pass

async def fetch_token():
    pass
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["create_auth", "AuthService", "fetch_token"]);
    }

    #[test]
    fn test_python_nested_not_captured() {
        let src = r#"
class Outer:
    class Inner:
        pass
    def method(self):
        pass

def top_level():
    def nested():
        pass
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Outer".to_string()));
        assert!(symbols.contains(&"top_level".to_string()));
        assert!(!symbols.contains(&"Inner".to_string()));
        assert!(!symbols.contains(&"method".to_string()));
        assert!(!symbols.contains(&"nested".to_string()));
    }

    #[test]
    fn test_python_dunder_excluded() {
        let src = r#"
def __init__(self):
    pass

def __repr__(self):
    pass

def public_func():
    pass
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"public_func".to_string()));
        assert!(!symbols.contains(&"__init__".to_string()));
    }

    #[test]
    fn test_python_all_overrides() {
        let src = r#"
__all__ = ["_special", "Public"]

def _special():
    pass

class Public:
    pass

class AlsoPublicButNotInAll:
    pass
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"_special".to_string()));
        assert!(symbols.contains(&"Public".to_string()));
        assert!(!symbols.contains(&"AlsoPublicButNotInAll".to_string()));
    }

    #[test]
    fn test_decorated_functions() {
        let src = r#"
@dataclass
class Config:
    host: str

@staticmethod
def create():
    pass
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Config".to_string()));
        assert!(symbols.contains(&"create".to_string()));
    }

    #[test]
    fn test_conditional_import_init() {
        // __init__.py pattern with conditional imports
        let src = r#"
__all__ = ["Router", "middleware"]

from .router import Router
from .middleware import middleware

try:
    from .optional import OptionalFeature
except ImportError:
    pass
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Router", "middleware"]);
    }
}
