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

    #[test]
    fn test_java_comments_stripped() {
        let src = r#"
// public class FakeClass {}
/* public interface FakeInterface {} */
/**
 * Javadoc comment
 * public enum FakeEnum {}
 */
public class RealClass {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"RealClass".to_string()));
        assert!(!symbols.contains(&"FakeClass".to_string()));
        assert!(!symbols.contains(&"FakeInterface".to_string()));
        assert!(!symbols.contains(&"FakeEnum".to_string()));
    }

    #[test]
    fn test_java_generics() {
        let src = r#"
public class Repository<T extends Entity> {
    public <E> List<E> findAll() {}
    public Map<String, Object> getMetadata() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Repository".to_string()));
        assert!(symbols.contains(&"findAll".to_string()));
        assert!(symbols.contains(&"getMetadata".to_string()));
    }

    #[test]
    fn test_java_annotation_type() {
        let src = r#"
public @interface Cacheable {
    int ttl() default 300;
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Cacheable".to_string()));
    }

    #[test]
    fn test_java_private_and_protected_excluded() {
        let src = r#"
public class Service {
    private String secret;
    protected void onInit() {}
    void packagePrivate() {}
    public String getName() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Service".to_string()));
        assert!(symbols.contains(&"getName".to_string()));
        assert!(!symbols.contains(&"secret".to_string()));
        assert!(!symbols.contains(&"onInit".to_string()));
        assert!(!symbols.contains(&"packagePrivate".to_string()));
    }

    #[test]
    fn test_java_sealed_class() {
        let src = r#"
public sealed class Shape permits Circle, Rectangle {
    public String describe() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Shape".to_string()));
        assert!(symbols.contains(&"describe".to_string()));
    }

    #[test]
    fn test_java_static_final_combined() {
        let src = r#"
public class Constants {
    public static final int MAX_SIZE = 100;
    public static final String VERSION = "1.0";
    public static void init() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Constants".to_string()));
        assert!(symbols.contains(&"MAX_SIZE".to_string()));
        assert!(symbols.contains(&"VERSION".to_string()));
        assert!(symbols.contains(&"init".to_string()));
    }
}
