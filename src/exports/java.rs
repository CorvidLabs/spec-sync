use regex::Regex;
use std::sync::LazyLock;

static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"//.*$").unwrap());

static COMMENT_MULTI: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/").unwrap());

/// Java public declarations: public class, interface, enum, record, @interface (annotation)
static JAVA_TYPE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*public\s+(?:static\s+)?(?:final\s+)?(?:abstract\s+)?(?:sealed\s+)?(?:class|interface|enum|record|@interface)\s+(\w+)",
    )
    .unwrap()
});

/// Java public methods and fields
static JAVA_MEMBER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*public\s+(?:static\s+)?(?:final\s+)?(?:synchronized\s+)?(?:abstract\s+)?(?:native\s+)?(?:<[^>]+>\s+)?(?:\w+(?:<[^>]*>)?(?:\[\])*)\s+(\w+)\s*[({;=]",
    )
    .unwrap()
});

/// Extract public symbols from Java source code.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = COMMENT_SINGLE.replace_all(content, "");
    let stripped = COMMENT_MULTI.replace_all(&stripped, "");

    let mut symbols = Vec::new();

    for caps in JAVA_TYPE.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            symbols.push(name.as_str().to_string());
        }
    }

    for caps in JAVA_MEMBER.captures_iter(&stripped) {
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
    fn test_java_exports() {
        let src = r#"
package com.example.auth;

public class AuthService {
    public static final String DEFAULT_TOKEN = "abc";
    public String validate(String token) {}
    private void internalCheck() {}
    public int getTimeout() {}
}

public interface Authenticator {}
public enum AuthStatus { ACTIVE, EXPIRED }
public record UserProfile(String name, int age) {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"AuthService".to_string()));
        assert!(symbols.contains(&"Authenticator".to_string()));
        assert!(symbols.contains(&"AuthStatus".to_string()));
        assert!(symbols.contains(&"UserProfile".to_string()));
        assert!(symbols.contains(&"DEFAULT_TOKEN".to_string()));
        assert!(symbols.contains(&"validate".to_string()));
        assert!(symbols.contains(&"getTimeout".to_string()));
        assert!(!symbols.contains(&"internalCheck".to_string()));
    }

    #[test]
    fn test_java_abstract() {
        let src = r#"
public abstract class BaseController {
    public abstract void handle();
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"BaseController".to_string()));
        assert!(symbols.contains(&"handle".to_string()));
    }
}
