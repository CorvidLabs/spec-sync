use regex::Regex;
use std::sync::LazyLock;

static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"//.*$").unwrap());
static COMMENT_MULTI: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/").unwrap());

/// Scala public declarations: class, object, trait, enum, def, val, var, type.
///
/// The name capture accepts either a normal `\w+` identifier or a run of
/// Scala operator characters, so symbolic/operator method names (`def
/// +(other: T): T`) are captured too — not just alphanumeric ones. When the
/// name starts with a word character the `\w+` alternative always wins
/// (regex alternation is leftmost-first), so this doesn't change behavior
/// for ordinary identifiers; the operator alternative only ever fires for
/// names that are symbolic from their very first character.
static SCALA_DECL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*(?:(?:implicit|lazy|case|override|sealed|abstract|final)\s+)*(?:class|object|trait|enum|def|val|var|type)\s+(\w+|[!#%&*+\-/:<=>?@^|~]+)",
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

/// Extract public symbols from Scala source code.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = COMMENT_SINGLE.replace_all(content, "");
    let stripped = COMMENT_MULTI.replace_all(&stripped, "");

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
            if let Some(caps) = SCALA_ENUM_CASE.captures(line) {
                if let Some(names) = caps.get(1) {
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
    fn throwaway_real_learnxinyminutes_regex() {
        let src = include_str!(
            "/private/tmp/claude-501/-Users-leif-Development--CorvidLabs-spec-sync/1429498c-236f-41e9-839d-cd71a8ca63b8/scratchpad/scala_work/real.scala"
        );
        let symbols = extract_exports(src);
        eprintln!("REGEX symbol count: {}", symbols.len());
        for s in &symbols {
            eprintln!("  {:?}", s);
        }
    }
}
