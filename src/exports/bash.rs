use regex::Regex;
use std::sync::LazyLock;

/// Single-line `#` comments. Bash has no block-comment syntax.
static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)#.*$").unwrap());

/// Heredoc start marker: `<<WORD`, `<<-WORD`, `<<'WORD'`, or `<<"WORD"`.
/// Used to blank out heredoc bodies before scanning for declarations, since
/// heredoc payloads are string literals that commonly contain text which
/// *looks* like function declarations or `export -f` statements but isn't
/// real code in this script (e.g. a script that generates another script,
/// or a `cat <<EOF` usage/help block).
static HEREDOC_START: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<<(-)?[^\S\n]*(?:'([A-Za-z_]\w*)'|"([A-Za-z_]\w*)"|([A-Za-z_]\w*))"#).unwrap()
});

/// `export -f name` — marks an already-defined function for export to
/// subshells. This is the strongest, most deliberate "public API" signal a
/// bash script author can give, so its presence anywhere in the file takes
/// precedence over every other heuristic (mirrors Python's `__all__`).
/// Supports multiple space-separated names on one `export -f` line
/// (`export -f foo bar baz`), which bash allows. Capture stops at `;`, `&`,
/// or `|` so a chained command on the same line (`export -f foo; echo hi`)
/// doesn't leak unrelated words as bogus exported names.
static EXPORT_F: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[^\S\n]*export[^\S\n]+-f[^\S\n]+([^;&|\n]+)").unwrap());

/// A single exported name inside an `export -f` argument list.
///
/// Bash function names may contain hyphens (common in subcommand-style
/// scripts, e.g. `pre-commit`, `git-diff-index`), so the tail of the name
/// allows `-` alongside `\w`. Without this, `export -f foo-bar baz` would be
/// mis-split into three bogus names (`foo`, `bar`, `baz`) instead of the
/// two real ones (`foo-bar`, `baz`).
static EXPORT_F_NAME: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[A-Za-z_][\w-]*").unwrap());

/// Explicit `function name { ... }` or `function name() { ... }` form.
///
/// See `EXPORT_F_NAME` above for why the name allows hyphens.
static FUNCTION_KEYWORD_DECL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^[^\S\n]*function[^\S\n]+([A-Za-z_][\w-]*)[^\S\n]*(?:\(\))?").unwrap()
});

/// POSIX form: `name() { ... }` with no `function` keyword.
///
/// See `EXPORT_F_NAME` above for why the name allows hyphens.
static POSIX_FUNCTION_DECL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[^\S\n]*([A-Za-z_][\w-]*)[^\S\n]*\(\)").unwrap());

/// Blank out heredoc bodies (and their terminator line), leaving the
/// heredoc-start line itself intact so any real code preceding the `<<` is
/// still scanned. This prevents heredoc payload text — which is a string
/// literal, not executable code — from being misread as function
/// declarations or `export -f` statements.
///
/// Handles the `<<-WORD` form (terminator may be indented with tabs) as
/// well as quoted (`<<'WORD'`, `<<"WORD"`) and bare (`<<WORD`) delimiters.
/// Line count is preserved (blanked lines stay as empty lines) so this can
/// be composed with the other line-anchored regexes unchanged.
fn strip_heredocs(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut out = String::with_capacity(content.len());
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        out.push_str(line);
        out.push('\n');
        i += 1;
        if let Some(caps) = HEREDOC_START.captures(line) {
            let dash = caps.get(1).is_some();
            let delim = caps
                .get(2)
                .or_else(|| caps.get(3))
                .or_else(|| caps.get(4))
                .map(|m| m.as_str())
                .unwrap_or("");
            while i < lines.len() {
                let candidate = lines[i];
                let trimmed = if dash {
                    candidate.trim_start_matches('\t')
                } else {
                    candidate
                };
                i += 1;
                if trimmed.trim_end_matches('\r') == delim {
                    break;
                }
                out.push('\n');
            }
        }
    }
    out
}

/// Extract exported symbols from a bash/sh script.
///
/// Bash has no native visibility keyword. The strongest signal an author can
/// give is an explicit `export -f name` statement, which marks a function for
/// export to subshells — this is treated the same way Python's `__all__` is:
/// if any `export -f` statements exist anywhere in the file, the union of all
/// named functions is the authoritative export list.
///
/// If no `export -f` statements exist, fall back to every function
/// declaration in the file — both `function name { ... }` / `function
/// name() { ... }` and the bare POSIX `name() { ... }` form — excluding
/// names starting with `_`, the common real-world "internal helper"
/// convention in bash scripts.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = COMMENT_SINGLE.replace_all(content, "");
    let stripped = strip_heredocs(&stripped);

    // `export -f` is the authoritative export list when present.
    let mut exported: Vec<String> = Vec::new();
    for caps in EXPORT_F.captures_iter(&stripped) {
        if let Some(names) = caps.get(1) {
            for name_cap in EXPORT_F_NAME.captures_iter(names.as_str()) {
                let n = name_cap.get(0).unwrap().as_str().to_string();
                if !exported.contains(&n) {
                    exported.push(n);
                }
            }
        }
    }
    if !exported.is_empty() {
        return exported;
    }

    // Fallback: all function declarations (both forms), excluding those
    // whose name starts with `_`, collected in source order and deduped
    // (a function can legitimately be declared with both forms across a
    // script's lifetime, or redeclared).
    let mut hits: Vec<(usize, String)> = Vec::new();

    for caps in FUNCTION_KEYWORD_DECL.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str();
            if !n.starts_with('_') {
                hits.push((name.start(), n.to_string()));
            }
        }
    }

    for caps in POSIX_FUNCTION_DECL.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str();
            if !n.starts_with('_') {
                hits.push((name.start(), n.to_string()));
            }
        }
    }

    hits.sort_by_key(|(pos, _)| *pos);

    let mut symbols: Vec<String> = Vec::new();
    for (_, n) in hits {
        if !symbols.contains(&n) {
            symbols.push(n);
        }
    }
    symbols
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bash_export_f_is_authoritative() {
        let src = r#"
#!/usr/bin/env bash

greet() {
    echo "hi $1"
}

_log() {
    echo "[log] $1" >&2
}

helper() {
    echo "unused elsewhere"
}

export -f greet
"#;
        let symbols = extract_exports(src);
        // Only what's explicitly exported, even though `helper` and `_log`
        // are also declared in the file.
        assert_eq!(symbols, vec!["greet"]);
    }

    #[test]
    fn test_bash_export_f_multiple_names_one_line() {
        let src = r#"
build() {
    echo building
}

test_run() {
    echo testing
}

deploy() {
    echo deploying
}

export -f build test_run
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["build", "test_run"]);
        assert!(!symbols.contains(&"deploy".to_string()));
    }

    #[test]
    fn test_bash_export_f_union_across_multiple_statements() {
        let src = r#"
alpha() { echo a; }
beta() { echo b; }
gamma() { echo g; }

export -f alpha
export -f beta
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"alpha".to_string()));
        assert!(symbols.contains(&"beta".to_string()));
        assert!(!symbols.contains(&"gamma".to_string()));
    }

    #[test]
    fn test_bash_no_export_f_falls_back_to_all_functions() {
        let src = r#"
function setup() {
    echo "setting up"
}

teardown() {
    echo "tearing down"
}

_internal_helper() {
    echo "should not be exported"
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["setup", "teardown"]);
        assert!(!symbols.contains(&"_internal_helper".to_string()));
    }

    #[test]
    fn test_bash_function_keyword_form_without_parens() {
        let src = r#"
function process_file {
    echo "processing $1"
}

function _cleanup {
    rm -f /tmp/scratch
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["process_file"]);
    }

    #[test]
    fn test_bash_comments_stripped_before_matching() {
        let src = r#"
# export -f fake_export
# a function that isn't real: fake_func() { echo no; }

real_func() {
    echo "real"  # not a function decl: not_a_func()
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["real_func"]);
        assert!(!symbols.contains(&"fake_export".to_string()));
        assert!(!symbols.contains(&"fake_func".to_string()));
        assert!(!symbols.contains(&"not_a_func".to_string()));
    }

    #[test]
    fn test_bash_mixed_declaration_styles_deduped() {
        // Realistic pattern seen in shared shell libraries: a mix of POSIX
        // and `function`-keyword styles in the same file, with local
        // variables inside functions that must not be picked up as
        // top-level declarations.
        let src = r#"
function log_info() {
    local msg="$1"
    echo "[INFO] $msg"
}

log_error() {
    local msg="$1"
    echo "[ERROR] $msg" >&2
}

retry() {
    local attempts=0
    local max=3
    while [ "$attempts" -lt "$max" ]; do
        attempts=$((attempts + 1))
    done
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["log_info", "log_error", "retry"]);
        assert!(!symbols.contains(&"msg".to_string()));
        assert!(!symbols.contains(&"attempts".to_string()));
        assert!(!symbols.contains(&"max".to_string()));
    }

    #[test]
    fn test_bash_underscore_prefixed_excluded_in_fallback() {
        let src = r#"
_validate_input() {
    [ -n "$1" ]
}

public_entrypoint() {
    _validate_input "$1" || exit 1
    echo "ok"
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["public_entrypoint"]);
    }

    #[test]
    fn test_bash_indented_functions_in_conditional_blocks() {
        // Functions are sometimes declared inside `if`/case blocks for
        // platform-specific overrides; indentation shouldn't matter since
        // bash has no true block scoping for function declarations.
        let src = r#"
if [ "$(uname)" = "Darwin" ]; then
    open_browser() {
        open "$1"
    }
else
    open_browser() {
        xdg-open "$1"
    }
fi
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["open_browser"]);
    }

    #[test]
    fn test_bash_heredoc_function_text_not_exported() {
        // A script that generates another script via heredoc. The heredoc
        // body is a string literal containing text that *looks* like a
        // function declaration but is just payload data written to another
        // file, not a real function declared in this script.
        let src = r#"
deploy() {
    cat > /tmp/generated.sh <<'EOF'
#!/usr/bin/env bash
not_a_real_function() {
    echo "this only exists inside the generated file"
}
EOF
    echo "wrote script"
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["deploy"]);
        assert!(!symbols.contains(&"not_a_real_function".to_string()));
    }

    #[test]
    fn test_bash_export_f_inside_heredoc_does_not_hijack_authoritative_list() {
        // Usage/help text sometimes documents `export -f` inside a heredoc
        // block. Since it's string content, not an actual export
        // statement, it must not become the authoritative export list —
        // the real fallback (all non-`_`-prefixed functions) should apply.
        let src = r#"
real_func() {
    echo "real"
}

show_help() {
    cat <<EOF
Usage:
  export -f real_func
  Then call real_func from a subshell.
EOF
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["real_func", "show_help"]);
    }

    #[test]
    fn test_bash_multiple_export_f_semicolon_same_line_no_bogus_names() {
        // `export -f foo; export -f bar` on one line is valid bash. The
        // capture for the first statement must stop at `;` rather than
        // swallowing the rest of the line (which would leak the literal
        // words `export` and `f` from the second statement as bogus
        // exported names).
        let src = r#"
build_all() { echo build; }
test_all() { echo test; }

export -f build_all; export -f test_all
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"build_all".to_string()));
        assert!(!symbols.contains(&"export".to_string()));
        assert!(!symbols.contains(&"f".to_string()));
    }

    #[test]
    fn test_bash_export_f_with_no_functions_defined_yields_empty() {
        // export -f referencing a name that was never actually declared as a
        // function anywhere in the file is still the authoritative signal
        // per spec (we don't cross-validate against declarations) but with
        // an empty export list overall we should fall back correctly.
        let src = r#"
NAME=world
echo "hello $NAME"
"#;
        let symbols = extract_exports(src);
        assert!(symbols.is_empty());
    }

    #[test]
    fn test_bash_hyphenated_posix_function_name_included() {
        // Regression test for a bug where POSIX_FUNCTION_DECL's name class
        // was `[A-Za-z_]\w*` (no hyphen). Since `\w` doesn't include `-`,
        // the regex needed whitespace/`(` immediately after "foo" but hit
        // `-` instead, so `foo-bar() { ... }` matched nothing at all and
        // was silently dropped from the output. Hyphenated names are a
        // common bash convention for subcommand-style scripts (e.g.
        // `git-diff-index`, `pre-commit`).
        let src = r#"
foo-bar() {
    echo hi
}

plain() {
    echo plain
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["foo-bar", "plain"]);
    }

    #[test]
    fn test_bash_hyphenated_function_keyword_form() {
        // Same hyphen bug as above, but for the `function name { ... }`
        // form via FUNCTION_KEYWORD_DECL's name class.
        let src = r#"
function pre-commit {
    echo "checking"
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["pre-commit"]);
    }

    #[test]
    fn test_bash_export_f_hyphenated_name_not_mis_split() {
        // Regression test for a bug where EXPORT_F_NAME's character class
        // was `[A-Za-z_]\w*` (no hyphen), so it stopped matching at the
        // hyphen and resumed matching on the remainder. This caused
        // `export -f foo-bar baz` to be split into THREE bogus names
        // (`foo`, `bar`, `baz`) instead of the two real ones (`foo-bar`,
        // `baz`).
        let src = r#"
foo-bar() { echo a; }
baz() { echo b; }

export -f foo-bar baz
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["foo-bar", "baz"]);
        assert!(!symbols.contains(&"foo".to_string()));
        assert!(!symbols.contains(&"bar".to_string()));
    }

    #[test]
    fn test_bash_allman_brace_style_functions() {
        // Allman style (opening brace on its own line) is valid bash for
        // both the POSIX and `function`-keyword forms. The trailing
        // `[^\S\n]*(?:\(\))?` / `\(\)` in the declaration regexes must not
        // require the brace to be on the same line, since the regexes only
        // match through the name (and optional parens), not the brace.
        let src = r#"
posix_style()
{
    echo "posix"
}

function keyword_style()
{
    echo "keyword with parens"
}

function keyword_no_parens
{
    echo "keyword without parens"
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(
            symbols,
            vec!["posix_style", "keyword_style", "keyword_no_parens"]
        );
    }

    #[test]
    fn test_bash_export_f_references_undeclared_function_is_still_authoritative() {
        // By design, `export -f` is treated as authoritative with no
        // cross-validation against actual function declarations in the
        // file. A name exported via `export -f` that was never declared
        // anywhere (e.g. defined in a sourced file, or simply a stale
        // export) must still surface in the output verbatim.
        let src = r#"
declared_func() {
    echo "I exist"
}

export -f declared_func never_declared_anywhere
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["declared_func", "never_declared_anywhere"]);
    }

    #[test]
    fn test_bash_heredoc_dash_quoted_delimiter_body_ignored() {
        // `<<-'EOF'` combines the dash form (terminator may be indented
        // with tabs, common for indenting heredocs inside functions) with a
        // quoted delimiter (disables parameter/command expansion in the
        // body). Both properties together must still be recognized by
        // HEREDOC_START and the body correctly blanked out, so payload text
        // that looks like a function declaration or `export -f` isn't
        // picked up as real code.
        let src = "deploy() {\n\tcat > /tmp/out.sh <<-'EOF'\n\tfake_func() { echo no; }\n\texport -f fake_func\n\tEOF\n\techo done\n}\n";
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["deploy"]);
        assert!(!symbols.contains(&"fake_func".to_string()));
    }

    #[test]
    fn test_bash_variable_only_export_forms_not_surfaced() {
        // This backend is function-name-only by design: `export FOO=bar`,
        // `declare -x FOO`, and `readonly FOO=bar` export/declare shell
        // *variables*, not functions, and must never appear in the output
        // even though they share the `export`/`declare`/`readonly` keyword
        // vocabulary with the function-oriented forms this backend does
        // recognize.
        let src = r#"
export FOO=bar
declare -x BAZ=qux
readonly QUUX=corge

real_func() {
    echo "real"
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["real_func"]);
        assert!(!symbols.contains(&"FOO".to_string()));
        assert!(!symbols.contains(&"BAZ".to_string()));
        assert!(!symbols.contains(&"QUUX".to_string()));
    }

    #[test]
    fn test_bash_hyphenated_underscore_prefixed_excluded_in_fallback() {
        // Combines the hyphen fix with the existing `_`-prefix exclusion
        // rule: a hyphenated internal helper must still be excluded from
        // the fallback (no `export -f` present) list.
        let src = r#"
_internal-helper() {
    echo "should not be exported"
}

public-entrypoint() {
    _internal-helper
    echo "ok"
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["public-entrypoint"]);
    }
}
