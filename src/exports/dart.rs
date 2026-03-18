use regex::Regex;
use std::sync::LazyLock;

static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"//.*$").unwrap());

static COMMENT_MULTI: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/").unwrap());

/// Dart top-level declarations: class, mixin, enum, extension, typedef
/// In Dart, anything NOT prefixed with _ is public.
static DART_TYPE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^(?:abstract\s+)?(?:class|mixin|enum|extension|typedef)\s+([A-Z]\w*)").unwrap()
});

/// Dart top-level functions and variables (public = no underscore prefix)
static DART_TOPLEVEL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^(?:Future<[^>]*>\s+|Stream<[^>]*>\s+|void\s+|int\s+|double\s+|String\s+|bool\s+|List<[^>]*>\s+|Map<[^>]*>\s+|Set<[^>]*>\s+|dynamic\s+|\w+\s+)([a-zA-Z]\w*)\s*[({=;]",
    )
    .unwrap()
});

/// Dart top-level `final` and `const` declarations
static DART_CONST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^(?:final|const)\s+(?:\w+\s+)?([a-zA-Z]\w*)\s*[=;]").unwrap()
});

/// Extract public symbols from Dart source code.
/// In Dart, identifiers starting with _ are private; everything else is public.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = COMMENT_SINGLE.replace_all(content, "");
    let stripped = COMMENT_MULTI.replace_all(&stripped, "");

    let mut symbols = Vec::new();

    for caps in DART_TYPE.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str();
            if !n.starts_with('_') {
                symbols.push(n.to_string());
            }
        }
    }

    for caps in DART_CONST.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str();
            if !n.starts_with('_') && !symbols.contains(&n.to_string()) {
                symbols.push(n.to_string());
            }
        }
    }

    for caps in DART_TOPLEVEL.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str();
            if !n.starts_with('_') && !symbols.contains(&n.to_string()) {
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
    fn test_dart_exports() {
        let src = r#"
class AuthService {
  String validate(String token) {}
}

abstract class BaseController {}
mixin LoggerMixin {}
enum AuthStatus { active, expired }
typedef AuthCallback = void Function(String);
const defaultTtl = 3600;
final String apiVersion = "1.0";
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"AuthService".to_string()));
        assert!(symbols.contains(&"BaseController".to_string()));
        assert!(symbols.contains(&"LoggerMixin".to_string()));
        assert!(symbols.contains(&"AuthStatus".to_string()));
        assert!(symbols.contains(&"AuthCallback".to_string()));
        assert!(symbols.contains(&"defaultTtl".to_string()));
        assert!(symbols.contains(&"apiVersion".to_string()));
    }

    #[test]
    fn test_dart_private() {
        let src = r#"
class _InternalHelper {}
void _privateFunc() {}
const _secret = "hidden";
class PublicClass {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"PublicClass".to_string()));
        assert!(!symbols.contains(&"_InternalHelper".to_string()));
        assert!(!symbols.contains(&"_privateFunc".to_string()));
        assert!(!symbols.contains(&"_secret".to_string()));
    }
}
