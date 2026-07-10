use regex::Regex;
use std::sync::LazyLock;

// `(?m)` is required so `$` anchors to each line's end, not just the end of
// the whole (potentially multi-line) source string: without it, `.` (which
// never crosses `\n`) can only ever reach `$` on the file's literal last
// line, so a `;` comment on any earlier line is left completely unstripped.
static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m);.*$").unwrap());

/// Common Lisp `#| ... |#` block comments (non-nested — nesting is rare in
/// practice and not worth a hand-rolled balanced-delimiter scanner here).
static COMMENT_BLOCK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)#\|.*?\|#").unwrap());

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
    let content = COMMENT_BLOCK.replace_all(content, "");
    let stripped = COMMENT_SINGLE.replace_all(&content, "");

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
