use tree_sitter::{Node, Parser, Tree};

/// Parse Erlang source into a tree-sitter AST.
fn parse_erlang(content: &str) -> Option<Tree> {
    let mut parser = Parser::new();
    let language = tree_sitter_erlang::LANGUAGE.into();
    parser.set_language(&language).ok()?;
    parser.parse(content, None)
}

/// Extract exported symbols from Erlang source using tree-sitter AST.
///
/// Erlang exports are primarily attribute-driven: function names that appear
/// inside one or more `-export([name/arity, ...]).` attributes count,
/// regardless of what's actually defined in the module body. Multiple
/// `-export` attributes accumulate, and quoted atom names (e.g. `'Foo'`)
/// have their surrounding quotes stripped.
///
/// The other route to a public function is the `-compile(export_all).`
/// compiler directive (or `export_all` as one of several options inside
/// `-compile([...]).`), which exports every top-level function defined in
/// the module regardless of any `-export` list. `-callback` specs are never
/// exports themselves -- they declare a contract for implementing modules --
/// so they're deliberately not walked here.
pub fn extract_exports(content: &str) -> Vec<String> {
    let tree = match parse_erlang(content) {
        Some(t) => t,
        None => return Vec::new(),
    };

    let root = tree.root_node();
    let src = content.as_bytes();
    let mut symbols = Vec::new();
    let mut export_all = false;

    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        match child.kind() {
            "export_attribute" => collect_export_funs(&child, src, &mut symbols),
            "compile_options_attribute" if compile_options_has_export_all(&child, src) => {
                export_all = true;
            }
            _ => {}
        }
    }

    if export_all {
        let mut cursor = root.walk();
        for child in root.children(&mut cursor) {
            if child.kind() == "fun_decl" {
                collect_fun_decl_name(&child, src, &mut symbols);
            }
        }
    }

    symbols
}

/// Collect exported function names from an `export_attribute` node's `fa`
/// (function/arity) children.
///
/// A `?MACRO` placeholder used as an export-list entry (e.g.
/// `-export([add/2, ?SOME_MACRO, sub/2]).`) cannot be statically resolved to
/// a real function name, and tree-sitter-erlang represents it as a
/// `macro_call_expr` in the `fun` field rather than a plain `atom` -- it must
/// never be pushed as a symbol name verbatim (that would report a literal
/// `"?SOME_MACRO"` as an exported function, which is never a real, callable
/// name). Worse, the presence of that macro token can corrupt the parse of
/// the *next* list entry into an `ERROR` node attached to the same `fa`
/// (observed: `sub/2` following `?SOME_MACRO` gets folded into
/// `fa fun: (macro_call_expr) (ERROR (atom))`), which would otherwise silently
/// drop a legitimate, real export. To recover it without over-trusting
/// `ERROR` node internals, any `atom` node found nested inside such an
/// `ERROR` child is treated as an additional recovered name -- mirroring the
/// regex reference, which token-matches `name/arity` shapes and is
/// unaffected by this parse corruption.
fn collect_export_funs(export_attr: &Node, src: &[u8], symbols: &mut Vec<String>) {
    let mut cursor = export_attr.walk();
    for fa in export_attr.children_by_field_name("funs", &mut cursor) {
        match fa.child_by_field_name("fun") {
            Some(fun) if fun.kind() == "atom" => {
                let text = fun.utf8_text(src).unwrap_or_default();
                let name = text.trim_matches('\'').to_string();
                if !symbols.contains(&name) {
                    symbols.push(name);
                }
            }
            _ => {
                // Not a plain atom (e.g. a `?MACRO` placeholder) -- don't
                // emit its raw text as a name, but recover any real atom
                // that got swallowed into a sibling ERROR node.
                let mut inner = fa.walk();
                for descendant in fa.children(&mut inner) {
                    if descendant.kind() == "ERROR" {
                        collect_error_atoms(&descendant, src, symbols);
                    }
                }
            }
        }
    }
}

/// Recover plain `atom` names nested inside an `ERROR` node (see
/// `collect_export_funs` for why this happens).
fn collect_error_atoms(node: &Node, src: &[u8], symbols: &mut Vec<String>) {
    if node.kind() == "atom" {
        let text = node.utf8_text(src).unwrap_or_default();
        let name = text.trim_matches('\'').to_string();
        if !name.is_empty() && !symbols.contains(&name) {
            symbols.push(name);
        }
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_error_atoms(&child, src, symbols);
    }
}

/// Check whether a `compile_options_attribute` node's `options` field
/// contains `export_all`, either as the option directly
/// (`-compile(export_all).`) or as one entry in an option list
/// (`-compile([export_all, debug_info]).`).
fn compile_options_has_export_all(compile_attr: &Node, src: &[u8]) -> bool {
    let Some(options) = compile_attr.child_by_field_name("options") else {
        return false;
    };
    is_export_all_atom(&options, src) || {
        let mut cursor = options.walk();
        options
            .children_by_field_name("exprs", &mut cursor)
            .any(|expr| is_export_all_atom(&expr, src))
    }
}

/// Whether a node is a bare `atom` node whose text is `export_all`.
fn is_export_all_atom(node: &Node, src: &[u8]) -> bool {
    node.kind() == "atom" && node.utf8_text(src).unwrap_or_default() == "export_all"
}

/// Collect the function name from a top-level `fun_decl` node's
/// `function_clause` (skipping macro-call clauses, which have no `name`
/// field). Used only when `-compile(export_all)` is in effect, since it
/// exports every function defined in the module.
fn collect_fun_decl_name(fun_decl: &Node, src: &[u8], symbols: &mut Vec<String>) {
    let Some(clause) = fun_decl.child_by_field_name("clause") else {
        return;
    };
    let Some(name_node) = clause.child_by_field_name("name") else {
        return;
    };
    let text = name_node.utf8_text(src).unwrap_or_default();
    let name = text.trim_matches('\'').to_string();
    if !name.is_empty() && !symbols.contains(&name) {
        symbols.push(name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_erlang_exports() {
        let src = r#"
-module(math_utils).
-export([add/2,
         sub/2]).
-export([mul/2]).
-export(['DummyClass'/0]).

add(A, B) -> A + B.
sub(A, B) -> A - B.
mul(A, B) -> A * B.
'DummyClass'() -> ok.
helper() -> ok.
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"add".to_string()));
        assert!(symbols.contains(&"sub".to_string()));
        assert!(symbols.contains(&"mul".to_string()));
        assert!(symbols.contains(&"DummyClass".to_string()));
        assert!(!symbols.contains(&"helper".to_string()));
    }

    #[test]
    fn test_erlang_ignores_export_text_in_comment() {
        // AST-native: text that looks like an -export attribute inside a
        // comment must not be captured (regex with comment-stripping also
        // handles this, but AST does it for free by never treating comment
        // bytes as attribute nodes).
        let src = r#"
-module(m).
%% -export([fake/0]).
-export([real/0]).

real() -> ok.
fake() -> ok.
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"real".to_string()));
        assert!(!symbols.contains(&"fake".to_string()));
    }

    #[test]
    fn test_erlang_no_export_attribute() {
        // A plain top-level function with no -export at all is never
        // captured, since Erlang export semantics are attribute-driven.
        let src = r#"
-module(m).

add(A, B) -> A + B.
"#;
        let symbols = extract_exports(src);
        assert!(symbols.is_empty());
    }

    #[test]
    fn test_erlang_compile_export_all() {
        // -compile(export_all). is a real, common Erlang directive (eunit
        // test modules, escripts, quick tooling, legacy production code)
        // that exports every top-level function with no -export list at
        // all.
        let src = r#"
-module(scratch_calc).
-compile(export_all).

add(A, B) -> A + B.
sub(A, B) -> A - B.
mul(A, B) -> A * B.
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"add".to_string()));
        assert!(symbols.contains(&"sub".to_string()));
        assert!(symbols.contains(&"mul".to_string()));
    }

    #[test]
    fn test_erlang_compile_export_all_in_option_list_and_non_export_all_compile_stays_empty() {
        // export_all can appear as one entry among several inside
        // -compile([...]), and a -compile attribute lacking export_all
        // (e.g. only parse_transform) must not export anything.
        let src_list_form = r#"
-module(scratch_utils).
-compile([export_all, debug_info]).

double(X) -> X * 2.
triple(X) -> X * 3.
"#;
        let symbols = extract_exports(src_list_form);
        assert!(symbols.contains(&"double".to_string()));
        assert!(symbols.contains(&"triple".to_string()));

        let src_no_export_all = r#"
-module(m).
-compile([{parse_transform, lager_transform}]).

add(A, B) -> A + B.
"#;
        let symbols = extract_exports(src_no_export_all);
        assert!(symbols.is_empty());
    }

    #[test]
    fn test_erlang_export_list_and_callback_in_behaviour_module() {
        // A realistic multi-export, multi-behaviour module: -callback specs
        // declare a contract for implementing modules but are not
        // themselves exports, and must not be picked up alongside the real
        // -export lists.
        let src = r#"
-module(validator).
-behaviour(gen_server).

-export([start_link/0, validate/1, init/1]).
-export([handle_call/3]).

-callback validate(term()) -> boolean().
-callback describe() -> binary().

start_link() -> ok.
validate(X) -> true.
init(X) -> {ok, X}.
handle_call(_, _, State) -> {reply, ok, State}.
describe_internal() -> ok.
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"start_link".to_string()));
        assert!(symbols.contains(&"validate".to_string()));
        assert!(symbols.contains(&"init".to_string()));
        assert!(symbols.contains(&"handle_call".to_string()));
        assert!(!symbols.contains(&"describe".to_string()));
        assert!(!symbols.contains(&"describe_internal".to_string()));
    }

    #[test]
    fn test_export_type_attribute_is_not_a_function_export() {
        // `-export_type([...])` exports a *type*, a distinct grammar node
        // (`export_type_attribute`) from `export_attribute`; it must never be
        // conflated with a function export.
        let src = r#"
-module(m).
-export_type([my_type/0]).
-type my_type() :: integer().
"#;
        let symbols = extract_exports(src);
        assert!(symbols.is_empty(), "got {symbols:?}");
    }

    #[test]
    fn test_export_list_with_comment_between_entries() {
        let src = "-module(m).\n-export([\n    add/2,\n    %% sub is deprecated\n    sub/2\n]).\nadd(A,B)->A+B.\nsub(A,B)->A-B.\n";
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"add".to_string()));
        assert!(symbols.contains(&"sub".to_string()));
    }

    #[test]
    fn test_multi_clause_function_name_deduped_under_export_all() {
        // Semicolon-separated clauses of the same function share one
        // top-level `fun_decl`, so this isn't really a dedup test of the
        // *walk* so much as confirmation that a multi-clause function under
        // `-compile(export_all)` is reported exactly once.
        let src = "-module(m).\n-compile(export_all).\nfoo(0) -> zero;\nfoo(N) -> N.\n";
        let symbols = extract_exports(src);
        assert_eq!(symbols.iter().filter(|s| *s == "foo").count(), 1);
    }

    #[test]
    fn test_record_and_define_are_not_exports() {
        let src = "-module(m).\n-compile(export_all).\n-record(foo, {a,b}).\n-define(BAR, 1).\nreal() -> ok.\n";
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["real".to_string()]);
    }

    #[test]
    fn test_ifdef_wrapped_export_attribute_still_captured() {
        // tree-sitter-erlang doesn't implement conditional compilation --
        // `-ifdef`/`-endif` are flat sibling directives, not wrapper nodes --
        // so an `-export` "inside" one is still a direct child of the root
        // and must still be found by the top-level scan.
        let src = "-module(m).\n-ifdef(TEST).\n-export([test_helper/0]).\n-endif.\ntest_helper() -> ok.\n";
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"test_helper".to_string()),
            "got {symbols:?}"
        );
    }

    #[test]
    fn test_macro_placeholder_in_export_list_does_not_break_other_entries() {
        // Regression test: a `?MACRO` placeholder used as an export-list
        // entry (`-export([add/2, ?SOME_MACRO, sub/2]).`) is represented by
        // tree-sitter-erlang as a `macro_call_expr` in the `fun` field of its
        // `fa` node, not a plain `atom`. Before the fix, the code blindly
        // used `fun.utf8_text()` regardless of node kind, so it (a) reported
        // the literal garbage name `"?SOME_MACRO"` as if it were a real
        // export, and (b) lost the following real entry `sub/2` entirely --
        // the malformed macro token corrupts the grammar's parse of the next
        // list item into an `ERROR` node attached to the same `fa`, and nothing
        // recovered it. Both are now fixed: non-atom `fun` fields are skipped
        // (no more garbage name), and atoms found inside a sibling `ERROR`
        // node are recovered as real names.
        let src = "-module(m).\n-export([add/2, ?SOME_MACRO, sub/2]).\nadd(A,B) -> A+B.\nsub(A,B) -> A-B.\n";
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"add".to_string()), "got {symbols:?}");
        assert!(symbols.contains(&"sub".to_string()), "got {symbols:?}");
        assert!(
            !symbols.iter().any(|s| s.contains("SOME_MACRO")),
            "must not leak the raw macro placeholder as a name: {symbols:?}"
        );
    }

    #[test]
    fn test_export_reports_name_regardless_of_defined_arity() {
        // The `-export` list is the sole authority on what's public --
        // there's no cross-check against the arity of an actual `fun_decl`
        // in the module body, matching the regex reference's behavior.
        let src = "-module(m).\n-export([foo/1]).\nfoo(A, B) -> A + B.\n";
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["foo".to_string()]);
    }
}
