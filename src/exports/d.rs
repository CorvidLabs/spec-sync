use regex::Regex;
use std::sync::LazyLock;

static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"//.*$").unwrap());

static COMMENT_MULTI: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/").unwrap());

/// D nested `/+ +/` comments aren't truly regex-representable (they nest arbitrarily), but a
/// single non-nested pass handles the common case seen in real code, matching the same
/// pragmatic approach other backends take with block comments.
static COMMENT_NESTING: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\+.*?\+/").unwrap());

/// D top-level declarations (everything not marked private/protected/package is public by
/// default — the opposite direction of Java/C++, same direction as Kotlin). We match:
/// class, struct, interface, enum, union, template, and function/variable declarations
/// (`ReturnType name(...)`, `auto name(...)`, or a template `T name(T)(...)`).
/// Then exclude lines that start with private/protected/package.
static D_DECL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*(?:(?:public|private|protected|package)\s+)?(?:(?:static|final|abstract|override|synchronized|const|immutable|nothrow|pure|@safe|@trusted|@system|@nogc)\s+)*(?:class|struct|interface|union|enum)\s+(\w+)",
    )
    .unwrap()
});

/// Function declarations, including templated functions like `T foo(T)(T x) { ... }` and
/// `auto`-returning functions. Captures the function name (the identifier immediately
/// preceding the first `(`), not the return type.
static D_FUNC_DECL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*(?:(?:public|private|protected|package)\s+)?(?:(?:static|final|abstract|override|synchronized|const|immutable|nothrow|pure|@safe|@trusted|@system|@nogc)\s+)*(?:[A-Za-z_]\w*(?:!\([^)]*\)|\!\w+)?(?:\[\])?(?:\s*\*)?)\s+(\w+)\s*(?:\([^)]*\))?\s*\(",
    )
    .unwrap()
});

/// Top-level `enum` member list (manifest constants without a preceding type, e.g.
/// `enum MAX = 100;`), which is a distinct D idiom from `enum Color { ... }` type
/// declarations — this one declares a single manifest constant, not a type.
static D_MANIFEST_CONST: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[^\S\n]*enum\s+(\w+)\s*=").unwrap());

/// Plain typed field/variable declarations with no parentheses at all, e.g. `int width;`
/// or `float x = 1.0;` — the shape a struct/class field or a module-level variable takes
/// (distinct from `D_FUNC_DECL`, which requires a parameter list).
static D_FIELD_DECL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*(?:(?:public|private|protected|package)\s+)?(?:(?:static|final|abstract|override|const|immutable|shared|__gshared|nothrow|pure)\s+)*[A-Za-z_]\w*(?:!\([^)]*\)|\!\w+)?(?:\[\])?(?:\s*\*)*\s+(\w+)\s*(?:=[^;]*)?;",
    )
    .unwrap()
});

/// Reserved words that can superficially look like a declared symbol name to the
/// permissive `D_FUNC_DECL`/`D_FIELD_DECL` patterns (e.g. `for (...)`, `return value;`)
/// but are actually control-flow/statement keywords, not declarations.
fn is_reserved_word(word: &str) -> bool {
    matches!(
        word,
        "if" | "while"
            | "for"
            | "foreach"
            | "foreach_reverse"
            | "switch"
            | "catch"
            | "return"
            | "cast"
            | "typeof"
            | "assert"
            | "with"
            | "scope"
            | "version"
            | "static"
            | "debug"
            | "mixin"
            | "else"
            | "case"
            | "break"
            | "continue"
            | "goto"
            | "throw"
            | "new"
            | "delete"
            | "import"
            | "module"
    )
}

/// Strip the contents of simple double-quoted string literals and single-quoted char literals
/// from a line (escapes respected) before brace-counting, so a brace character living inside a
/// literal — e.g. `"a { b"` or `'{'` — can't be mistaken for a real scope delimiter. This does
/// not attempt D's full string-literal grammar (raw `r"..."`, backtick, hex, or delimited `q{}`
/// strings) — matching the same pragmatic, common-case approach already used for comments.
fn strip_literals_for_brace_scan(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' | '\'' => {
                let quote = c;
                while let Some(next) = chars.next() {
                    if next == '\\' {
                        chars.next();
                    } else if next == quote {
                        break;
                    }
                }
            }
            _ => out.push(c),
        }
    }
    out
}

/// Detect visibility — private/protected/package lines should be excluded. D also supports
/// a "labeled" visibility block (`private:` on its own line, affecting all following
/// declarations until the next label or end of scope) which we handle separately.
static NON_PUBLIC_LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[^\S\n]*(?:private|protected|package)\s+").unwrap());

/// A `private:`/`protected:`/`package:` or `public:` label line, which switches the default
/// visibility for all subsequent declarations in the current scope until another label
/// appears or the scope closes.
static VISIBILITY_LABEL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^[^\S\n]*(private|protected|package|public)\s*:\s*$").unwrap()
});

/// Detect a line that opens a "type body" scope (class/struct/interface/union/enum), as
/// opposed to a function body or any other block. Declarations directly inside a type body
/// are real export candidates; declarations nested inside a function body — or any other
/// block, like a `for`/`if`/lambda — are local and are never exported.
static TYPE_BODY_OPEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*(?:(?:public|private|protected|package)\s+)?(?:(?:static|final|abstract)\s+)*(?:class|struct|interface|union|enum)\b",
    )
    .unwrap()
});

/// Extract exported symbols from D source code.
/// In D, everything at module scope is public by default unless marked private/protected/package.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = COMMENT_NESTING.replace_all(content, "");
    let stripped = COMMENT_MULTI.replace_all(&stripped, "");
    let stripped = COMMENT_SINGLE.replace_all(&stripped, "");

    let mut symbols = Vec::new();
    // Tracks nested `{ }` scopes: true = a class/struct/interface/union/enum body (member
    // declarations are legitimate export candidates), false = a function body or any other
    // block (declarations inside are local and never exported). An empty stack means top level.
    let mut scope_stack: Vec<bool> = Vec::new();
    // Tracks the "label" visibility (`private:` / `public:` etc) currently in effect for each
    // scope level; `true` means the label default is non-public.
    let mut label_stack: Vec<bool> = vec![false];

    for line in stripped.lines() {
        let trimmed = line.trim();

        // A bare visibility label switches the default for subsequent declarations in this
        // scope; it does not itself declare a symbol.
        if let Some(caps) = VISIBILITY_LABEL.captures(line) {
            let label = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            if let Some(top) = label_stack.last_mut() {
                *top = label != "public";
            }
            continue;
        }

        let in_exportable_scope = scope_stack.last().copied().unwrap_or(true);
        let label_hides = label_stack.last().copied().unwrap_or(false);
        let explicit_visibility = NON_PUBLIC_LINE.is_match(line);
        // Whether *this* declaration line itself sits in exported territory. A type body that
        // opens here should only be treated as an export-worthy scope (see `opens_type_body`
        // below) when this line itself qualifies — otherwise members of a `private`-marked type,
        // or of a type nested inside a function body, would incorrectly leak as exports even
        // though the type itself was correctly excluded.
        let is_exportable_here = in_exportable_scope
            && !explicit_visibility
            && (!label_hides || trimmed.starts_with("public"));

        if is_exportable_here {
            if let Some(caps) = D_DECL.captures(line) {
                if let Some(name) = caps.get(1) {
                    let n = name.as_str().to_string();
                    if !symbols.contains(&n) {
                        symbols.push(n);
                    }
                }
            } else if let Some(caps) = D_MANIFEST_CONST.captures(line) {
                if let Some(name) = caps.get(1) {
                    let n = name.as_str().to_string();
                    if !symbols.contains(&n) {
                        symbols.push(n);
                    }
                }
            } else if let Some(caps) = D_FUNC_DECL.captures(line) {
                if let Some(name) = caps.get(1) {
                    let n = name.as_str().to_string();
                    if !is_reserved_word(&n) && !symbols.contains(&n) {
                        symbols.push(n);
                    }
                }
            } else if let Some(caps) = D_FIELD_DECL.captures(line)
                && let Some(name) = caps.get(1)
            {
                let n = name.as_str().to_string();
                if !is_reserved_word(&n) && !symbols.contains(&n) {
                    symbols.push(n);
                }
            }
        }

        // Only treat a freshly-opened type body as an "exportable scope" (and thus a candidate
        // for its members to be captured) if the opening declaration itself was in exported
        // territory. Without this guard, a `private struct Foo { ... }` or a struct declared
        // inside a function body would still push `true`, leaking its members as exports even
        // though the type itself is correctly excluded above.
        let opens_type_body = is_exportable_here && TYPE_BODY_OPEN.is_match(line);
        // Strip (simple, non-nested) double-quoted string and single-quoted char literal
        // contents before counting braces, so a stray `{`/`}` inside a literal — e.g.
        // `writefln("Value: %s}", x);` or `if (c == '{') {` — can't desynchronize the scope
        // stack for the rest of the file. This mirrors the pragmatic, common-case-only
        // approach already used for comments above.
        let brace_scan_line = strip_literals_for_brace_scan(line);
        for ch in brace_scan_line.chars() {
            match ch {
                '{' => {
                    scope_stack.push(opens_type_body);
                    label_stack.push(false);
                }
                '}' => {
                    scope_stack.pop();
                    label_stack.pop();
                    if label_stack.is_empty() {
                        label_stack.push(false);
                    }
                }
                _ => {}
            }
        }
    }

    symbols
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_d_module_level_public_by_default() {
        let src = r#"
module myapp.auth;

import std.stdio;

class AuthService {
    void login() {}
}

struct Point {
    int x;
    int y;
}

interface Shape {
    double area();
}

enum Color { Red, Green, Blue }

void connect() {}

int retryCount = 3;
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"AuthService".to_string()));
        assert!(symbols.contains(&"Point".to_string()));
        assert!(symbols.contains(&"Shape".to_string()));
        assert!(symbols.contains(&"Color".to_string()));
        assert!(symbols.contains(&"connect".to_string()));
    }

    #[test]
    fn test_d_private_excluded() {
        let src = r#"
private class Internal {}
private struct Cache {}
private void helper() {}
protected int protectedField;
package void packageOnly() {}
class Public {}
void publicFn() {}
"#;
        let symbols = extract_exports(src);
        assert!(!symbols.contains(&"Internal".to_string()));
        assert!(!symbols.contains(&"Cache".to_string()));
        assert!(!symbols.contains(&"helper".to_string()));
        assert!(!symbols.contains(&"packageOnly".to_string()));
        assert!(symbols.contains(&"Public".to_string()));
        assert!(symbols.contains(&"publicFn".to_string()));
    }

    #[test]
    fn test_d_comment_stripping() {
        let src = r#"
// class FakeClass {}
/* struct FakeStruct {
   private int x;
} */
/+ interface NestedComment {
    void ghost();
} +/
class RealClass { // trailing comment
    void realMethod() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"RealClass".to_string()));
        assert!(symbols.contains(&"realMethod".to_string()));
        assert!(!symbols.contains(&"FakeClass".to_string()));
        assert!(!symbols.contains(&"FakeStruct".to_string()));
        assert!(!symbols.contains(&"NestedComment".to_string()));
        assert!(!symbols.contains(&"ghost".to_string()));
    }

    #[test]
    fn test_d_template_function() {
        // Realistic D idiom: a templated function with a separate compile-time parameter
        // list followed by the runtime parameter list, e.g. `T max(T)(T a, T b) { ... }`.
        let src = r#"
T max(T)(T a, T b) {
    return a > b ? a : b;
}

private T clamp(T)(T val, T lo, T hi) {
    return val < lo ? lo : (val > hi ? hi : val);
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"max".to_string()));
        assert!(!symbols.contains(&"clamp".to_string()));
    }

    #[test]
    fn test_d_visibility_label_block() {
        // D supports a "labeled" visibility affecting all declarations until the next label,
        // distinct from a per-declaration modifier.
        let src = r#"
class Widget {
private:
    int secretState;
    void hiddenHelper() {}

public:
    void render() {}
    int width;
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Widget".to_string()));
        assert!(symbols.contains(&"render".to_string()));
        assert!(symbols.contains(&"width".to_string()));
        assert!(!symbols.contains(&"secretState".to_string()));
        assert!(!symbols.contains(&"hiddenHelper".to_string()));
    }

    #[test]
    fn test_d_manifest_constant_enum() {
        // A single manifest constant declared with `enum NAME = value;` (no braces) is a
        // distinct idiom from `enum Color { ... }` type declarations.
        let src = r#"
enum MAX_CONNECTIONS = 100;
private enum INTERNAL_LIMIT = 5;
enum Status { Active, Inactive }
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"MAX_CONNECTIONS".to_string()));
        assert!(symbols.contains(&"Status".to_string()));
        assert!(!symbols.contains(&"INTERNAL_LIMIT".to_string()));
    }

    #[test]
    fn test_d_locals_not_exported() {
        let src = r#"
void computeTotal() {
    int subtotal = 10;
    auto tax = subtotal * 0.08;
    for (int i = 0; i < 3; i++) {
        int adjusted = i * 2;
    }
}

private void helper() {
    int secretLocal = 42;
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["computeTotal".to_string()]);
    }

    #[test]
    fn test_d_struct_with_methods_and_ufcs_style_free_function() {
        let src = r#"
struct Vector3 {
    float x;
    float y;
    float z;

    float length() {
        return 0.0;
    }
}

// UFCS-friendly free function operating on Vector3
float dot(Vector3 a, Vector3 b) {
    return a.x * b.x + a.y * b.y + a.z * b.z;
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Vector3".to_string()));
        assert!(symbols.contains(&"length".to_string()));
        assert!(symbols.contains(&"dot".to_string()));
    }

    #[test]
    fn test_d_private_struct_members_not_leaked() {
        // Regression test: members of a `private` struct/class must not be treated as
        // exported just because they sit inside a "type body" scope. Only the outer
        // declaration's own visibility should gate whether its body counts as exportable.
        let src = r#"
private struct Cache {
    int hitCount;
    void store() {}
}

private class Session {
    string token;
}

void publicFn() {}
"#;
        let symbols = extract_exports(src);
        assert!(!symbols.contains(&"Cache".to_string()));
        assert!(!symbols.contains(&"hitCount".to_string()));
        assert!(!symbols.contains(&"store".to_string()));
        assert!(!symbols.contains(&"Session".to_string()));
        assert!(!symbols.contains(&"token".to_string()));
        assert!(symbols.contains(&"publicFn".to_string()));
    }

    #[test]
    fn test_d_locally_nested_struct_members_not_leaked() {
        // Regression test: a struct declared inside a function body is local. Its own name
        // is (correctly) never captured since the enclosing function scope isn't exportable,
        // but its *members* were leaking as top-level exports because the type-body scope was
        // still pushed as "exportable" regardless of the surrounding context.
        let src = r#"
void process() {
    struct Local {
        int secretField;
        void secretMethod() {}
    }
    Local l;
}

void publicApi() {}
"#;
        let symbols = extract_exports(src);
        assert!(!symbols.contains(&"Local".to_string()));
        assert!(!symbols.contains(&"secretField".to_string()));
        assert!(!symbols.contains(&"secretMethod".to_string()));
        assert!(symbols.contains(&"process".to_string()));
        assert!(symbols.contains(&"publicApi".to_string()));
    }

    #[test]
    fn test_d_brace_inside_string_and_char_literals_does_not_desync_scope() {
        // Regression test: a brace character living inside a string or char literal (a
        // realistic case for format strings / lexer-style code) must not be counted as a
        // real scope delimiter, or every declaration for the rest of the file after an
        // unbalanced literal brace would be silently mis-scoped.
        let src = r#"
string greeting = "hello {name}, welcome!";
string weird = "a { b";

void logMessage() {
    writefln("progress: %s}", 42);
}

struct Lexer {
    char current;

    bool isOpenBrace() {
        return current == '{';
    }
}

void afterEverything() {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"logMessage".to_string()));
        assert!(symbols.contains(&"Lexer".to_string()));
        assert!(symbols.contains(&"isOpenBrace".to_string()));
        assert!(symbols.contains(&"afterEverything".to_string()));
    }

    #[test]
    fn test_d_attributes_do_not_block_match() {
        let src = r#"
@safe pure nothrow int add(int a, int b) {
    return a + b;
}

@nogc void freeStanding() {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"add".to_string()));
        assert!(symbols.contains(&"freeStanding".to_string()));
    }
}
