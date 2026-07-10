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
            "compile_options_attribute" => {
                if compile_options_has_export_all(&child, src) {
                    export_all = true;
                }
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
fn collect_export_funs(export_attr: &Node, src: &[u8], symbols: &mut Vec<String>) {
    let mut cursor = export_attr.walk();
    for fa in export_attr.children_by_field_name("funs", &mut cursor) {
        if let Some(fun) = fa.child_by_field_name("fun") {
            let text = fun.utf8_text(src).unwrap_or_default();
            let name = text.trim_matches('\'').to_string();
            if !symbols.contains(&name) {
                symbols.push(name);
            }
        }
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
}
