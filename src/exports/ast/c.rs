use tree_sitter::{Node, Parser, Tree};

/// Parse C source into a tree-sitter AST.
fn parse_c(content: &str) -> Option<Tree> {
    let mut parser = Parser::new();
    let language = tree_sitter_c::LANGUAGE.into();
    parser.set_language(&language).ok()?;
    parser.parse(content, None)
}

/// Extract exported symbols from C source using tree-sitter AST.
///
/// Handles:
/// - Top-level `struct`/`union`/`enum` names (standalone or nested inside a `typedef`)
/// - Typedef aliases, including anonymous struct/enum bodies and
///   tag/alias mismatches (`typedef struct Foo_s { ... } Foo;` -> `Foo`)
/// - Function-pointer typedefs (`typedef int (*Callback)(int);` -> `Callback`)
/// - Top-level non-`static` function definitions and prototypes (declarations)
/// - Pointer-returning functions (e.g. `char* get_name()`)
/// - Declarations wrapped in `#ifdef`/`#ifndef`/`#elif`/`#else` preprocessor
///   conditionals and in `extern "C" { ... }` linkage-specification blocks
///   (including nested combinations of the two), both extremely common in
///   real C headers
/// - Correctly ignores text inside string/comment literals (AST-native)
pub fn extract_exports(content: &str) -> Vec<String> {
    let tree = match parse_c(content) {
        Some(t) => t,
        None => return Vec::new(),
    };

    let root = tree.root_node();
    let src = content.as_bytes();
    let mut symbols = Vec::new();

    // Flatten out preprocessor-conditional and `extern "C"` wrapper nodes so
    // declarations nested inside them (at any depth) are visited exactly
    // like true top-level declarations. Without this, a header's routine
    // `#ifdef __cplusplus` / `extern "C" { ... }` guard -- or any declaration
    // guarded by a plain `#ifdef FEATURE_X` -- silently hid every
    // declaration inside it, since the loops below only ever walked `root`'s
    // direct children.
    let mut top_level = Vec::new();
    collect_effective_top_level(root, &mut top_level);

    for child in &top_level {
        collect_type_name(child, src, &mut symbols);
    }

    for child in &top_level {
        collect_function_name(child, src, &mut symbols);
    }

    // Recover function definitions that tree-sitter's error recovery nested
    // inside another function's body. Standard C has no such thing as a
    // function definition nested inside another function's body (that's only
    // a rare, non-portable GNU extension); in practice this shape almost
    // always means an unrelated syntax error earlier in the *outer* function
    // (e.g. a statement missing its trailing `;`) confused error recovery
    // into swallowing everything up to EOF into that one function's body,
    // dragging every subsequent real top-level function definition down with
    // it. Without this pass, one stray typo deep inside an early function can
    // silently erase every function defined after it in the file. This does
    // not attempt the same recovery for struct/union/enum/typedef names,
    // since genuinely local type definitions nested inside a function body
    // are common and legitimate, unlike nested function definitions.
    collect_nested_function_names(&root, false, src, &mut symbols);

    symbols
}

/// Recursively expand `node`'s children into `out`, treating preprocessor
/// conditional branches (`preproc_ifdef` -- covers both `#ifdef` and
/// `#ifndef`, plus any `preproc_elif`/`preproc_else` alternative branch) and
/// `extern "C" { ... }` linkage-specification blocks as transparent: their
/// contents are still genuinely top-level declarations, just textually
/// wrapped, so they must be visited exactly like any other top-level node.
/// Handles arbitrary nesting (an `extern "C"` block inside an `#ifdef`, an
/// `#ifdef` inside an `extern "C"` block, `#ifdef`s nested inside each
/// other, etc.) since each expansion recurses back into this same function.
fn collect_effective_top_level<'tree>(node: Node<'tree>, out: &mut Vec<Node<'tree>>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "preproc_ifdef" | "preproc_elif" | "preproc_else" => {
                collect_effective_top_level(child, out);
            }
            "linkage_specification" => {
                if let Some(body) = child.child_by_field_name("body") {
                    collect_effective_top_level(body, out);
                }
            }
            _ => out.push(child),
        }
    }
}

/// Recursively look for `function_definition` nodes nested inside another
/// `function_definition`'s subtree (see comment at the call site) and collect
/// their names via the normal `collect_function_name` rules (respecting
/// `static` and de-duplication). `inside_function` tracks whether an
/// ancestor `function_definition` has already been entered.
fn collect_nested_function_names(
    node: &Node,
    inside_function: bool,
    src: &[u8],
    symbols: &mut Vec<String>,
) {
    let is_function_def = node.kind() == "function_definition";
    if is_function_def && inside_function {
        collect_function_name(node, src, symbols);
    }
    let next_inside = inside_function || is_function_def;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_nested_function_names(&child, next_inside, src, symbols);
    }
}

/// Collect a struct/union/enum name from a top-level node. Handles the name
/// appearing standalone (`struct Foo;`), nested inside a `typedef`, nested
/// inside a `declaration` (e.g. `struct Foo { ... } instance;` or a
/// struct-returning prototype `struct Foo make_foo(void);`), or nested inside
/// a `function_definition`'s return type (`struct Foo make_foo(void) { ... }`).
/// No de-duplication (mirrors the reference regex, which does not de-dupe
/// type names).
fn collect_type_name(node: &Node, src: &[u8], symbols: &mut Vec<String>) {
    match node.kind() {
        "struct_specifier" | "union_specifier" | "enum_specifier" => {
            if let Some(name) = get_field_text(node, "name", src) {
                symbols.push(name);
            }
        }
        "type_definition" => {
            if let Some(inner) = node.child_by_field_name("type") {
                collect_type_name(&inner, src, symbols);
            }
            // The typedef's own `declarator` field holds the alias — the
            // actual public name callers use (`Point3D p;`, `Callback cb;`).
            // This is the only place that name appears when the underlying
            // struct/union/enum is anonymous, and it's the *real* public name
            // (as opposed to the internal tag) when the tag and alias differ,
            // e.g. `typedef struct Foo_s { ... } Foo;`.
            //
            // A `type_definition` can carry *multiple* `declarator`-field
            // children — e.g. `typedef struct { ... } A, B;` or `typedef
            // struct { ... } Point, *PointPtr;` — since tree-sitter-c's
            // grammar allows a comma-separated list of declarators here, all
            // sharing the same `declarator` field name. Using
            // `child_by_field_name` (first match only) silently dropped
            // every alias after the first comma; `children_by_field_name`
            // visits all of them.
            let mut cursor = node.walk();
            for declarator in node.children_by_field_name("declarator", &mut cursor) {
                if let Some(alias) = typedef_alias_name(&declarator, src) {
                    symbols.push(alias);
                }
            }
        }
        "declaration" | "function_definition" => {
            if let Some(inner) = node.child_by_field_name("type") {
                collect_type_name(&inner, src, symbols);
            }
        }
        _ => {}
    }
}

/// Given the `declarator` field of a `type_definition` node, drill through
/// any pointer/array/parenthesized/function wrapper layers to find the
/// terminal `type_identifier` — the typedef's actual alias name. Handles
/// plain aliases (`typedef struct { ... } Point3D;` -> `Point3D`) as well as
/// function-pointer typedefs (`typedef int (*Callback)(int, void*);` ->
/// `Callback`, whose declarator is a `function_declarator` wrapping a
/// `parenthesized_declarator` wrapping a `pointer_declarator` wrapping the
/// `type_identifier`).
fn typedef_alias_name(node: &Node, src: &[u8]) -> Option<String> {
    match node.kind() {
        "type_identifier" => Some(node.utf8_text(src).unwrap_or_default().to_string()),
        // `parenthesized_declarator` wraps its inner declarator as a plain
        // (unnamed) child — e.g. the `(*Callback)` in a function-pointer
        // typedef — rather than exposing it under a `declarator` field like
        // its sibling wrapper kinds do.
        "parenthesized_declarator" => {
            let inner = node.named_child(0)?;
            typedef_alias_name(&inner, src)
        }
        "pointer_declarator" | "array_declarator" | "function_declarator" => {
            let inner = node.child_by_field_name("declarator")?;
            typedef_alias_name(&inner, src)
        }
        _ => None,
    }
}

/// Collect a non-static top-level function name from a `function_definition`
/// (has a body) or a `declaration` (a prototype ending in `;`). De-dupes
/// against symbols already collected, mirroring the reference regex.
fn collect_function_name(node: &Node, src: &[u8], symbols: &mut Vec<String>) {
    match node.kind() {
        "function_definition" | "declaration" => {
            if has_static(node, src) {
                return;
            }
            if let Some(name) = function_name_from_declarator(node, src)
                && !symbols.contains(&name)
            {
                symbols.push(name);
            }
        }
        _ => {}
    }
}

/// Check whether a `function_definition`/`declaration` node has a `static`
/// storage class specifier child.
fn has_static(node: &Node, src: &[u8]) -> bool {
    let mut cursor = node.walk();
    node.children(&mut cursor).any(|child| {
        child.kind() == "storage_class_specifier"
            && child.utf8_text(src).unwrap_or_default() == "static"
    })
}

/// Walk a node's `declarator` field, unwrapping `pointer_declarator` layers
/// (for pointer-returning functions like `char* get_name()`), to find the
/// name inside the innermost `function_declarator`.
///
/// Returns `None` (no match) rather than a best-effort guess in two cases
/// that the reference regex also fails to match:
/// - The node has more than one `declarator` field, i.e. a comma-separated
///   multi-declarator statement like `int foo(), bar();`. The regex requires
///   `{`/`;` immediately after the closing paren, so it never matches any
///   name in such a statement; matching only the first declarator here would
///   silently accept text the regex rejects.
/// - The `function_declarator`'s own `declarator` is not a plain `identifier`
///   — e.g. a function-pointer *variable* like `int (*fp)(int, int);`, whose
///   innermost declarator is a `parenthesized_declarator`/`pointer_declarator`
///   wrapping `fp`, not a function name at all. Naively taking that node's
///   source text would yield a garbage, non-identifier symbol like `"(*fp)"`.
fn function_name_from_declarator(node: &Node, src: &[u8]) -> Option<String> {
    let mut cursor = node.walk();
    if node
        .children_by_field_name("declarator", &mut cursor)
        .count()
        != 1
    {
        return None;
    }

    let mut declarator = node.child_by_field_name("declarator")?;
    loop {
        match declarator.kind() {
            "pointer_declarator" => {
                declarator = declarator.child_by_field_name("declarator")?;
            }
            "function_declarator" => {
                let inner = declarator.child_by_field_name("declarator")?;
                if inner.kind() != "identifier" {
                    return None;
                }
                return Some(inner.utf8_text(src).unwrap_or_default().to_string());
            }
            _ => return None,
        }
    }
}

fn get_field_text(node: &Node, field: &str, src: &[u8]) -> Option<String> {
    node.child_by_field_name(field)
        .map(|n| n.utf8_text(src).unwrap_or_default().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ifdef_wrapped_declarations_captured() {
        // Bug found via adversarial testing: declarations guarded by a plain
        // `#ifdef FEATURE_X` (an extremely common real-world header idiom)
        // were silently dropped entirely. tree-sitter-c nests them under a
        // `preproc_ifdef` node's own direct children (no "body" field), and
        // the extraction loops previously only ever walked `root`'s direct
        // children, so `preproc_ifdef` itself was invisible to both
        // `collect_type_name` and `collect_function_name` and everything
        // inside it vanished. Fixed by flattening `preproc_ifdef` (and
        // `linkage_specification`) children into the effective top level
        // before running either pass.
        let src = r#"
#ifdef FEATURE_X
int feature_x_fn(void) {
    return 1;
}
struct FeatureXStruct { int a; };
#endif

int always_present(void) {
    return 0;
}
"#;
        let symbols = extract_exports(src);
        for name in ["feature_x_fn", "FeatureXStruct", "always_present"] {
            assert!(
                symbols.contains(&name.to_string()),
                "missing {name}: {symbols:?}"
            );
        }
    }

    #[test]
    fn test_extern_c_block_declarations_captured() {
        // Bug found via adversarial testing: the routine C/C++ interop idiom
        // `#ifdef __cplusplus / extern "C" { ... } / #endif` (present in
        // most real-world public C headers) silently dropped every
        // declaration inside it -- `extern "C" { ... }` parses as a
        // `linkage_specification` node with its own `body` field, doubly
        // nested inside the `preproc_ifdef` guard, so neither extraction
        // pass ever reached `exported_fn`. Fixed alongside the `#ifdef` bug
        // above; the fix handles arbitrary nesting of the two wrapper kinds.
        let src = r#"
#ifdef __cplusplus
extern "C" {
#endif

void exported_fn(int x);

#ifdef __cplusplus
}
#endif
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"exported_fn".to_string()),
            "expected exported_fn in {symbols:?}"
        );
    }

    #[test]
    fn test_preproc_else_branch_declarations_captured() {
        // Both branches of an `#ifdef ... #else ... #endif` are real,
        // reachable (under different build configurations) top-level
        // declarations and must both be captured, not just the `#ifdef`
        // branch.
        let src = r#"
#ifdef USE_FAST_PATH
int compute(void) { return 1; }
#else
int compute(void) { return 2; }
#endif
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"compute".to_string()),
            "expected compute in {symbols:?}"
        );
    }

    #[test]
    fn test_kr_style_function_definition_captured() {
        // Old K&R-style function definition: parameter names appear bare in
        // the parameter list, with their types declared separately between
        // the parameter list and the body.
        let src = r#"
int add(a, b)
    int a;
    int b;
{
    return a + b;
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"add".to_string()), "got {symbols:?}");
    }

    #[test]
    fn test_inline_without_static_is_captured() {
        // A plain `inline` (no `static`) function still has external linkage
        // in C, so it must be captured -- only `static` excludes a
        // definition, not `inline` on its own.
        let src = "inline int add(int a, int b) { return a + b; }\n";
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["add"]);
    }

    #[test]
    fn test_anonymous_union_member_not_misattributed() {
        // An anonymous union nested as a struct member has no name of its
        // own and must not produce a spurious symbol; only the struct name
        // is captured.
        let src = r#"
struct Msg {
    int type;
    union {
        int i;
        float f;
    };
};
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Msg"]);
    }

    #[test]
    fn test_explicit_extern_prototype_captured() {
        // A redundant-but-common explicit `extern` on a prototype is not
        // `static`, so it must still be captured as exported.
        let src = "extern int explicit_extern_fn(int x);\n";
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["explicit_extern_fn"]);
    }

    #[test]
    fn test_macro_annotated_prototype_still_captured() {
        // A common "export macro" idiom (e.g. a `MODULE_EXPORT`/
        // `__declspec(dllexport)`-style macro prefixing a prototype), left
        // unexpanded since no preprocessor runs. The leading unknown
        // identifier must not prevent the real function name from being
        // captured.
        let src = "MODULE_EXPORT int module_fn(int x);\n";
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"module_fn".to_string()),
            "got {symbols:?}"
        );
    }

    #[test]
    fn test_pointer_array_variable_not_captured_as_function() {
        // `int *arr[10];` is a top-level variable declaration (an array of
        // pointers), not a function -- its declarator is an
        // `array_declarator`, not a `function_declarator`, so it must not be
        // captured.
        let src = "int *arr[10];\n";
        let symbols = extract_exports(src);
        assert!(symbols.is_empty(), "expected no symbols, got {symbols:?}");
    }

    #[test]
    fn test_signal_style_declarator_matches_regex_parity() {
        // The classic convoluted libc `signal()` prototype: a function
        // returning a pointer to a function, itself taking a function
        // pointer parameter. `function_name_from_declarator` only unwraps
        // `pointer_declarator` layers before requiring a `function_declarator`
        // wrapping a plain `identifier`; here the outer `function_declarator`
        // (for `signal` itself) wraps a `parenthesized_declarator` around a
        // `pointer_declarator` around the inner `function_declarator`, so it
        // does not match and nothing is captured. Verified this is not a
        // regression: the reference regex backend also fails to match this
        // line (its declarator-less pattern can't handle it either), so
        // returning empty here preserves parity rather than diverging from
        // it.
        let src = "int (*signal(int sig, void (*func)(int)))(int);\n";
        let symbols = extract_exports(src);
        let regex_symbols = crate::exports::c::extract_exports(src);
        assert!(symbols.is_empty(), "got {symbols:?}");
        assert!(
            regex_symbols.is_empty(),
            "expected regex parity (both empty), regex got {regex_symbols:?}"
        );
    }

    #[test]
    fn test_function_pointer_variable_not_captured_as_garbage() {
        // `int (*fp)(int, int);` declares a function-pointer *variable*, not a
        // function. Its `function_declarator`'s own declarator is a
        // `parenthesized_declarator`/`pointer_declarator` wrapping `fp`, not a
        // plain identifier. Naively taking that node's source text produced a
        // garbage, non-identifier symbol like `"(*fp)"`; it must be excluded,
        // matching the reference regex (which never matches this line either).
        let src = "int (*fp)(int, int);\n";
        let symbols = extract_exports(src);
        assert!(symbols.is_empty(), "expected no symbols, got {symbols:?}");
    }

    #[test]
    fn test_multi_declarator_prototype_not_captured() {
        // `int foo(), bar();` is a single comma-separated multi-declarator
        // declaration. The reference regex requires `{`/`;` immediately after
        // the closing paren, so it never matches a name here; capturing only
        // the first declarator would accept text the regex rejects.
        let src = "int foo(), bar();\n";
        let symbols = extract_exports(src);
        assert!(symbols.is_empty(), "expected no symbols, got {symbols:?}");
    }

    #[test]
    fn test_struct_with_trailing_instance_name_captures_type() {
        // `struct Foo { ... } instance;` is a very common C idiom: the struct
        // specifier is nested inside a `declaration` node (as its `type`
        // field) rather than appearing as a standalone top-level node.
        let src = "struct Foo { int x; } inst;\nenum Color { RED } c;\n";
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Foo".to_string()));
        assert!(symbols.contains(&"Color".to_string()));
    }

    #[test]
    fn test_struct_return_type_captured() {
        // A function returning a named struct by value (prototype or
        // definition) nests the struct_specifier as the `type` field of the
        // `declaration`/`function_definition` node.
        let src = "struct Foo make_foo(void);\n";
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Foo".to_string()));
        assert!(symbols.contains(&"make_foo".to_string()));
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
    fn test_function_prototype_captured() {
        // Non-static top-level prototypes (declarations without a body) count too,
        // matching the reference regex which only requires a trailing `{` or `;`.
        let src = r#"
void declared_only(int x);
static int hidden_proto(int x);
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"declared_only".to_string()));
        assert!(!symbols.contains(&"hidden_proto".to_string()));
    }

    #[test]
    fn test_typedef_struct_name_captured() {
        let src = r#"
typedef struct Point {
    int x;
    int y;
} Point;

typedef struct {
    int a;
} Anonymous;
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Point".to_string()));
        assert!(symbols.contains(&"Anonymous".to_string()));
    }

    #[test]
    fn test_forward_declaration_captured() {
        let src = "struct ForwardOnly;\n";
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"ForwardOnly".to_string()));
    }

    #[test]
    fn test_anonymous_typedef_and_tag_alias_mismatch_captured() {
        // Anonymous struct/enum bodies have no name of their own — the
        // typedef's trailing alias is the *only* place the public name
        // appears. And when the tag and alias differ, the alias (not the
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
        // `typedef int (*Callback)(int, void*);` declares a public type
        // alias whose declarator is a function_declarator wrapping a
        // parenthesized/pointer declarator around the alias identifier, not
        // a struct/union/enum specifier — it must still be captured.
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
        // The member itself (`area`) is not a top-level declaration and must
        // not be captured as a function or type; only the struct name is.
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
    fn test_ignores_text_in_strings_and_comments() {
        // AST-native: text that merely resembles a declaration inside a string
        // or comment must not be captured, unlike a naive regex scan.
        let src = r#"
// struct FakeComment {};
// int fake_comment_fn() {}
const char *msg = "struct FakeString {}; int fake_string_fn() {}";

int real_fn(void) {
    return 0;
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["real_fn"]);
        assert!(!symbols.contains(&"FakeComment".to_string()));
        assert!(!symbols.contains(&"fake_comment_fn".to_string()));
        assert!(!symbols.contains(&"FakeString".to_string()));
        assert!(!symbols.contains(&"fake_string_fn".to_string()));
    }

    #[test]
    fn test_typedef_multi_declarator_all_aliases_captured() {
        // Regression: `typedef struct { ... } A, B;` declares TWO public
        // aliases in one statement (verified empirically: tree-sitter-c's
        // `type_definition` node carries a repeated `declarator` field, one
        // child per comma-separated name — `children_by_field_name` returns
        // both `A` and `B`). The code previously used
        // `child_by_field_name("declarator")`, which only ever returns the
        // *first* match, silently dropping every alias after the first
        // comma. This is a real-world idiom (e.g. libc's
        // `typedef struct __sFILE { ... } FILE;`-style headers commonly
        // declare `Foo, *FooPtr` together) and the AST backend returned a
        // nonempty-but-incomplete result, which never triggers the
        // regex-fallback safety net (that only fires on a fully empty
        // result).
        let src = "typedef struct { int x; } A, B;\n";
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["A", "B"], "got {symbols:?}");
    }

    #[test]
    fn test_typedef_multi_declarator_with_pointer_alias_captured() {
        // Regression: same underlying bug as
        // `test_typedef_multi_declarator_all_aliases_captured`, but the
        // second declarator is a pointer alias (`*PointPtr`) rather than a
        // plain name -- confirms the fix's `children_by_field_name` loop
        // correctly re-drills through `typedef_alias_name` for every
        // declarator, not just plain `type_identifier` ones.
        let src = "typedef struct { int x; } Point, *PointPtr;\n";
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Point", "PointPtr"], "got {symbols:?}");
    }

    #[test]
    fn test_function_pointer_array_typedef_captured() {
        // `typedef int (*Callback[10])(int);` declares an array of function
        // pointers. Verified via real parse tree: the alias `Callback` sits
        // at the bottom of function_declarator -> parenthesized_declarator
        // (unnamed child) -> pointer_declarator -> array_declarator
        // (`declarator` field) -> type_identifier (`declarator` field).
        // `typedef_alias_name` already drills through all of these kinds
        // generically; this locks in that it actually works end-to-end.
        let src = "typedef int (*Callback[10])(int);\n";
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Callback"], "got {symbols:?}");
    }

    #[test]
    fn test_array_typedef_captured() {
        // `typedef char Buffer[256];` — a plain (non-pointer, non-function)
        // array typedef. The alias sits under a single `array_declarator`
        // layer with no intervening pointer/function wrapper.
        let src = "typedef char Buffer[256];\n";
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Buffer"], "got {symbols:?}");
    }

    #[test]
    fn test_multidimensional_array_typedef_captured() {
        // `typedef int Array2D[10][20];` nests two `array_declarator` layers
        // (one per dimension) around the alias identifier; the recursive
        // `array_declarator` arm in `typedef_alias_name` must unwrap both.
        let src = "typedef int Array2D[10][20];\n";
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Array2D"], "got {symbols:?}");
    }

    #[test]
    fn test_leading_attribute_macro_prototype_captured() {
        // A leading `__attribute__((...))` (or similarly-shaped compiler
        // annotation) is a real, separately-parsed `attribute_specifier`
        // node inside the `declaration`, not folded into the `type` field --
        // verified via real parse tree. It must not disturb the `type`
        // and `declarator` field lookups that locate the function name.
        let src = "__attribute__((visibility(\"default\"))) int attr_fn(void);\n";
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["attr_fn"], "got {symbols:?}");
    }

    #[test]
    fn test_trailing_attribute_macro_prototype_captured() {
        // A GNU-style trailing `__attribute__((...))` after the parameter
        // list is nested inside the `function_declarator` itself (as an
        // extra unfielded child alongside the `declarator`/`parameters`
        // fields, verified via real parse tree). It must not prevent the
        // function name from being captured.
        let src = "void trailing_attr_fn(void) __attribute__((noreturn));\n";
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["trailing_attr_fn"], "got {symbols:?}");
    }

    #[test]
    fn test_multi_level_pointer_return_captured() {
        // `int **double_ptr_fn(void);` — a function returning a
        // pointer-to-pointer. `function_name_from_declarator`'s loop must
        // unwrap more than one nested `pointer_declarator` layer before
        // reaching the `function_declarator`.
        let src = "int **double_ptr_fn(void);\n";
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["double_ptr_fn"], "got {symbols:?}");
    }

    #[test]
    fn test_inline_static_reversed_modifier_order_excluded() {
        // Unusual-but-legal modifier ordering: `inline static` (storage
        // class after `inline`) rather than the conventional `static
        // inline`. `has_static` scans *all* direct children for a
        // `storage_class_specifier` regardless of position, so both
        // orderings must be excluded identically -- confirmed empirically
        // via real parse tree, which shows two sibling
        // `storage_class_specifier` nodes in either order.
        let conventional = "static inline int static_inline_fn(void) { return 0; }\n";
        let reversed = "inline static int inline_static_fn(void) { return 0; }\n";
        assert!(extract_exports(conventional).is_empty());
        assert!(extract_exports(reversed).is_empty());
    }

    #[test]
    fn test_mixed_multi_declarator_prototype_not_captured() {
        // `int foo(void), *bar(void);` -- a comma-separated multi-declarator
        // statement where the declarators have different shapes (a plain
        // function and a pointer-returning function). Like the simpler
        // `int foo(), bar();` case, the `declaration` node carries two
        // `declarator`-field children, so the declarator-count guard in
        // `function_name_from_declarator` must reject it entirely rather
        // than capturing just the first one -- verified this also matches
        // the reference regex backend (which requires `{`/`;` immediately
        // after the closing paren and so never matches either name here).
        let src = "int foo(void), *bar(void);\n";
        let symbols = extract_exports(src);
        let regex_symbols = crate::exports::c::extract_exports(src);
        assert!(symbols.is_empty(), "got {symbols:?}");
        assert!(
            regex_symbols.is_empty(),
            "expected regex parity (both empty), regex got {regex_symbols:?}"
        );
    }
}
