use regex::Regex;
use std::sync::LazyLock;

/// Clojure top-level defining forms. Group 1 is the head keyword — it captures
/// the private-suffix convention directly (`defn-`, `def-`) since Clojure marks
/// privacy on the *defining form*, not the name: `(defn- foo ...)` defines a
/// private function literally named `foo`. Group 2 is any metadata preceding
/// the name: `^:private` / `^{:private true}` (the equivalent long-hand way to
/// mark `defn`/`def` private), or a bare/namespaced type-hint symbol like
/// `^String`, `^long`, or `^java.lang.String` (e.g. `(defn ^String greeting
/// [x] ...)`). Type hints are extremely common on `defn` return values and
/// must not prevent the name itself (group 3) from matching — a version of
/// this regex that only recognized `^{...}`/`^:kw` metadata (not bare symbol
/// hints) failed to match the whole form at all for a hinted `defn`, silently
/// dropping the function from the export list entirely.
///
/// The hint after `^` is deliberately *optional* (not just an alternation of
/// required forms): a string-form type hint like `^"[Ljava.lang.String;"`
/// (used for Java array types) is itself a string literal, so it gets fully
/// erased — quotes and all — by [`strip_comments_and_strings`] before this
/// regex ever runs, leaving a bare dangling `^` with nothing recognizable
/// after it. Requiring one of `{...}`/`:kw`/symbol to follow `^` would make
/// the whole match fail on exactly that (rare but real) case; treating the
/// hint as optional lets a content-less `^` still be consumed as one
/// (now-empty) metadata token so the name after it is never lost.
///
/// Group 3 is the name itself. `defrecord`/`deftype`/`defprotocol`/`defmacro`
/// have no private convention in common use, so they're always treated as
/// public. `defmulti`/`defonce` behave like `def` (public unless
/// dash-suffixed or `^:private`-tagged).
static CLOJURE_DEF: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\(\s*(defn-|defn|def-|def|defmacro|defrecord|deftype|defprotocol|defmulti|defonce)\s+((?:\^(?:\{[^}]*\}|:\S+|[A-Za-z_][\w.]*)?\s+)*)([A-Za-z0-9_*+!?<>=.\-]+)",
    )
    .unwrap()
});

/// The start of a `defprotocol` form, up through its name. `defprotocol`
/// bodies list method signatures as bare `(name [args] ...)` forms (they don't
/// reuse `defn`), so they need separate handling from `CLOJURE_DEF` above.
static PROTOCOL_START: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\(\s*defprotocol\s+[A-Za-z0-9_*+!?<>=.\-]+").unwrap());

/// A method signature line inside a `defprotocol` body: `(name [args] ...)`,
/// optionally with multiple arity vectors on the same line.
static PROTOCOL_METHOD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\s*\(\s*([A-Za-z][A-Za-z0-9_*+!?<>=.\-]*)\s*\[").unwrap());

/// Extract public symbols from Clojure source code.
pub fn extract_exports(content: &str) -> Vec<String> {
    // Pass 1: blank `;` comments and `"…"` string literals in one linear scan
    // so a `;` inside a string can't swallow real code after it on the same
    // line, and code-shaped text inside a docstring (very common in `defn`
    // doc examples) can't be misread as a real declaration.
    let stripped = strip_comments_and_strings(content);
    // Pass 2: blank the bodies of `(comment ...)` forms and `#_(...)`
    // reader-discarded forms. Both are real Clojure mechanisms for keeping
    // code checked in but never evaluated (REPL scratch blocks and
    // one-off "disable this def" respectively); anything defined inside
    // never becomes a var and must not be reported as a live export. Safe
    // to paren-balance now that pass 1 removed every stray `(`/`)` that
    // could have been hiding inside a comment or string.
    let stripped = strip_dead_forms(&stripped);

    let mut symbols = Vec::new();

    for caps in CLOJURE_DEF.captures_iter(&stripped) {
        let head = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let meta = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let name = match caps.get(3) {
            Some(n) => n.as_str().to_string(),
            None => continue,
        };

        // Private via the `defn-`/`def-` defining-form convention, or via
        // `^:private` / `^{:private ...}` metadata on the name.
        let is_private = head.ends_with('-') || meta.contains("private");
        if is_private {
            continue;
        }

        if !symbols.contains(&name) {
            symbols.push(name);
        }
    }

    extract_protocol_methods(&stripped, &mut symbols);

    symbols
}

/// Scan every `defprotocol` form and pull out its method names. The body is
/// taken as everything from the protocol name up to the next top-level
/// (column-zero) form, or end of input — `regex` has no lookaround, so the
/// boundary is found with a plain substring search rather than in-regex.
fn extract_protocol_methods(stripped: &str, symbols: &mut Vec<String>) {
    for m in PROTOCOL_START.find_iter(stripped) {
        let rest = &stripped[m.end()..];
        let body_end = rest
            .match_indices("\n(")
            .map(|(i, _)| i + 1)
            .next()
            .unwrap_or(rest.len());
        let body = &rest[..body_end];

        for caps in PROTOCOL_METHOD.captures_iter(body) {
            if let Some(name) = caps.get(1) {
                let n = name.as_str().to_string();
                if !symbols.contains(&n) {
                    symbols.push(n);
                }
            }
        }
    }
}

/// Blank the contents of `;` line comments and `"…"` string literals in a
/// single left-to-right pass, so neither can be misread as the other and
/// nothing inside either leaks into the def-matching regexes below:
///
/// - A `;` living inside a string (e.g. `"See ; below"`) must not be
///   mistaken for a comment start, which would silently truncate the rest
///   of that line — including any real declaration sharing the line after
///   the string.
/// - A `"` living inside a `;` comment must not be mistaken for the start
///   of a string, which would swallow real code as "string content" until
///   the next stray quote turns up.
/// - Docstrings routinely quote example code (`"Example: (defn foo ...)"`),
///   which would otherwise be misread as a real top-level declaration.
///
/// Blanked characters are replaced with a space (newlines are preserved) so
/// the `(?m)`-anchored regexes downstream stay aligned to the original
/// line structure.
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
        out.push(c);
        i += 1;
    }
    out
}

/// Blank the bodies of `(comment ...)` forms and `#_(...)` reader-discarded
/// forms. Must run after [`strip_comments_and_strings`] so that a paren
/// hiding inside a `;` comment or a string literal (e.g. a docstring
/// containing literal `)` prose) can never be miscounted while balancing
/// parens to find a form's real end.
///
/// - `(comment ...)` is `clojure.core`'s "rich comment" macro: it reads its
///   body (so it must be syntactically valid Clojure) but never evaluates
///   it. Scratch `defn`/`def` forms left inside one — an extremely common
///   REPL-driven-development idiom — never become vars and must not be
///   reported as live exports.
/// - `#_` is the reader's discard macro: it parses and throws away exactly
///   the next form. `#_(defn old-fn [] ...)` is a common way to disable a
///   single top-level def without deleting it.
fn strip_dead_forms(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < n {
        if chars[i] == '(' {
            let mut k = i + 1;
            while k < n && chars[k].is_whitespace() {
                k += 1;
            }
            if starts_with_word(&chars[k..], "comment") {
                let form_end = matching_paren_end(&chars, i);
                for &c in &chars[i..form_end] {
                    out.push(if c == '\n' { '\n' } else { ' ' });
                }
                i = form_end;
                continue;
            }
        }

        if chars[i] == '#' && i + 1 < n && chars[i + 1] == '_' {
            let mut j = i + 2;
            while j < n && chars[j].is_whitespace() {
                j += 1;
            }
            if j < n && chars[j] == '(' {
                let form_end = matching_paren_end(&chars, j);
                for &c in &chars[i..form_end] {
                    out.push(if c == '\n' { '\n' } else { ' ' });
                }
                i = form_end;
                continue;
            }
        }

        out.push(chars[i]);
        i += 1;
    }
    out
}

/// True if `chars` starts with the complete token `word` — i.e. `word`
/// followed by whitespace, a `)`, or end of input — rather than merely
/// being a prefix of a longer symbol (`comment-out`, `commentary`).
fn starts_with_word(chars: &[char], word: &str) -> bool {
    let word_len = word.chars().count();
    if chars.len() < word_len || !chars[..word_len].iter().copied().eq(word.chars()) {
        return false;
    }
    match chars.get(word_len) {
        None => true,
        Some(c) => c.is_whitespace() || *c == ')',
    }
}

/// Given the index of an opening `(`, return the index just past its
/// matching `)` (depth-balanced). Falls back to the end of input for
/// unbalanced/truncated source rather than panicking.
fn matching_paren_end(chars: &[char], open_idx: usize) -> usize {
    let mut depth = 0i32;
    let mut i = open_idx;
    while i < chars.len() {
        match chars[i] {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return i + 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    chars.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_defn_and_def() {
        let src = r#"
(defn greet
  "Says hello."
  [name]
  (str "Hello, " name))

(def default-timeout 5000)
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"greet".to_string()));
        assert!(symbols.contains(&"default-timeout".to_string()));
    }

    #[test]
    fn test_defn_dash_marks_private() {
        let src = r#"
(defn public-api [x]
  (helper x))

(defn- helper [x]
  (* x 2))
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"public-api".to_string()));
        assert!(
            !symbols.contains(&"helper".to_string()),
            "defn- must be excluded, got {symbols:?}"
        );
    }

    #[test]
    fn test_private_metadata_forms_excluded() {
        let src = r#"
(defn ^:private secret-fn [x] x)
(def ^:private secret-val 42)
(defn ^{:doc "internal" :private true} another-secret [] nil)
(defn visible-fn [] :ok)
"#;
        let symbols = extract_exports(src);
        assert!(!symbols.contains(&"secret-fn".to_string()));
        assert!(!symbols.contains(&"secret-val".to_string()));
        assert!(
            !symbols.contains(&"another-secret".to_string()),
            "got {symbols:?}"
        );
        assert!(symbols.contains(&"visible-fn".to_string()));
    }

    #[test]
    fn test_def_dash_rare_private_form() {
        // `def-` isn't a clojure.core macro, but some codebases define their
        // own; treat it the same as `defn-` for consistency.
        let src = "(def- internal-const 1)\n(def public-const 2)\n";
        let symbols = extract_exports(src);
        assert!(!symbols.contains(&"internal-const".to_string()));
        assert!(symbols.contains(&"public-const".to_string()));
    }

    #[test]
    fn test_comment_stripping_is_multiline() {
        // A commented-out defn on a line that is NOT the file's last line
        // must not leak through — regression for the `(?m)` flag on `$`.
        let src = r#"
;; (defn fake-fn [] :not-real)
;; TODO: remove old-helper below
(defn real-fn [] :ok)
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"real-fn".to_string()));
        assert!(!symbols.contains(&"fake-fn".to_string()));
        assert!(!symbols.contains(&"old-helper".to_string()));
    }

    #[test]
    fn test_docstring_code_example_not_leaked_as_export() {
        // Doc examples inside a real `defn`'s docstring commonly show
        // sample usage as literal `(defn ...)`/`(def ...)` snippets. That
        // text lives inside a string literal and must never be read as a
        // real declaration.
        let src = r#"
(defn parse-config
  "Parses a config map.

  Example:
    (defn my-handler [x] (parse-config x))
    (def my-secret 1)"
  [x]
  x)
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"parse-config".to_string()));
        assert!(
            !symbols.contains(&"my-handler".to_string()),
            "docstring example code leaked through as an export, got {symbols:?}"
        );
        assert!(
            !symbols.contains(&"my-secret".to_string()),
            "docstring example code leaked through as an export, got {symbols:?}"
        );
    }

    #[test]
    fn test_semicolon_inside_string_does_not_eat_rest_of_line() {
        // A `;` inside a string literal is ordinary string content, not a
        // comment start, and must not swallow a real declaration that
        // shares the line after the string.
        let src = r#"(def notice "See ; below for details") (def real-var 1)"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"real-var".to_string()),
            "semicolon inside a string literal wrongly treated as a comment start, got {symbols:?}"
        );
    }

    #[test]
    fn test_comment_macro_body_not_exported() {
        // `(comment ...)` is a real Clojure macro that reads but never
        // evaluates its body — a ubiquitous "rich comment" scratch-code
        // idiom. Anything defined inside one never becomes a var.
        let src = r#"
(comment
  (defn scratch-fn [x] (+ x 1))
  (def scratch-val 42))

(defn real-fn [x] (+ x 1))
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"real-fn".to_string()));
        assert!(
            !symbols.contains(&"scratch-fn".to_string()),
            "(comment ...) body should not count as a live export, got {symbols:?}"
        );
        assert!(
            !symbols.contains(&"scratch-val".to_string()),
            "(comment ...) body should not count as a live export, got {symbols:?}"
        );
    }

    #[test]
    fn test_reader_discard_form_not_exported() {
        // `#_(...)` discards exactly the next form at read time — another
        // common way to disable a single top-level def in place.
        let src = "#_(defn disabled-fn [x] x)\n(defn enabled-fn [x] x)\n";
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"enabled-fn".to_string()));
        assert!(
            !symbols.contains(&"disabled-fn".to_string()),
            "#_(...) discarded form should not count as a live export, got {symbols:?}"
        );
    }

    #[test]
    fn test_comment_like_names_not_mistaken_for_comment_macro() {
        // `comment-out` and `commentary` are real, distinct symbol names;
        // the `(comment ...)` stripper must not treat them as the
        // `comment` macro just because they share a prefix.
        let src = r#"
(defn comment-out [x] x)
(def commentary "notes")
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"comment-out".to_string()),
            "got {symbols:?}"
        );
        assert!(
            symbols.contains(&"commentary".to_string()),
            "got {symbols:?}"
        );
    }

    #[test]
    fn test_defrecord_deftype_defmacro_always_public() {
        let src = r#"
(defrecord Point [x y])
(deftype MutableBox [^:volatile-mutable val])
(defmacro unless [test body] `(if (not ~test) ~body))
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Point".to_string()));
        assert!(symbols.contains(&"MutableBox".to_string()));
        assert!(symbols.contains(&"unless".to_string()));
    }

    #[test]
    fn test_defprotocol_and_its_methods_are_public() {
        let src = r#"
(defprotocol Shape
  "A protocol for 2D shapes."
  (area [this] "Returns the area (in square units) of this shape.")
  (perimeter [this] [this unit]))

(defrecord Circle [radius]
  Shape
  (area [this] (* Math/PI radius radius))
  (perimeter [this] (* 2 Math/PI radius)))
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Shape".to_string()), "got {symbols:?}");
        assert!(symbols.contains(&"area".to_string()), "got {symbols:?}");
        assert!(
            symbols.contains(&"perimeter".to_string()),
            "got {symbols:?}"
        );
        assert!(symbols.contains(&"Circle".to_string()));
    }

    #[test]
    fn test_main_entry_point_and_dynamic_var() {
        // `-main` (name literally starts with a dash) and `^:dynamic` vars are
        // both common, real-world public declarations.
        let src = r#"
(def ^:dynamic *config* {:env "dev"})

(defn -main
  [& args]
  (println "starting" args))
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"*config*".to_string()));
        assert!(symbols.contains(&"-main".to_string()), "got {symbols:?}");
    }

    #[test]
    fn test_bare_symbol_type_hint_does_not_hide_the_def() {
        // Regression: `(defn ^String greeting ...)` is a very common
        // real-world shape (return-type hint on a public function). Before
        // the fix, the metadata group only recognized `^{...}`/`^:kw` forms,
        // so a bare symbol type hint like `^String` made the *entire* regex
        // fail to match at this position -- silently dropping `greeting`
        // and `compute` from the export list rather than just misreading
        // the hint.
        let src = "(defn ^String greeting [x] (str \"Hi \" x))\n(defn ^long compute [x] (* x 2))\n";
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"greeting".to_string()), "got {symbols:?}");
        assert!(symbols.contains(&"compute".to_string()), "got {symbols:?}");
    }

    #[test]
    fn test_namespaced_and_string_form_type_hints() {
        // Namespaced symbol hints (`^java.lang.String`) are valid Clojure
        // metadata shorthand and must not swallow the name. The string-form
        // hint used for Java array types (`^"[Ljava.lang.String;"`) is a
        // string literal, so `strip_comments_and_strings` erases it (quotes
        // and all) before `CLOJURE_DEF` ever runs, leaving a bare `^`; the
        // name after it must still be found rather than the whole match
        // failing.
        let src = r#"
(defn ^java.lang.String full-name [x] x)
(defn ^"[Ljava.lang.String;" string-array [x] x)
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"full-name".to_string()),
            "got {symbols:?}"
        );
        assert!(
            symbols.contains(&"string-array".to_string()),
            "got {symbols:?}"
        );
    }

    #[test]
    fn test_type_hint_stacked_with_private_metadata_still_excluded() {
        // A type hint stacked together with `^:private` metadata (order:
        // type hint first, then the privacy tag) must still be recognized
        // as private -- the hint alone shouldn't distract from the
        // `contains("private")` check on the whole metadata group.
        let src = "(defn ^String ^:private secret-greeting [x] x)\n(defn ^String public-greeting [x] x)\n";
        let symbols = extract_exports(src);
        assert!(
            !symbols.contains(&"secret-greeting".to_string()),
            "got {symbols:?}"
        );
        assert!(
            symbols.contains(&"public-greeting".to_string()),
            "got {symbols:?}"
        );
    }

    #[test]
    fn test_multiple_defs_on_same_line() {
        // Verified-correct existing behavior: the regex is unanchored, so
        // two top-level forms sharing one physical line are both found.
        let src = "(def a 1) (def b 2)\n";
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"a".to_string()), "got {symbols:?}");
        assert!(symbols.contains(&"b".to_string()), "got {symbols:?}");
    }

    #[test]
    fn test_defmulti_and_defonce_included() {
        let src = r#"
(defonce app-state (atom {}))

(defmulti render :shape-type)

(defmethod render :circle [shape]
  (draw-circle shape))
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"app-state".to_string()));
        assert!(symbols.contains(&"render".to_string()));
    }
}
