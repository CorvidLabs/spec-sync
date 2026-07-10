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

/// A `let` member written inline after a one-liner module's own `=`, e.g.
/// `module Bar = let x = 1`, as opposed to the far more common shape where the `=`
/// ends the line and the member sits on its own, more-indented, line below (which the
/// scope-stack walk below already handles). Because the walk below dispatches each
/// *line* through a single if-else chain, a line matched by `MODULE_DECL` never also
/// gets a chance to match `LET_DECL` — without this, `x` would be silently dropped
/// even though it's a real public member of a public module. Anchored (no `(?m)`) to
/// the start of the remainder of the line *right after* the module match, with only
/// whitespace and the `=` permitted in between, so it can't misfire on an unrelated
/// local `let` buried later in some other kind of line (e.g. `let x = let y = 1 in y`,
/// where `y` is a genuinely local binding regardless of `x`'s own visibility).
static MODULE_INLINE_LET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*=\s*let\s+((?:(?:rec|private|internal|inline|mutable)\s+)*)([A-Za-z_][\w']*)")
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
            let module_exportable = exportable && !is_private;
            // A one-liner module body's inline member is exportable only if both the
            // enclosing module is exportable AND the member itself isn't separately
            // marked `private`/`internal` (e.g. `module Bar = let private x = 1`).
            if let Some(inline_caps) =
                MODULE_INLINE_LET.captures(&line[caps.get(0).unwrap().end()..])
            {
                let inline_is_private = inline_caps
                    .get(1)
                    .is_some_and(|m| has_private_modifier(m.as_str()));
                if module_exportable
                    && !inline_is_private
                    && let Some(name) = inline_caps.get(2)
                {
                    push_symbol(&mut symbols, name.as_str());
                }
            }
            // A private/internal module hides everything nested under it too.
            scope_stack.push((indent, module_exportable));
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

    #[test]
    fn test_fsharp_active_pattern_private_and_internal_variants_excluded() {
        // Like plain `let`/`type`, an active pattern's own case labels must respect an
        // explicit `private`/`internal` modifier on the defining `let`, even though the
        // exported names (the case labels) are extracted from inside the banana
        // brackets rather than being the `let`-bound identifier itself.
        let src = r#"
let private (|Even|Odd|) n =
    if n % 2 = 0 then Even else Odd

let (|Parseable|_|) (s: string) =
    match System.Int32.TryParse(s) with
    | true, v -> Some v
    | _ -> None

let internal (|Hidden|_|) s = Some s
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Parseable".to_string()]);
        assert!(!symbols.contains(&"Even".to_string()));
        assert!(!symbols.contains(&"Odd".to_string()));
        assert!(!symbols.contains(&"Hidden".to_string()));
    }

    #[test]
    fn test_fsharp_computation_expression_locals_not_exported() {
        // A computation-expression body (`maybe { ... }`) is, from this extractor's
        // point of view, just more-indented code inside whatever `let`/`type` opened
        // it — the CE's `{ }` braces aren't tracked at all (this backend is purely
        // indentation-driven), so `let`s inside the CE body fall out as local the same
        // way any other nested `let` would. Only the builder type, the builder
        // instance, and the two outer bindings that hold CE results are top-level.
        let src = r#"
type MaybeBuilder() =
    member this.Bind(x, f) = Option.bind f x
    member this.Return(x) = Some x

let maybe = MaybeBuilder()

let result =
    maybe {
        let x = Some 1
        let y = Some 2
        return x + y
    }

let inlineResult = maybe { let z = Some 3 in return z }
"#;
        let symbols = extract_exports(src);
        assert_eq!(
            symbols,
            vec![
                "MaybeBuilder".to_string(),
                "maybe".to_string(),
                "result".to_string(),
                "inlineResult".to_string(),
            ]
        );
        assert!(!symbols.contains(&"x".to_string()));
        assert!(!symbols.contains(&"y".to_string()));
        assert!(
            !symbols.contains(&"z".to_string()),
            "z is a same-line CE-local, not a sibling top-level let"
        );
    }

    #[test]
    fn test_fsharp_deeply_nested_private_module_hides_public_looking_members() {
        // A `let` that would look exported in isolation is still hidden when the
        // *module* it lives in is `private` — visibility of a nested scope is
        // determined by walking outward, not by the innermost declaration's own
        // (lack of a) modifier. `Outer` itself stays visible, and a sibling `let`
        // declared back at `Outer`'s own indentation after the private nested module
        // closes must still be exported (dedenting out of `Inner` must not also pop
        // `Outer`'s frame).
        let src = r#"
module Outer =
    module private Inner =
        let publicLookingButHidden = 1
        type AlsoHidden = { X: int }
    let visible = 2
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Outer".to_string(), "visible".to_string()]);
        assert!(!symbols.contains(&"Inner".to_string()));
        assert!(!symbols.contains(&"publicLookingButHidden".to_string()));
        assert!(!symbols.contains(&"AlsoHidden".to_string()));
    }

    #[test]
    fn test_fsharp_module_rec_and_autoopen_attribute() {
        // `[<AutoOpen>]` is just an attribute (already stripped by `ATTRIBUTE` before
        // keyword matching) and `module rec` is a recognized modifier prefix on
        // `MODULE_DECL` — neither should prevent the module name or its members from
        // being treated as an ordinary exportable module.
        let src = r#"
[<AutoOpen>]
module Extensions =
    let extend x = x

module rec Recursive =
    let a x = if x = 0 then 0 else b (x - 1)
    let b x = a x
"#;
        let symbols = extract_exports(src);
        assert_eq!(
            symbols,
            vec![
                "Extensions".to_string(),
                "extend".to_string(),
                "Recursive".to_string(),
                "a".to_string(),
                "b".to_string(),
            ]
        );
    }

    #[test]
    fn test_fsharp_multiline_signature_wrapping_before_equals() {
        // A signature that wraps its parameter list (and return-type annotation) onto
        // several lines before the `=` must still be recognized by the name captured
        // on the declaration's *first* line — the continuation lines are just more
        // deeply indented and never match any declaration keyword themselves, so they
        // fall out as harmless non-matching lines rather than confusing the scope
        // tracker into treating them as siblings.
        let src = r#"
let processData
    (items: seq<int>)
    (predicate: int -> bool)
    : seq<int> =
    items |> Seq.filter predicate

let private hiddenMultiline
    (x: int)
    : int =
    x

let bar y = y
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["processData".to_string(), "bar".to_string()]);
        assert!(!symbols.contains(&"hiddenMultiline".to_string()));
    }

    #[test]
    fn test_fsharp_oneliner_module_inline_public_member_exported() {
        // Regression test for a real bug: `module Bar = let x = 1` puts the member on
        // the *same* line as the module's own `=`, rather than on a separate,
        // more-indented line below (the far more common shape). Because each line is
        // dispatched through a single if-else chain of regexes, `MODULE_DECL` used to
        // claim the whole line and the inline `let x = 1` was silently dropped
        // entirely — even though `x` is exactly as public as `Bar` itself. This is now
        // handled by `MODULE_INLINE_LET` matching the remainder of the line right
        // after the module's `=`.
        let src = r#"
module Bar = let x = 1

let after = 2
"#;
        let symbols = extract_exports(src);
        assert_eq!(
            symbols,
            vec!["Bar".to_string(), "x".to_string(), "after".to_string()]
        );
    }

    #[test]
    fn test_fsharp_oneliner_module_private_hides_module_and_inline_member() {
        // The mirror image of the previous test: when the *module* itself is
        // `private`, the inline member must be hidden too (a public-looking `let`
        // cannot escape a private enclosing module just by being written inline).
        let src = r#"
module private Bar = let x = 1

let after = 2
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["after".to_string()]);
        assert!(!symbols.contains(&"Bar".to_string()));
        assert!(!symbols.contains(&"x".to_string()));
    }

    #[test]
    fn test_fsharp_oneliner_module_public_but_inline_member_itself_private() {
        // The inline member's own modifier is independent of the module's: even when
        // `Bar` is public, `module Bar = let private x = 1` must still hide `x`.
        let src = r#"
module Bar = let private x = 1

let after = 2
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Bar".to_string(), "after".to_string()]);
        assert!(!symbols.contains(&"x".to_string()));
    }

    #[test]
    fn test_fsharp_dedented_looking_string_inside_nested_module_scope() {
        // The triple-quote-stripping tests above cover a top-level string; this
        // exercises the same hazard one level deeper, where a bug in indentation
        // tracking could plausibly manifest differently (e.g. by popping the
        // enclosing module's scope frame instead of just failing to find fake
        // exports). The embedded fake `type`/`let` lines start at column 0 — more
        // dedented than even the enclosing `module Outer =` — and must not be mistaken
        // for a dedent back to real top-level content, nor confuse recognition of the
        // real `let realNested` sibling that follows.
        let src = r#"
module Outer =
    let template =
        """
type FakeNestedType = { X: int }
let fakeNestedLet x = x
"""
    let realNested = 3
"#;
        let symbols = extract_exports(src);
        assert_eq!(
            symbols,
            vec![
                "Outer".to_string(),
                "template".to_string(),
                "realNested".to_string()
            ]
        );
        assert!(!symbols.contains(&"FakeNestedType".to_string()));
        assert!(!symbols.contains(&"fakeNestedLet".to_string()));
    }
}
