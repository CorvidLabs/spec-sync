use tree_sitter::{Parser, Tree};

/// Parse TypeScript/JavaScript source into a tree-sitter AST.
fn parse_ts(content: &str) -> Option<Tree> {
    let mut parser = Parser::new();
    let language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into();
    parser.set_language(&language).ok()?;
    parser.parse(content, None)
}

/// Extract exported symbols from TypeScript/JavaScript source using tree-sitter AST.
///
/// Handles:
/// - `export function/class/interface/type/const/enum Name`
/// - `export default class/function Name`
/// - `export { Name, Foo as Bar }`
/// - `export type { Name }`
/// - `export * as Ns from './module'`
/// - `export * from './module'` (with optional resolver)
/// - Conditional exports (`if (...)  { export ... }` — still captured)
/// - Computed property names in export lists are skipped
pub fn extract_exports(content: &str) -> Vec<String> {
    extract_exports_with_resolver(content, None)
}

/// Extract exports, optionally resolving wildcard re-exports via a file resolver.
#[allow(clippy::type_complexity)]
pub fn extract_exports_with_resolver(
    content: &str,
    resolver: Option<&dyn Fn(&str) -> Option<String>>,
) -> Vec<String> {
    let tree = match parse_ts(content) {
        Some(t) => t,
        None => return Vec::new(),
    };

    let root = tree.root_node();
    let src = content.as_bytes();
    let mut symbols = Vec::new();

    collect_exports(&root, src, &mut symbols, resolver);

    symbols
}

#[allow(clippy::type_complexity)]
fn collect_exports(
    node: &tree_sitter::Node,
    src: &[u8],
    symbols: &mut Vec<String>,
    resolver: Option<&dyn Fn(&str) -> Option<String>>,
) {
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        match child.kind() {
            // export function name() {}
            // export class Name {}
            // export interface Name {}
            // export type Name = ...
            // export const name = ...
            // export enum Name { ... }
            // export abstract class Name {}
            // export async function name() {}
            "export_statement" => {
                handle_export_statement(&child, src, symbols, resolver);
            }
            // Recurse into other nodes (e.g. if blocks containing exports)
            _ => {
                collect_exports(&child, src, symbols, resolver);
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn handle_export_statement(
    node: &tree_sitter::Node,
    src: &[u8],
    symbols: &mut Vec<String>,
    resolver: Option<&dyn Fn(&str) -> Option<String>>,
) {
    let mut cursor = node.walk();

    // Check for `export * from` or `export * as Ns from`
    // Check for export_clause: `export { ... }`
    // Check for declaration: `export function/class/...`
    // Check for `export default`

    let mut has_default = false;
    let mut has_wildcard = false;
    let mut namespace_name: Option<String> = None;
    let mut from_path: Option<String> = None;

    for child in node.children(&mut cursor) {
        match child.kind() {
            "default" => {
                has_default = true;
            }
            // `export function name() {}` etc.
            "function_declaration" | "generator_function_declaration" => {
                if let Some(name) = get_child_by_field(&child, "name", src) {
                    symbols.push(name);
                }
            }
            "class_declaration" | "abstract_class_declaration" => {
                if let Some(name) = get_child_by_field(&child, "name", src) {
                    symbols.push(name);
                }
            }
            "interface_declaration" => {
                if let Some(name) = get_child_by_field(&child, "name", src) {
                    symbols.push(name);
                }
            }
            "type_alias_declaration" => {
                if let Some(name) = get_child_by_field(&child, "name", src) {
                    symbols.push(name);
                }
            }
            "enum_declaration" => {
                if let Some(name) = get_child_by_field(&child, "name", src) {
                    symbols.push(name);
                }
            }
            "lexical_declaration" => {
                // export const name = ... or export let name = ...
                extract_variable_names(&child, src, symbols);
            }
            "variable_declaration" => {
                extract_variable_names(&child, src, symbols);
            }
            // export { Foo, Bar as Baz }
            "export_clause" => {
                extract_export_clause(&child, src, symbols);
            }
            // export * (bare wildcard, without namespace)
            "*" => {
                has_wildcard = true;
            }
            // export * as Ns from '...'
            "namespace_export" => {
                has_wildcard = true;
                let mut ns_cursor = child.walk();
                for ns_child in child.children(&mut ns_cursor) {
                    if ns_child.kind() == "identifier" {
                        namespace_name =
                            Some(ns_child.utf8_text(src).unwrap_or_default().to_string());
                    }
                }
            }
            // from './module' — the string node contains the path
            "string" => {
                // Extract the string_fragment child for the actual path
                let mut str_cursor = child.walk();
                for str_child in child.children(&mut str_cursor) {
                    if str_child.kind() == "string_fragment" {
                        let path = str_child.utf8_text(src).unwrap_or_default();
                        if !path.is_empty() {
                            from_path = Some(path.to_string());
                        }
                    }
                }
            }
            // export default expression (identifier)
            "identifier" if has_default => {
                let name = child.utf8_text(src).unwrap_or_default();
                if !is_keyword(name) {
                    symbols.push(name.to_string());
                }
            }
            _ => {}
        }
    }

    // Handle wildcard re-exports
    if has_wildcard {
        if let Some(ns) = namespace_name {
            // export * as Ns from '...' — emit namespace name
            symbols.push(ns);
        } else if let Some(path) = &from_path {
            // export * from '...' — resolve if we have a resolver
            if let Some(resolver) = resolver
                && let Some(target_content) = resolver(path) {
                    let target_symbols = extract_exports(&target_content);
                    symbols.extend(target_symbols);
                }
        }
    }
}

/// Extract names from `export { Foo, Bar as Baz }`.
fn extract_export_clause(node: &tree_sitter::Node, src: &[u8], symbols: &mut Vec<String>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "export_specifier" {
            // If there's an alias ("as Bar"), use the alias. Otherwise use the name.
            let alias = child.child_by_field_name("alias");
            let name_node = child.child_by_field_name("name");

            if let Some(alias_node) = alias {
                let text = alias_node.utf8_text(src).unwrap_or_default();
                if !text.is_empty() {
                    symbols.push(text.to_string());
                }
            } else if let Some(name_node) = name_node {
                let text = name_node.utf8_text(src).unwrap_or_default();
                if !text.is_empty() {
                    symbols.push(text.to_string());
                }
            }
        }
    }
}

/// Extract variable names from `const x = ...` or `let { a, b } = ...`.
fn extract_variable_names(node: &tree_sitter::Node, src: &[u8], symbols: &mut Vec<String>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "variable_declarator"
            && let Some(name_node) = child.child_by_field_name("name") {
                match name_node.kind() {
                    "identifier" => {
                        let name = name_node.utf8_text(src).unwrap_or_default();
                        symbols.push(name.to_string());
                    }
                    // Destructuring: export const { a, b } = ...
                    "object_pattern" => {
                        extract_pattern_names(&name_node, src, symbols);
                    }
                    "array_pattern" => {
                        extract_pattern_names(&name_node, src, symbols);
                    }
                    _ => {}
                }
            }
    }
}

/// Extract identifiers from destructuring patterns.
fn extract_pattern_names(node: &tree_sitter::Node, src: &[u8], symbols: &mut Vec<String>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "identifier" => {
                let name = child.utf8_text(src).unwrap_or_default();
                symbols.push(name.to_string());
            }
            "shorthand_property_identifier_pattern" => {
                let name = child.utf8_text(src).unwrap_or_default();
                symbols.push(name.to_string());
            }
            "pair_pattern" => {
                // { key: value } — the value is the binding
                if let Some(value) = child.child_by_field_name("value") {
                    extract_pattern_names(&value, src, symbols);
                }
            }
            "object_pattern" | "array_pattern" => {
                extract_pattern_names(&child, src, symbols);
            }
            _ => {}
        }
    }
}

/// Get the text of a named field child.
fn get_child_by_field(node: &tree_sitter::Node, field: &str, src: &[u8]) -> Option<String> {
    node.child_by_field_name(field)
        .map(|n| n.utf8_text(src).unwrap_or_default().to_string())
}

fn is_keyword(s: &str) -> bool {
    matches!(
        s,
        "new"
            | "function"
            | "class"
            | "abstract"
            | "async"
            | "true"
            | "false"
            | "null"
            | "undefined"
    )
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
    fn test_re_exports_with_alias() {
        let src = r#"
export { Foo, Bar as Baz } from './module';
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Foo".to_string()));
        assert!(symbols.contains(&"Baz".to_string()));
        assert!(!symbols.contains(&"Bar".to_string()));
    }

    #[test]
    fn test_wildcard_namespace() {
        let src = r#"
export * as Utils from './utils';
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Utils"]);
    }

    #[test]
    fn test_wildcard_with_resolver() {
        let src = r#"
export * from './helpers';
export function main() {}
"#;
        let helper = r#"
export function helperA() {}
export function helperB() {}
"#;
        let resolver = |path: &str| -> Option<String> {
            if path == "./helpers" {
                Some(helper.to_string())
            } else {
                None
            }
        };
        let symbols = extract_exports_with_resolver(src, Some(&resolver));
        assert!(symbols.contains(&"main".to_string()));
        assert!(symbols.contains(&"helperA".to_string()));
        assert!(symbols.contains(&"helperB".to_string()));
    }

    #[test]
    fn test_default_export() {
        let src = r#"
export default class MyApp {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"MyApp".to_string()));
    }

    #[test]
    fn test_async_abstract() {
        let src = r#"
export async function fetchData() {}
export abstract class BaseService {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"fetchData".to_string()));
        assert!(symbols.contains(&"BaseService".to_string()));
    }

    #[test]
    fn test_comments_not_exported() {
        // AST naturally ignores comments — no exports inside comments
        let src = r#"
// export function notExported() {}
/* export class AlsoNot {} */
export function realExport(): void {}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["realExport"]);
    }

    #[test]
    fn test_conditional_export() {
        // Tree-sitter can see exports inside if blocks
        let src = r#"
if (process.env.NODE_ENV === 'development') {
    export function debugHelper() {}
}
export function main() {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"main".to_string()));
        assert!(symbols.contains(&"debugHelper".to_string()));
    }

    #[test]
    fn test_export_type_clause() {
        let src = r#"
export type { MyType, AnotherType as Renamed } from './types';
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"MyType".to_string()));
        assert!(symbols.contains(&"Renamed".to_string()));
    }
}
