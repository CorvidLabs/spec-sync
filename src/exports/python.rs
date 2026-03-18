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
}
