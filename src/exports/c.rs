use regex::Regex;
use std::sync::LazyLock;

static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"//.*$").unwrap());
static COMMENT_MULTI: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/").unwrap());

/// C types: struct, union, enum
static C_TYPE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[^\S\n]*(?:struct|union|enum)\s+(\w+)").unwrap());

/// `typedef struct/union/enum { ... } Alias;` — captures the optional tag
/// (group 1, e.g. `Foo_s`) and the trailing typedef alias (group 2, e.g.
/// `Foo`), the actual public name callers use (`Foo f;`). This is the only
/// place the name appears when the body is anonymous (`typedef struct { ... }
/// Point3D;`), and it's the real public name (as opposed to the internal tag)
/// when the tag and alias differ. `C_TYPE` above never matches these lines
/// because they start with `typedef`, not `struct`/`union`/`enum`, at bol.
static C_TYPEDEF_ALIAS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?ms)^[^\S\n]*typedef\s+(?:struct|union|enum)\s*(\w+)?\s*\{.*?\}\s*(\w+)\s*;")
        .unwrap()
});

/// `typedef <return-type> (*Alias)(<params>);` — a function-pointer typedef,
/// e.g. `typedef int (*Callback)(int, void*);`. The prefix before the first
/// `(` excludes `{`/`}`/newline so this can never span past a struct/enum
/// body into an unrelated function-pointer *member* declaration inside it
/// (e.g. `struct Shape { double (*area)(struct Shape*); };`, which doesn't
/// start with `typedef` anyway).
static C_TYPEDEF_FN_PTR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^[^\S\n]*typedef\s+[^(;{}\n]*\(\s*\*\s*(\w+)\s*\)\s*\(").unwrap()
});

/// `typedef struct/union/enum Tag Alias;` — re-typedefs an already-tagged struct/union/
/// enum (defined elsewhere, or even in the same translation unit further down) under a
/// public alias with no body of its own, e.g. `typedef struct Point Point;` (the classic
/// "opaque handle" idiom) or `typedef struct Foo_s Foo;` (tag/alias mismatch, no braces).
/// `C_TYPEDEF_ALIAS` above never matches these lines because it requires a `{ ... }`
/// body; `C_TYPE` never matches them either because the line starts with `typedef`, not
/// `struct`/`union`/`enum`, at bol. Captures only the alias (group 1): the tag itself is
/// either an internal-only name or is separately captured by `C_TYPE` wherever its actual
/// `struct { ... };` definition appears.
static C_TYPEDEF_TAG_ALIAS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^[^\S\n]*typedef\s+(?:struct|union|enum)\s+\w+\s+(\w+)\s*;").unwrap()
});

/// C top-level functions (which could include keywords, filtered in rust). The parameter
/// list allows one level of nested parens (`(?:[^()]|\([^()]*\))*`) so a callback
/// parameter of function-pointer type (e.g. `int register_callback(int (*cb)(int),
/// void *ctx);`) doesn't break the match: a plain `[^)]*` parameter list stops at the
/// callback's own closing `)`, leaving the real terminator (`;`/`{`) unreachable from
/// that position and silently dropping the entire declaration, not just the callback
/// parameter. A trailing GCC/Clang `__attribute__((...))` — a common way to annotate
/// a declaration with e.g. `noreturn`/`warn_unused_result` — is likewise optional
/// between the closing `)` and the terminator for the same reason.
static C_FUNCTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*(?:[\w*&]+\s+)+\*?(\w+)\s*\((?:[^()]|\([^()]*\))*\)\s*(?:__attribute__\s*\(\([^()]*\)\)\s*)?[{;]",
    )
    .unwrap()
});

/// `C_FUNCTION`'s "one-or-more word-then-space" prefix is meant to match a return-type
/// (and qualifiers like `static`/`const`) before a function name, but a bare `return
/// someFunc(a, b);` statement has exactly the same shape — `return` satisfies the
/// prefix, and `someFunc` gets captured as if it were a declared function name, leaking
/// whatever it calls (even a `static`/file-private function) as an export. Unlike the
/// keyword filter on the *captured name*, this checks whether `return` is what's
/// actually filling the prefix slot.
fn starts_with_return(full_match: &str) -> bool {
    full_match
        .trim_start()
        .strip_prefix("return")
        .is_some_and(|rest| rest.starts_with(|c: char| c.is_whitespace()))
}

/// Extract public symbols from C source code.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = COMMENT_SINGLE.replace_all(content, "");
    let stripped = COMMENT_MULTI.replace_all(&stripped, "");

    let keywords: std::collections::HashSet<&str> = [
        "static",
        "private",
        "protected",
        "return",
        "if",
        "while",
        "for",
        "switch",
        "typedef",
        "struct",
        "class",
        "union",
        "enum",
        "namespace",
        "using",
        "friend",
        "inline",
        "virtual",
        "const",
        "constexpr",
        "extern",
        "void",
        "int",
        "char",
        "float",
        "double",
    ]
    .iter()
    .cloned()
    .collect();

    let mut symbols = Vec::new();

    for caps in C_TYPE.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            symbols.push(name.as_str().to_string());
        }
    }

    for caps in C_TYPEDEF_ALIAS.captures_iter(&stripped) {
        if let Some(tag) = caps.get(1) {
            symbols.push(tag.as_str().to_string());
        }
        if let Some(alias) = caps.get(2) {
            symbols.push(alias.as_str().to_string());
        }
    }

    for caps in C_TYPEDEF_TAG_ALIAS.captures_iter(&stripped) {
        if let Some(alias) = caps.get(1) {
            symbols.push(alias.as_str().to_string());
        }
    }

    for caps in C_TYPEDEF_FN_PTR.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            symbols.push(name.as_str().to_string());
        }
    }

    for caps in C_FUNCTION.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str().to_string();
            if !keywords.contains(n.as_str()) {
                let full_match = caps.get(0).unwrap().as_str();
                if !full_match.contains("static")
                    && !starts_with_return(full_match)
                    && !symbols.contains(&n)
                {
                    symbols.push(n);
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
    fn test_anonymous_typedef_and_tag_alias_mismatch_captured() {
        // Anonymous struct/enum bodies have no name of their own — the
        // typedef's trailing alias is the *only* place the public name
        // appears. When the tag and alias differ, the alias (not the
        // internal tag) is the name callers actually use (`Foo f;`).
        let src = r#"
typedef struct {
    int x;
    int y;
} Point3D;

typedef enum { RED, GREEN, BLUE } Color;

typedef struct Foo_s {
    int value;
} Foo;
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"Point3D".to_string()),
            "expected Point3D in {symbols:?}"
        );
        assert!(
            symbols.contains(&"Color".to_string()),
            "expected Color in {symbols:?}"
        );
        assert!(
            symbols.contains(&"Foo".to_string()),
            "expected the typedef alias Foo (not just the internal tag Foo_s) in {symbols:?}"
        );
    }

    #[test]
    fn test_function_pointer_typedef_captured() {
        // `typedef void (*LogHandler)(...);` declares a public type alias
        // used as a parameter type in another public function's signature;
        // it must be captured alongside that function.
        let src = r#"
typedef void (*LogHandler)(const char *msg, void *userdata);

void context_set_log_handler(Context *ctx, LogHandler handler);
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"LogHandler".to_string()),
            "expected LogHandler in {symbols:?}"
        );
        assert!(symbols.contains(&"context_set_log_handler".to_string()));
    }

    #[test]
    fn test_struct_with_function_pointer_member_not_misattributed() {
        // A poor-man's vtable: a struct whose member is a function pointer.
        // The member itself (`area`) is not a typedef and must not be
        // captured as a type alias; only the struct name is.
        let src = r#"
struct Shape {
    double (*area)(struct Shape *self);
};
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Shape".to_string()));
        assert!(!symbols.contains(&"area".to_string()));
    }

    #[test]
    fn test_return_call_not_misparsed_as_declaration() {
        // Regression test: `C_FUNCTION`'s "one-or-more word-then-space" prefix (meant to
        // match a return type/qualifiers) is also satisfied by the single word `return`,
        // so `return helper_sum(a, b);` was misparsed as a declaration of `helper_sum`
        // -- leaking it as exported even though it's `static` (file-private).
        let src = r#"
static int helper_sum(int a, int b) {
    return a + b;
}

int public_wrapper(int a, int b) {
    return helper_sum(a, b);
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"public_wrapper".to_string()));
        assert!(!symbols.contains(&"helper_sum".to_string()));
    }

    #[test]
    fn test_c_exports() {
        let src = r#"
// This is a comment
/* Multi-line
   comment */
struct User {
    int id;
};
union Data {
    int i;
};
enum Color { RED, GREEN };

int calculate_sum(int a, int b) {
    return a + b;
}

static void helper_func() {
}

char* get_name() {
    return "C";
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"User".to_string()));
        assert!(symbols.contains(&"Data".to_string()));
        assert!(symbols.contains(&"Color".to_string()));
        assert!(symbols.contains(&"calculate_sum".to_string()));
        assert!(symbols.contains(&"get_name".to_string()));
        assert!(!symbols.contains(&"helper_func".to_string()));
    }

    #[test]
    fn test_function_pointer_callback_parameter_not_dropped() {
        // Bug found via adversarial probing: `C_FUNCTION`'s parameter-list group was a
        // flat `[^)]*`, which can never contain a `)`. A callback parameter of
        // function-pointer type (an extremely common C API idiom, e.g. qsort/bsearch
        // style comparators) has its own closing `)` before the real parameter list
        // ends, so the flat pattern stopped there and the terminator (`;`/`{`) was
        // unreachable from that position -- the *entire* declaration silently failed
        // to match and `register_callback` was dropped, not just the callback param.
        let src = r#"
int register_callback(int (*cb)(int, void*), void *ctx);
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"register_callback".to_string()),
            "expected register_callback in {symbols:?}"
        );
        assert!(
            !symbols.contains(&"cb".to_string()),
            "the callback parameter name itself must not be captured"
        );
    }

    #[test]
    fn test_attribute_decorated_declaration_captured() {
        // Bug found via adversarial probing: a trailing GCC/Clang
        // `__attribute__((...))` between the closing `)` and the terminating `;`/`{`
        // (a common way to annotate a public declaration, e.g. `noreturn`) made the
        // fixed `\s*[{;]` tail fail to match immediately after the params, dropping
        // the whole declaration.
        let src = r#"
void die(void) __attribute__((noreturn));
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"die".to_string()),
            "expected die in {symbols:?}"
        );
    }

    #[test]
    fn test_typedef_tag_realias_without_body_captured() {
        // Bug found via adversarial probing: `typedef struct Tag Alias;` (the classic
        // "opaque handle"/tag-realias idiom, with no `{ ... }` body on this line) matched
        // neither `C_TYPE` (line doesn't start with `struct`/`union`/`enum`) nor
        // `C_TYPEDEF_ALIAS` (requires a brace body), so the public alias was dropped
        // entirely even though it's the name every caller actually uses.
        let src = r#"
typedef struct Point Point;
typedef struct Foo_s Foo;
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"Point".to_string()),
            "expected Point in {symbols:?}"
        );
        assert!(
            symbols.contains(&"Foo".to_string()),
            "expected Foo in {symbols:?}"
        );
    }

    #[test]
    fn test_multiline_return_type_signature_captured() {
        // Documents correct (already-working) behavior: a return type on its own line
        // followed by the function name/params on the next line is real, if unusual, C
        // style. `C_FUNCTION`'s inter-token separator is plain `\s+` (not `[^\S\n]+`),
        // so it already spans the newline between "int" and the name -- this is not a
        // gap, just an under-documented case worth pinning down.
        let src = r#"
int
multiline_signature(int a, int b) {
    return a + b;
}
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"multiline_signature".to_string()),
            "expected multiline_signature in {symbols:?}"
        );
    }
}
