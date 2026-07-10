use regex::Regex;
use std::sync::LazyLock;

static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"//.*$").unwrap());

static COMMENT_MULTI: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/").unwrap());

/// Dart top-level declarations: class, mixin, enum, extension, typedef
/// In Dart, anything NOT prefixed with _ is public.
static DART_TYPE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^(?:abstract\s+)?(?:class|mixin|enum|extension|typedef)\s+([A-Z]\w*)").unwrap()
});

/// Dart top-level functions and variables, AND container members (methods,
/// getters/setters, fields) declared inside a class/mixin/interface/extension
/// body. Dart has no per-member visibility keyword — everything not prefixed
/// with `_` is public, including members of an already-public container — so
/// unlike Swift/Kotlin we can't anchor on an access modifier. We instead anchor
/// on horizontal whitespace only (`^[^\S\n]*`, matching the Kotlin extractor's
/// approach) so realistically-indented (dartfmt-formatted) member declarations
/// still match, not just column-0 top-level declarations.
///
/// A leading type token (or the `get`/`set` keyword) is still required before
/// the name so we don't match arbitrary indented statements inside method
/// bodies (e.g. `return foo();`, `await bar();`); the fallback "custom type"
/// branch is restricted to `[A-Z]\w*` (Dart's own type-naming convention)
/// rather than any lowercase word, which is what keeps control-flow keywords
/// like `return`/`await` from being misread as a return type.
static DART_TOPLEVEL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*(?:static\s+)?(?:(?:Future<[^>]*>|Stream<[^>]*>|void|int|double|String|bool|List<[^>]*>|Map<[^>]*>|Set<[^>]*>|dynamic|[A-Z]\w*)\s+(?:get\s+|set\s+)?|(?:get|set)\s+)([a-zA-Z]\w*)\s*[({=;]",
    )
    .unwrap()
});

/// Dart `final` and `const` declarations — top-level or as container (class/
/// mixin/extension) fields. See DART_TOPLEVEL for why the anchor allows
/// leading horizontal whitespace instead of requiring column 0.
static DART_CONST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^[^\S\n]*(?:static\s+)?(?:final|const)\s+(?:\w+\s+)?([a-zA-Z]\w*)\s*[=;]")
        .unwrap()
});

/// Dart top-level function declarations with NO explicit return type. Dart
/// infers a return type when one isn't written, so `main() {}` and
/// `example1() {}` are just as public as `void main() {}` — but they don't
/// have any type token for DART_TOPLEVEL to anchor on. Unlike DART_TOPLEVEL,
/// this is restricted to true column 0 (no leading whitespace at all, not
/// even the horizontal-whitespace allowance used for container members):
/// indented statements inside a function/class body (`print(x);`,
/// `if (x) { ... }`, a nested helper function) are never anchored at column
/// 0 in realistically-formatted Dart, so this stays a low-risk, high-value
/// way to pick up exactly top-level declarations. A keyword denylist guards
/// against the rare case of a control-flow statement sitting at column 0.
static DART_BARE_FUNCTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^([a-zA-Z_]\w*)\s*\([^)]*\)\s*(?:async\s*\*?\s*)?(?:\{|=>)").unwrap()
});

const DART_KEYWORDS: &[&str] = &[
    "if", "for", "while", "switch", "catch", "do", "else", "try", "finally", "return", "await",
    "yield", "assert", "new", "throw", "with", "on",
];

/// Detect a line that opens a container body (class/mixin/enum/extension), as
/// opposed to a function/method/getter/setter body or any other block
/// (if/for/while/closure). Declarations directly inside a container body are
/// real export candidates; declarations nested inside a function/method body
/// are locals and are never exported, regardless of whether they happen to
/// look like a typed top-level declaration.
static CONTAINER_BODY_OPEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^[^\S\n]*(?:abstract\s+)?(?:class|mixin|enum|extension)\b").unwrap()
});

/// Extract public symbols from Dart source code.
/// In Dart, identifiers starting with _ are private; everything else is public.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = COMMENT_SINGLE.replace_all(content, "");
    let stripped = COMMENT_MULTI.replace_all(&stripped, "");

    let mut symbols = Vec::new();
    // Tracks nested `{ }` scopes: true = a class/mixin/enum/extension body
    // (member declarations are legitimate export candidates), false = a
    // function/method/getter/setter body or any other block (declarations
    // inside are local and never exported). An empty stack means top level.
    let mut scope_stack: Vec<bool> = Vec::new();

    for line in stripped.lines() {
        let in_exportable_scope = scope_stack.last().copied().unwrap_or(true);

        if in_exportable_scope {
            for caps in DART_TYPE.captures_iter(line) {
                if let Some(name) = caps.get(1) {
                    let n = name.as_str();
                    if !n.starts_with('_') && !symbols.contains(&n.to_string()) {
                        symbols.push(n.to_string());
                    }
                }
            }

            for caps in DART_CONST.captures_iter(line) {
                if let Some(name) = caps.get(1) {
                    let n = name.as_str();
                    if !n.starts_with('_') && !symbols.contains(&n.to_string()) {
                        symbols.push(n.to_string());
                    }
                }
            }

            for caps in DART_TOPLEVEL.captures_iter(line) {
                if let Some(name) = caps.get(1) {
                    let n = name.as_str();
                    if !n.starts_with('_') && !symbols.contains(&n.to_string()) {
                        symbols.push(n.to_string());
                    }
                }
            }

            for caps in DART_BARE_FUNCTION.captures_iter(line) {
                if let Some(name) = caps.get(1) {
                    let n = name.as_str();
                    if !n.starts_with('_')
                        && !DART_KEYWORDS.contains(&n)
                        && !symbols.contains(&n.to_string())
                    {
                        symbols.push(n.to_string());
                    }
                }
            }
        }

        // Only treat a freshly-opened body as an "exportable scope" (and thus a candidate
        // for its members to be captured) if it's a container body, not a function/method
        // body -- without this, a local variable declared inside an ordinary function body
        // (`void doWork() { String result = compute(); }`) would be treated as sitting
        // directly in an exportable scope and leak as a top-level export.
        let opens_container_body = in_exportable_scope && CONTAINER_BODY_OPEN.is_match(line);
        for ch in line.chars() {
            match ch {
                '{' => scope_stack.push(opens_container_body),
                '}' => {
                    scope_stack.pop();
                }
                _ => {}
            }
        }
    }

    symbols
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dart_abstract_class_interface_members() {
        // Finding #1: an abstract class used as an interface (dartfmt-indented
        // method/getter signatures with no bodies) previously had its entire
        // contract dropped — only the class name itself was captured, because
        // DART_TOPLEVEL/DART_CONST were anchored at column 0 and never matched
        // realistically-indented member declarations.
        let src = r#"
abstract class Authenticator {
  String get name;
  bool validate(String token);
  Future<void> refresh();
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Authenticator".to_string()));
        assert!(symbols.contains(&"name".to_string()), "abstract getter");
        assert!(symbols.contains(&"validate".to_string()), "abstract method");
        assert!(
            symbols.contains(&"refresh".to_string()),
            "abstract method with Future<void> return"
        );
    }

    #[test]
    fn test_dart_class_and_mixin_members_indented() {
        // Finding #1 continued: mixin methods and class fields/methods/getters/
        // setters at normal indentation must be captured, while private
        // (underscore-prefixed) members stay excluded.
        let src = r#"
mixin Loggable {
  final List<String> _logBuffer = [];
  void log(String message) { _logBuffer.add(message); }
  void clearLog() => _logBuffer.clear();
}

class UserRepository {
  final ApiClient _client;
  static const int maxPageSize = 50;
  int get cachedCount => _cache.length;
  set pageSize(int value) { _pageSize = value; }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Loggable".to_string()));
        assert!(symbols.contains(&"log".to_string()));
        assert!(symbols.contains(&"clearLog".to_string()));
        assert!(!symbols.contains(&"_logBuffer".to_string()));
        assert!(symbols.contains(&"UserRepository".to_string()));
        assert!(
            symbols.contains(&"maxPageSize".to_string()),
            "static const class field"
        );
        assert!(!symbols.contains(&"_client".to_string()));
        assert!(
            symbols.contains(&"cachedCount".to_string()),
            "instance getter"
        );
        assert!(symbols.contains(&"pageSize".to_string()), "instance setter");
    }

    #[test]
    fn test_dart_getter_setter_keyword_position() {
        // Finding #2: `get`/`set` sits between the return-type prefix and the
        // identifier, so the regex previously tried (and failed) to capture
        // `get`/`set` itself as the name. Also covers the related
        // inconsistency where an explicit `void` return type on a setter
        // (`void set count(...)`) was dropped even though the typeless
        // `set count(...)` form worked.
        let src = r#"
extension StringValidation on String {
  bool get isValidEmail => contains('@');
}

class Widget {
  String get label;
  set pageSize(int value) {}
  void set count(int v) {}
}
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"isValidEmail".to_string()),
            "extension getter with arrow body"
        );
        assert!(symbols.contains(&"label".to_string()), "abstract getter");
        assert!(
            symbols.contains(&"pageSize".to_string()),
            "setter without explicit return type"
        );
        assert!(
            symbols.contains(&"count".to_string()),
            "setter with explicit void return type"
        );
    }

    #[test]
    fn test_dart_exports() {
        let src = r#"
class AuthService {
  String validate(String token) {}
}

abstract class BaseController {}
mixin LoggerMixin {}
enum AuthStatus { active, expired }
typedef AuthCallback = void Function(String);
const defaultTtl = 3600;
final String apiVersion = "1.0";
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"AuthService".to_string()));
        assert!(symbols.contains(&"BaseController".to_string()));
        assert!(symbols.contains(&"LoggerMixin".to_string()));
        assert!(symbols.contains(&"AuthStatus".to_string()));
        assert!(symbols.contains(&"AuthCallback".to_string()));
        assert!(symbols.contains(&"defaultTtl".to_string()));
        assert!(symbols.contains(&"apiVersion".to_string()));
    }

    #[test]
    fn test_dart_locals_inside_function_body_not_exported() {
        // Regression test: a plain typed local variable declared inside a function or
        // method body must not leak as a top-level export just because it has the same
        // "Type name = ..." shape as a real field/top-level variable declaration.
        let src = r#"
void doWork() {
  String result = compute();
  int count = 0;
}

class Service {
  void process() {
    String localValue = "temp";
  }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"doWork".to_string()));
        assert!(symbols.contains(&"Service".to_string()));
        assert!(symbols.contains(&"process".to_string()));
        assert!(!symbols.contains(&"result".to_string()));
        assert!(!symbols.contains(&"count".to_string()));
        assert!(!symbols.contains(&"localValue".to_string()));
    }

    #[test]
    fn test_dart_private() {
        let src = r#"
class _InternalHelper {}
void _privateFunc() {}
const _secret = "hidden";
class PublicClass {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"PublicClass".to_string()));
        assert!(!symbols.contains(&"_InternalHelper".to_string()));
        assert!(!symbols.contains(&"_privateFunc".to_string()));
        assert!(!symbols.contains(&"_secret".to_string()));
    }

    #[test]
    fn test_dart_comments_stripped() {
        let src = r#"
// class FakeClass {}
/* enum FakeEnum {} */
/// Documentation comment
class RealClass {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"RealClass".to_string()));
        assert!(!symbols.contains(&"FakeClass".to_string()));
        assert!(!symbols.contains(&"FakeEnum".to_string()));
    }

    #[test]
    fn test_dart_abstract_class() {
        let src = r#"
abstract class Repository {
  Future<void> save();
}

abstract class Disposable {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Repository".to_string()));
        assert!(symbols.contains(&"Disposable".to_string()));
    }

    #[test]
    fn test_dart_future_stream_return() {
        let src = r#"
Future<String> fetchData() async {}
Stream<int> countUp() async* {}
void doNothing() {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"fetchData".to_string()));
        assert!(symbols.contains(&"countUp".to_string()));
        assert!(symbols.contains(&"doNothing".to_string()));
    }

    #[test]
    fn test_dart_const_vs_final() {
        let src = r#"
const int maxRetries = 3;
final timeout = Duration(seconds: 30);
const apiUrl = "https://example.com";
final _internal = "hidden";
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"maxRetries".to_string()));
        assert!(symbols.contains(&"timeout".to_string()));
        assert!(symbols.contains(&"apiUrl".to_string()));
        assert!(!symbols.contains(&"_internal".to_string()));
    }

    #[test]
    fn test_dart_extension_and_mixin() {
        let src = r#"
extension StringExt on String {}
mixin Serializable {}
enum Status { active, inactive }
typedef Callback = void Function(String);
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"StringExt".to_string()));
        assert!(symbols.contains(&"Serializable".to_string()));
        assert!(symbols.contains(&"Status".to_string()));
        assert!(symbols.contains(&"Callback".to_string()));
    }

    #[test]
    fn test_dart_learnxinyminutes_bare_toplevel_functions() {
        // Real excerpt (learnxinyminutes.com/docs/dart, adapted only by
        // trimming the surrounding tutorial commentary) covering the most
        // idiomatic Dart declaration style: top-level functions with NO
        // explicit return type, culminating in `main()` itself. Before this
        // fix, DART_TOPLEVEL required a leading type token, so `main` and
        // every `exampleN` function here were silently dropped even though
        // they're the program's actual public entry points.
        let src = r#"
example31() {
    findVolume31(int length, int breath, [int? height]) {
      print('length = $length, breath = $breath, height = $height');
    }

    findVolume31(10,20,30); //valid
    findVolume31(10,20); //also valid
}

example32() {
    findVolume32(int length, int breath, {int? height}) {
    print('length = $length, breath = $breath, height = $height');
    }

    findVolume32(10,20,height:30);//valid & we can see the parameter name is mentioned here.
    findVolume32(10,20);//also valid
}

main() {
  print("Learn Dart in 15 minutes!");
  [
    example31, example32
  ].forEach((ef) => ef());
}
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"example31".to_string()),
            "bare top-level function"
        );
        assert!(
            symbols.contains(&"example32".to_string()),
            "bare top-level function"
        );
        assert!(
            symbols.contains(&"main".to_string()),
            "bare top-level main entry point"
        );
        // Nested nested helper functions (`findVolume31`/`findVolume32`) are
        // indented, not at column 0, so they correctly stay out of scope for
        // the bare-function pattern (they're locals, not top-level exports).
        assert!(!symbols.contains(&"findVolume31".to_string()));
        assert!(!symbols.contains(&"findVolume32".to_string()));
    }

    #[test]
    fn test_dart_learnxinyminutes_generics_and_const_collections() {
        // Real excerpt (learnxinyminutes.com/docs/dart) covering a generic
        // class declaration and top-level `const` List/Map literals plus
        // explicitly-typed `List<String>`/`Map<String,dynamic>` top-level
        // variables -- idiomatic Dart collection/generics syntax not
        // exercised by the hand-written tests above.
        let src = r#"
class GenericExample<T>{
  void printType(){
    print("$T");
  }
  // methods can also have generics
  genericMethod<M>(){
    print("class:$T, method: $M");
  }
}

const example8List = ["Example8 const array"];
const  example8Map = {"someKey": "Example8 const map"};
/// Declare List or Maps as Objects.
 List<String> explicitList = new List<String>.empty();
 Map<String,dynamic> explicitMaps = new Map<String,dynamic>();
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"GenericExample".to_string()),
            "generic class name"
        );
        assert!(
            symbols.contains(&"printType".to_string()),
            "generic class method"
        );
        assert!(
            symbols.contains(&"example8List".to_string()),
            "top-level const list"
        );
        assert!(
            symbols.contains(&"example8Map".to_string()),
            "top-level const map"
        );
        assert!(
            symbols.contains(&"explicitList".to_string()),
            "top-level List<String> variable"
        );
        assert!(
            symbols.contains(&"explicitMaps".to_string()),
            "top-level Map<String,dynamic> variable"
        );
    }
}
