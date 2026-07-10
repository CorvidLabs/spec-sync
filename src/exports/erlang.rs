use regex::Regex;
use std::sync::LazyLock;

static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"%.*$").unwrap());

/// Erlang export attribute: -export([f1/1, f2/2]).
static ERL_EXPORT_ATTR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)-export\s*\(\s*\[\s*([^]]*)\s*\]\s*\)").unwrap());

/// Function name within export list: name/arity
static ERL_FUN_ARITY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"'?\b(\w+)'?/\d+").unwrap());

/// `-compile(export_all).` or `-compile([export_all, ...]).`: a compiler
/// directive that exports every top-level function in the module, regardless
/// of any (or absent) `-export` list.
static ERL_COMPILE_EXPORT_ALL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)-compile\s*\(\s*(?:\[[^]]*\bexport_all\b[^]]*\]|export_all)\s*\)").unwrap()
});

/// Top-level function clause head: an identifier (or quoted atom) starting
/// at column 0 immediately followed by `(`. Attributes, specs, records,
/// macros, etc. all start with `-` or `?` and are never matched. Only used
/// to enumerate every defined function when `-compile(export_all)` is
/// present, since export_all makes every top-level function public.
static ERL_FUN_DEF: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^(?:'([^']+)'|([A-Za-z_][A-Za-z0-9_@]*))\s*\(").unwrap());

/// Extract public symbols from Erlang source code.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = COMMENT_SINGLE.replace_all(content, "");

    let mut symbols = Vec::new();

    for caps in ERL_EXPORT_ATTR.captures_iter(&stripped) {
        if let Some(list_match) = caps.get(1) {
            let list_str = list_match.as_str();
            for f_caps in ERL_FUN_ARITY.captures_iter(list_str) {
                if let Some(name) = f_caps.get(1) {
                    let n = name.as_str().to_string();
                    if !symbols.contains(&n) {
                        symbols.push(n);
                    }
                }
            }
        }
    }

    if ERL_COMPILE_EXPORT_ALL.is_match(&stripped) {
        for caps in ERL_FUN_DEF.captures_iter(&stripped) {
            let name = caps
                .get(1)
                .or_else(|| caps.get(2))
                .map(|m| m.as_str().to_string());
            if let Some(n) = name
                && !symbols.contains(&n)
            {
                symbols.push(n);
            }
        }
    }

    symbols
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
    fn test_erlang_compile_export_all() {
        // -compile(export_all). exports every top-level function in the
        // module with no -export list at all -- common in eunit test
        // modules, escripts, and legacy code.
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
    fn test_erlang_compile_export_all_list_form() {
        // export_all can also appear as one option among several inside the
        // -compile([...]) option list form.
        let src = r#"
-module(scratch_utils).
-compile([export_all, debug_info]).

double(X) -> X * 2.
triple(X) -> X * 3.
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"double".to_string()));
        assert!(symbols.contains(&"triple".to_string()));
    }

    #[test]
    fn test_erlang_no_export_all_other_compile_options_stay_unexported() {
        // A -compile attribute without export_all (e.g. only
        // parse_transform) must not cause every function to be exported.
        let src = r#"
-module(m).
-compile([{parse_transform, lager_transform}]).

add(A, B) -> A + B.
"#;
        let symbols = extract_exports(src);
        assert!(symbols.is_empty());
    }

    #[test]
    fn test_erlang_export_list_and_callback_in_behaviour_module() {
        // A realistic multi-export, multi-behaviour module: -callback specs
        // declare a contract but are not themselves exports, and must not
        // be picked up alongside the real -export lists.
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
