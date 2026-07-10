use regex::Regex;
use std::sync::LazyLock;

/// Lisp/Scheme definitions: (defun name ...), (defmacro name ...), (defvar name ...),
/// (defparameter name ...), (defstruct name ...). `defstruct` only captures the struct
/// type name itself (a best-effort regex can't synthesize the implicit constructor/
/// predicate/accessor functions the way the AST walker does) and only matches the bare
/// `(defstruct name ...)` form, not the `(defstruct (name :option ...) ...)` form.
static LISP_DEF: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)\(\s*(?:defun|defmacro|defvar|defparameter|defstruct)\s+([\w\-*!?<>]+)")
        .unwrap()
});

/// Extract public symbols from Lisp source code.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = strip_comments_and_strings(content);

    let mut symbols = Vec::new();

    for caps in LISP_DEF.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str().to_string();
            if !symbols.contains(&n) {
                symbols.push(n);
            }
        }
    }

    symbols
}

/// Blank `;` line comments, `"…"` string literals, and (properly nested)
/// `#| … |#` block comments in a single left-to-right pass.
///
/// A single scan — rather than independent regex passes run in some fixed
/// order — is required because any fixed pass order is wrong in the opposite
/// direction: running a block-comment stripper before line comments are
/// stripped lets a `#|`/`|#` marker that's merely prose inside a `;` comment
/// (e.g. `; see #| syntax carefully`) kick off a real block-comment match
/// that swallows genuine code up to some unrelated, later `|#`. Running line
/// comments first instead lets a `;` inside a genuine `#| ... |#` block
/// comment (ordinary reference prose, e.g. "old approach; deprecated")
/// truncate that line before its own `|#` closer is reached, which can then
/// eat real code up to the next stray `|#`. A `;` or `#|` living inside a
/// string literal (e.g. `(defvar *msg* "foo ; bar")`) has the same problem
/// in both orderings: it gets misread as a comment start and swallows the
/// rest of the line, including any real `defun` sharing that line. A single
/// scan resolves each `;`, `"`, and `#|` strictly in the order they're
/// actually encountered, matching how a real Lisp reader tokenizes.
///
/// `#| ... |#` nesting is tracked with a depth counter: Common Lisp
/// explicitly allows arbitrary nesting of block comments (e.g. an outer
/// `#| ... #| ... |# ... |#` comment wrapping a smaller one), and a naive
/// non-nested stripper stops at the *first* `|#`, leaking the "still
/// commented out" tail — including any code hiding in it — back into the
/// output.
///
/// Blanked characters are replaced with a space (newlines preserved) so the
/// `(?m)`-anchored `LISP_DEF` regex downstream stays aligned to the original
/// line structure. An unterminated `"` or `#|` blanks to end of input rather
/// than panicking or looping.
fn strip_comments_and_strings(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < n {
        let c = chars[i];
        if c == ';' {
            while i < n && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if c == '"' {
            i += 1;
            while i < n {
                if chars[i] == '\\' && i + 1 < n {
                    i += 2;
                    continue;
                }
                if chars[i] == '"' {
                    i += 1;
                    break;
                }
                if chars[i] == '\n' {
                    out.push('\n');
                }
                i += 1;
            }
            continue;
        }
        if c == '#' && i + 1 < n && chars[i + 1] == '|' {
            let mut depth = 1u32;
            i += 2;
            while i < n && depth > 0 {
                if chars[i] == '#' && i + 1 < n && chars[i + 1] == '|' {
                    depth += 1;
                    i += 2;
                } else if chars[i] == '|' && i + 1 < n && chars[i + 1] == '#' {
                    depth -= 1;
                    i += 2;
                } else {
                    if chars[i] == '\n' {
                        out.push('\n');
                    }
                    i += 1;
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
    fn test_ignores_block_comments() {
        // `#| ... |#` block comments can wrap dead/commented-out
        // definitions; only `;`-line comments were stripped before, so
        // these leaked into the extracted symbol list as false positives.
        let src = "#|\nOld implementation, kept for reference:\n(defun legacy-parse (x) (parse-legacy x))\n(defparameter *old-limit* 10)\n|#\n(defun real-parse (x) (parse x))\n";
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["real-parse"]);
    }

    #[test]
    fn test_lisp_commented_out_defun_on_non_last_line_not_leaked() {
        // Regression test: `COMMENT_SINGLE` previously lacked `(?m)`, so `$` only
        // anchored to the end of the whole file, not each line -- a `;` comment was
        // only ever stripped if it happened to be on the file's literal last line. A
        // commented-out `(defun fake ...)` on any earlier line leaked straight through
        // since `LISP_DEF` searches unanchored.
        let src = "; (defun fake (x) x)\n(defun real-fn (x) (+ x 1))\n";
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"real-fn".to_string()));
        assert!(!symbols.contains(&"fake".to_string()));
    }

    #[test]
    fn test_defstruct_name_captured() {
        let src = "(defstruct point x y)\n(defun distance (p1 p2) (+ (point-x p1) (point-y p2)))\n";
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"point".to_string()));
        assert!(symbols.contains(&"distance".to_string()));
    }

    #[test]
    fn test_semicolon_comment_containing_block_marker_does_not_swallow_real_code() {
        // Regression: with independent regex passes (block comments
        // stripped before line comments), a `;` comment merely *mentioning*
        // `#|` as prose kicked off a real block-comment match that ran all
        // the way to the next unrelated `|#`, silently eating the entire
        // `real-fn` defun in between. Before the fix this produced only
        // `["another-fn"]`.
        let src = "; see #| syntax carefully\n(defun real-fn (x) x)\n#| actual comment |#\n(defun another-fn (x) x)\n";
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"real-fn".to_string()), "got {symbols:?}");
        assert!(
            symbols.contains(&"another-fn".to_string()),
            "got {symbols:?}"
        );
    }

    #[test]
    fn test_semicolon_inside_string_does_not_eat_rest_of_line() {
        // Regression: a `;` inside a string literal was previously misread
        // as a comment start (no string-literal awareness existed at all),
        // truncating the rest of the line -- including a real `defun`
        // sharing that line. Before the fix this produced only `["*msg*"]`.
        let src = "(defvar *msg* \"foo ; bar\") (defun real-fn (x) x)\n";
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"*msg*".to_string()), "got {symbols:?}");
        assert!(symbols.contains(&"real-fn".to_string()), "got {symbols:?}");
    }

    #[test]
    fn test_nested_block_comment_is_fully_stripped() {
        // Common Lisp explicitly allows nested `#| ... #| ... |# ... |#`
        // block comments. A naive non-nested stripper stops at the first
        // `|#`, leaking the "still commented out" tail -- including
        // `secret-fn` -- back into the output. Depth-tracking fixes this.
        let src = "#| outer comment\n#| inner nested |#\n(defun secret-fn (x) x)\nstill commented out\n|#\n(defun real-fn (x) x)\n";
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"real-fn".to_string()), "got {symbols:?}");
        assert!(
            !symbols.contains(&"secret-fn".to_string()),
            "nested block comment leaked, got {symbols:?}"
        );
    }

    #[test]
    fn test_string_containing_parens_and_hash_pipe_sequence() {
        // A docstring/string argument containing literal parens and a
        // `#|`-looking sequence must not be mistaken for real code or a
        // block-comment opener.
        let src = r#"(defvar *note* "see (foo #| bar) for details") (defun real-fn (x) x)"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"*note*".to_string()), "got {symbols:?}");
        assert!(symbols.contains(&"real-fn".to_string()), "got {symbols:?}");
    }

    #[test]
    fn test_lisp_exports() {
        let src = r#"
;; Lisp program
(defun add-numbers (a b)
  (+ a b))

(defparameter *global-var* 42)

(defmacro with-something (&body body)
  `(progn ,@body))
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"add-numbers".to_string()));
        assert!(symbols.contains(&"*global-var*".to_string()));
        assert!(symbols.contains(&"with-something".to_string()));
    }
}
