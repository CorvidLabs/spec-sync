use regex::Regex;
use std::sync::LazyLock;

/// OCaml has no visibility keyword in a `.ml` file. Real access control lives in the
/// sibling `.mli` signature file (which restricts what's visible to other modules);
/// with no `.mli` present, EVERY top-level binding is public by default. So there is
/// nothing to search for like `pub`/`public` — instead this matches the shapes of
/// top-level (column 0) definitions: `let`/`let rec`/`and`, `type`, `module`
/// (including `module type` and `module rec`), and `exception`. Anchoring each
/// alternative on `^` in multiline mode is what keeps this to *top-level* bindings —
/// a `let` indented inside a function body (a local `let x = ... in ...`) or a
/// `type`/`module` nested inside a `struct ... end` never starts at column 0, so it
/// is correctly left uncaptured.
///
/// - `let rec name ...` / `let name ...`: a value binding. `let () = ...` — the
///   conventional unit-pattern entry point, OCaml's rough equivalent of `main` — is
///   naturally excluded: `(` is not a word character, so `(\w+)` never matches it.
///   `let module M = ...`, `let open M in ...`, and `let exception E in ...` are real
///   (if uncommon) local-scope forms that also start with `let`; the regex crate has
///   no look-ahead to exclude them inline, so `extract_exports` below drops a plain-
///   `let` capture when it's literally the keyword `module`/`open`/`exception` (none
///   of which can ever be a real OCaml identifier) rather than a bound name.
/// - `and name ...`: the continuation of a preceding top-level `let rec`, `type`, or
///   `module rec` group (mutually-recursive definitions) — still top-level, still
///   public.
/// - `type name = ...` / `type 'a name = ...` / `type ('a, 'b) name = ...` /
///   `type nonrec name = ...`: the optional group skips a single type variable or a
///   parenthesized list of them so the type's own name is captured, not its first
///   parameter.
/// - `module Name = ...` / `module Name (Arg : Sig) = ...` / `module type Name = ...`
///   / `module rec Name = ...`.
/// - `exception Name` / `exception Name of ty`.
static OCAML_DECL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^(?:let\s+rec\s+(\w+)|let\s+(\w+)|and\s+(\w+)|type\s+(?:nonrec\s+)?(?:'\w+\s+|\(\s*'\w+(?:\s*,\s*'\w+)*\s*\)\s+)?(\w+)|module\s+(?:type\s+|rec\s+)?(\w+)|exception\s+(\w+))",
    )
    .unwrap()
});

/// Top-level operator definition, e.g. `let (>>=) m f = ...`, `let rec (>>=) ...`,
/// or a mutually-recursive continuation `and (<|>) ...`. Custom infix operators
/// are extremely common in real-world OCaml (monadic `>>=`, applicative `<*>`,
/// pipeline `|>`-style combinators, ...) and are just as public as any other
/// top-level `let` binding, but `OCAML_DECL` above can never match them: its
/// `let\s+(\w+)` alternative requires a `\w` (word) character immediately
/// after the keyword, and `(` is not one. The character class here is
/// OCaml's own operator-symbol alphabet, so `let () = ...` (the conventional
/// unit-pattern entry point, deliberately treated as a non-export) is safely
/// excluded: empty parens contain zero operator characters, so the `+`
/// quantifier never matches.
static OCAML_OPERATOR_DECL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^(?:let\s+rec\s+|let\s+|and\s+)\(([!$%&*+\-./:<=>?@^|~]+)\)").unwrap()
});

/// OCaml attribute payloads — `[@attr]`, `[@@attr]`, `[@@@attr]` — blanked out (with a
/// single space) before `OCAML_DECL` runs. An attribute can sit directly between a
/// keyword and its name (`let[@inline] compute x = ...`), and without stripping it
/// first the name would simply never follow `let\s+` (there'd be a `[` there
/// instead). Non-nested: real-world attribute payloads (`[@warning "-27"]`,
/// `[@@deriving show]`) don't themselves contain a `]`.
static ATTRIBUTE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[@+[^\[\]]*\]").unwrap());

/// Extract exported symbols from OCaml source code.
///
/// With no `.mli` in view, every top-level `let`/`let rec`/`and`/`type`/`module`/
/// `exception` binding is public by construction — see `OCAML_DECL` for the exact
/// mechanism.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = strip_comments_and_strings(content);
    let stripped = ATTRIBUTE.replace_all(&stripped, " ");

    let mut hits: Vec<(usize, String)> = Vec::new();

    for caps in OCAML_DECL.captures_iter(&stripped) {
        for i in 1..=6 {
            if let Some(m) = caps.get(i) {
                let name = m.as_str();
                // Group 2 is the plain `let name ...` alternative. `let module M =
                // ...`, `let open M in ...`, and `let exception E in ...` are local-
                // scope forms that also start with `let`; since none of `module`/
                // `open`/`exception` can ever be a real OCaml identifier, a capture
                // of exactly that keyword here means we matched one of those forms
                // rather than a bound name, and should be dropped. `rec` is
                // excluded for the same reason but a different route: it's always
                // a reserved keyword, never a legal binding name, so a plain-`let`
                // capture of literally "rec" only ever happens when the dedicated
                // `let\s+rec\s+(\w+)` alternative failed to match a following
                // identifier -- e.g. `let rec (>>>) f g x = ...`, where the bound
                // name is an operator handled separately by
                // `OCAML_OPERATOR_DECL`. Without this, that line would additionally
                // (and wrongly) report a top-level binding literally named "rec".
                let is_local_scope_keyword =
                    i == 2 && matches!(name, "module" | "open" | "exception" | "rec");
                if !is_local_scope_keyword {
                    hits.push((m.start(), name.to_string()));
                }
                break;
            }
        }
    }

    for caps in OCAML_OPERATOR_DECL.captures_iter(&stripped) {
        if let Some(m) = caps.get(1) {
            hits.push((m.start(), m.as_str().to_string()));
        }
    }

    hits.sort_by_key(|(pos, _)| *pos);

    let mut symbols = Vec::new();
    for (_, name) in hits {
        push(&name, &mut symbols);
    }
    symbols
}

fn push(name: &str, symbols: &mut Vec<String>) {
    // `let _ = ...` is a deliberate "evaluate and discard" binding, not a named
    // export — `_` is not an identifier that any other module could reference.
    if name == "_" {
        return;
    }
    let n = name.to_string();
    if !symbols.contains(&n) {
        symbols.push(n);
    }
}

/// Blank comments and string-like literals in one linear pass so neither is misread
/// as code and nothing inside them leaks a `let`/`type`/`module`/`exception` keyword
/// into a match. Handles, at each position: `(* ... *)` block comments (OCaml
/// comments nest, unlike C-family `/* */`); quoted string literals `{id|...|id}` for
/// any lowercase-identifier delimiter, including the empty delimiter `{|...|}`
/// (only treated as one if a `|` actually follows the candidate id — otherwise this
/// is ordinary record/list syntax like `{ x = 1 }` or `{x with y = 1}` and is left
/// alone); `'"'`/`'\"'` char literals (whose interior `"` would otherwise open a
/// string); and regular `"..."` strings with `\` escapes. Newlines are preserved so
/// the `(?m)` callers stay aligned.
///
/// A block comment that is the first thing on its line also has the whitespace
/// between it and whatever follows swallowed (see `line_start` below) — otherwise
/// a same-line leading comment like `(* doc *) let x = 1` would leave a stray
/// space in front of `let`, pushing it off column 0 and hiding a genuine
/// top-level export from `OCAML_DECL`'s `^` anchor.
fn strip_comments_and_strings(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    // Tracks whether only whitespace (or nothing) has appeared since the last
    // newline / start of file — i.e. whether we're still positioned at the true
    // start of the current logical line. `OCAML_DECL` anchors on `^` (column 0)
    // to detect top-level declarations, so a same-line leading comment like
    // `(* doc *) let x = 1` must not leave behind the whitespace that separated
    // it from `let` — that stray space would shift `let` off column 0 in the
    // stripped text and cause a real top-level export to be missed, even though
    // a leading comment never nests the code that follows it. See the trailing-
    // whitespace swallow below the block-comment branch.
    let mut line_start = true;
    while i < n {
        let c = chars[i];
        let comment_started_at_line_start = line_start;

        // Block comment `(* ... *)` — OCaml comments nest.
        if c == '(' && i + 1 < n && chars[i + 1] == '*' {
            i += 2;
            let mut depth = 1;
            while i < n && depth > 0 {
                if chars[i] == '(' && i + 1 < n && chars[i + 1] == '*' {
                    depth += 1;
                    i += 2;
                } else if chars[i] == '*' && i + 1 < n && chars[i + 1] == ')' {
                    depth -= 1;
                    i += 2;
                } else {
                    if chars[i] == '\n' {
                        out.push('\n');
                    }
                    i += 1;
                }
            }
            // If this comment was the first thing on its line, it isn't real
            // indentation — swallow the whitespace between it and whatever
            // follows (more comments, or the actual next token) so that token
            // keeps its true column-0 position in the stripped output.
            if comment_started_at_line_start {
                while i < n && (chars[i] == ' ' || chars[i] == '\t') {
                    i += 1;
                }
            }
            continue;
        }

        // Quoted string literal `{id|...|id}`.
        if c == '{' {
            let mut j = i + 1;
            while j < n
                && (chars[j].is_ascii_lowercase() || chars[j].is_ascii_digit() || chars[j] == '_')
            {
                j += 1;
            }
            if j < n && chars[j] == '|' {
                let id: Vec<char> = chars[i + 1..j].to_vec();
                let mut closer = vec!['|'];
                closer.extend(id);
                closer.push('}');

                let mut k = j + 1;
                let mut found = None;
                while k < n {
                    if chars[k..].starts_with(&closer[..]) {
                        found = Some(k + closer.len());
                        break;
                    }
                    if chars[k] == '\n' {
                        out.push('\n');
                    }
                    k += 1;
                }
                i = found.unwrap_or(n);
                continue;
            }
        }

        // Char literal containing a double quote — `'"'` or `'\"'`. Blanked so its
        // `"` is not read as a string opener. Other char literals (`'a'`, `'\n'`)
        // hold no `"` and are harmless, so they are left as-is.
        if c == '\'' {
            if i + 2 < n && chars[i + 1] == '"' && chars[i + 2] == '\'' {
                i += 3;
                continue;
            }
            if i + 3 < n && chars[i + 1] == '\\' && chars[i + 2] == '"' && chars[i + 3] == '\'' {
                i += 4;
                continue;
            }
        }

        // Regular `"..."` string literal, honoring `\` escapes.
        if c == '"' {
            i += 1;
            while i < n {
                if chars[i] == '\\' {
                    if i + 1 < n && chars[i + 1] == '\n' {
                        out.push('\n');
                    }
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
        line_start = if c == '\n' {
            true
        } else if c == ' ' || c == '\t' {
            line_start
        } else {
            false
        };
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ocaml_basic_exports() {
        let src = r#"
let greeting = "hello"

let add x y = x + y

let rec factorial n =
  if n <= 1 then 1 else n * factorial (n - 1)

type point = { x : int; y : int }

module Config = struct
  let default_timeout = 30
end

exception Invalid_input of string

let () =
  print_endline greeting
"#;
        let symbols = extract_exports(src);
        assert_eq!(
            symbols,
            vec![
                "greeting",
                "add",
                "factorial",
                "point",
                "Config",
                "Invalid_input",
            ]
        );
        // The unit-pattern entry point is not a meaningful named export.
        assert!(!symbols.contains(&"()".to_string()));
    }

    #[test]
    fn test_ocaml_excludes_local_bindings() {
        // Bindings inside a function body (indented `let ... in`) are local scope,
        // not top-level exports, even though they use the same `let` keyword.
        let src = r#"
let compute total =
  let tax = total *. 0.08 in
  let shipping = 5.0 in
  total +. tax +. shipping

let _ = compute 100.0
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["compute"]);
        assert!(!symbols.contains(&"tax".to_string()));
        assert!(!symbols.contains(&"shipping".to_string()));
        // `let _ = ...` discards its value; `_` is not a referenceable name.
        assert!(!symbols.contains(&"_".to_string()));
    }

    #[test]
    fn test_ocaml_strips_comments() {
        let src = r#"
(* let fake_export = 1 *)
let real_export = 2

(* a nested (* comment *) still hides this *)
let another_real = 3

(** doc comment mentioning let phantom = 1 in prose *)
let third_real = 4
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["real_export", "another_real", "third_real"]);
        assert!(!symbols.contains(&"fake_export".to_string()));
        assert!(!symbols.contains(&"phantom".to_string()));
    }

    #[test]
    fn test_ocaml_mutually_recursive_let_and_type() {
        let src = r#"
let rec is_even n =
  if n = 0 then true else is_odd (n - 1)

and is_odd n =
  if n = 0 then false else is_even (n - 1)

type tree =
  | Leaf
  | Node of tree * int * tree

and forest = tree list
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["is_even", "is_odd", "tree", "forest"]);
    }

    #[test]
    fn test_ocaml_modules_and_module_types() {
        let src = r#"
module type COMPARABLE = sig
  type t
  val compare : t -> t -> int
end

module IntSet = struct
  type t = int list
  let empty = []
end

module rec Even : sig
  val is_even : int -> bool
end = struct
  let is_even n = n = 0 || Odd.is_odd (n - 1)
end

and Odd : sig
  val is_odd : int -> bool
end = struct
  let is_odd n = n <> 0 && Even.is_even (n - 1)
end
"#;
        let symbols = extract_exports(src);
        for name in ["COMPARABLE", "IntSet", "Even", "Odd"] {
            assert!(
                symbols.contains(&name.to_string()),
                "missing {name}: {symbols:?}"
            );
        }
        // Nested `type t`/`let empty`/`val` inside module bodies are indented, not
        // top-level, and `val` isn't a binding keyword we track at all.
        assert!(!symbols.contains(&"t".to_string()));
    }

    #[test]
    fn test_ocaml_type_parameters() {
        let src = r#"
type 'a option_list = 'a option list

type ('a, 'b) result = Ok of 'a | Error of 'b

type nonrec t = int
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["option_list", "result", "t"]);
    }

    #[test]
    fn test_ocaml_strings_and_char_literals_do_not_hide_exports() {
        let src = r#"
let quote_char = '"'

let escaped_quote = '\"'

let template = "not real code: let fake = 1 ;; module Fake = struct end"

let real_after_string = 42
"#;
        let symbols = extract_exports(src);
        assert_eq!(
            symbols,
            vec![
                "quote_char",
                "escaped_quote",
                "template",
                "real_after_string"
            ]
        );
        assert!(!symbols.contains(&"fake".to_string()));
        assert!(!symbols.contains(&"Fake".to_string()));
    }

    #[test]
    fn test_ocaml_quoted_string_literals() {
        // `{|...|}`-style quoted strings (and named-delimiter variants like
        // `{sql|...|sql}`) are common for embedding raw text such as SQL or regexes.
        // Content inside must not leak a fake export, but ordinary record syntax
        // (which also opens with `{`) must be unaffected.
        let src = r#"
let query = {sql|SELECT * FROM users WHERE let = 1|sql}

let raw = {|this text has "unbalanced quotes and a let binding = 1|}

let point = { x = 1; y = 2 }

let updated point = { point with x = 5 }
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["query", "raw", "point", "updated"]);
    }

    #[test]
    fn test_ocaml_attributes_between_keyword_and_name() {
        // Attributes can sit directly between `let` and the name with no space
        // (`let[@inline] ...`), or after the definition (`[@@deriving ...]`).
        let src = r#"
let[@inline] compute x = x + 1

let [@warning "-32"] unused_helper () = ()

type t = int [@@deriving show, eq]
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"compute".to_string()), "{symbols:?}");
        assert!(
            symbols.contains(&"unused_helper".to_string()),
            "{symbols:?}"
        );
        assert!(symbols.contains(&"t".to_string()), "{symbols:?}");
    }

    #[test]
    fn test_ocaml_operator_definitions_are_exported() {
        // REGRESSION: `OCAML_DECL`'s plain-`let` alternative requires a `\w`
        // (word) character right after `let`, so `let (>>=) m f = ...` never
        // matched at all -- custom infix operators (monadic bind, `<|>`,
        // pipeline combinators, ...) are extremely common in real OCaml and
        // were previously dropped from the export list entirely. Before the
        // fix this returned `["value"]`, silently missing ">>=".
        let src = r#"
let (>>=) m f = match m with
  | None -> None
  | Some x -> f x

let rec (>>>) f g x = g (f x)

let value = 42
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec![">>=", ">>>", "value"]);
    }

    #[test]
    fn test_ocaml_operator_unit_pattern_still_excluded() {
        // The operator-definition fix must not accidentally start matching
        // the conventional `let () = ...` entry-point idiom -- empty parens
        // hold zero operator-symbol characters, so it stays excluded.
        let src = r#"
let () =
  print_endline "starting"

let real_binding = 1
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["real_binding"]);
        assert!(!symbols.contains(&"()".to_string()));
    }

    #[test]
    fn test_ocaml_mutually_recursive_operator_and_value() {
        // `and` can continue a `let rec` group into an operator definition,
        // just as it can for plain names -- exercised separately from
        // `test_ocaml_mutually_recursive_let_and_type` since operators use a
        // dedicated matcher (`OCAML_OPERATOR_DECL`), not `OCAML_DECL`.
        let src = r#"
let rec is_zero n = n = 0

and (=~=) a b = is_zero (a - b)
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["is_zero", "=~="]);
    }

    #[test]
    fn test_ocaml_comment_containing_unmatched_quote_and_parens() {
        // A doc comment mentioning quoted code or example syntax (including
        // an unmatched `"` and a `(*`-shaped substring) must be stripped as
        // one unit without desyncing the string/comment scanner, regardless
        // of what punctuation it contains.
        let src = r#"
(* Don't use "let x = 1" here; see also the (* nested example *) above. *)
let real_export = 1
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["real_export"]);
    }

    #[test]
    fn test_ocaml_same_line_leading_comment_does_not_hide_export() {
        // A short doc/annotation comment placed before a one-line top-level
        // binding, on the *same* line, is a real style seen in small utility
        // modules and version/constant files. The comment sits at true column 0
        // (nothing indents it), so the `let` that follows it is still top-level
        // and public — it must not be dropped just because the comment ate the
        // literal column-0 position on that line.
        let src = r#"
(* Public API version string. *) let version = "1.0.0"

(** Maximum retry count before giving up. *) let max_retries = 5

(* deprecated, use [max_retries] instead *) let old_retry_limit = 3

let real_after_all_that = 42
"#;
        let symbols = extract_exports(src);
        assert_eq!(
            symbols,
            vec![
                "version",
                "max_retries",
                "old_retry_limit",
                "real_after_all_that",
            ],
            "{symbols:?}"
        );
    }
}
