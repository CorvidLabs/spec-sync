use regex::Regex;
use std::sync::LazyLock;

/// Triple-quoted string literals (`"""..."""`, including interpolated
/// `s"""..."""`/`raw"""..."""` variants). Scala triple-quoted strings can't
/// nest, so a lazy, non-greedy match from the first `"""` to the next one is
/// always correct (unlike block comments below). Without this, arbitrary
/// text inside a multi-line string literal — including text that happens to
/// look like `class Foo` or `private class Foo` — would be scanned as if it
/// were real source and could leak fake symbols into the export list.
static STRING_TRIPLE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("(?s)\"\"\".*?\"\"\"").unwrap());

/// Scala public declarations: class, object, trait, enum, def, val, var, type.
///
/// The name capture accepts either a normal `\w+` identifier or a run of
/// Scala operator characters, so symbolic/operator method names (`def
/// +(other: T): T`) are captured too — not just alphanumeric ones. When the
/// name starts with a word character the `\w+` alternative always wins
/// (regex alternation is leftmost-first), so this doesn't change behavior
/// for ordinary identifiers; the operator alternative only ever fires for
/// names that are symbolic from their very first character.
///
/// The modifier list includes `package` so `package object Foo { ... }`
/// (Scala's package-scoped singleton, itself a public top-level
/// declaration) is recognized — without it, the leading `package` token
/// before `object` made the whole line fail to match since only
/// class/object/trait/def/val/var/type keywords were treated as the head of
/// a declaration.
static SCALA_DECL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*(?:(?:implicit|lazy|case|override|sealed|abstract|final|package)\s+)*(?:class|object|trait|enum|def|val|var|type)\s+(\w+|[!#%&*+\-/:<=>?@^|~]+)",
    )
    .unwrap()
});

/// Scala 3 bare enum cases, e.g. `case Red, Green, Blue` or
/// `case Success(value: A)`. These have no leading
/// class/object/trait/def/val/var/type keyword for `SCALA_DECL` to match, so
/// they need a dedicated pattern. Callers only try this after `SCALA_DECL`
/// has already failed to match the line, so `case class`/`case object`
/// lines (handled by `SCALA_DECL`'s `case`-modifier prefix) never reach
/// here; callers should also skip any line containing `=>` before testing
/// this, so a pattern-match arm's scrutinee isn't mistaken for a case
/// declaration.
static SCALA_ENUM_CASE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^[^\S\n]*case\s+([A-Za-z_]\w*(?:\s*,\s*[A-Za-z_]\w*)*)\s*(?:[(:]|$)").unwrap()
});

/// Exclude private/protected declarations
static SCALA_PRIVATE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[^\S\n]*(?:private|protected)").unwrap());

/// Determine whether the characters starting right after an opening `'`
/// form a valid Scala char literal body, and if so, how many characters
/// (not counting the closing `'`) belong to it.
///
/// This exists so [`strip_comments`] can tell a real char literal (`'"'`,
/// `'\''`, `'\\'`) apart from a bare/symbol-literal apostrophe (`'foo`,
/// Scala 2's deprecated `Symbol` literal syntax, or a stray `'` used as a
/// type-variance-style marker) which has no closing quote at all. Without
/// this check, treating every `'` as opening a char literal would make the
/// scanner swallow arbitrary following text — up to the next unrelated `'`
/// it happens to find, or the end of the line — as if it were literal
/// content.
///
/// Handles a single ordinary character (`'x'`) and backslash escapes
/// (`'\\'`, `'\''`, `'\n'`, `'A'`), bounded to a small lookahead window
/// so a non-char-literal apostrophe can't stall the scan.
fn char_literal_body_len(mut lookahead: impl Iterator<Item = char>) -> Option<usize> {
    match lookahead.next()? {
        '\\' => {
            // Escape sequence: search a bounded window for the closing `'`.
            let mut consumed = 1usize; // the backslash itself
            for _ in 0..8 {
                match lookahead.next() {
                    Some('\'') => return Some(consumed),
                    Some('\n') | None => return None,
                    Some(_) => consumed += 1,
                }
            }
            None
        }
        '\'' | '\n' => None,
        _ => {
            if lookahead.next() == Some('\'') {
                Some(1)
            } else {
                None
            }
        }
    }
}

/// Strip Scala `//` line comments and nested `/* ... */` block comments,
/// while treating plain (single-line) double-quoted string and char
/// literals as opaque.
///
/// Two real bugs motivated the string/char-literal awareness:
///
/// 1. A `//`/`/*` embedded inside a plain string (`"http://example.com"`,
///    `"/* not a comment */"`) is source *text*, not a real comment
///    delimiter, and must not be treated as one.
/// 2. Critically, a stray `/*` inside a plain string with no matching `*/`
///    in that same string (e.g. `"start /* fake open"`) would otherwise
///    make the old char-scanning block-comment stripper open a *real*
///    unterminated comment and silently swallow every declaration for the
///    rest of the file — a total, silent data-loss bug, not just a
///    cosmetic one.
///
/// Char literals are consumed atomically via [`char_literal_body_len`] so a
/// quote inside one (`'"'`) can never be misread as opening a string.
///
/// Scala (unlike Java/C) also allows block comments to nest
/// (`/* /* inner */ still commented */`); a depth counter is tracked so a
/// naive first-`*/`-wins close (which would leak the outer comment's tail,
/// including lines that look exactly like real declarations) never happens.
///
/// Triple-quoted string literals are stripped separately (via
/// `STRING_TRIPLE`) *before* this function runs, since they can span many
/// lines and can't nest — this function only needs to reason about
/// single-line string/char literals.
fn strip_comments(input: &str) -> String {
    #[derive(Clone, Copy)]
    enum State {
        Normal,
        LineComment,
        BlockComment(u32),
        Str,
    }

    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut state = State::Normal;

    while let Some(c) = chars.next() {
        state = match state {
            State::LineComment => {
                if c == '\n' {
                    out.push('\n');
                    State::Normal
                } else {
                    State::LineComment
                }
            }
            State::BlockComment(depth) => {
                if c == '/' && chars.peek() == Some(&'*') {
                    chars.next();
                    State::BlockComment(depth + 1)
                } else if c == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    if depth > 1 {
                        State::BlockComment(depth - 1)
                    } else {
                        State::Normal
                    }
                } else {
                    // Preserve newlines so per-line scanning downstream
                    // still sees one input line per output line even when
                    // a comment spans several lines.
                    if c == '\n' {
                        out.push('\n');
                    }
                    State::BlockComment(depth)
                }
            }
            State::Str => {
                out.push(c);
                if c == '\\' {
                    // Consume the escaped character verbatim too, so an
                    // escaped quote (`\"`) can't be mistaken for the
                    // closing delimiter.
                    if let Some(next) = chars.peek().copied() {
                        out.push(next);
                        chars.next();
                    }
                    State::Str
                } else if c == '"' {
                    State::Normal
                } else if c == '\n' {
                    // Plain strings can't span lines in real Scala; if the
                    // input is malformed/unterminated, bail out here rather
                    // than let a missing closing quote swallow the rest of
                    // the file.
                    State::Normal
                } else {
                    State::Str
                }
            }
            State::Normal => {
                if c == '\'' {
                    if let Some(body_len) = char_literal_body_len(chars.clone()) {
                        out.push('\'');
                        for _ in 0..body_len {
                            if let Some(ch) = chars.next() {
                                out.push(ch);
                            }
                        }
                        if let Some(ch) = chars.next() {
                            out.push(ch); // closing quote
                        }
                    } else {
                        out.push(c);
                    }
                    State::Normal
                } else if c == '"' {
                    out.push(c);
                    State::Str
                } else if c == '/' && chars.peek() == Some(&'/') {
                    chars.next();
                    State::LineComment
                } else if c == '/' && chars.peek() == Some(&'*') {
                    chars.next();
                    State::BlockComment(1)
                } else {
                    out.push(c);
                    State::Normal
                }
            }
        };
    }

    out
}

/// Extract public symbols from Scala source code.
pub fn extract_exports(content: &str) -> Vec<String> {
    // Strip triple-quoted string literals first so text that merely looks
    // like a declaration inside a multi-line string (or a `//`/`/*` inside
    // one) isn't misread as real source by the steps below.
    let stripped = STRING_TRIPLE.replace_all(content, "");
    let stripped = strip_comments(&stripped);

    let mut symbols = Vec::new();

    for line in stripped.lines() {
        if SCALA_PRIVATE.is_match(line) {
            continue;
        }
        if let Some(caps) = SCALA_DECL.captures(line) {
            if let Some(name) = caps.get(1) {
                let mut n = name.as_str().to_string();
                // Scala setter methods (`def name_=(value: T): Unit`) are a
                // single identifier ending in `=`, but `\w+` can't include
                // that trailing `=`. Splice it back on when the captured
                // name ends with `_` and is immediately (no space) followed
                // by `=`, matching the real setter naming convention.
                if n.ends_with('_') && line.as_bytes().get(name.end()) == Some(&b'=') {
                    n.push('=');
                }
                if !symbols.contains(&n) {
                    symbols.push(n);
                }
            }
        } else if !line.contains("=>") {
            // Scala 3 enum case lines don't match SCALA_DECL at all (no
            // leading keyword), so try the dedicated case pattern. Lines
            // containing `=>` are match arms, not case declarations.
            if let Some(caps) = SCALA_ENUM_CASE.captures(line)
                && let Some(names) = caps.get(1)
            {
                for raw in names.as_str().split(',') {
                    let case_name = raw.trim();
                    if !case_name.is_empty() && case_name != "_" {
                        let n = case_name.to_string();
                        if !symbols.contains(&n) {
                            symbols.push(n);
                        }
                    }
                }
            }
        }
    }

    symbols
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scala3_enum_declaration_and_cases() {
        // Scala 3 `enum` is the idiomatic way to write sum types/ADTs. Both
        // the enum's own name and its cases (bare `case Red, Green, Blue`
        // and parameterized `case Success(value: A)`) are public API and
        // must be captured, while the pattern-match arms inside a method
        // body (`case Success(_) => true`) must NOT be mistaken for case
        // declarations.
        let src = r#"
package com.example.model

enum Color:
  case Red, Green, Blue

enum Result[+A]:
  case Success(value: A)
  case Failure(error: String)

  def isSuccess: Boolean = this match
    case Success(_) => true
    case Failure(_) => false
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Color".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"Red".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"Green".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"Blue".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"Result".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"Success".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"Failure".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"isSuccess".to_string()), "{symbols:?}");
    }

    #[test]
    fn test_abstract_type_member_and_symbolic_operator_def() {
        // Abstract type members (`type Item`, family polymorphism/type-member
        // DI) and symbolic operator methods (`def +`, common in numeric/DSL
        // types) are both idiomatic, public Scala API that a plain `\w+`
        // capture misses entirely. The self-type annotation (`self: Logger
        // =>`) must not leak in as a false symbol.
        let src = r#"
trait Container {
  self: Logger =>
  type Item
  def get: Item
  def +(other: Container): Container = this
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Container".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"Item".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"get".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"+".to_string()), "{symbols:?}");
        assert!(!symbols.contains(&"Logger".to_string()), "{symbols:?}");
    }

    #[test]
    fn test_setter_method_name_not_mangled() {
        // Property-style getter/setter pairs are idiomatic Scala. The setter
        // `def name_=(value: String): Unit` is a single identifier
        // (`name_=`) — it must not be truncated to the fabricated name
        // `name_`, which wouldn't match a spec/doc referencing the real
        // setter name.
        let src = r#"
trait Container {
  def name: String
  def name_=(value: String): Unit
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"name".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"name_=".to_string()), "{symbols:?}");
        assert!(!symbols.contains(&"name_".to_string()), "{symbols:?}");
    }

    #[test]
    fn test_package_object_name_is_exported() {
        // BUG (fixed): `package object util { ... }` is a real, public,
        // top-level Scala declaration, but before the fix `SCALA_DECL`'s
        // modifier list didn't include `package`, so the leading `package`
        // token before `object` made the whole line fail to match and
        // `util` was silently dropped from the export list. Its public
        // member `helper` must still be reported, and the private member
        // must still be excluded.
        let src = r#"
package object util {
  def helper: Int = 1
  private def hiddenHelper: Int = 2
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"util".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"helper".to_string()), "{symbols:?}");
        assert!(
            !symbols.contains(&"hiddenHelper".to_string()),
            "{symbols:?}"
        );
    }

    #[test]
    fn test_qualified_private_modifiers_excluded() {
        // `private[this]` and `private[pkgname]` are package/instance
        // scoped access modifiers, not fully public — they must be excluded
        // just like bare `private`, across class/case class/object. Since
        // the exclusion check only requires the line to start with the
        // literal token "private" (the `[qualifier]` that follows is never
        // inspected), this already worked correctly before this task; this
        // test locks that verified behavior in permanently.
        let src = r#"
private[this] class GhostThis
private[mypkg] class GhostPkg
private[this] case class GhostCaseThis(x: Int)
private[mypkg] object GhostObjPkg
"#;
        let symbols = extract_exports(src);
        assert!(!symbols.contains(&"GhostThis".to_string()), "{symbols:?}");
        assert!(!symbols.contains(&"GhostPkg".to_string()), "{symbols:?}");
        assert!(
            !symbols.contains(&"GhostCaseThis".to_string()),
            "{symbols:?}"
        );
        assert!(!symbols.contains(&"GhostObjPkg".to_string()), "{symbols:?}");
    }

    #[test]
    fn test_nested_case_class_flat_export_convention() {
        // This extractor doesn't track brace nesting/scope depth anywhere
        // (confirmed by reading the implementation — it's a pure per-line
        // scan), so a public nested case class inside a class body is
        // reported flatly alongside top-level symbols, consistent with how
        // trait members are already treated elsewhere in this file. A
        // `private` nested case class must still be excluded.
        let src = r#"
class Outer {
  case class Inner(x: Int)
  private case class HiddenInner(x: Int)
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Outer".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"Inner".to_string()), "{symbols:?}");
        assert!(!symbols.contains(&"HiddenInner".to_string()), "{symbols:?}");
    }

    #[test]
    fn test_multiline_class_header_with_generics_and_extends() {
        // Real Scala style commonly wraps a generic class's `extends`/`with`
        // clauses onto their own lines when the header gets long. The class
        // name must still be captured from just its own first line, and
        // members further down in the body must still be found.
        let src = r#"
class Generic[A, B]
    extends SomeBase[A]
    with SomeMixin[B] {
  def method: Unit = ()
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Generic".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"method".to_string()), "{symbols:?}");
    }

    #[test]
    fn test_one_liner_class_and_object_stubs() {
        // Minimal one-line stub declarations (`class Foo {}`, `object Bar
        // {}`), common in scaffolding/placeholder code, must still have
        // their names captured even though the whole declaration — keyword,
        // name, and empty body — is packed onto a single line.
        let src = r#"
class OneLiner {}
object OneLinerObj {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"OneLiner".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"OneLinerObj".to_string()), "{symbols:?}");
    }

    #[test]
    fn test_sealed_and_final_modifier_combinations() {
        // Common real-world modifier stacks for ADT-style hierarchies:
        // `sealed trait`, `sealed abstract class`, and `final case class`.
        // Each combination must still yield the declared name.
        let src = r#"
sealed trait Animal
sealed abstract class Shape
final case class Point(x: Int, y: Int)
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Animal".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"Shape".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"Point".to_string()), "{symbols:?}");
    }

    #[test]
    fn test_triple_quoted_string_contents_not_treated_as_declarations() {
        // BUG (fixed): multi-line triple-quoted string literals were never
        // stripped before scanning, so text that merely *looks* like a
        // declaration inside a string body (e.g. embedded sample code,
        // templates, or doc text) was scanned as if it were real source and
        // leaked a fake symbol into the export list. The real declaration
        // that follows the string must still be found.
        let src = r#"
val s = """
class FakeExportInString
private class AlsoFakeInString
"""
class RealAfterString
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"s".to_string()), "{symbols:?}");
        assert!(
            !symbols.contains(&"FakeExportInString".to_string()),
            "{symbols:?}"
        );
        assert!(
            !symbols.contains(&"AlsoFakeInString".to_string()),
            "{symbols:?}"
        );
        assert!(
            symbols.contains(&"RealAfterString".to_string()),
            "{symbols:?}"
        );
    }

    #[test]
    fn test_nested_block_comments_fully_stripped() {
        // BUG (fixed): Scala (unlike Java/C) allows nested block comments
        // (`/* /* inner */ still commented */`). The old comment stripper
        // was a single lazy regex (`/\*.*?\*/`) that closed on the *first*
        // `*/` — i.e. the end of the inner comment — leaving everything
        // between that point and the real outer `*/` behind as live text.
        // That leftover text included a line that looked exactly like a
        // real declaration (`class ShouldStillBeCommented`), which leaked
        // into the export list. The nesting-aware stripper must consume the
        // whole outer comment, and the real declaration after it must still
        // be found.
        let src = r#"
/* outer
   /* inner */
   still inside comment
class ShouldStillBeCommented
*/
class RealAfterComment
"#;
        let symbols = extract_exports(src);
        assert!(
            !symbols.contains(&"ShouldStillBeCommented".to_string()),
            "{symbols:?}"
        );
        assert!(
            symbols.contains(&"RealAfterComment".to_string()),
            "{symbols:?}"
        );
    }

    #[test]
    fn test_scala_exports() {
        let src = r#"
package com.test
object Main {
  def main(args: Array[String]): Unit = {}
}
case class User(name: String)
trait Printable {
  def print(): Unit
}
private class Helper
protected def hiddenFunc = 42
lazy val config = "dev"
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Main".to_string()));
        assert!(symbols.contains(&"main".to_string()));
        assert!(symbols.contains(&"User".to_string()));
        assert!(symbols.contains(&"Printable".to_string()));
        assert!(symbols.contains(&"config".to_string()));
        assert!(!symbols.contains(&"Helper".to_string()));
        assert!(!symbols.contains(&"hiddenFunc".to_string()));
    }

    #[test]
    fn test_line_comment_with_stray_block_comment_opener_does_not_swallow_rest_of_file() {
        // BUG (fixed): the old line-comment regex (`//.*$`) had no `(?m)`
        // flag, so `$` anchored to the *end of the whole haystack* rather
        // than end-of-line. In practice this meant a `//` comment was only
        // ever stripped when it happened to sit on the literal last line of
        // the file — everywhere else, the comment's raw text (including any
        // `//`) survived untouched into the block-comment pass. If that
        // untouched comment text contained a stray `/*` with no matching
        // `*/` inside it, the nesting-aware block-comment stripper would
        // treat it as a genuine unterminated comment opening and silently
        // swallow every declaration for the rest of the file (verified: it
        // previously reduced this exact input down to just `["a"]`). The
        // rewritten single-pass `strip_comments` state machine strips a
        // `//` comment per-line regardless of its position in the file, so
        // the stray `/*` inside it never reaches the block-comment logic.
        let src = r#"
val a = 1 // see /* generics note
class RealDeclShouldSurvive
val b = 2
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"a".to_string()), "{symbols:?}");
        assert!(
            symbols.contains(&"RealDeclShouldSurvive".to_string()),
            "{symbols:?}"
        );
        assert!(symbols.contains(&"b".to_string()), "{symbols:?}");
    }

    #[test]
    fn test_unterminated_slash_star_inside_plain_string_does_not_swallow_rest_of_file() {
        // BUG (fixed): a plain (non-triple) double-quoted string was never
        // treated as opaque before scanning. If its contents included a
        // `/*` with no matching `*/` inside that same string, the old
        // purely char-based block-comment stripper (unaware of string
        // quoting) mistook it for a real unterminated comment opening and
        // swallowed every subsequent declaration in the file (verified: it
        // previously reduced this exact input down to just `["s"]`). The
        // rewritten `strip_comments` now tracks a string state entered on
        // an unescaped `"` and exited on the next unescaped `"` (or at
        // end-of-line, since plain Scala strings can't span lines), so a
        // `/*`/`*/`/`//` inside one is never mistaken for a real comment
        // delimiter.
        let src = r#"
val s = "start /* fake open, no close in this string"
class RealDeclAfterString
val b = 2
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"s".to_string()), "{symbols:?}");
        assert!(
            symbols.contains(&"RealDeclAfterString".to_string()),
            "{symbols:?}"
        );
        assert!(symbols.contains(&"b".to_string()), "{symbols:?}");
    }

    #[test]
    fn test_slash_star_and_double_slash_inside_plain_string_not_treated_as_comment() {
        // A self-contained `/* ... */` and a `//`-shaped substring
        // (`http://...`) inside an ordinary string must not be treated as
        // real comment delimiters — the string's own text, and the real
        // declaration that follows, must both still be found. This is the
        // less catastrophic sibling of the swallow-the-rest-of-file bug
        // above: even when the embedded sequence happens to be
        // self-terminating (or harmless) on its own line, it must still be
        // recognized as *string content*, not comment syntax.
        let src = r#"
val commentLike = "/* not a comment */"
val urlLike = "http://example.com"
class RealAfterBothStrings
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"commentLike".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"urlLike".to_string()), "{symbols:?}");
        assert!(
            symbols.contains(&"RealAfterBothStrings".to_string()),
            "{symbols:?}"
        );
    }

    #[test]
    fn test_char_literal_containing_quote_is_not_mistaken_for_a_string() {
        // A char literal whose content is itself a quote character (`'"'`)
        // must not be misread as the start of a string literal — if it
        // were, the scanner's string-tracking state would desync for the
        // rest of the line (and, before the string-awareness fix, could
        // even desync the block-comment logic across the rest of the
        // file). Escaped variants (`'\''`, `'\\'`) must work the same way.
        let src = r#"
val quoteChar = '"'
val escapedQuote = '\''
val backslash = '\\'
class RealAfterCharLiterals
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"quoteChar".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"escapedQuote".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"backslash".to_string()), "{symbols:?}");
        assert!(
            symbols.contains(&"RealAfterCharLiterals".to_string()),
            "{symbols:?}"
        );
    }

    #[test]
    fn test_char_literal_adjacent_to_real_block_comment_still_stripped() {
        // A char literal (`'/'`) immediately followed by a genuine block
        // comment on the same line must not desync the comment-opening
        // detection — the real comment must still be recognized and
        // stripped, and the declaration after it must still be found.
        let src = r#"
val slash = '/' /* trailing comment */
class RealAfterSlashChar
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"slash".to_string()), "{symbols:?}");
        assert!(
            symbols.contains(&"RealAfterSlashChar".to_string()),
            "{symbols:?}"
        );
    }

    #[test]
    fn test_single_quote_inside_plain_string_is_ordinary_text() {
        // A `'` inside a double-quoted string ("it's fine") is just
        // ordinary text — it must not be mistaken for the start of a char
        // literal (which would desync the scanner looking for a closing
        // `'`), and the string plus the following declaration must both
        // still be found.
        let src = r#"
val s = "it's fine"
class AfterApostropheString
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"s".to_string()), "{symbols:?}");
        assert!(
            symbols.contains(&"AfterApostropheString".to_string()),
            "{symbols:?}"
        );
    }

    #[test]
    fn test_interpolated_string_with_nested_quoted_subexpression() {
        // String interpolation with an embedded `${ ... }` sub-expression
        // that itself contains nested double-quoted strings (and, in the
        // second case, unbalanced-looking braces from a `Map` call) is
        // realistic idiomatic Scala. The scanner doesn't understand
        // interpolation syntax at all — it only tracks quote parity — but
        // because every nested string literal is itself a balanced
        // open/close pair, the parity naturally comes out even and the
        // true closing quote of the outer string is still found correctly.
        let src = r#"
val x = true
val greeting = s"outer ${ if (x) "a" else "b" } tail"
val m = s"${ Map(1 -> 2) }"
class RealAfterBothInterpolations
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"x".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"greeting".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"m".to_string()), "{symbols:?}");
        assert!(
            symbols.contains(&"RealAfterBothInterpolations".to_string()),
            "{symbols:?}"
        );
    }

    #[test]
    fn test_qualified_private_after_triple_quoted_string_on_previous_line() {
        // No parsing state should leak across the triple-quoted-string
        // stripping boundary: a `private[pkg]`-qualified declaration
        // immediately after a stripped triple-quoted string must still be
        // excluded, and the real public declaration after it must still be
        // found.
        let src = r#"
val doc = """
some text
"""
private[myapp] class GhostAfterTripleString
class RealAfterTripleString
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"doc".to_string()), "{symbols:?}");
        assert!(
            !symbols.contains(&"GhostAfterTripleString".to_string()),
            "{symbols:?}"
        );
        assert!(
            symbols.contains(&"RealAfterTripleString".to_string()),
            "{symbols:?}"
        );
    }

    #[test]
    fn test_three_levels_of_nested_block_comments() {
        // The nesting-aware comment stripper must correctly track depth
        // three levels deep (not just the two levels the first-pass
        // regression test covered), fully consuming the whole outer
        // comment regardless of how many inner comments it contains.
        let src = r#"
/* level1 /* level2 /* level3 */ still2 */ still1 */
class RealAfterDeepNesting
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"RealAfterDeepNesting".to_string()),
            "{symbols:?}"
        );
    }

    #[test]
    fn test_unterminated_block_comment_at_eof_does_not_panic_or_leak() {
        // A block comment that never closes before end-of-file must not
        // panic or hang, and must not leak the "commented-out" declaration
        // inside it as if it were live source. The real declaration before
        // the comment opened must still be found.
        let src = r#"
class BeforeComment
/* unterminated comment
class InsideUnterminatedShouldBeHidden
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"BeforeComment".to_string()),
            "{symbols:?}"
        );
        assert!(
            !symbols.contains(&"InsideUnterminatedShouldBeHidden".to_string()),
            "{symbols:?}"
        );
    }
}
