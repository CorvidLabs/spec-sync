use regex::Regex;
use std::sync::LazyLock;

static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"//.*$").unwrap());

static COMMENT_MULTI: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/").unwrap());

/// C# public/internal types: class, struct, interface, enum, record, delegate
static CS_TYPE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*public\s+(?:static\s+)?(?:partial\s+)?(?:sealed\s+)?(?:abstract\s+)?(?:class|struct|interface|enum|record|delegate)\s+(\w+)",
    )
    .unwrap()
});

/// C# public methods and properties
static CS_MEMBER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*public\s+(?:static\s+)?(?:virtual\s+)?(?:override\s+)?(?:abstract\s+)?(?:async\s+)?(?:new\s+)?(?:\w+(?:<[^>]*>)?(?:\[\])?(?:\?)?)\s+(\w+)\s*[({;]",
    )
    .unwrap()
});

/// Extract public symbols from C# source code.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = COMMENT_SINGLE.replace_all(content, "");
    let stripped = COMMENT_MULTI.replace_all(&stripped, "");

    let mut symbols = Vec::new();

    for caps in CS_TYPE.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            symbols.push(name.as_str().to_string());
        }
    }

    for caps in CS_MEMBER.captures_iter(&stripped) {
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
    fn test_csharp_exports() {
        let src = r#"
namespace Example.Auth;

public class AuthService {
    public string Validate(string token) {}
    private void InternalCheck() {}
    public static AuthService Instance { get; }
    public int Timeout;
}

public interface IAuthenticator {}
public enum AuthStatus { Active, Expired }
public record UserProfile(string Name, int Age);
public struct Config {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"AuthService".to_string()));
        assert!(symbols.contains(&"Validate".to_string()));
        assert!(symbols.contains(&"IAuthenticator".to_string()));
        assert!(symbols.contains(&"AuthStatus".to_string()));
        assert!(symbols.contains(&"UserProfile".to_string()));
        assert!(symbols.contains(&"Config".to_string()));
        assert!(!symbols.contains(&"InternalCheck".to_string()));
    }

    #[test]
    fn test_csharp_async() {
        let src = r#"
public class Service {
    public async Task<string> FetchData() {}
    public virtual void OnUpdate() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Service".to_string()));
        assert!(symbols.contains(&"FetchData".to_string()));
        assert!(symbols.contains(&"OnUpdate".to_string()));
    }
}
