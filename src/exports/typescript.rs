use regex::Regex;
use std::sync::LazyLock;
use tree_sitter::{Parser, Tree};

/// export function/class/interface/type/const/enum name
///
/// The `const\s+enum` alternative must be listed before the bare `const`
/// alternative: `export const enum Name` would otherwise match the `const`
/// branch and greedily capture the following `enum` keyword as the name
/// instead of `Name`. `declare` is accepted as an optional modifier so
/// ambient declarations (`export declare function/class/const ...`) are not
/// silently dropped.
///
/// JavaScript identifiers may spell Unicode scalar values as `\uXXXX` or
/// `\u{...}`. Capture the complete source spelling so canonicalization can
/// decode it rather than reporting the prefix before the backslash.
const JS_IDENTIFIER: &str = r"(?:\w|\\u(?:\{[0-9A-Fa-f]{1,6}\}|[0-9A-Fa-f]{4}))+";

static EXPORT_DECL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"export\s+(?:declare\s+)?(?:async\s+)?(?:abstract\s+)?(?:const\s+enum|function|class|interface|type|const|enum)\s+({JS_IDENTIFIER})"
    ))
    .unwrap()
});

/// export type { Name, Name2 }
static RE_EXPORT_TYPE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"export\s+type\s*\{([^}]+)\}").unwrap());

/// export { Name, Name2 }
static RE_EXPORT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"export\s*\{([^}]+)\}").unwrap());

/// export * from './module' or export * as name from './module'
static WILDCARD_EXPORT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r#"export\s+\*\s+(?:as\s+({JS_IDENTIFIER})\s+)?from\s+['"]([^'"]+)['"]"#
    ))
    .unwrap()
});

/// export default function/class name or export default expression
static EXPORT_DEFAULT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"export\s+default\s+(?:(?:abstract\s+)?(?:function|class)\s+({JS_IDENTIFIER})|({JS_IDENTIFIER})\s*[;\n])"
    ))
    .unwrap()
});

/// export = name (CommonJS-style default export)
static EXPORT_EQUALS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"export\s*=\s*({JS_IDENTIFIER})")).unwrap());

/// `exports.name = value` or `module.exports.name = value`.
static COMMONJS_PROPERTY_EXPORT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)(?:^|[^\w$.])(?:module\s*\.\s*)?exports\s*\.\s*([A-Za-z_$][A-Za-z0-9_$]*)\s*=\s*",
    )
    .unwrap()
});

/// Start of a `module.exports = { ... }` object assignment.
static COMMONJS_OBJECT_EXPORT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)(?:^|[^\w$.])module\s*\.\s*exports\s*=\s*\{").unwrap());

/// Decode JavaScript Unicode escapes in an identifier to its symbol identity.
///
/// The raw spelling remains available to the caller for diagnostics until the
/// canonical name is accepted. Invalid scalar values and malformed escapes are
/// rejected instead of emitting a truncated or partially decoded name.
fn decode_js_identifier_escapes(raw_name: &str) -> Option<String> {
    if !raw_name.contains('\\') {
        return Some(raw_name.to_string());
    }

    let bytes = raw_name.as_bytes();
    let mut canonical = String::with_capacity(raw_name.len());
    let mut copied_through = 0;
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] != b'\\' {
            index += 1;
            continue;
        }

        canonical.push_str(&raw_name[copied_through..index]);
        if bytes.get(index + 1) != Some(&b'u') {
            return None;
        }

        let (digits_start, digits_end, next_index) = if bytes.get(index + 2) == Some(&b'{') {
            let digits_start = index + 3;
            let close_offset = bytes[digits_start..]
                .iter()
                .position(|byte| *byte == b'}')?;
            let digits_end = digits_start + close_offset;
            if !(1..=6).contains(&(digits_end - digits_start)) {
                return None;
            }
            (digits_start, digits_end, digits_end + 1)
        } else {
            let digits_start = index + 2;
            let digits_end = digits_start.checked_add(4)?;
            if digits_end > bytes.len() {
                return None;
            }
            (digits_start, digits_end, digits_end)
        };

        let digits = raw_name.get(digits_start..digits_end)?;
        if !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return None;
        }
        let value = u32::from_str_radix(digits, 16).ok()?;
        canonical.push(char::from_u32(value)?);

        copied_through = next_index;
        index = next_index;
    }

    canonical.push_str(&raw_name[copied_through..]);
    Some(canonical)
}

fn push_js_identifier(symbols: &mut Vec<String>, raw_name: &str) {
    if let Some(canonical_name) = decode_js_identifier_escapes(raw_name) {
        push_unique(symbols, &canonical_name);
    }
}

/// Extract exported symbols from TypeScript/JavaScript source (without file resolution).
///
/// Supports:
/// - Direct exports: `export function/class/interface/type/const/enum Name`
/// - Re-exports: `export { Name }` and `export type { Name }`
/// - Namespace re-exports: `export * as Name from './module'`
/// - Default exports: `export default class Name`
/// - CommonJS-style default exports: `export = Name`
/// - Static CommonJS exports: `exports.name = value` and `module.exports = { name }`
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
    // Strip `//` and `/* */` comments while leaving string/template-literal
    // CONTENT untouched (see strip_comments_preserving_strings doc): a naive
    // `//.*$` / `/\*.*?\*/` regex pair (as this used to be) reads INSIDE
    // quoted strings too, so a `//` in a URL string (`"http://..."`) could
    // truncate a real export sharing that line, and a `/*`-like sequence in
    // any string could merge with a later real `*/` and swallow every real
    // export in between.
    let stripped = strip_comments_preserving_strings(content);

    let mut symbols = Vec::new();

    // Direct exports: export function/class/interface/type/const/enum
    for caps in EXPORT_DECL.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            // A malformed escape after an otherwise valid prefix must not
            // resurrect the old truncated identity (`caf\u{...}` -> `caf`).
            if stripped.as_bytes().get(name.end()) != Some(&b'\\') {
                push_js_identifier(&mut symbols, name.as_str());
            }
        }
    }

    // Default exports: export default class/function Name
    for caps in EXPORT_DEFAULT.captures_iter(&stripped) {
        if let Some(name) = caps.get(1).or_else(|| caps.get(2)) {
            let raw_name = name.as_str();
            let Some(canonical_name) = decode_js_identifier_escapes(raw_name) else {
                continue;
            };
            // Skip keyword-like default exports (e.g. `export default new ...`)
            if ![
                "new",
                "function",
                "class",
                "abstract",
                "async",
                "true",
                "false",
                "null",
                "undefined",
            ]
            .contains(&canonical_name.as_str())
            {
                push_unique(&mut symbols, &canonical_name);
            }
        }
    }

    // CommonJS-style default export: export = Name
    for caps in EXPORT_EQUALS.captures_iter(&stripped) {
        if let Some(name) = caps.get(1)
            && stripped.as_bytes().get(name.end()) != Some(&b'\\')
        {
            push_js_identifier(&mut symbols, name.as_str());
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
                    push_js_identifier(&mut symbols, final_name);
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
                    push_js_identifier(&mut symbols, final_name);
                }
            }
        }
    }

    // Wildcard re-exports: export * from './module' / export * as Ns from './module'
    for caps in WILDCARD_EXPORT.captures_iter(&stripped) {
        if let Some(alias) = caps.get(1) {
            // export * as Ns from '...' — the namespace name itself is the export
            push_js_identifier(&mut symbols, alias.as_str());
        } else if let Some(resolver) = resolver {
            // export * from '...' — resolve the target module and pull its exports
            let path = caps.get(2).unwrap().as_str();
            if let Some(target_content) = resolver(path) {
                // Recurse without resolver to avoid infinite loops
                let target_symbols = extract_exports_with_resolver(&target_content, None);
                for symbol in target_symbols {
                    push_unique(&mut symbols, &symbol);
                }
            }
        }
    }

    for symbol in extract_commonjs_exports(content) {
        push_unique(&mut symbols, &symbol);
    }

    symbols
}

/// Extract statically named CommonJS exports without executing source code.
///
/// The scanner masks comments and literals before matching assignments. Object
/// exports include identifier shorthand, named identifier properties, and
/// identifier-named methods. Computed keys and spreads remain intentionally
/// unresolved.
pub(super) fn extract_commonjs_exports(content: &str) -> Vec<String> {
    let masked = mask_comments_and_literals(content);
    let scope_tree = parse_scope_tree(content);
    let mut symbols = Vec::new();

    let mut property_cursor = 0;
    while let Some(captures) = COMMONJS_PROPERTY_EXPORT.captures_at(&masked, property_cursor) {
        let Some(name) = captures.get(1) else {
            break;
        };
        if is_plain_assignment(&masked, captures.get(0).map_or(0, |matched| matched.end()))
            && scope_tree
                .as_ref()
                .is_none_or(|tree| is_module_scope(tree, name.start()))
        {
            push_unique(&mut symbols, name.as_str());
        }

        // Resume inside the match so a chained RHS assignment can reuse the
        // first assignment operator as its left boundary.
        property_cursor = name.end().max(property_cursor + 1);
    }

    for matched in COMMONJS_OBJECT_EXPORT.find_iter(&masked) {
        if scope_tree
            .as_ref()
            .is_some_and(|tree| !is_module_scope(tree, matched.start()))
        {
            continue;
        }
        let open_brace = matched.end() - 1;
        extract_commonjs_object_keys(&masked, open_brace, &mut symbols);
    }

    symbols
}

fn parse_scope_tree(content: &str) -> Option<Tree> {
    let mut parser = Parser::new();
    let language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into();
    parser.set_language(&language).ok()?;
    parser.parse(content, None)
}

fn is_module_scope(tree: &Tree, byte_offset: usize) -> bool {
    let Some(mut node) = tree
        .root_node()
        .descendant_for_byte_range(byte_offset, byte_offset.saturating_add(1))
    else {
        return true;
    };

    loop {
        if matches!(
            node.kind(),
            "function_declaration"
                | "function_expression"
                | "generator_function_declaration"
                | "generator_function"
                | "arrow_function"
                | "method_definition"
        ) {
            return false;
        }
        let Some(parent) = node.parent() else {
            return true;
        };
        node = parent;
    }
}

fn is_plain_assignment(masked: &str, after_match: usize) -> bool {
    masked.as_bytes()[after_match..]
        .iter()
        .copied()
        .find(|byte| !byte.is_ascii_whitespace())
        .is_none_or(|byte| !matches!(byte, b'=' | b'>'))
}

fn mask_comments_and_literals(content: &str) -> String {
    let bytes = content.as_bytes();
    let mut masked = bytes.to_vec();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'/' && bytes.get(index + 1) == Some(&b'/') {
            masked[index] = b' ';
            masked[index + 1] = b' ';
            index += 2;
            while index < bytes.len() && bytes[index] != b'\n' {
                masked[index] = b' ';
                index += 1;
            }
            continue;
        }

        if bytes[index] == b'/' && bytes.get(index + 1) == Some(&b'*') {
            masked[index] = b' ';
            masked[index + 1] = b' ';
            index += 2;
            while index < bytes.len() {
                if bytes[index] == b'*' && bytes.get(index + 1) == Some(&b'/') {
                    masked[index] = b' ';
                    masked[index + 1] = b' ';
                    index += 2;
                    break;
                }
                if bytes[index] != b'\n' {
                    masked[index] = b' ';
                }
                index += 1;
            }
            continue;
        }

        if bytes[index] == b'/'
            && let Some(end) = regex_literal_end(bytes, index)
        {
            for byte in &mut masked[index..end] {
                if *byte != b'\n' {
                    *byte = b' ';
                }
            }
            index = end;
            continue;
        }

        if matches!(bytes[index], b'\'' | b'"' | b'`') {
            let quote = bytes[index];
            masked[index] = b' ';
            index += 1;
            while index < bytes.len() {
                if bytes[index] == b'\\' && index + 1 < bytes.len() {
                    masked[index] = b' ';
                    masked[index + 1] = b' ';
                    index += 2;
                    continue;
                }

                let current = bytes[index];
                if current != b'\n' {
                    masked[index] = b' ';
                }
                index += 1;

                if current == quote || (quote != b'`' && current == b'\n') {
                    break;
                }
            }
            continue;
        }

        index += 1;
    }

    String::from_utf8(masked).expect("masking valid source preserves UTF-8")
}

fn regex_literal_end(bytes: &[u8], start: usize) -> Option<usize> {
    let previous = bytes[..start]
        .iter()
        .rev()
        .copied()
        .find(|byte| !byte.is_ascii_whitespace());
    if previous.is_some_and(|byte| {
        !matches!(
            byte,
            b'=' | b'('
                | b'['
                | b'{'
                | b','
                | b':'
                | b';'
                | b'!'
                | b'?'
                | b'&'
                | b'|'
                | b'+'
                | b'-'
                | b'*'
                | b'%'
                | b'^'
                | b'~'
                | b'<'
                | b'>'
        )
    }) {
        return None;
    }

    let mut index = start + 1;
    let mut in_character_class = false;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' if index + 1 < bytes.len() => index += 2,
            b'[' => {
                in_character_class = true;
                index += 1;
            }
            b']' => {
                in_character_class = false;
                index += 1;
            }
            b'/' if !in_character_class => {
                index += 1;
                while index < bytes.len() && bytes[index].is_ascii_alphabetic() {
                    index += 1;
                }
                return Some(index);
            }
            b'\n' => return None,
            _ => index += 1,
        }
    }
    None
}

fn extract_commonjs_object_keys(masked: &str, open_brace: usize, symbols: &mut Vec<String>) {
    let bytes = masked.as_bytes();
    let mut index = open_brace + 1;
    let mut brace_depth = 1usize;
    let mut bracket_depth = 0usize;
    let mut paren_depth = 0usize;
    let mut property_start = true;

    while index < bytes.len() {
        if property_start && brace_depth == 1 && bracket_depth == 0 && paren_depth == 0 {
            while index < bytes.len() && bytes[index].is_ascii_whitespace() {
                index += 1;
            }
            if index >= bytes.len() || bytes[index] == b'}' {
                return;
            }

            if is_identifier_start(bytes[index]) {
                let name_start = index;
                index += 1;
                while index < bytes.len() && is_identifier_continue(bytes[index]) {
                    index += 1;
                }
                let name = &masked[name_start..index];
                let mut after_name = index;
                while after_name < bytes.len() && bytes[after_name].is_ascii_whitespace() {
                    after_name += 1;
                }

                if matches!(bytes.get(after_name), Some(b':' | b',' | b'}' | b'(')) {
                    push_unique(symbols, name);
                } else if matches!(name, "async" | "get" | "set") {
                    let second_start = after_name;
                    if second_start < bytes.len() && is_identifier_start(bytes[second_start]) {
                        let mut second_end = second_start + 1;
                        while second_end < bytes.len() && is_identifier_continue(bytes[second_end])
                        {
                            second_end += 1;
                        }
                        let mut after_second = second_end;
                        while after_second < bytes.len()
                            && bytes[after_second].is_ascii_whitespace()
                        {
                            after_second += 1;
                        }
                        if bytes.get(after_second) == Some(&b'(') {
                            push_unique(symbols, &masked[second_start..second_end]);
                        }
                    }
                }
            }

            property_start = false;
        }

        if index >= bytes.len() {
            return;
        }

        match bytes[index] {
            b'{' => brace_depth += 1,
            b'}' => {
                if brace_depth == 1 {
                    return;
                }
                brace_depth -= 1;
            }
            b'[' => bracket_depth += 1,
            b']' => bracket_depth = bracket_depth.saturating_sub(1),
            b'(' => paren_depth += 1,
            b')' => paren_depth = paren_depth.saturating_sub(1),
            b',' if brace_depth == 1 && bracket_depth == 0 && paren_depth == 0 => {
                property_start = true;
            }
            _ => {}
        }
        index += 1;
    }
}

fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$')
}

fn is_identifier_continue(byte: u8) -> bool {
    is_identifier_start(byte) || byte.is_ascii_digit()
}

fn push_unique(symbols: &mut Vec<String>, name: &str) {
    if !symbols.iter().any(|symbol| symbol == name) {
        symbols.push(name.to_string());
    }
}

/// Strip `//` line comments and `/* */` block comments in one linear pass
/// while treating single-quoted, double-quoted, and backtick template-literal
/// CONTENT as opaque: a comment-like sequence (`//`, `/*`) inside a string is
/// never mistaken for a real comment delimiter. Unlike the Go/Rust backends'
/// string handling, string/template content is copied through UNCHANGED
/// (not blanked) -- downstream regexes here (WILDCARD_EXPORT, RE_EXPORT's
/// `from '...'` clause) depend on the quoted import path text surviving
/// verbatim. `\` escapes are honored; a single/double-quoted string bails at
/// a raw newline (JS string literals cannot contain one), so a genuinely
/// unterminated quote cannot swallow the rest of the file. A template literal
/// may span multiple lines. Nested `${ ... }` interpolation containing
/// further backticks/comments is not specially tracked (rare, left for
/// future work).
fn strip_comments_preserving_strings(content: &str) -> String {
    let chars: Vec<char> = content.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(content.len());
    let mut i = 0;
    while i < n {
        let c = chars[i];
        // Line comment: skip to end of line (keep the newline).
        if c == '/' && i + 1 < n && chars[i + 1] == '/' {
            i += 2;
            while i < n && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        // Block comment (does not nest in JS/TS).
        if c == '/' && i + 1 < n && chars[i + 1] == '*' {
            i += 2;
            while i < n {
                if chars[i] == '*' && i + 1 < n && chars[i + 1] == '/' {
                    i += 2;
                    break;
                }
                if chars[i] == '\n' {
                    out.push('\n');
                }
                i += 1;
            }
            continue;
        }
        // Single/double-quoted string: copied through verbatim, just scanned
        // over so a `//`/`/*` inside it is never treated as a real comment.
        if c == '\'' || c == '"' {
            let quote = c;
            out.push(c);
            i += 1;
            while i < n {
                if chars[i] == '\\' && i + 1 < n {
                    out.push(chars[i]);
                    out.push(chars[i + 1]);
                    i += 2;
                    continue;
                }
                out.push(chars[i]);
                let ended = chars[i] == quote || chars[i] == '\n';
                i += 1;
                if ended {
                    break;
                }
            }
            continue;
        }
        // Backtick template literal: copied through verbatim; may span lines.
        if c == '`' {
            out.push(c);
            i += 1;
            while i < n {
                if chars[i] == '\\' && i + 1 < n {
                    out.push(chars[i]);
                    out.push(chars[i + 1]);
                    i += 2;
                    continue;
                }
                out.push(chars[i]);
                let ended = chars[i] == '`';
                i += 1;
                if ended {
                    break;
                }
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
    fn test_unicode_escape_identifiers_are_canonicalized_for_direct_exports() {
        let src = r#"
export const caf\u0061 = 1;
export function \u{62}et\u0061() {}
export interface \u0047amma {}
export const cafa = 2;
"#;

        let symbols = extract_exports(src);

        assert_eq!(symbols, vec!["cafa", "beta", "Gamma"]);
        assert!(!symbols.contains(&"caf".to_string()));
    }

    #[test]
    fn test_unicode_escape_identifiers_are_canonicalized_for_export_paths() {
        let src = r#"
export default class Def\u0061ult {}
export = Mod\u0075le;
export type { SourceType as T\u0079pe };
export { Source as ali\u0061s, Loc\u0061l };
export * as N\u0061mespace from './module';
"#;

        let symbols = extract_exports(src);

        assert_eq!(
            symbols,
            vec!["Default", "Module", "Type", "alias", "Local", "Namespace"]
        );

        let default_expression = extract_exports(r"export default v\u0061lue;");
        assert_eq!(default_expression, vec!["value"]);
    }

    #[test]
    fn test_unicode_escape_identifiers_are_canonicalized_through_resolver() {
        let resolver = |path: &str| {
            (path == "./dependency").then(|| r"export function dep\u0065ndency() {}".to_string())
        };

        let symbols =
            extract_exports_with_resolver("export * from './dependency';", Some(&resolver));

        assert_eq!(symbols, vec!["dependency"]);
    }

    #[test]
    fn test_invalid_unicode_escape_identifiers_are_not_truncated() {
        let src = r#"
export const tooLong\u{1234567} = 1;
export const surrogate\uD800 = 2;
export const outOfRange\u{110000} = 3;
export const valid = 4;
"#;

        let symbols = extract_exports(src);

        assert_eq!(symbols, vec!["valid"]);
        assert!(!symbols.contains(&"tooLong".to_string()));
    }

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

    #[test]
    fn test_export_equals() {
        // CommonJS-style default export: `export = Name` exports the identifier
        // as the module's default.
        let src = r#"
class AuthService {}
export = AuthService;
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["AuthService"]);
    }

    #[test]
    fn test_export_equals_no_spaces() {
        let src = r#"export=AuthService;"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["AuthService"]);
    }

    #[test]
    fn test_commonjs_direct_and_object_exports() {
        let src = r#"
exports.direct = createDirect();
module.exports.qualified = createQualified();
module.exports = {
    shorthand,
    named: createNamed(),
    method() { return true; },
    async asyncMethod() { return true; },
    nested: { value: true },
};
"#;
        let symbols = extract_exports(src);
        assert_eq!(
            symbols,
            vec![
                "direct",
                "qualified",
                "shorthand",
                "named",
                "method",
                "asyncMethod",
                "nested"
            ]
        );
    }

    #[test]
    fn test_commonjs_ignores_non_static_and_non_code_names() {
        let src = r#"
// exports.commentOnly = true;
const text = "module.exports.stringOnly = true";
const template = `exports.templateOnly = true`;
module.exports = {
    [computed]: value,
    ...extra,
    visible,
};
other.exports.notModule = true;
exports.notAssignment == true;
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["visible"]);
    }

    #[test]
    fn test_commonjs_mixed_with_esm_is_deduplicated() {
        let src = r#"
export const shared = true;
exports.shared = shared;
module.exports = { shared, commonOnly };
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["shared", "commonOnly"]);
    }

    #[test]
    fn test_commonjs_chains_ignore_function_scopes_and_regex_literals() {
        let src = r#"
exports.first = exports.second = value;
exports.third = module.exports.fourth = value;
const expression = /exports.regexOnly = module.exports = { hidden }/gi;
function fill(exports) {
    exports.functionOnly = true;
}
const run = (module) => {
    module.exports.arrowOnly = true;
};
"#;

        let symbols = extract_exports(src);

        assert_eq!(symbols, vec!["first", "second", "third", "fourth"]);
    }

    #[test]
    fn test_export_equals_does_not_match_equality() {
        // `export == Foo` is not valid TS, but `export =` should not greedily
        // consume an `=` inside an expression. This input has no real export.
        let src = r#"const x = 1; export = x;"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["x"]);
    }

    #[test]
    fn test_const_enum_export() {
        // Regression: `export const enum` is a common idiomatic TS pattern
        // (compile-time-only enums). The alternation used to match the bare
        // `const` branch first and then greedily capture the following
        // `enum` keyword as the symbol name instead of `Direction`.
        let src = r#"
export const enum Direction {
    Up,
    Down,
    Left,
    Right,
}
export const MAX_RETRIES = 3;
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Direction", "MAX_RETRIES"]);
    }

    #[test]
    fn test_declare_ambient_exports() {
        // Regression: `export declare function/class/const` ambient
        // declarations were dropped entirely because `declare` wasn't
        // recognized as a modifier between `export` and the declaration
        // keyword.
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
    fn test_learnxinyminutes_module_namespace_export() {
        // Regression: real excerpt from learnxinyminutes.com/typescript.
        // Legacy `module Foo { ... }` namespace syntax nests its `export`
        // one level inside a `module` block rather than at the top level.
        // This was previously untested even though both the regex and AST
        // backends already recurse/scan past the enclosing block.
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
    fn test_string_containing_slash_star_does_not_swallow_real_export() {
        // Real bug found this session: the old comment stripping was two bare
        // regexes (`//.*$`, `/\*.*?\*/`) run directly on raw source with NO
        // string-literal awareness. A string containing a `/*`-like sequence,
        // with a LATER real block comment elsewhere in the file, let the
        // string's `/*` merge with the real comment's `*/` and swallow every
        // real export in between -- `shouldBeFound` was silently dropped.
        let src = r#"
const pattern = "/* this looks like a comment start";
export function shouldBeFound() {}
/* a real comment */
export function afterRealComment() {}
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"shouldBeFound".to_string()),
            "{symbols:?}"
        );
        assert!(
            symbols.contains(&"afterRealComment".to_string()),
            "{symbols:?}"
        );
    }

    #[test]
    fn test_url_string_same_line_as_export_not_truncated() {
        // Real bug found this session: a `//` inside a URL string
        // (extremely common: fetch calls, import paths, doc links), with a
        // real export on the SAME line after it, was misread as a line
        // comment by the old non-string-aware COMMENT_SINGLE regex,
        // truncating the rest of the line and dropping `afterUrl` entirely.
        let src = r#"const API_URL = "http://example.com"; export function afterUrl() {}"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"afterUrl".to_string()), "{symbols:?}");
    }

    #[test]
    fn test_template_literal_with_comment_markers_does_not_hide_export() {
        // A backtick template literal (SQL/GraphQL) containing text that
        // looks like comment syntax (`--`, `/* */`) must not corrupt parsing
        // of the real export that follows.
        let src = r#"
const sql = `select * from t -- /* not a real comment */`;
export function afterTemplate() {}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["afterTemplate"]);
    }

    #[test]
    fn test_multiline_template_literal_with_unterminated_fake_comment() {
        // A multi-line template literal whose body contains an UNCLOSED
        // `/*`-like sequence (never matched by a `*/` inside the template)
        // must not merge with a real block comment appearing later in the
        // file and swallow the real export between them -- the template's
        // own closing backtick, not a `*/`, is what ends it.
        let src = "
const doc = `
This block mentions /* an opening marker with no matching close
inside the template.
`;
export function realExport() {}
/* genuinely unrelated trailing comment */
";
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["realExport"]);
    }

    #[test]
    fn test_escaped_quote_in_string_does_not_confuse_scanner() {
        // A string containing an escaped quote followed by a `//`-like
        // sequence, with a real export on the same line, must not have the
        // escaped quote mistaken for the string's terminator (which would
        // leave the scanner still "inside" a string one character early,
        // misreading the true closing quote as raw text and then the real
        // `//` as a genuine comment).
        let src =
            r#"const s = "she said \"hi\" // not a comment"; export function afterEscaped() {}"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"afterEscaped".to_string()), "{symbols:?}");
    }

    #[test]
    fn test_wildcard_export_import_path_survives_string_aware_stripping() {
        // The new string-aware comment stripper must NOT blank out quoted
        // import-path text (only the old naive regex-based approach would
        // have been safe to blank strings entirely) -- WILDCARD_EXPORT and
        // RE_EXPORT depend on `from '...'` surviving verbatim so the path can
        // still be captured/resolved.
        let src = r#"
export * as Utils from './utils';
export { Foo } from './module';
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Utils".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"Foo".to_string()), "{symbols:?}");
    }

    #[test]
    fn test_learnxinyminutes_interfaces_and_classes_real_excerpt() {
        // Real excerpt (verbatim) from learnxinyminutes.com/typescript,
        // covering structural interfaces, a class with constructor
        // shorthand properties, `implements`, inheritance via `extends`,
        // and the legacy namespace `module` export. None of the
        // interfaces/classes here are exported except `Square` nested in
        // `Geometry` -- this locks in that the extractor doesn't spuriously
        // pick up un-exported top-level declarations while still finding
        // the genuinely nested export, and that a realistic slice with this
        // much syntax breadth doesn't panic or produce garbage.
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
}
