use regex::Regex;
use std::sync::LazyLock;

/// __all__ = ["Name1", "Name2"]
static ALL_DECL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"__all__\s*=\s*\[([^\]]*)\]"#).unwrap());

/// Top-level def name( or class Name
static TOP_LEVEL_DECL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^(?:def|class|async def)\s+(\w+)").unwrap());

/// Quoted string in __all__
static QUOTED: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"["'](\w+)["']"#).unwrap());

/// Extract exported symbols from Python source code.
/// If `__all__` is defined, use that. Otherwise, all top-level
/// functions and classes that don't start with `_` are considered public.
pub fn extract_exports(content: &str) -> Vec<String> {
    // Check for __all__ first
    if let Some(caps) = ALL_DECL.captures(content)
        && let Some(list) = caps.get(1)
    {
        let mut symbols = Vec::new();
        for name_cap in QUOTED.captures_iter(list.as_str()) {
            if let Some(name) = name_cap.get(1) {
                symbols.push(name.as_str().to_string());
            }
        }
        return symbols;
    }

    // Fallback: top-level def/class that don't start with _
    let mut symbols = Vec::new();
    for caps in TOP_LEVEL_DECL.captures_iter(content) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str();
            if !n.starts_with('_') {
                symbols.push(n.to_string());
            }
        }
    }

    symbols
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

DEFAULT_TTL = 3600
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
    fn test_python_all_single_quotes() {
        let src = r#"
__all__ = ['Foo', 'bar_func']

class Foo:
    pass

def bar_func():
    pass

class _Hidden:
    pass
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Foo", "bar_func"]);
    }

    #[test]
    fn test_python_all_overrides_conventions() {
        // When __all__ is present, even underscore-prefixed names are exported if listed
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
    fn test_python_decorators_ignored() {
        let src = r#"
@dataclass
class Config:
    host: str

@staticmethod
def create():
    pass

@app.route("/")
async def index():
    pass
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Config".to_string()));
        assert!(symbols.contains(&"create".to_string()));
        assert!(symbols.contains(&"index".to_string()));
    }

    #[test]
    fn test_python_nested_not_captured() {
        // Only top-level (no indentation) defs/classes should be captured
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
        // Inner and method are indented, should not be captured
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
        assert!(!symbols.contains(&"__repr__".to_string()));
    }
}
