use regex::Regex;
use std::sync::LazyLock;

static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"//.*$").unwrap());

static COMMENT_MULTI: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/").unwrap());

/// Swift public/open declarations:
/// public/open func, class, struct, enum, protocol, typealias, var, let, actor
static SWIFT_DECL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)(?:public|open)\s+(?:static\s+)?(?:class\s+)?(?:func|class|struct|enum|protocol|typealias|var|let|actor|init)\s+(\w+)",
    )
    .unwrap()
});

/// Swift init doesn't have a name — detect public init separately
static SWIFT_INIT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)(?:public|open)\s+(?:required\s+)?(?:convenience\s+)?init\s*\(").unwrap()
});

/// Extract exported (public/open) symbols from Swift source code.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = COMMENT_SINGLE.replace_all(content, "");
    let stripped = COMMENT_MULTI.replace_all(&stripped, "");

    let mut symbols = Vec::new();

    for caps in SWIFT_DECL.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            symbols.push(name.as_str().to_string());
        }
    }

    // Count public inits (they don't have a standalone name)
    if SWIFT_INIT.is_match(&stripped) {
        symbols.push("init".to_string());
    }

    symbols
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swift_exports() {
        let src = r#"
public class AuthService {
    public var token: String
    public let apiVersion: Int
    public func validate() -> Bool {}
    private func internalCheck() {}
    public static func shared() -> AuthService {}
}
public struct Config {}
public enum AuthStatus { case active, expired }
public protocol Authenticator {}
public typealias Token = String
open class BaseController {}
public actor SessionManager {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"AuthService".to_string()));
        assert!(symbols.contains(&"token".to_string()));
        assert!(symbols.contains(&"apiVersion".to_string()));
        assert!(symbols.contains(&"validate".to_string()));
        assert!(symbols.contains(&"shared".to_string()));
        assert!(symbols.contains(&"Config".to_string()));
        assert!(symbols.contains(&"AuthStatus".to_string()));
        assert!(symbols.contains(&"Authenticator".to_string()));
        assert!(symbols.contains(&"Token".to_string()));
        assert!(symbols.contains(&"BaseController".to_string()));
        assert!(symbols.contains(&"SessionManager".to_string()));
        assert!(!symbols.contains(&"internalCheck".to_string()));
    }

    #[test]
    fn test_swift_init() {
        let src = r#"
public class Foo {
    public init(name: String) {}
    public convenience init() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Foo".to_string()));
        assert!(symbols.contains(&"init".to_string()));
    }

    #[test]
    fn test_swift_open() {
        let src = r#"
open class BaseView {
    open func layoutSubviews() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"BaseView".to_string()));
        assert!(symbols.contains(&"layoutSubviews".to_string()));
    }
}
