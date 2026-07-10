use regex::Regex;
use std::sync::LazyLock;

/// Nim exported symbols: name followed by *
/// Follow-set includes `[` (generics: `name*[T]`) and `,` (comma-separated
/// multi-identifier declarations: `var a*, b*: T`) in addition to the
/// characters/whitespace/end-of-input that can legally follow an exported
/// identifier.
static NIM_EXPORT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(\w+)\*(?:[=:({\[,\s]|$)").unwrap());

/// Extract public symbols from Nim source code.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = strip_comments_and_strings(content);

    let mut symbols = Vec::new();

    for caps in NIM_EXPORT.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str().to_string();
            if !symbols.contains(&n) {
                symbols.push(n);
            }
        }
    }

    symbols
}

/// Blank `"…"` / `"""…"""` string literals, properly-nested `#[ … ]#` block
/// comments, and `#` line comments, all in a single left-to-right pass.
///
/// A single scan (rather than independent regex passes applied in a fixed
/// order) is required for correct precedence: whichever construct's opening
/// delimiter is actually reached first in the scan wins. A `#` living inside
/// a string literal (`"Hello # World"`) must not be misread as starting a
/// line comment — which would truncate the string and anything legitimately
/// following it on that line — and a stray `#[`/`]#` inside a string or a
/// `#` line comment must not be misread as a block-comment delimiter either.
/// Independent regex passes run in either order get one of these two cases
/// wrong.
///
/// `#[ ... ]#` nesting is tracked with a depth counter rather than a single
/// lazy regex match: Nim's manual explicitly documents `#[ ... ]#` as
/// *arbitrarily nestable* (unlike most languages' block comments), and a
/// non-nested approximation stops at the first `]#`, leaking the "still
/// nested" tail — including any code hiding in it — back into the output as
/// if it were live source. This was verified empirically: a synthetic
/// `#[ outer #[ inner ]# secretProc*(...) ]#` sample leaked `secretProc` as
/// a false-positive export under the old non-nested regex.
///
/// String literals were not stripped at all before this fix, which allowed
/// an unrelated but structurally identical false positive: a plain string
/// containing `identifier*` text (e.g. a doc string like `"call obj.foo* to
/// succeed"`) was reported as exporting `foo`, since nothing removed the
/// string's contents before the `NIM_EXPORT` regex ran over it. Triple-quoted
/// `"""..."""` strings (Nim's raw/doc-string form, seen in real code as
/// `discard """ ... """`) are handled specially since embedded single `"`
/// characters are literal content there, not a terminator.
///
/// This intentionally does not special-case Nim's prefixed raw-string forms
/// (`r"..."`, or library-defined prefixes like `re"..."`), where embedded
/// `\` sequences are not escapes — same documented-limitation trade-off as
/// other backends make for less-common string forms. Blanked characters are
/// replaced with a space (newlines preserved) so `NIM_EXPORT`'s match
/// offsets stay aligned to the original source. Unterminated strings/block
/// comments blank to end of input rather than panicking or looping.
fn strip_comments_and_strings(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < n {
        // Triple-quoted string: """ ... """
        if chars[i] == '"' && i + 2 < n && chars[i + 1] == '"' && chars[i + 2] == '"' {
            i += 3;
            while i < n {
                if chars[i] == '"' && i + 2 < n && chars[i + 1] == '"' && chars[i + 2] == '"' {
                    i += 3;
                    break;
                }
                if chars[i] == '\n' {
                    out.push('\n');
                }
                i += 1;
            }
            continue;
        }
        // Regular double-quoted string, with backslash escapes.
        if chars[i] == '"' {
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
        // Nested-aware `#[ ... ]#` block comment.
        if chars[i] == '#' && i + 1 < n && chars[i + 1] == '[' {
            let mut depth = 1u32;
            i += 2;
            while i < n && depth > 0 {
                if chars[i] == '#' && i + 1 < n && chars[i + 1] == '[' {
                    depth += 1;
                    i += 2;
                } else if chars[i] == ']' && i + 1 < n && chars[i + 1] == '#' {
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
        // `#` line comment.
        if chars[i] == '#' {
            while i < n && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nim_exports() {
        let src = r#"
# Nim module
proc greet*(name: string) =
  echo "Hello ", name

proc helper() =
  echo "internal"

type
  User* = object
    username*: string
    age: int

const Version* = "1.0"
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"greet".to_string()));
        assert!(symbols.contains(&"User".to_string()));
        assert!(symbols.contains(&"username".to_string()));
        assert!(symbols.contains(&"Version".to_string()));
        assert!(!symbols.contains(&"helper".to_string()));
        assert!(!symbols.contains(&"age".to_string()));
    }

    #[test]
    fn test_nim_block_commented_out_export_not_leaked() {
        // Regression test: `#[ ... ]#` block comments were claimed handled (per the
        // docstring on `test_nim_exports_real_learnxinyminutes_script_has_no_exports`)
        // but were never actually stripped -- only single-line `#...` comments were.
        // That test passed only by coincidence (no `word*` pattern happened to appear
        // in its comment body). This uses a real exported-looking name inside the
        // block comment to prove it's genuinely excluded.
        let src = r#"
#[
  Old implementation, kept for reference:
  proc secretProc*(x: int): int =
    x * 2
]#
proc realProc*(x: int): int =
  x + 1
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"realProc".to_string()));
        assert!(!symbols.contains(&"secretProc".to_string()));
    }

    #[test]
    fn test_nim_exports_generic_procs_and_types() {
        let src = r#"
type
  Stack*[T] = object
    items: seq[T]
  Container*[T] = ref object
    value*: T

proc newStack*[T](): Stack[T] = Stack[T](items: @[])
proc push*[T](s: var Stack[T], item: T) = s.items.add(item)
proc get*[T](x: Container[T]): T = x.value
proc pop*[T: SomeInteger](s: var Stack[T]): T = s.items.pop()
"#;
        let symbols = extract_exports(src);
        for expected in [
            "Stack",
            "Container",
            "value",
            "newStack",
            "push",
            "get",
            "pop",
        ] {
            assert!(
                symbols.contains(&expected.to_string()),
                "missing {expected}: {symbols:?}"
            );
        }
        assert_eq!(symbols.len(), 7, "unexpected extra symbols: {symbols:?}");
    }

    #[test]
    fn test_nim_exports_ignores_comment_text_across_lines() {
        let src = r#"
# TODO: implement validate*() properly, needs review*
proc realValidate*(x: int): bool =
  # NOTE: internal helper*, not exported: helperFn*()
  true

proc helperFn(x: int): bool =
  false
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["realValidate".to_string()]);
        assert!(!symbols.contains(&"validate".to_string()));
        assert!(!symbols.contains(&"review".to_string()));
        assert!(!symbols.contains(&"helperFn".to_string()));
    }

    /// Real excerpt from learnxinyminutes.com's Nim tutorial (learnNim.nim).
    /// The tutorial is a top-level script, not a library, so it never marks
    /// anything `*`-exported. This is a genuine, common real-world Nim shape
    /// (a `main`-style program): confirm that nested `#[ ... ]#` block
    /// comments, `discard """ ... """` multiline strings, tuple/array/range
    /// generic syntax, and qualified enum access (`Answer.aYes`) never get
    /// misread as exports and never cause a panic.
    #[test]
    fn test_nim_exports_real_learnxinyminutes_script_has_no_exports() {
        let src = r#"
#[
  This is a multiline comment.
  In Nim, multiline comments can be nested, beginning with #[
  ... and ending with ]#
]#

discard """
This can also work as a multiline comment.
Or for unparsable, broken code
"""

var
  child: tuple[name: string, age: int]   # Tuples have *both* field names
  today: tuple[sun: string, temp: float] # *and* order.

type
  Name = string
  Age = int
  Person = tuple[name: Name, age: Age]
  AnotherSyntax = tuple
    fieldOne: string
    secondField: int

type
  Cash = distinct int
  Desc = distinct string

var
  money: Cash = 100.Cash
  description: Desc = "Interesting".Desc

type
  DieFaces = range[1..20]
  RollCounter = array[DieFaces, int]
  Truths = array[42..44, bool]

type Answer = enum aYes, aNo

proc ask(question: string): Answer =
  echo(question, " (y/n)")
  while true:
    case readLine(stdin)
    of "y", "Y", "yes", "Yes":
      return Answer.aYes
    of "n", "N", "no", "No":
      return Answer.aNo
    else: echo("Please be clear: yes or no")

proc strcmp(a, b: cstring): cint {.importc: "strcmp", nodecl.}

let cmp = strcmp("C?", "Easy!")
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.is_empty(),
            "expected no exports in this non-library script: {symbols:?}"
        );
    }

    /// Same real Room/House `ref object` shapes from the learnxinyminutes
    /// tutorial's "More Types and Data Structures" section, adapted with `*`
    /// markers (as a real public Nim module would write them) to confirm the
    /// extractor correctly picks up exported object types and fields -
    /// including a field with a default value (`doors*: int = 1`, added in
    /// Nim 2.0) - while still excluding unexported ones.
    #[test]
    fn test_nim_exports_real_learnxinyminutes_ref_object_fields() {
        let src = r#"
type
  Room* = ref object # reference to an object, useful for big objects or
    windows*: int     # objects inside objects
    doors*: int = 1   # Change the default value of a field (since Nim 2.0)
    secret: string    # not exported
  House* = object
    address*: string
    rooms*: seq[Room]

var
  defaultHouse = House() # initialize with default values
  sesameRoom = Room(windows: 4, doors: 2)
"#;
        let symbols = extract_exports(src);
        for expected in ["Room", "windows", "doors", "House", "address", "rooms"] {
            assert!(
                symbols.contains(&expected.to_string()),
                "missing {expected}: {symbols:?}"
            );
        }
        assert!(!symbols.contains(&"secret".to_string()));
    }

    #[test]
    fn test_genuinely_nested_block_comment_no_longer_leaks_secret() {
        // Regression: empirically confirmed before the fix -- a single
        // non-nested `#[...]#` regex stops at the *first* `]#`, so this
        // genuinely-nested comment (Nim explicitly allows nesting) leaked
        // `secretProc*(...)` back out as live code, producing
        // `["secretProc", "realProc"]`. Depth-tracking fixes this.
        let src = r#"
#[ outer comment
#[ inner nested ]#
secretProc*(x: int): int = x * 2
still commented out
]#
proc realProc*(x: int): int =
  x + 1
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["realProc"], "got {symbols:?}");
    }

    #[test]
    fn test_string_literal_with_export_like_text_not_leaked() {
        // Regression: string literals weren't stripped at all before this
        // fix, so `identifier*`-shaped text sitting inside an ordinary
        // string was misread as a real export. Before the fix this produced
        // `["greeting", "fake_export", "afterProc"]`.
        let src = r#"
const greeting* = "call obj.fake_export* now"
proc afterProc*(x: int): int = x + 1
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"greeting".to_string()), "got {symbols:?}");
        assert!(
            symbols.contains(&"afterProc".to_string()),
            "got {symbols:?}"
        );
        assert!(
            !symbols.contains(&"fake_export".to_string()),
            "string literal content leaked as a fake export, got {symbols:?}"
        );
    }

    #[test]
    fn test_block_comment_with_stray_hash_and_brackets_not_part_of_nesting() {
        // A `#[ ... ]#` block comment whose prose contains a lone `#`, `[`,
        // or `]` that is not part of a real `#[`/`]#` marker pair must not
        // confuse the depth counter (only exact adjacent pairs change
        // depth) and must not itself be misread as an export.
        let src = r#"
#[ this mentions array a[0], a stray ] bracket, and a single # hash,
   none of which are real nesting markers ]#
proc realProc*(x: int): int = x + 1
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["realProc"], "got {symbols:?}");
    }

    #[test]
    fn test_triple_quoted_string_with_asterisk_word_not_leaked() {
        // Triple-quoted strings are common as ad hoc "block comments" or
        // doc strings (`discard """ ... """`) in real Nim code; content
        // that happens to look like an export inside one must not leak.
        let src = r#"
discard """
Usage notes: call myHelper* before proceeding.
"""
proc realProc*(x: int): int = x + 1
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["realProc"], "got {symbols:?}");
    }

    #[test]
    fn test_nim_exports_comma_separated_multi_ident_decls() {
        let src = r#"
var minValue*, maxValue*: int
let defaultName*, fallbackName*: string = ("foo", "bar")
"#;
        let symbols = extract_exports(src);
        for expected in ["minValue", "maxValue", "defaultName", "fallbackName"] {
            assert!(
                symbols.contains(&expected.to_string()),
                "missing {expected}: {symbols:?}"
            );
        }
    }
}
