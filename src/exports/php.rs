use regex::Regex;
use std::sync::LazyLock;

static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"//.*$").unwrap());

static COMMENT_MULTI: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/").unwrap());

static COMMENT_HASH: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)#[^\[].*$").unwrap());

/// PHP public type declarations: class, interface, trait, enum
static PHP_TYPE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*(?:abstract\s+|final\s+)?(?:readonly\s+)?(?:class|interface|trait|enum)\s+(\w+)",
    )
    .unwrap()
});

/// PHP public function declarations (top-level or explicitly public)
static PHP_FUNCTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^[^\S\n]*(?:public\s+)?(?:static\s+)?function\s+(\w+)").unwrap()
});

/// PHP class method with explicit visibility — skip private/protected
static PHP_PRIVATE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[^\S\n]*(?:private|protected)\s+").unwrap());

/// PHP const declarations at class or top level
static PHP_CONST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^[^\S\n]*(?:public\s+)?const\s+(\w+)").unwrap()
});

/// Extract public symbols from PHP source code.
/// Classes, interfaces, traits, enums are always included.
/// Functions and constants are included unless marked private/protected.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = COMMENT_SINGLE.replace_all(content, "");
    let stripped = COMMENT_MULTI.replace_all(&stripped, "");
    let stripped = COMMENT_HASH.replace_all(&stripped, "");

    let mut symbols = Vec::new();

    // Types are always public in PHP (visibility is per-member, not per-type)
    for caps in PHP_TYPE.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            symbols.push(name.as_str().to_string());
        }
    }

    // Functions: include unless the line starts with private/protected
    for caps in PHP_FUNCTION.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str();
            // Skip constructor and magic methods
            if n.starts_with("__") {
                continue;
            }
            // Check if the whole match line starts with private/protected
            let line_start = &stripped[..caps.get(0).unwrap().start()];
            let line_begin = line_start.rfind('\n').map_or(0, |i| i + 1);
            let full_line = &stripped[line_begin..caps.get(0).unwrap().end()];
            if PHP_PRIVATE.is_match(full_line) {
                continue;
            }
            if !symbols.contains(&n.to_string()) {
                symbols.push(n.to_string());
            }
        }
    }

    // Constants
    for caps in PHP_CONST.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str().to_string();
            let line_start = &stripped[..caps.get(0).unwrap().start()];
            let line_begin = line_start.rfind('\n').map_or(0, |i| i + 1);
            let full_line = &stripped[line_begin..caps.get(0).unwrap().end()];
            if PHP_PRIVATE.is_match(full_line) {
                continue;
            }
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
    fn test_php_class_and_methods() {
        let src = r#"<?php

namespace App\Auth;

class AuthService {
    public const DEFAULT_TTL = 3600;
    private const INTERNAL_KEY = "secret";

    public function validate(string $token): bool {}
    private function internalCheck(): void {}
    public static function create(): self {}
    protected function helper(): void {}
}

interface Authenticator {
    public function authenticate(): bool;
}

abstract class BaseController {}

enum Status {
    case Active;
    case Expired;
}

trait Loggable {
    public function log(): void {}
}

function standalone_helper(): void {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"AuthService".to_string()));
        assert!(symbols.contains(&"Authenticator".to_string()));
        assert!(symbols.contains(&"BaseController".to_string()));
        assert!(symbols.contains(&"Status".to_string()));
        assert!(symbols.contains(&"Loggable".to_string()));
        assert!(symbols.contains(&"validate".to_string()));
        assert!(symbols.contains(&"create".to_string()));
        assert!(symbols.contains(&"DEFAULT_TTL".to_string()));
        assert!(symbols.contains(&"standalone_helper".to_string()));
        assert!(!symbols.contains(&"internalCheck".to_string()));
        assert!(!symbols.contains(&"helper".to_string()));
        assert!(!symbols.contains(&"INTERNAL_KEY".to_string()));
    }

    #[test]
    fn test_php_final_readonly() {
        let src = r#"<?php
final class Config {}
readonly class ValueObject {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Config".to_string()));
        assert!(symbols.contains(&"ValueObject".to_string()));
    }

    #[test]
    fn test_php_skips_magic_methods() {
        let src = r#"<?php
class Foo {
    public function __construct() {}
    public function __toString(): string {}
    public function getName(): string {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Foo".to_string()));
        assert!(symbols.contains(&"getName".to_string()));
        assert!(!symbols.contains(&"__construct".to_string()));
        assert!(!symbols.contains(&"__toString".to_string()));
    }
}
