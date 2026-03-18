use regex::Regex;
use std::sync::LazyLock;

static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)//.*$").unwrap());

static COMMENT_MULTI: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/").unwrap());

/// export function/class/interface/type/const/enum name
static EXPORT_DECL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"export\s+(?:async\s+)?(?:abstract\s+)?(?:function|class|interface|type|const|enum)\s+(\w+)")
        .unwrap()
});

/// export type { Name, Name2 }
static RE_EXPORT_TYPE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"export\s+type\s*\{([^}]+)\}").unwrap());

/// export { Name, Name2 }
static RE_EXPORT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"export\s*\{([^}]+)\}").unwrap());

/// Extract exported symbols from TypeScript/JavaScript source.
pub fn extract_exports(content: &str) -> Vec<String> {
    // Strip comments
    let stripped = COMMENT_SINGLE.replace_all(content, "");
    let stripped = COMMENT_MULTI.replace_all(&stripped, "");

    let mut symbols = Vec::new();

    // Direct exports: export function/class/interface/type/const/enum
    for caps in EXPORT_DECL.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            symbols.push(name.as_str().to_string());
        }
    }

    // Re-export type: export type { Name }
    for caps in RE_EXPORT_TYPE.captures_iter(&stripped) {
        if let Some(names) = caps.get(1) {
            for name in names.as_str().split(',') {
                let name = name.trim();
                // Handle "Foo as Bar"
                let final_name = name.split(" as ").last().unwrap_or(name).trim();
                if !final_name.is_empty() {
                    symbols.push(final_name.to_string());
                }
            }
        }
    }

    // Re-export: export { Name } (but not if it's "export type {")
    for caps in RE_EXPORT.captures_iter(&stripped) {
        let full = caps.get(0).unwrap().as_str();
        if full.contains("export type") {
            continue;
        }
        if let Some(names) = caps.get(1) {
            for name in names.as_str().split(',') {
                let name = name.trim();
                // Handle "type Foo" prefix
                let name = name.strip_prefix("type ").unwrap_or(name);
                let final_name = name.split(" as ").last().unwrap_or(name).trim();
                if !final_name.is_empty() {
                    symbols.push(final_name.to_string());
                }
            }
        }
    }

    symbols
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_exports() {
        let src = r#"
export function createAuth(config: Config): Auth {}
export class AuthService {}
export interface AuthConfig {}
export type TokenType = string;
export const DEFAULT_TTL = 3600;
export enum AuthStatus { Active, Expired }
"#;
        let symbols = extract_exports(src);
        assert_eq!(
            symbols,
            vec![
                "createAuth",
                "AuthService",
                "AuthConfig",
                "TokenType",
                "DEFAULT_TTL",
                "AuthStatus"
            ]
        );
    }

    #[test]
    fn test_comments_stripped() {
        let src = r#"
// export function notExported() {}
/* export class AlsoNot {} */
export function realExport(): void {}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["realExport"]);
    }

    #[test]
    fn test_re_exports() {
        let src = r#"
export { Foo, Bar as Baz } from './module';
export type { MyType } from './types';
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Foo".to_string()));
        assert!(symbols.contains(&"Baz".to_string()));
        assert!(symbols.contains(&"MyType".to_string()));
    }
}
