use regex::Regex;
use std::sync::LazyLock;

/// Swift public/open declarations:
/// public/open func, class, struct, enum, protocol, typealias, var, let, actor
static SWIFT_DECL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)(?:public|open)\s+(?:(?:final|static|class)\s+)*(?:func|class|struct|enum|protocol|typealias|var|let|actor|init)\s+(\w+)",
    )
    .unwrap()
});

/// Swift init doesn't have a name — detect public init separately
static SWIFT_INIT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)(?:public|open)\s+(?:required\s+)?(?:convenience\s+)?init\s*\(").unwrap()
});

/// Extract exported (public/open) symbols from Swift source code.
pub fn extract_exports(content: &str) -> Vec<String> {
    // Strip strings AND comments in one pass so a declaration-shaped token inside a
    // string literal (e.g. a code-gen template `let t = "public final class X {}"`)
    // is not extracted as a phantom export, and a `"` in a comment is not read as a
    // string (and vice-versa).
    let stripped = strip_swift_strings_and_comments(content);

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

/// Blank Swift string literals (`"..."` and `"""..."""`), `//` line comments, and
/// (nesting) `/* */` block comments in one linear pass, so neither is misread as
/// the other and no declaration inside a string is extracted. Newlines preserved.
fn strip_swift_strings_and_comments(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < n {
        let c = chars[i];
        // Line comment.
        if c == '/' && i + 1 < n && chars[i + 1] == '/' {
            i += 2;
            while i < n && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        // Block comment (Swift block comments nest).
        if c == '/' && i + 1 < n && chars[i + 1] == '*' {
            i += 2;
            let mut depth = 1;
            while i < n && depth > 0 {
                if chars[i] == '/' && i + 1 < n && chars[i + 1] == '*' {
                    depth += 1;
                    i += 2;
                } else if chars[i] == '*' && i + 1 < n && chars[i + 1] == '/' {
                    depth -= 1;
                    i += 2;
                } else {
                    if chars[i] == '\n' {
                        out.push('\n');
                    }
                    i += 1;
                }
            }
            continue;
        }
        // Multi-line string literal `"""..."""`.
        if c == '"' && i + 2 < n && chars[i + 1] == '"' && chars[i + 2] == '"' {
            i += 3;
            while i < n {
                if chars[i] == '\\' {
                    i += 2;
                    continue;
                }
                if chars[i] == '"' && i + 2 < n && chars[i + 1] == '"' && chars[i + 2] == '"' {
                    i += 3;
                    break;
                }
                if chars[i] == '\n' {
                    out.push('\n');
                }
                i += 1;
            }
            continue;
        }
        // Single-line string literal `"..."`.
        if c == '"' {
            i += 1;
            while i < n {
                if chars[i] == '\\' {
                    i += 2;
                    continue;
                }
                if chars[i] == '"' {
                    i += 1;
                    break;
                }
                if chars[i] == '\n' {
                    out.push('\n');
                }
                i += 1;
            }
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
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

    #[test]
    fn test_swift_final_modifiers_exported() {
        // Finding #4: `final` (and combinations with static/class) between the
        // access keyword and the declaration keyword must not hide the export.
        let src = r#"
public final class Repository {}
open final class BaseService {}
public final func recompute() -> Int {}
public static let shared = Repository()
public class RegularClass {}
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"Repository".to_string()),
            "public final class"
        );
        assert!(
            symbols.contains(&"BaseService".to_string()),
            "open final class"
        );
        assert!(
            symbols.contains(&"recompute".to_string()),
            "public final func"
        );
        assert!(symbols.contains(&"shared".to_string()), "public static let");
        assert!(
            symbols.contains(&"RegularClass".to_string()),
            "plain public class still works"
        );
    }

    #[test]
    fn test_swift_declaration_inside_string_not_extracted() {
        // A declaration-shaped token inside a string literal (e.g. a code-gen
        // template) must not be extracted as a phantom export.
        let src = r#"
public let template = "public final class Injected {}"
public final class Real {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"template".to_string()));
        assert!(symbols.contains(&"Real".to_string()));
        assert!(
            !symbols.contains(&"Injected".to_string()),
            "declaration is inside a string literal"
        );
    }

    #[test]
    fn test_swift_declaration_inside_multiline_string_not_extracted() {
        let src = r#"
public let generator = """
public final class Generated {}
public struct AlsoFake {}
"""
public struct Keep {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"generator".to_string()));
        assert!(symbols.contains(&"Keep".to_string()));
        assert!(!symbols.contains(&"Generated".to_string()));
        assert!(!symbols.contains(&"AlsoFake".to_string()));
    }
}
