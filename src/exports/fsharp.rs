use regex::Regex;
use std::sync::LazyLock;

static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"//.*$").unwrap());

/// Triple-quoted raw strings: `"""...."""`. Unlike a regular `"..."` string (which can't
/// span multiple lines in ordinary use and so can't be mistaken for a sibling top-level
/// declaration), these routinely span many lines — F# code commonly uses them to embed
/// SQL, JSON, GraphQL, or example/generated source. If such a blob contains a line that
/// starts at column 0 with `let`/`type`/`module` (extremely plausible for an embedded
/// code sample), the indentation-based scope tracker below would otherwise mistake it for
/// a real dedent back to top level and extract a phantom export. Stripped before
/// `COMMENT_MULTI`/`COMMENT_SINGLE` so a `(*`/`//`-like substring inside one of these
/// strings can't be misread as starting a comment either.
static TRIPLE_QUOTE_STRING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("\"\"\"[\\s\\S]*?\"\"\"").unwrap());

/// Verbatim strings: `@"...."`, where a doubled `""` is an escaped quote. Like triple-quoted
/// strings, these can span multiple lines (verbatim strings are F#'s other common home for
/// embedded SQL/regex/templates) and are stripped for the same reason.
static VERBATIM_STRING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("@\"(?:\"\"|[^\"])*\"").unwrap());

/// F# block comments: `(* ... *)`. F# technically allows these to nest, but — matching
/// every other regex-based backend's simplified (non-nesting) treatment of its own
/// block-comment syntax in this codebase — we strip the first non-greedy `(* ... *)`
/// span. Stripped before `COMMENT_SINGLE` (mirroring ruby.rs's ordering) so a `//`
/// occurring inside a block comment (e.g. `(* see // notes *)`) can't be mistaken for
/// a line comment and eat the block comment's own closing `*)`.
static COMMENT_MULTI: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)\(\*.*?\*\)").unwrap());

/// Inline attribute annotations, e.g. `[<Struct>]`, `[<CustomEquality; NoComparison>]`.
/// Stripped (per-line, see below) so a same-line `[<Attr>] type Foo = ...` still lines
/// up with the `^type` (etc.) anchors used by the declaration regexes below.
static ATTRIBUTE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[<[^>]*>\]\s*").unwrap());

/// `let` / `and` bindings — functions, values, and `let rec ... and ...` mutually
/// recursive groups (an `and` can also continue a mutually recursive `type` group;
/// either way the continuation is captured the same way). Modifiers (`rec`,
/// `private`, `internal`, `inline`, `mutable`) can appear in any order in real code,
/// so they're captured as one blob (group 1) and inspected in Rust rather than
/// pinning down a single canonical ordering in the regex itself.
static LET_DECL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*(?:let|and)\s+((?:(?:rec|private|internal|inline|mutable)\s+)*)([A-Za-z_][\w']*)",
    )
    .unwrap()
});

/// Active pattern definitions: `let (|Even|Odd|) x = ...` (exhaustive) or
/// `let (|Prefix|_|) s = ...` (partial). What a consuming spec actually pattern-matches
/// against is the individual case label(s) inside the banana brackets, not the literal
/// `(|Even|Odd|)` token, so those are extracted (the `_` wildcard case of a partial
/// active pattern is not a real name and is dropped).
static ACTIVE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^[^\S\n]*let\s+((?:(?:private|internal)\s+)*)\(\|([^)]+)\|\)").unwrap()
});

/// `type` declarations: records, unions, classes, interfaces, type aliases, measures...
static TYPE_DECL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^[^\S\n]*type\s+((?:(?:private|internal)\s+)*)([A-Za-z_][\w']*)").unwrap()
});

/// `module` declarations, including recursive (`module rec Foo = `) and dotted
/// lightweight top-level module/namespace-style names (`module MyApp.Utils`, no `=`).
static MODULE_DECL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^[^\S\n]*module\s+((?:(?:rec|private|internal)\s+)*)([A-Za-z_][\w'.]*)")
        .unwrap()
});

/// True if the captured modifier blob contains an explicit `private`/`internal`.
fn has_private_modifier(modifiers: &str) -> bool {
    modifiers
        .split_whitespace()
        .any(|w| w == "private" || w == "internal")
}

fn push_symbol(symbols: &mut Vec<String>, name: &str) {
    let n = name.to_string();
    if !symbols.contains(&n) {
        symbols.push(n);
    }
}

/// Extract exported symbols from F# source code.
///
/// F# `let`, `type`, and `module` declarations at the top level of a file (or the top
/// level of a `module Foo = ` block) are PUBLIC by default — the opposite direction of
/// C-family languages: `private` is an explicit keyword that hides one declaration
/// (`internal` is treated the same way here, since it's not visible outside the
/// assembly and so isn't part of the public surface a consuming spec/module can see).
///
/// F# has no braces — nesting is purely indentation-driven — so instead of the
/// brace-depth tracking used by the C-family backends, this walks lines tracking an
/// indentation-based scope stack: a `module Foo = ` block keeps propagating
/// exportability to its (more-indented) members, while a `let`/`and` function body or
/// a `type` body does not — nested `let`s inside either are always local (a class's
/// `let`-bound fields are private implementation details in F#; only its `member`s are
/// public API surface, and this backend does not attempt to extract members).
pub fn extract_exports(content: &str) -> Vec<String> {
    // Multi-line string literals are stripped first, before comments: their contents can
    // legitimately contain `(*`/`//`-like text, and — more importantly — can contain
    // lines that start at column 0 looking exactly like top-level declarations. Same
    // newline-preserving rationale as the comment stripping below: deleting the embedded
    // newlines would corrupt the indentation of whatever code follows the string.
    let stripped = TRIPLE_QUOTE_STRING.replace_all(content, |caps: &regex::Captures| {
        "\n".repeat(caps[0].matches('\n').count())
    });
    let stripped = VERBATIM_STRING.replace_all(&stripped, |caps: &regex::Captures| {
        "\n".repeat(caps[0].matches('\n').count())
    });

    // Multi-line comments are stripped next (matching ruby.rs's ordering) and with a
    // newline-preserving replacement: unlike the brace-tracking backends, this
    // extractor's scope resolution depends on each line's own indentation, so merging
    // lines together by deleting the newlines inside a stripped comment would corrupt
    // the indentation of whatever followed it.
    let stripped = COMMENT_MULTI.replace_all(&stripped, |caps: &regex::Captures| {
        "\n".repeat(caps[0].matches('\n').count())
    });
    let stripped = COMMENT_SINGLE.replace_all(&stripped, "");

    let mut symbols: Vec<String> = Vec::new();
    // Stack of (indentation of the line that opened this scope, is this scope's
    // content exportable). Empty stack == true top level (exportable).
    let mut scope_stack: Vec<(usize, bool)> = Vec::new();

    for raw_line in stripped.lines() {
        if raw_line.trim().is_empty() {
            continue;
        }
        // Indentation is measured on the line as written, BEFORE stripping any inline
        // `[<Attr>]` prefix — a same-line attribute is still at the declaration's real
        // column, so stripping it first (and only then measuring indentation) would
        // shift `[<Measure>] type kg` to look more nested than it actually is.
        let indent = raw_line.len() - raw_line.trim_start().len();
        // Attribute-stripped copy used only for keyword matching below.
        let line = &ATTRIBUTE.replace_all(raw_line, "")[..];

        // A line at or before the indentation of the scope that opened means we've
        // dedented out of it (a sibling declaration, or a return to an outer scope).
        while let Some(&(frame_indent, _)) = scope_stack.last() {
            if frame_indent >= indent {
                scope_stack.pop();
            } else {
                break;
            }
        }
        let exportable = scope_stack.last().map(|&(_, e)| e).unwrap_or(true);

        if let Some(caps) = ACTIVE_PATTERN.captures(line) {
            let is_private = caps
                .get(1)
                .is_some_and(|m| has_private_modifier(m.as_str()));
            if exportable
                && !is_private
                && let Some(cases) = caps.get(2)
            {
                for case in cases.as_str().split('|') {
                    let case = case.trim();
                    if !case.is_empty() && case != "_" {
                        push_symbol(&mut symbols, case);
                    }
                }
            }
            scope_stack.push((indent, false));
        } else if let Some(caps) = MODULE_DECL.captures(line) {
            let is_private = caps
                .get(1)
                .is_some_and(|m| has_private_modifier(m.as_str()));
            if exportable
                && !is_private
                && let Some(name) = caps.get(2)
            {
                push_symbol(&mut symbols, name.as_str());
            }
            // A private/internal module hides everything nested under it too.
            scope_stack.push((indent, exportable && !is_private));
        } else if let Some(caps) = TYPE_DECL.captures(line) {
            let is_private = caps
                .get(1)
                .is_some_and(|m| has_private_modifier(m.as_str()));
            if exportable
                && !is_private
                && let Some(name) = caps.get(2)
            {
                push_symbol(&mut symbols, name.as_str());
            }
            scope_stack.push((indent, false));
        } else if let Some(caps) = LET_DECL.captures(line) {
            let is_private = caps
                .get(1)
                .is_some_and(|m| has_private_modifier(m.as_str()));
            if exportable
                && !is_private
                && let Some(name) = caps.get(2)
            {
                push_symbol(&mut symbols, name.as_str());
            }
            scope_stack.push((indent, false));
        }
    }

    symbols
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fsharp_top_level_exports() {
        let src = r#"
module MyApp.Auth

let createToken user = user

type Credentials = { Username: string; Password: string }

let private hashPassword pw = pw

module Utils =
    let normalize s = s
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"createToken".to_string()));
        assert!(symbols.contains(&"Credentials".to_string()));
        assert!(symbols.contains(&"Utils".to_string()));
        assert!(symbols.contains(&"normalize".to_string()));
        assert!(
            !symbols.contains(&"hashPassword".to_string()),
            "explicit private let must be excluded"
        );
    }

    #[test]
    fn test_fsharp_private_and_internal_excluded() {
        let src = r#"
let publicValue = 1
let private hiddenValue = 2
let internal assemblyOnlyValue = 3
type PublicType = { Id: int }
type private HiddenType = { Secret: string }
type internal InternalType = { Data: int }
module private HiddenModule =
    let x = 1
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"publicValue".to_string()));
        assert!(symbols.contains(&"PublicType".to_string()));
        assert!(!symbols.contains(&"hiddenValue".to_string()));
        assert!(!symbols.contains(&"assemblyOnlyValue".to_string()));
        assert!(!symbols.contains(&"HiddenType".to_string()));
        assert!(!symbols.contains(&"InternalType".to_string()));
        assert!(!symbols.contains(&"HiddenModule".to_string()));
    }

    #[test]
    fn test_fsharp_comments_stripped() {
        let src = r#"
// let fakeFromLineComment x = x
(* let fakeFromBlockComment x = x
   type FakeType = { X: int } *)
/// Adds two numbers together.
let add a b = a + b
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["add".to_string()]);
    }

    #[test]
    fn test_fsharp_locals_inside_function_not_exported() {
        // `let` bindings nested inside a function body are local variables, not a
        // second layer of top-level public API, even though F# uses the very same
        // `let` keyword for both.
        let src = r#"
let computeTotal items =
    let subtotal = List.sum items
    let tax = subtotal * 0.08
    let total = subtotal + tax
    total

let private helper x =
    let secretLocal = x * 2
    secretLocal
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["computeTotal".to_string()]);
    }

    #[test]
    fn test_fsharp_nested_module_members_exported() {
        // Members nested under a public `module Foo = ` block are still part of the
        // public surface — only the indentation changes, there's no `pub` keyword to
        // find (the crux of "public by default unless marked private").
        let src = r#"
module Payments =
    let charge amount = amount

    module Internal =
        let rawCharge amount = amount

    let refund amount = -amount
"#;
        let symbols = extract_exports(src);
        assert_eq!(
            symbols,
            vec![
                "Payments".to_string(),
                "charge".to_string(),
                "Internal".to_string(),
                "rawCharge".to_string(),
                "refund".to_string()
            ]
        );
    }

    #[test]
    fn test_fsharp_let_rec_and_mutually_recursive() {
        let src = r#"
let rec isEven n =
    if n = 0 then true else isOdd (n - 1)
and isOdd n =
    if n = 0 then false else isEven (n - 1)

let rec private isEvenHidden n =
    if n = 0 then true else isOddHidden (n - 1)
and isOddHidden n =
    if n = 0 then false else isEvenHidden (n - 1)
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"isEven".to_string()));
        assert!(symbols.contains(&"isOdd".to_string()));
        // Per the task's guidance, an `and`-continuation is treated as public by
        // default even though it continues a `let rec private` group.
        assert!(!symbols.contains(&"isEvenHidden".to_string()));
        assert!(symbols.contains(&"isOddHidden".to_string()));
    }

    #[test]
    fn test_fsharp_active_patterns() {
        // Active patterns are a distinctive F# idiom: the exported names a consuming
        // spec matches against are the case labels, not the `(|...|)` token itself.
        let src = r#"
let (|Even|Odd|) n =
    if n % 2 = 0 then Even else Odd

let (|Prefix|_|) (prefix: string) (s: string) =
    if s.StartsWith(prefix) then Some(s.Substring(prefix.Length)) else None
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Even".to_string()));
        assert!(symbols.contains(&"Odd".to_string()));
        assert!(symbols.contains(&"Prefix".to_string()));
        assert!(
            !symbols.contains(&"_".to_string()),
            "partial active pattern wildcard is not a real name"
        );
    }

    #[test]
    fn test_fsharp_discriminated_union_and_record_fields_not_leaked() {
        let src = r#"
type Shape =
    | Circle of float
    | Square of float
    | Rectangle of width: float * height: float

type Point =
    { X: float
      Y: float }
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Shape".to_string(), "Point".to_string()]);
    }

    #[test]
    fn test_fsharp_class_with_private_let_fields_and_members() {
        // `let`-bound fields inside a class-style type definition are private
        // implementation details in F# (only `member`s would be public, and this
        // backend intentionally does not extract members), so they must not leak.
        let src = r#"
type Counter(initial: int) =
    let mutable count = initial
    member this.Value = count
    member this.Increment() = count <- count + 1
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Counter".to_string()]);
    }

    #[test]
    fn test_fsharp_verbatim_string_contents_not_exported() {
        // A verbatim string (common for embedding SQL) that happens to contain a line
        // starting at column 0 with `let` must not be mistaken for a real top-level
        // sibling declaration by the indentation-based scope tracker.
        let src = r#"
let sqlTemplate = @"
SELECT * FROM users WHERE id = @id
let notReal = 1
"

let realExport = 42
"#;
        let symbols = extract_exports(src);
        assert_eq!(
            symbols,
            vec!["sqlTemplate".to_string(), "realExport".to_string()]
        );
    }

    #[test]
    fn test_fsharp_triple_quote_string_contents_not_exported() {
        // Triple-quoted strings are F#'s other common home for multi-line embedded
        // content (e.g. an example/generated code snippet in a doc-generation tool).
        // Fake-looking `type`/`module`/`let` lines inside one must not leak as exports.
        let src = r#"
let codeSample = """
type FakeType = { X: int }
module FakeModule =
    let fakeFn x = x
"""

let realExport2 = 1
"#;
        let symbols = extract_exports(src);
        assert_eq!(
            symbols,
            vec!["codeSample".to_string(), "realExport2".to_string()]
        );
    }

    #[test]
    fn test_fsharp_learnxinyminutes_real_source() {
        let src = std::fs::read_to_string(
            "/private/tmp/claude-501/-Users-leif-Development--CorvidLabs-spec-sync/1429498c-236f-41e9-839d-cd71a8ca63b8/scratchpad/fsharp_real.fs",
        )
        .unwrap();
        let symbols = extract_exports(&src);
        println!("{symbols:#?}");
    }

    #[test]
    fn test_fsharp_attribute_same_line_as_type() {
        let src = r#"
[<Struct>]
type Vector2 = { X: float; Y: float }

[<Measure>] type kg
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Vector2".to_string()));
        assert!(symbols.contains(&"kg".to_string()));
    }
}
