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

/// export * from './module' or export * as name from './module'
static WILDCARD_EXPORT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"export\s+\*\s+(?:as\s+(\w+)\s+)?from\s+['"]([^'"]+)['"]"#).unwrap()
});

/// export default function/class name or export default expression
static EXPORT_DEFAULT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"export\s+default\s+(?:(?:abstract\s+)?(?:function|class)\s+(\w+)|(\w+)\s*[;\n])")
        .unwrap()
});

/// Extract exported symbols from TypeScript/JavaScript source (without file resolution).
///
/// Supports:
/// - Direct exports: `export function/class/interface/type/const/enum Name`
/// - Re-exports: `export { Name }` and `export type { Name }`
/// - Namespace re-exports: `export * as Name from './module'`
/// - Default exports: `export default class Name`
///
/// For wildcard `export * from` support, use `extract_exports_with_resolver`.
#[cfg_attr(not(test), allow(dead_code))]
pub fn extract_exports(content: &str) -> Vec<String> {
    extract_exports_with_resolver(content, None)
}

/// Function signature for resolving import paths to file content.
type ImportResolver<'a> = dyn Fn(&str) -> Option<String> + 'a;

/// Extract exports, optionally resolving wildcard re-exports via a file resolver.
/// The resolver maps a relative import path to the file content at that path.
pub fn extract_exports_with_resolver(
    content: &str,
    resolver: Option<&ImportResolver<'_>>,
) -> Vec<String> {
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

    // Default exports: export default class/function Name
    for caps in EXPORT_DEFAULT.captures_iter(&stripped) {
        if let Some(name) = caps.get(1).or_else(|| caps.get(2)) {
            let n = name.as_str();
            // Skip keyword-like default exports (e.g. `export default new ...`)
            if !["new", "function", "class", "abstract", "async", "true", "false", "null", "undefined"].contains(&n) {
                symbols.push(n.to_string());
            }
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

    // Wildcard re-exports: export * from './module' / export * as Ns from './module'
    for caps in WILDCARD_EXPORT.captures_iter(&stripped) {
        if let Some(alias) = caps.get(1) {
            // export * as Ns from '...' — the namespace name itself is the export
            symbols.push(alias.as_str().to_string());
        } else if let Some(resolver) = resolver {
            // export * from '...' — resolve the target module and pull its exports
            let path = caps.get(2).unwrap().as_str();
            if let Some(target_content) = resolver(path) {
                // Recurse without resolver to avoid infinite loops
                let target_symbols = extract_exports_with_resolver(&target_content, None);
                symbols.extend(target_symbols);
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

    #[test]
    fn test_wildcard_namespace_export() {
        let src = r#"
export * as Utils from './utils';
export * as Types from './types';
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Utils", "Types"]);
    }

    #[test]
    fn test_wildcard_export_with_resolver() {
        let src = r#"
export * from './helpers';
export function main() {}
"#;
        let helper_content = r#"
export function helperA() {}
export function helperB() {}
export const HELPER_CONST = 42;
"#;
        let resolver = |path: &str| -> Option<String> {
            if path == "./helpers" {
                Some(helper_content.to_string())
            } else {
                None
            }
        };
        let symbols = extract_exports_with_resolver(src, Some(&resolver));
        assert!(symbols.contains(&"main".to_string()));
        assert!(symbols.contains(&"helperA".to_string()));
        assert!(symbols.contains(&"helperB".to_string()));
        assert!(symbols.contains(&"HELPER_CONST".to_string()));
    }

    #[test]
    fn test_wildcard_export_without_resolver() {
        // Without a resolver, wildcard exports are silently skipped
        let src = r#"
export * from './helpers';
export function main() {}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["main"]);
    }

    #[test]
    fn test_default_export_class() {
        let src = r#"
export default class MyApp {}
export function helper() {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"MyApp".to_string()));
        assert!(symbols.contains(&"helper".to_string()));
    }

    #[test]
    fn test_async_and_abstract_exports() {
        let src = r#"
export async function fetchData() {}
export abstract class BaseService {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"fetchData".to_string()));
        assert!(symbols.contains(&"BaseService".to_string()));
    }
}
