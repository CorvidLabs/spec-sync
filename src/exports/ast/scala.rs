use tree_sitter::{Node, Parser, Tree};

/// Parse Scala source into a tree-sitter AST.
fn parse_scala(content: &str) -> Option<Tree> {
    let mut parser = Parser::new();
    let language = tree_sitter_scala::LANGUAGE.into();
    parser.set_language(&language).ok()?;
    parser.parse(content, None)
}

/// Extract public symbols from Scala source using tree-sitter AST.
///
/// Mirrors the reference regex extractor's breadth: it walks the *whole*
/// tree (not just top-level items), collecting `class`/`object`/`trait`/`enum`
/// definitions, Scala 3 enum cases, abstract/alias `type` members, and
/// `def`/`val`/`var` declarations at any nesting depth, excluding only those
/// whose own declaration carries a `private` or `protected` modifier. It does
/// not model true Scala visibility scoping (e.g. a public member nested
/// inside a private class is still captured), matching the line-based
/// regex's blindness to block scope.
pub fn extract_exports(content: &str) -> Vec<String> {
    let tree = match parse_scala(content) {
        Some(t) => t,
        None => return Vec::new(),
    };

    let root = tree.root_node();
    let src = content.as_bytes();
    let mut symbols = Vec::new();

    collect_decls(&root, src, &mut symbols);

    symbols
}

/// Recursively walk every node, recording exported declaration names.
fn collect_decls(node: &Node, src: &[u8], symbols: &mut Vec<String>) {
    if !has_private_or_protected_modifier(node)
        && let Some(name) = decl_name(node, src)
        && !symbols.contains(&name)
    {
        symbols.push(name);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_decls(&child, src, symbols);
    }
}

/// Extract the declared name for the node kinds the regex spec captures.
fn decl_name(node: &Node, src: &[u8]) -> Option<String> {
    match node.kind() {
        "class_definition"
        | "object_definition"
        | "trait_definition"
        | "enum_definition"
        | "function_definition"
        | "function_declaration"
        // Scala 3 `given` instances (`given intShow: Show[Int] with ...`) are
        // real, nameable, importable public API (e.g. `import Foo.given`, or
        // a direct reference to the synthesized name) — the grammar gives
        // them their own `given_definition` node with a `name` field, just
        // like `def`/`class`. Anonymous givens (`given Show[Int] with ...`,
        // no name) simply have no `name` field, so `get_field_text` returns
        // `None` for them without any special-casing needed.
        | "given_definition" => get_field_text(node, "name", src),
        // Scala 3 enum cases (`case Red, Green, Blue` and `case Success(value: A)`)
        // each parse as their own node with a `name` field — a bare case list like
        // `case Red, Green, Blue` produces one `simple_enum_case` per name, so no
        // special comma-splitting is needed here (unlike the reference regex).
        "simple_enum_case" | "full_enum_case" => get_field_text(node, "name", src),
        // Abstract type members (`type Item`) and type aliases (`type Alias = Foo`)
        // both parse as `type_definition`, matching the reference regex's blindness
        // to that distinction (it just matches any `type <Name>` line).
        "type_definition" => get_field_text(node, "name", src),
        "val_definition" | "var_definition" => {
            // Only plain identifier patterns count, matching the regex's
            // `\s+(\w+)` requirement right after `val`/`var` (tuple/other
            // destructuring patterns don't match the reference regex either).
            let pattern = node.child_by_field_name("pattern")?;
            match pattern.kind() {
                "identifier" => pattern.utf8_text(src).ok().map(|s| s.to_string()),
                // Multi-name bindings (`val a, b = 1`) parse their pattern
                // as an `identifiers` node wrapping each name. The regex's
                // `\s+(\w+)` only ever captures the first name on the line
                // (`\w+` stops at the comma), so mirror that instead of
                // dropping the declaration entirely.
                "identifiers" => {
                    let mut cursor = pattern.walk();
                    pattern
                        .children(&mut cursor)
                        .find(|c| c.kind() == "identifier")
                        .and_then(|n| n.utf8_text(src).ok().map(|s| s.to_string()))
                }
                _ => None,
            }
        }
        // Abstract `val`/`var` members with no initializer (e.g. inside a
        // trait: `val apiKey: String`) parse as a distinct node kind from
        // their defining counterparts above, but the line-based regex draws
        // no such distinction — it matches any `val`/`var` line regardless
        // of whether it has a body. Mirror that by capturing these too.
        "val_declaration" | "var_declaration" => get_field_text(node, "name", src),
        _ => None,
    }
}

/// Check whether a declaration node's own `modifiers` child contains a
/// `private` or `protected` token (including qualified forms like
/// `private[this]` or `protected[pkg]`).
fn has_private_or_protected_modifier(node: &Node) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "modifiers" {
            return contains_access_keyword(&child);
        }
    }
    false
}

fn contains_access_keyword(node: &Node) -> bool {
    if node.kind() == "private" || node.kind() == "protected" {
        return true;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if contains_access_keyword(&child) {
            return true;
        }
    }
    false
}

fn get_field_text(node: &Node, field: &str, src: &[u8]) -> Option<String> {
    node.child_by_field_name(field)
        .map(|n| n.utf8_text(src).unwrap_or_default().to_string())
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
    fn test_abstract_type_member_captured() {
        // Abstract type members (`type Item`, family polymorphism/type-
        // member DI) are common, idiomatic parts of a trait's public
        // contract. The self-type annotation (`self: Logger =>`) must not
        // leak in as a false symbol.
        let src = r#"
trait Container {
  self: Logger =>
  type Item
  def get: Item
  def name: String
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Container".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"Item".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"get".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"name".to_string()), "{symbols:?}");
        assert!(!symbols.contains(&"Logger".to_string()), "{symbols:?}");
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
    fn test_ignores_declarations_in_comments_and_strings() {
        // AST-native: tree-sitter never treats comment/string contents as
        // code, unlike the regex spec which merely strips `//` and `/* */`
        // comments textually before scanning (and never touches strings —
        // a decl-shaped string body would already survive the regex too,
        // demonstrating the AST is at least as precise here).
        let src = r#"
// class FakeCommented
/* def alsoFake(): Unit = {} */
val message = "class FakeInString"
def real(): Unit = {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"real".to_string()));
        assert!(symbols.contains(&"message".to_string()));
        assert!(!symbols.contains(&"FakeCommented".to_string()));
        assert!(!symbols.contains(&"alsoFake".to_string()));
        assert!(!symbols.contains(&"FakeInString".to_string()));
    }

    #[test]
    fn test_captures_abstract_val_var_declarations() {
        // Abstract `val`/`var` members (no initializer) parse as a
        // different tree-sitter node kind (`val_declaration` /
        // `var_declaration`) than their defining counterparts, but the
        // line-based regex has no such distinction.
        let src = r#"
trait Config {
  val apiKey: String
  var counter: Int
  private val secret: String
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"apiKey".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"counter".to_string()), "{symbols:?}");
        assert!(!symbols.contains(&"secret".to_string()), "{symbols:?}");
    }

    #[test]
    fn test_captures_first_name_in_multi_name_bindings() {
        // `val a, b = 1` / `var c, d = 1` bind multiple names on one line.
        // The reference regex's `val\s+(\w+)` only ever captures the first
        // name (`\w+` stops at the comma) — it doesn't lose the
        // declaration entirely, so the AST port shouldn't either.
        let src = r#"
val a, b = 1
var c, d = 2
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"a".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"c".to_string()), "{symbols:?}");
    }

    #[test]
    fn test_matches_regex_blindness_to_nesting_scope() {
        // Deliberate parity with the reference regex: it has no notion of
        // block-scoped visibility, so a member nested inside a private
        // class is still captured as long as its own line/declaration
        // isn't itself marked private/protected.
        let src = r#"
private class Helper {
  def innerMethod(): Unit = {}
}
"#;
        let symbols = extract_exports(src);
        assert!(!symbols.contains(&"Helper".to_string()));
        assert!(symbols.contains(&"innerMethod".to_string()));
    }

    #[test]
    fn test_learnxinyminutes_real_oop_excerpt() {
        // Real excerpt (learnxinyminutes.com/docs/scala, OOP section): a
        // class with a public var, a public method, and a private method; a
        // companion object; a case class; and a trait with abstract members
        // plus a subclass. Exercises real-world visibility mixing (public
        // API alongside a `private def`) in idiomatic Scala source rather
        // than a hand-written toy example.
        let src = r#"
class Dog(br: String) {
  var breed: String = br

  def bark = "Woof, woof!"

  private def sleep(hours: Int) =
    println(s"I'm sleeping for $hours hours")
}

val mydog = new Dog("greyhound")

object Dog {
  def allKnownBreeds = List("pitbull", "shepherd", "retriever")
  def createDog(breed: String) = new Dog(breed)
}

case class Person(name: String, phoneNumber: String)

val george = Person("George", "1234")
val otherGeorge = george.copy(phoneNumber = "9876")

trait Dog {
	def breed: String
	def color: String
	def bark: Boolean = true
	def bite: Boolean
}
class SaintBernard extends Dog {
	val breed = "Saint Bernard"
	val color = "brown"
	def bite = false
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Dog".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"breed".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"bark".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"mydog".to_string()), "{symbols:?}");
        assert!(
            symbols.contains(&"allKnownBreeds".to_string()),
            "{symbols:?}"
        );
        assert!(symbols.contains(&"createDog".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"Person".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"george".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"otherGeorge".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"color".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"bite".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"SaintBernard".to_string()), "{symbols:?}");
        assert!(!symbols.contains(&"sleep".to_string()), "{symbols:?}");
    }

    #[test]
    fn test_learnxinyminutes_real_pattern_matching_excerpt() {
        // Real excerpt (learnxinyminutes.com/docs/scala, Pattern Matching
        // section): a def matching on a case class, a val bound to a
        // compiled regex, and a def with a large match block covering
        // literal/type/guard/tuple/list patterns. Regression-tests that
        // clean, idiomatic real-world `match` usage is captured correctly
        // (see also the regex extractor's sibling test, which documents a
        // known AST-only gap when this exact declaration sequence is
        // preceded by unrelated corrupted input elsewhere in the file).
        let src = r#"
def matchPerson(person: Person): String = person match {
  case Person("George", number) => "We found George! His number is " + number
  case Person("Kate", number)   => "We found Kate! Her number is " + number
  case Person(name, number)     => "We matched someone : " + name + ", phone : " + number
}

val email = "(.*)@(.*)".r

def matchEverything(obj: Any): String = obj match {
  case "Hello world" => "Got the string Hello world"
  case x: Double => "Got a Double: " + x
  case x: Int if x > 10000 => "Got a pretty big number!"
  case Person(name, number) => s"Got contact info for $name!"
  case email(name, domain) => s"Got email address $name@$domain"
  case (a: Int, b: Double, c: String) => s"Got a tuple: $a, $b, $c"
  case List(1, b, c) => s"Got a list with three elements and starts with 1: 1, $b, $c"
  case List(List((1, 2, "YAY"))) => "Got a list of list of tuple"
  case _ => "Got unknown object"
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"matchPerson".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"email".to_string()), "{symbols:?}");
        assert!(
            symbols.contains(&"matchEverything".to_string()),
            "{symbols:?}"
        );
    }

    #[test]
    fn test_private_qualified_access_modifiers_excluded() {
        // `private[this]` and `private[pkg]` are partial-privacy qualifiers,
        // not full `private` — but they're still not part of a type/trait's
        // *public* API surface (they're visible only within the enclosing
        // instance or a named package, never from an arbitrary external
        // importer). Verified via `parse_scala(..).root_node().to_sexp()`
        // that both wrap an `access_modifier` node that still contains a
        // `private`-kind node regardless of the `[qualifier]` suffix, so
        // `contains_access_keyword`'s recursive search already excludes them
        // correctly — this documents verified-correct behavior, not a bug.
        let src = r#"
class Outer {
  private[this] def onlyThis: Int = 1
  private[mypkg] def onlyPkg: Int = 2
  private def fullyPrivate: Int = 3
  def public: Int = 4
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Outer", "public"], "{symbols:?}");
    }

    #[test]
    fn test_scala3_given_instance_name_captured() {
        // Regression test: `given_definition` was previously missing from
        // `decl_name`'s match arms entirely, so a named Scala 3 `given`
        // instance (`given intShow: Show[Int] with ...`) was silently
        // dropped even though the walker still recursed *into* its body
        // (capturing the nested `show` method) — an inconsistent partial
        // miss, not a parse failure, so the overall non-empty result never
        // triggered the regex fallback that would have masked the gap.
        // `given`-instance names are real, referenceable public API
        // (`import Foo.given`), and a `private given` must still be
        // excluded like any other private declaration.
        let src = r#"
trait Show[A] {
  def show(a: A): String
}

given intShow: Show[Int] with
  def show(a: Int): String = a.toString

private given hiddenShow: Show[Long] with
  def show(a: Long): String = a.toString
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"intShow".to_string()), "{symbols:?}");
        assert!(!symbols.contains(&"hiddenShow".to_string()), "{symbols:?}");
    }

    #[test]
    fn test_scala3_extension_methods_captured_without_extension_name() {
        // Scala 3 `extension (x: Int) def double: Int = ...` blocks have no
        // name of their own (only the methods declared inside do) — the
        // walker must still recurse into the `extension_definition` body and
        // capture `double` even though `decl_name` returns `None` for the
        // `extension_definition` node itself.
        let src = r#"
extension (x: Int)
  def double: Int = x * 2
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"double".to_string()), "{symbols:?}");
    }

    #[test]
    fn test_sealed_trait_and_sealed_abstract_class_captured() {
        // `sealed` restricts *where* subtypes may be defined (same file/
        // package), but the trait/class name itself remains public API —
        // the `sealed` modifier must not be confused with `private`/
        // `protected` and must not block capture.
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
    fn test_package_object_and_companion_object_members_captured() {
        // `package object` bodies and companion objects sharing a class's
        // name are both idiomatic, common Scala patterns; their public
        // members must be captured like any other object's, with private
        // members still excluded.
        let src = r#"
package object util {
  def helper: Int = 1
  private def hiddenHelper: Int = 2
}

class Dog(name: String)
object Dog {
  def apply(name: String): Dog = new Dog(name)
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"helper".to_string()), "{symbols:?}");
        assert!(
            !symbols.contains(&"hiddenHelper".to_string()),
            "{symbols:?}"
        );
        assert!(symbols.contains(&"Dog".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"apply".to_string()), "{symbols:?}");
    }

    #[test]
    fn test_multiline_variance_and_context_bound_type_params_captured() {
        // Multi-line type parameter lists with variance annotations
        // (`+A`/`-B`) and context bounds (`C: Ordering`) must not confuse
        // the `name` field lookup on the surrounding `class_definition`/
        // `function_definition` — the type-parameter list is a distinct
        // field the walker never has to parse to find the declared name.
        // (Primary-constructor `val` parameters like `val value: A` are a
        // distinct `class_parameter` node kind not handled by `decl_name`
        // at all — out of scope here, this test only exercises the
        // type-parameter-list shape.)
        let src = r#"
class Container[
    +A,
    -B
](val value: A) {
  def map[C: Ordering](f: A => C): Container[C, B] = ???
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Container".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"map".to_string()), "{symbols:?}");
    }
}
