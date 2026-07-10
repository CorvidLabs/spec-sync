use regex::Regex;
use std::sync::LazyLock;

// `(?m)` is required so `$` anchors to each line's end rather than only the
// end of the whole (potentially multi-line) source string; without it `.*`
// (which never crosses `\n`) can never reach `$` on any line but the last,
// so comments are never stripped from realistic multi-line files.
static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)#.*$").unwrap());

/// Nim exported symbols: name followed by *
/// Follow-set includes `[` (generics: `name*[T]`) and `,` (comma-separated
/// multi-identifier declarations: `var a*, b*: T`) in addition to the
/// characters/whitespace/end-of-input that can legally follow an exported
/// identifier.
static NIM_EXPORT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(\w+)\*(?:[=:({\[,\s]|$)").unwrap());

/// Extract public symbols from Nim source code.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = COMMENT_SINGLE.replace_all(content, "");

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
