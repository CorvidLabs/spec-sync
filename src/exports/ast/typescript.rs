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
/// - CommonJS-style default export: `export = Name`
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
    let mut has_equals = false;
    let mut namespace_name: Option<String> = None;
    let mut from_path: Option<String> = None;

    for child in node.children(&mut cursor) {
        match child.kind() {
            "default" => {
                has_default = true;
            }
            "=" => {
                has_equals = true;
            }
            // `export declare function/class/const ...` — tree-sitter wraps the
            // inner declaration in an `ambient_declaration` node rather than
            // exposing it directly as a child of `export_statement`. Unwrap it
            // and handle the inner declaration the same way as a non-ambient one
            // (ambient functions surface as `function_signature`, since they
            // have no body).
            "ambient_declaration" => {
                let mut ambient_cursor = child.walk();
                for ambient_child in child.children(&mut ambient_cursor) {
                    match ambient_child.kind() {
                        // `export declare namespace Foo { ... }` / `export
                        // declare module "path" { ... }` — see the
                        // `internal_module` | `module` arm below for why this
                        // needs its own recursive handling instead of the
                        // flat `handle_declaration_child` dispatch.
                        "internal_module" | "module" => {
                            handle_module_or_namespace(&ambient_child, src, symbols, resolver);
                        }
                        _ => handle_declaration_child(&ambient_child, src, symbols),
                    }
                }
            }
            // `export namespace Foo { ... }` (identifier-named, `internal_module`)
            // or `export module "path" { ... }` (string-named ambient external
            // module, `module`). Bug fix: before this arm existed, both kinds
            // fell through to the wildcard `_ => {}` case below with no
            // recursion at all, so `export namespace Foo { export function
            // bar(): void; }` produced a completely empty result — despite
            // being fully valid, common TypeScript — which is exactly the
            // condition that trips the AST-empty-triggers-regex-fallback
            // policy elsewhere in the codebase (masking this gap instead of
            // surfacing it). The regex fallback only "works" here by
            // accident, since it text-scans for `export function ...`
            // anywhere in the file with no namespace-scoping and so also
            // misses the namespace's own name.
            "internal_module" | "module" => {
                handle_module_or_namespace(&child, src, symbols, resolver);
            }
            // `export function name() {}` etc.
            "function_declaration" | "generator_function_declaration" | "function_signature" => {
                handle_declaration_child(&child, src, symbols);
            }
            "class_declaration" | "abstract_class_declaration" => {
                handle_declaration_child(&child, src, symbols);
            }
            "interface_declaration" => {
                handle_declaration_child(&child, src, symbols);
            }
            "type_alias_declaration" => {
                handle_declaration_child(&child, src, symbols);
            }
            "enum_declaration" => {
                handle_declaration_child(&child, src, symbols);
            }
            "lexical_declaration" => {
                // export const name = ... or export let name = ...
                handle_declaration_child(&child, src, symbols);
            }
            "variable_declaration" => {
                handle_declaration_child(&child, src, symbols);
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
                && let Some(target_content) = resolver(path)
            {
                let target_symbols = extract_exports(&target_content);
                symbols.extend(target_symbols);
            }
        }
    }

    // Handle CommonJS-style `export = Name`
    if has_equals {
        handle_export_equals(node, src, symbols);
    }
}

/// Handle an `internal_module` (`namespace Foo { ... }` / legacy `module Foo
/// { ... }`) or `module` (ambient external module, `module "path" { ... }`)
/// node reached via an `export`/`export declare` statement.
///
/// Pushes the namespace's own name (an `internal_module`'s `name` field is an
/// `identifier`, a real importable symbol) but not an ambient external
/// module's (its `name` field is a `string` literal path, not a symbol), then
/// recurses into the body to find further nested `export` statements —
/// mirroring how a bare, non-exported `module Foo { ... }` block already
/// gets its body scanned via `collect_exports`'s generic-recursion fallback.
#[allow(clippy::type_complexity)]
fn handle_module_or_namespace(
    node: &tree_sitter::Node,
    src: &[u8],
    symbols: &mut Vec<String>,
    resolver: Option<&dyn Fn(&str) -> Option<String>>,
) {
    if let Some(name) = node.child_by_field_name("name")
        && name.kind() != "string"
    {
        let text = name.utf8_text(src).unwrap_or_default();
        if !text.is_empty() {
            symbols.push(text.to_string());
        }
    }
    if let Some(body) = node.child_by_field_name("body") {
        collect_exports(&body, src, symbols, resolver);
    }
}

/// Extract the name from a CommonJS-style `export = expression` statement.
/// Captures the identifier name for `export = Foo`, or the declared name for
/// `export = class Foo {}` / `export = function foo() {}`.
fn handle_export_equals(node: &tree_sitter::Node, src: &[u8], symbols: &mut Vec<String>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "identifier" => {
                let name = child.utf8_text(src).unwrap_or_default();
                if !name.is_empty() {
                    symbols.push(name.to_string());
                }
                return;
            }
            "class"
            | "class_declaration"
            | "abstract_class_declaration"
            | "function_expression"
            | "generator_function"
            | "arrow_function" => {
                if let Some(name) = get_child_by_field(&child, "name", src)
                    && !name.is_empty()
                {
                    symbols.push(name);
                }
                return;
            }
            _ => {}
        }
    }
}

/// Handle a node representing a single declaration (function, class,
/// interface, type alias, enum, or variable/lexical declaration), pushing its
/// name(s) onto `symbols`. Shared between direct `export_statement` children
/// and the inner declaration wrapped by `ambient_declaration`
/// (`export declare function/class/const ...`), which tree-sitter nests one
/// level deeper instead of exposing directly as a child of `export_statement`.
fn handle_declaration_child(child: &tree_sitter::Node, src: &[u8], symbols: &mut Vec<String>) {
    match child.kind() {
        "function_declaration" | "generator_function_declaration" | "function_signature" => {
            if let Some(name) = get_child_by_field(child, "name", src) {
                symbols.push(name);
            }
        }
        "class_declaration" | "abstract_class_declaration" => {
            if let Some(name) = get_child_by_field(child, "name", src) {
                symbols.push(name);
            }
        }
        "interface_declaration" => {
            if let Some(name) = get_child_by_field(child, "name", src) {
                symbols.push(name);
            }
        }
        "type_alias_declaration" => {
            if let Some(name) = get_child_by_field(child, "name", src) {
                symbols.push(name);
            }
        }
        "enum_declaration" => {
            if let Some(name) = get_child_by_field(child, "name", src) {
                symbols.push(name);
            }
        }
        "lexical_declaration" | "variable_declaration" => {
            extract_variable_names(child, src, symbols);
        }
        _ => {}
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
            && let Some(name_node) = child.child_by_field_name("name")
        {
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

    #[test]
    fn test_ambient_declare_exports() {
        // `export declare ...` ambient declarations wrap their inner
        // declaration in an `ambient_declaration` node in tree-sitter's AST
        // rather than exposing it directly as a child of `export_statement`;
        // regression test for that unwrap.
        let src = r#"
export declare function debounce<T extends (...args: any[]) => void>(fn: T, wait: number): T;
export declare class EventEmitter {
    on(event: string, listener: (...args: any[]) => void): this;
}
export declare const VERSION: string;
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["debounce", "EventEmitter", "VERSION"]);
    }

    #[test]
    fn test_ambient_declare_mixed_with_normal_export() {
        // Regression: in a file mixing ambient declarations with a normal
        // export, `scan_exported_symbols_full`'s AST-with-regex-fallback only
        // falls back to regex when the AST result is empty. Before the fix,
        // this returned a non-empty but incomplete result (`createEmitter`
        // only), so the fallback never triggered and EventEmitter/VERSION
        // silently vanished with no error signal.
        let src = r#"
export declare class EventEmitter { on(event: string, listener: (...args: any[]) => void): this; }
export declare const VERSION: string;
export function createEmitter(): EventEmitter { return new EventEmitter(); }
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["EventEmitter", "VERSION", "createEmitter"]);
    }

    #[test]
    fn test_learnxinyminutes_module_namespace_export() {
        // Regression: real excerpt from learnxinyminutes.com/typescript.
        // Legacy `module Foo { ... }` namespace syntax nests its `export`
        // one level inside a `module`/`internal_module` node rather than at
        // the top level, requiring the generic recursion in
        // `collect_exports` to descend into it.
        let src = r#"
// Modules, "." can be used as separator for sub modules
module Geometry {
  export class Square {
    constructor(public sideLength: number = 0) {
    }
    area() {
      return Math.pow(this.sideLength, 2);
    }
  }
}

let s1 = new Geometry.Square(5);

// Local alias for referencing a module
import G = Geometry;

let s2 = new G.Square(10);
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Square"]);
    }

    #[test]
    fn test_learnxinyminutes_interfaces_and_classes_real_excerpt() {
        // Real excerpt (verbatim) from learnxinyminutes.com/typescript,
        // covering structural interfaces, a class with constructor
        // shorthand properties, `implements`, inheritance via `extends`,
        // and the legacy namespace `module` export. None of the
        // interfaces/classes here are exported except `Square` nested in
        // `Geometry` -- this locks in that the AST extractor doesn't
        // spuriously pick up un-exported top-level declarations while still
        // finding the genuinely nested export, and that a realistic slice
        // with this much syntax breadth parses cleanly without panicking.
        let src = r#"
// Interfaces are structural, anything that has the properties is compliant with
// the interface
interface Person {
  name: string;
  // Optional properties, marked with a "?"
  age?: number;
  // And of course functions
  move(): void;
}

// Object that implements the "Person" interface
// Can be treated as a Person since it has the name and move properties
let p: Person = { name: "Bobby", move: () => { } };
// Objects that have the optional property:
let validPerson: Person = { name: "Bobby", age: 42, move: () => { } };
// Is not a person because age is not a number
let invalidPerson: Person = { name: "Bobby", age: true };

// Classes - members are public by default
class Point {
  // Properties
  x: number;

  constructor(x: number, public y: number = 0) {
    this.x = x;
  }

  // Functions
  dist(): number { return Math.sqrt(this.x * this.x + this.y * this.y); }

  // Static members
  static origin = new Point(0, 0);
}

// Classes can be explicitly marked as implementing an interface.
class PointPerson implements Person {
    name: string
    move() {}
}

let p1 = new Point(10, 20);

// Inheritance
class Point3D extends Point {
  constructor(x: number, y: number, public z: number = 0) {
    super(x, y); // Explicit call to the super class constructor is mandatory
  }

  // Overwrite
  dist(): number {
    let d = super.dist();
    return Math.sqrt(d * d + this.z * this.z);
  }
}

// Modules, "." can be used as separator for sub modules
module Geometry {
  export class Square {
    constructor(public sideLength: number = 0) {
    }
    area() {
      return Math.pow(this.sideLength, 2);
    }
  }
}

let s1 = new Geometry.Square(5);
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Square"]);
    }

    #[test]
    fn test_export_equals_identifier() {
        let src = r#"
class AuthService {}
export = AuthService;
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["AuthService"]);
    }

    #[test]
    fn test_export_equals_class_expression() {
        let src = r#"export = class AuthService {};"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["AuthService"]);
    }

    #[test]
    fn test_export_equals_function_expression() {
        let src = r#"export = function createAuth() {};"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["createAuth"]);
    }

    #[test]
    fn test_export_namespace_recurses_and_captures_own_name() {
        // Bug (silent-fallback-masking): `export namespace Foo { ... }`
        // wraps its body in an `internal_module` node exposed as the
        // `export_statement`'s `declaration` field. Before this fix, that
        // node kind wasn't handled at all in `handle_export_statement`'s
        // flat match, so it silently fell through the wildcard `_ => {}`
        // arm with *no recursion whatsoever* -- unlike a bare, non-exported
        // `module Foo { ... }` block (see
        // `test_learnxinyminutes_module_namespace_export` below), which
        // still gets its body scanned via `collect_exports`'s generic
        // recursion fallback. The result: this fully valid, common
        // TypeScript pattern produced a completely empty `Vec`, which is
        // exactly the condition that trips the AST-empty-triggers-
        // regex-fallback policy elsewhere in the codebase. The regex
        // fallback only "half-recovers" here by accident (it text-scans for
        // `export function ...` with no namespace-scoping), and even then it
        // misses the namespace's own name `Foo` entirely.
        let src = r#"
export namespace Foo {
    export function bar(): void;
}
"#;
        let symbols = extract_exports(src);
        assert!(
            !symbols.is_empty(),
            "AST backend must not return empty for `export namespace Foo {{ ... }}`"
        );
        assert!(symbols.contains(&"Foo".to_string()));
        assert!(symbols.contains(&"bar".to_string()));
    }

    #[test]
    fn test_export_declare_namespace_recurses_and_captures_own_name() {
        // Same underlying bug as the non-declare case above, but reached via
        // the `ambient_declaration` unwrap path instead: `export declare
        // namespace Foo { ... }` nests its `internal_module` one level
        // deeper (inside `ambient_declaration`), and that per-child loop
        // dispatched exclusively through the non-recursive
        // `handle_declaration_child`, which also had no case for
        // `internal_module` -- so an ambient exported namespace with
        // multiple exported members inside it produced nothing at all.
        let src = r#"
export declare namespace Foo {
    export function bar(): void;
    export const baz: string;
}
"#;
        let symbols = extract_exports(src);
        assert!(
            !symbols.is_empty(),
            "AST backend must not return empty for `export declare namespace Foo {{ ... }}`"
        );
        assert_eq!(symbols, vec!["Foo", "bar", "baz"]);
    }

    #[test]
    fn test_declare_module_string_ambient_external_module() {
        // Verified-correct behavior (not a bug, but previously untested): a
        // bare (non-exported) `declare module "path" { ... }` ambient
        // external module augmentation is a top-level `ambient_declaration`
        // node reached directly by `collect_exports`'s generic recursion
        // fallback, so its nested exports were already found correctly. Its
        // `name` field is a string literal path, not an identifier, so
        // (unlike `export namespace Foo`) there is no symbol name of its own
        // to push -- only the members exported from inside it.
        let src = r#"
declare module "my-lib" {
    export function doThing(): void;
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["doThing"]);
        assert!(!symbols.contains(&"my-lib".to_string()));
    }

    #[test]
    fn test_anonymous_default_export_yields_no_symbol() {
        // Verified-correct (not a bug): `export default <anonymous
        // expression>` (an arrow function, an unnamed `function`
        // expression, or an unnamed `class`) has no declared name to
        // report at all -- there is nothing for a spec/doc reference to
        // call this symbol by. Both the AST and regex backends agree this
        // yields an empty result; this is a fundamentally different
        // situation from the `export namespace`/`export declare namespace`
        // cases above, where a real name (the namespace's own, plus its
        // members) existed but was being silently dropped.
        for src in [
            "export default () => {};\n",
            "export default function() {};\n",
            "export default class {};\n",
        ] {
            let symbols = extract_exports(src);
            assert!(
                symbols.is_empty(),
                "anonymous default export {src:?} should yield no symbol, got {symbols:?}"
            );
        }
    }

    #[test]
    fn test_decorated_export_class_captured() {
        // Verified-correct behavior: a decorator (`@Injectable()`) sitting
        // between nothing and `export class Foo {}` is exposed as a
        // separate `decorator` field on the `export_statement` node and
        // does not displace the `class_declaration`'s own `name` field, so
        // name extraction is unaffected.
        let src = "@Injectable()\nexport class FooService {}\n";
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["FooService"]);
    }

    #[test]
    fn test_export_specifier_level_type_prefix_not_captured_as_name() {
        // Verified-correct behavior: `export { type Bar } from './bar'`
        // marks a single specifier as type-only (as opposed to `export type
        // { Foo }`, which marks the whole clause type-only). Tree-sitter
        // exposes the specifier's `name` field as a plain `identifier`
        // ("Bar") with the `type` keyword as a separate sibling token, so
        // unlike the regex backend (which must explicitly
        // `strip_prefix("type ")`), the AST backend never risks capturing
        // the literal text "type Bar" as a symbol name.
        let src = "export type { Foo } from './foo';\nexport { type Bar } from './bar';\n";
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Foo".to_string()));
        assert!(symbols.contains(&"Bar".to_string()));
        assert!(!symbols.iter().any(|s| s.contains("type")));
    }

    #[test]
    fn test_multiline_generic_constraint_and_template_literal_type() {
        // Verified-correct behavior: a generic type parameter whose
        // constraint spans multiple lines, and a template literal type
        // (`` `hello ${string}` ``) as a type alias's value, don't disturb
        // the surrounding declaration's `name` field lookup.
        let generic_src = "export function identity<\n    T extends Record<string, unknown>\n>(value: T): T {\n    return value;\n}\n";
        let symbols = extract_exports(generic_src);
        assert_eq!(symbols, vec!["identity"]);

        let template_literal_src = "export type Greeting = `hello ${string}`;\n";
        let symbols = extract_exports(template_literal_src);
        assert_eq!(symbols, vec!["Greeting"]);
    }
}
