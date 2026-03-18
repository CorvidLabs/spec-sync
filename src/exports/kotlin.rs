use regex::Regex;
use std::sync::LazyLock;

static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"//.*$").unwrap());

static COMMENT_MULTI: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/").unwrap());

/// Kotlin top-level declarations (everything not marked private/internal/protected is public by default).
/// We match: fun, class, object, interface, typealias, val, var, enum class, data class, sealed class, annotation class
/// Then exclude lines that start with private/internal/protected.
static KT_DECL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*(?:(?:public|internal|private|protected)\s+)?(?:suspend\s+)?(?:inline\s+)?(?:data\s+|sealed\s+|enum\s+|annotation\s+|abstract\s+|open\s+)?(?:fun|class|object|interface|typealias|val|var)\s+(?:<[^>]+>\s+)?(\w+)",
    )
    .unwrap()
});

/// Detect visibility — private/internal/protected lines should be excluded
static PRIVATE_LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[^\S\n]*(?:private|internal|protected)\s+").unwrap());

/// Extract exported symbols from Kotlin source code.
/// In Kotlin, everything is public by default unless marked private/internal/protected.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = COMMENT_SINGLE.replace_all(content, "");
    let stripped = COMMENT_MULTI.replace_all(&stripped, "");

    let mut symbols = Vec::new();

    for line in stripped.lines() {
        let trimmed = line.trim();
        // Skip private/internal/protected declarations
        if PRIVATE_LINE.is_match(line) {
            continue;
        }

        if let Some(caps) = KT_DECL.captures(line)
            && let Some(name) = caps.get(1)
        {
            // Skip companion objects (they're not standalone exports)
            if trimmed.starts_with("companion") {
                continue;
            }
            symbols.push(name.as_str().to_string());
        }
    }

    symbols
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kotlin_exports() {
        let src = r#"
package com.example.auth

fun createAuth(config: Config): Auth {}
private fun internalHelper() {}
class AuthService {}
data class UserProfile(val name: String)
sealed class AuthState {}
object AuthManager {}
interface Authenticator {}
typealias Token = String
val DEFAULT_TTL = 3600
var globalInstance: Auth? = null
enum class Status { ACTIVE, EXPIRED }
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"createAuth".to_string()));
        assert!(symbols.contains(&"AuthService".to_string()));
        assert!(symbols.contains(&"UserProfile".to_string()));
        assert!(symbols.contains(&"AuthState".to_string()));
        assert!(symbols.contains(&"AuthManager".to_string()));
        assert!(symbols.contains(&"Authenticator".to_string()));
        assert!(symbols.contains(&"Token".to_string()));
        assert!(symbols.contains(&"DEFAULT_TTL".to_string()));
        assert!(symbols.contains(&"globalInstance".to_string()));
        assert!(symbols.contains(&"Status".to_string()));
        assert!(!symbols.contains(&"internalHelper".to_string()));
    }

    #[test]
    fn test_kotlin_visibility() {
        let src = r#"
public fun publicFun() {}
internal fun internalFun() {}
protected fun protectedFun() {}
private fun privateFun() {}
fun defaultPublicFun() {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"publicFun".to_string()));
        assert!(symbols.contains(&"defaultPublicFun".to_string()));
        assert!(!symbols.contains(&"internalFun".to_string()));
        assert!(!symbols.contains(&"protectedFun".to_string()));
        assert!(!symbols.contains(&"privateFun".to_string()));
    }

    #[test]
    fn test_kotlin_suspend() {
        let src = r#"
suspend fun fetchData(): Data {}
public suspend fun loadProfile(): Profile {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"fetchData".to_string()));
        assert!(symbols.contains(&"loadProfile".to_string()));
    }
}
