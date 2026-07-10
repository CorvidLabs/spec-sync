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
///
/// `template` is included alongside class/struct/interface/union/enum here: a bare
/// `template Name(Args) { ... }` block declares a named symbol the same way a class does
/// (distinct from a *templated function* like `T max(T)(T a, T b) { ... }`, which is already
/// captured by `D_FUNC_DECL` below).
static D_DECL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*(?:(?:public|private|protected|package)\s+)?(?:(?:static|final|abstract|override|synchronized|const|immutable|nothrow|pure|@safe|@trusted|@system|@nogc)\s+)*(?:class|struct|interface|union|enum|template)\s+(\w+)",
    )
    .unwrap()
});

/// Function declarations, including templated functions like `T foo(T)(T x) { ... }` and
/// `auto`-returning functions. Captures the function name (the identifier immediately
/// preceding the first `(`), not the return type.
///
/// `extern(C)`/`extern(C++)` (with an optional linkage argument in parens) is accepted as a
/// modifier so that bare `extern(C) void foo();`-style single-line declarations — common for
/// C-interop code — are recognized like any other function declaration.
static D_FUNC_DECL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*(?:(?:public|private|protected|package)\s+)?(?:(?:static|final|abstract|override|synchronized|const|immutable|nothrow|pure|@safe|@trusted|@system|@nogc|extern(?:\s*\([^)]*\))?)\s+)*(?:[A-Za-z_]\w*(?:!\([^)]*\)|\!\w+)?(?:\[\])?(?:\s*\*)?)\s+(\w+)\s*(?:\([^)]*\))?\s*\(",
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
        r"(?m)^[^\S\n]*(?:(?:public|private|protected|package)\s+)?(?:(?:static|final|abstract|override|const|immutable|shared|__gshared|nothrow|pure|extern(?:\s*\([^)]*\))?)\s+)*[A-Za-z_]\w*(?:!\([^)]*\)|\!\w+)?(?:\[\])?(?:\s*\*)*\s+(\w+)\s*(?:=[^;]*)?;",
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

/// Detect a line that opens a "type body" scope (class/struct/interface/union/enum/template), as
/// opposed to a function body or any other block. Declarations directly inside a type body
/// are real export candidates; declarations nested inside a function body — or any other
/// block, like a `for`/`if`/lambda — are local and are never exported. `template` is included
/// here to match `D_DECL` — a `template Name(Args) { ... }` block's own members are exportable
/// under the same rules as a class/struct's members.
static TYPE_BODY_OPEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*(?:(?:public|private|protected|package)\s+)?(?:(?:static|final|abstract)\s+)*(?:class|struct|interface|union|enum|template)\b",
    )
    .unwrap()
});

/// Detect an `extern(C)`/`extern(C++)` linkage block opener, e.g. `extern(C) {` (optionally
/// with the brace on the same line). Unlike a class/struct body, this block is *transparent*:
/// it doesn't declare a named type of its own, so declarations inside it should inherit the
/// exportability of the surrounding scope rather than being force-classified as a non-exported
/// function-body-like scope.
static EXTERN_BLOCK_OPEN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[^\S\n]*extern\s*\([^)]*\)\s*\{?\s*$").unwrap());

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
    // Carries the most recently computed "does this opening brace start an exportable-member
    // scope?" verdict across lines. This matters for Allman-style brace placement, where the
    // `{` that actually opens the scope sits on its own line, separate from the `class Foo`/
    // `extern(C)` line that determined whether the resulting scope should be treated as a type
    // body. Without carrying this forward, a brace-only line would be judged in isolation —
    // never matching `TYPE_BODY_OPEN`/`EXTERN_BLOCK_OPEN` — and the class/extern-block's own
    // members would be wrongly classified as living inside a plain (non-exportable) block.
    let mut last_scope_open_value = false;

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
        //
        // A blank line, or a line consisting only of brace characters (and whitespace),
        // carries no declaration content of its own — it's classified purely by whatever the
        // most recent "real" line determined (see `last_scope_open_value` above), so
        // Allman-style brace placement (and any blank lines separating a declaration from its
        // opening brace) doesn't lose the scope classification made on the declaration line
        // above it.
        let is_trivial_line = trimmed.is_empty() || trimmed.chars().all(|c| c == '{' || c == '}');
        let opens_type_body = if is_trivial_line {
            last_scope_open_value
        } else {
            // `EXTERN_BLOCK_OPEN` (a transparent `extern(C) { ... }` linkage block) is folded
            // into the same "is this a scope-opening line?" check as `TYPE_BODY_OPEN`: since
            // `is_exportable_here` already captures this line's own visibility gating, treating
            // the extern block the same way as a type body means its contents simply inherit
            // whatever exportability this line already had — exactly the transparent behavior
            // `extern(C)` should have (it declares no type of its own).
            let value = is_exportable_here
                && (TYPE_BODY_OPEN.is_match(line) || EXTERN_BLOCK_OPEN.is_match(line));
            last_scope_open_value = value;
            value
        };
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

    #[test]
    fn test_d_public_template_block_and_members_are_exported() {
        // Bug found via adversarial testing: a `template Name(Args) { ... }` block (distinct
        // from a *templated function* like `T max(T)(T a, T b) { ... }`, which already worked)
        // was not recognized at all — neither the template's own name, nor its members, were
        // captured — because `template` was missing from both `D_DECL`'s and
        // `TYPE_BODY_OPEN`'s keyword alternation, despite the doc comment on `D_DECL` already
        // (incorrectly) claiming templates were matched. Fixed by adding `template` to both.
        let src = r#"
public template Bar(T) {
    void baz() {
        int local = 1;
    }
    int width;
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Bar".to_string()));
        assert!(symbols.contains(&"baz".to_string()));
        assert!(symbols.contains(&"width".to_string()));
        assert!(!symbols.contains(&"local".to_string()));
    }

    #[test]
    fn test_d_private_template_block_and_members_are_excluded() {
        // Companion to the public-template fix above: a `private template Foo(T) { ... }`
        // must have its name and its members excluded, the same way a private class/struct is.
        let src = r#"
private template Foo(T) {
    void hidden() {}
    int secretField;
}

void publicFn() {}
"#;
        let symbols = extract_exports(src);
        assert!(!symbols.contains(&"Foo".to_string()));
        assert!(!symbols.contains(&"hidden".to_string()));
        assert!(!symbols.contains(&"secretField".to_string()));
        assert!(symbols.contains(&"publicFn".to_string()));
    }

    #[test]
    fn test_d_allman_style_class_members_are_exported() {
        // Real bug found via adversarial testing: when a class/struct is written Allman-style
        // (opening `{` alone on its own line, not on the same line as `class Foo`), the
        // per-line `opens_type_body` check was recomputed fresh against the brace-only line —
        // which never matches `TYPE_BODY_OPEN` (it has no `class`/`struct` keyword on it) — so
        // the scope was wrongly pushed as a non-exportable "function body" scope. That silently
        // dropped every member of an Allman-style-formatted public type from the export list,
        // even though the identical class in K&R style worked fine. Fixed by carrying the most
        // recently computed scope-open verdict forward across brace-only (and blank) lines.
        let src = r#"
class Widget
{
    void render() {}
    int width;
}

struct Point
{
    int x;
    int y;
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Widget".to_string()));
        assert!(symbols.contains(&"render".to_string()));
        assert!(symbols.contains(&"width".to_string()));
        assert!(symbols.contains(&"Point".to_string()));
        assert!(symbols.contains(&"x".to_string()));
        assert!(symbols.contains(&"y".to_string()));
    }

    #[test]
    fn test_d_allman_style_private_class_members_still_excluded() {
        // Companion to the Allman-brace fix above: confirm the carried-forward scope
        // classification still correctly excludes members of a `private` Allman-style class —
        // the fix must not accidentally make everything exportable regardless of visibility.
        let src = r#"
private class Hidden
{
    void secretMethod() {}
    int secretField;
}

void publicApi() {}
"#;
        let symbols = extract_exports(src);
        assert!(!symbols.contains(&"Hidden".to_string()));
        assert!(!symbols.contains(&"secretMethod".to_string()));
        assert!(!symbols.contains(&"secretField".to_string()));
        assert!(symbols.contains(&"publicApi".to_string()));
    }

    #[test]
    fn test_d_extern_c_block_members_are_exported() {
        // Real bug found via adversarial testing: an `extern(C) { ... }` linkage block doesn't
        // declare a type of its own — it's transparent — but it was being treated like any
        // other non-type brace block (function body, `if`/`for`, etc), so its member function
        // declarations were force-classified as local and dropped from the export list even
        // though they sit at effectively module (top-level) scope. Fixed by recognizing
        // `extern(C)`-style linkage block openers and having their contents inherit the
        // enclosing scope's exportability instead of being marked non-exportable.
        let src = r#"
extern(C) {
    void cFunction();
    int cGlobal;
}

void afterExternBlock() {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"cFunction".to_string()));
        assert!(symbols.contains(&"cGlobal".to_string()));
        assert!(symbols.contains(&"afterExternBlock".to_string()));
    }

    #[test]
    fn test_d_extern_c_bare_single_line_declaration_is_exported() {
        // Real bug found via adversarial testing: a bare, un-blocked `extern(C) void foo();`
        // declaration (very common for one-off C-interop bindings) failed to match
        // `D_FUNC_DECL` at all, because `extern(C)` wasn't recognized as an optional modifier
        // token, so the regex's required "type name(" shape never lined up. Fixed by adding
        // `extern(?:\s*\([^)]*\))?` to the modifier alternation in `D_FUNC_DECL`/`D_FIELD_DECL`.
        let src = r#"
extern(C) void cConnect();
extern (C++) int cAdd(int a, int b);
private extern(C) void cHiddenHelper();
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"cConnect".to_string()));
        assert!(symbols.contains(&"cAdd".to_string()));
        assert!(!symbols.contains(&"cHiddenHelper".to_string()));
    }

    #[test]
    fn test_d_visibility_label_restored_after_nested_scope_closes() {
        // Verified-correct (not a bug): a `private:` label's effect is scoped to the block it
        // appears in. A nested scope (e.g. a locally-declared struct) starts with its own
        // fresh default (public) label, and once that nested scope closes, the outer `private:`
        // label must still be in effect for subsequent declarations in the outer scope.
        let src = r#"
class Widget {
private:
    void hiddenBefore() {}
    struct Inner {
        void innerMethod() {}
    }
    void hiddenAfter() {}
public:
    void visible() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Widget".to_string()));
        assert!(symbols.contains(&"visible".to_string()));
        assert!(!symbols.contains(&"hiddenBefore".to_string()));
        assert!(!symbols.contains(&"hiddenAfter".to_string()));
        assert!(!symbols.contains(&"Inner".to_string()));
        assert!(!symbols.contains(&"innerMethod".to_string()));
    }

    #[test]
    fn test_d_nested_free_function_inside_function_not_leaked() {
        // Verified-correct (not a bug): a UFCS-style free function declared *inside* another
        // function's body is local to that function and must never be treated as a top-level
        // export, unlike a genuine top-level free function operating on the same type.
        let src = r#"
void outer() {
    void innerHelper() {
        int x = 1;
    }
    innerHelper();
}

void topLevelHelper() {}
"#;
        let symbols = extract_exports(src);
        assert!(!symbols.contains(&"innerHelper".to_string()));
        assert!(symbols.contains(&"outer".to_string()));
        assert!(symbols.contains(&"topLevelHelper".to_string()));
    }

    #[test]
    fn test_d_multiline_template_constrained_function_signature() {
        // Verified-correct (not a bug): a templated function whose constraint clause
        // (`if (...)`) spans a separate line from the parameter list, with the body brace on
        // yet another line, must still have its name captured and must not have the
        // constraint clause's own `if (...)` misparsed as a nested declaration.
        let src = r#"
T combine(T)(T a, T b)
if (is(T == int) || is(T == float))
{
    return a + b;
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["combine".to_string()]);
    }

    #[test]
    fn test_d_one_liner_type_stubs() {
        // Verified-correct (not a bug): fully one-line, empty-body type stubs (a common
        // placeholder/forward-declaration idiom) must still have their own name captured, and
        // a `private` one-liner stub must still be excluded, without either desyncing the
        // scope stack for declarations that follow.
        let src = r#"
class Foo {}
struct Bar {}
private class Hidden {}
void afterStubs() {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Foo".to_string()));
        assert!(symbols.contains(&"Bar".to_string()));
        assert!(!symbols.contains(&"Hidden".to_string()));
        assert!(symbols.contains(&"afterStubs".to_string()));
    }

    #[test]
    fn test_d_label_like_text_inside_string_literal_does_not_desync_labels() {
        // Verified-correct (not a bug): a string literal that happens to contain
        // label-like text (e.g. `"public: fake label"`) must not be mistaken for a real
        // `VISIBILITY_LABEL` line, since the label regex only matches when the *entire*
        // trimmed line is just `keyword\s*:\s*$` — not when that text is embedded mid-line
        // inside a string.
        let src = r#"
string label = "public: fake label";
private class Hidden {}
void afterFakeLabel() {}
"#;
        let symbols = extract_exports(src);
        assert!(!symbols.contains(&"Hidden".to_string()));
        assert!(symbols.contains(&"afterFakeLabel".to_string()));
    }
}
