use regex::Regex;
use std::sync::LazyLock;

static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"//.*$").unwrap());

static COMMENT_MULTI: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/").unwrap());

/// Go exports: func Name, type Name, var Name, const Name
/// In Go, anything starting with uppercase is exported.
static GO_DECL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^(?:func|type|var|const)\s+(?:\([^)]*\)\s+)?([A-Z]\w*)").unwrap()
});

/// Go method: func (receiver) Name(...)
static GO_METHOD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^func\s+\([^)]+\)\s+([A-Z]\w*)").unwrap());

/// Extract exported symbols from Go source code.
/// In Go, any top-level identifier starting with an uppercase letter is exported.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = COMMENT_SINGLE.replace_all(content, "");
    let stripped = COMMENT_MULTI.replace_all(&stripped, "");

    let mut symbols = Vec::new();

    for caps in GO_DECL.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            symbols.push(name.as_str().to_string());
        }
    }

    // Also capture exported methods
    for caps in GO_METHOD.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str().to_string();
            if !symbols.contains(&n) {
                symbols.push(n);
            }
        }
    }

    symbols
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_go_exports() {
        let src = r#"
package auth

func CreateAuth(config Config) Auth {}
func privateFunc() {}
type AuthService struct {}
type authInternal struct {}
const DefaultTTL = 3600
var GlobalInstance *Auth
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"CreateAuth".to_string()));
        assert!(symbols.contains(&"AuthService".to_string()));
        assert!(symbols.contains(&"DefaultTTL".to_string()));
        assert!(symbols.contains(&"GlobalInstance".to_string()));
        assert!(!symbols.contains(&"privateFunc".to_string()));
        assert!(!symbols.contains(&"authInternal".to_string()));
    }

    #[test]
    fn test_go_methods() {
        let src = r#"
package auth

func (a *Auth) Validate(token string) bool {}
func (a *Auth) internal() {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Validate".to_string()));
        assert!(!symbols.contains(&"internal".to_string()));
    }
}
