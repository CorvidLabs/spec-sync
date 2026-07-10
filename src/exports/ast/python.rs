use tree_sitter::{Parser, Tree};

/// Parse Python source into a tree-sitter AST.
fn parse_py(content: &str) -> Option<Tree> {
    let mut parser = Parser::new();
    let language = tree_sitter_python::LANGUAGE.into();
    parser.set_language(&language).ok()?;
    parser.parse(content, None)
}

/// Extract exported symbols from Python source using tree-sitter AST.
///
/// Handles:
/// - `__all__` list (takes precedence if present)
/// - Top-level `def`, `async def`, `class` (excluding `_`-prefixed)
/// - Top-level constant/variable assignments, typed or untyped (excluding `_`-prefixed)
/// - Conditional imports in `__init__.py` patterns
pub fn extract_exports(content: &str) -> Vec<String> {
    let tree = match parse_py(content) {
        Some(t) => t,
        None => return Vec::new(),
    };

    let root = tree.root_node();
    let src = content.as_bytes();

    // First, look for __all__ assignment
    if let Some(all_symbols) = find_dunder_all(&root, src) {
        return all_symbols;
    }

    // Fallback: top-level def/class not starting with _
    let mut symbols = Vec::new();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor) {
        match child.kind() {
            "function_definition" => {
                if let Some(name) = child.child_by_field_name("name") {
                    let n = name.utf8_text(src).unwrap_or_default();
                    if !n.starts_with('_') {
                        symbols.push(n.to_string());
                    }
                }
            }
            "class_definition" => {
                if let Some(name) = child.child_by_field_name("name") {
                    let n = name.utf8_text(src).unwrap_or_default();
                    if !n.starts_with('_') {
                        symbols.push(n.to_string());
                    }
                }
            }
            // `@decorator\nasync def ...` — decorated definitions
            "decorated_definition" => {
                let mut inner_cursor = child.walk();
                for inner in child.children(&mut inner_cursor) {
                    if (inner.kind() == "function_definition" || inner.kind() == "class_definition")
                        && let Some(name) = inner.child_by_field_name("name")
                    {
                        let n = name.utf8_text(src).unwrap_or_default();
                        if !n.starts_with('_') {
                            symbols.push(n.to_string());
                        }
                    }
                }
            }
            // Top-level constant/variable assignment: `NAME = ...` or
            // `NAME: Type = ...`. Tuple/list-unpacking targets (left is not a
            // plain identifier) are skipped.
            "expression_statement" => {
                let mut stmt_cursor = child.walk();
                for stmt_child in child.children(&mut stmt_cursor) {
                    if stmt_child.kind() == "assignment"
                        && let Some(left) = stmt_child.child_by_field_name("left")
                        && left.kind() == "identifier"
                    {
                        let n = left.utf8_text(src).unwrap_or_default();
                        if !n.starts_with('_') {
                            symbols.push(n.to_string());
                        }
                    }
                }
            }
            // Conditional imports: `if TYPE_CHECKING:` blocks at top level
            // We still don't export from these — they're type-only
            _ => {}
        }
    }

    symbols
}

/// Look for `__all__ = [...]` at the top level and extract symbol names.
fn find_dunder_all(root: &tree_sitter::Node, src: &[u8]) -> Option<Vec<String>> {
    let mut cursor = root.walk();

    for child in root.children(&mut cursor) {
        if child.kind() == "expression_statement" {
            let mut stmt_cursor = child.walk();
            for stmt_child in child.children(&mut stmt_cursor) {
                if stmt_child.kind() == "assignment" {
                    // Use `continue` rather than `?` on a missing field: a
                    // single malformed/ERROR-recovered assignment node must
                    // not abort scanning the rest of the file for a later,
                    // well-formed `__all__` assignment.
                    let Some(left) = stmt_child.child_by_field_name("left") else {
                        continue;
                    };
                    let left_text = left.utf8_text(src).unwrap_or_default();

                    if left_text == "__all__" {
                        let Some(right) = stmt_child.child_by_field_name("right") else {
                            continue;
                        };
                        let mut names = Vec::new();
                        collect_list_literals(&right, src, &mut names);
                        if !names.is_empty() {
                            return Some(names);
                        }
                    }
                }
            }
        }
    }

    None
}

/// Recursively harvest string-literal entries from every `list` node
/// reachable within `node`.
///
/// Handles a dynamically-built `__all__`, e.g. `__all__ = ["a"] + other`
/// (a `binary_operator` node whose left operand is a real list literal):
/// the visible list literal(s) still contribute their names even though a
/// bare identifier operand (`other`) can't be statically resolved. Without
/// this, any `__all__` built via concatenation was invisible to
/// `find_dunder_all` entirely (its RHS is a `binary_operator`, not a
/// `list`), silently discarding the explicit export restriction and falling
/// back to scanning every top-level public def/class -- including names
/// the module never intended to export.
fn collect_list_literals(node: &tree_sitter::Node, src: &[u8], names: &mut Vec<String>) {
    if node.kind() == "list" {
        names.extend(extract_string_list(node, src));
        return;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_list_literals(&child, src, names);
    }
}

/// Extract string values from a list literal: `["foo", "bar"]`.
fn extract_string_list(node: &tree_sitter::Node, src: &[u8]) -> Vec<String> {
    let mut names = Vec::new();
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        if child.kind() == "string" {
            // String node contains string_content child(ren)
            let text = child.utf8_text(src).unwrap_or_default();
            // Strip surrounding quotes
            let trimmed = text
                .trim_start_matches(['\'', '"'])
                .trim_end_matches(['\'', '"']);
            if !trimmed.is_empty() {
                names.push(trimmed.to_string());
            }
        }
    }

    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_learnxinyminutes_functions_excerpt() {
        // Real excerpt (learnxinyminutes.com/python) covering *args/**kwargs,
        // tuple-swap assignment, tuple-unpacking (must NOT surface as a
        // top-level symbol), an unindented docstring-as-comment between two
        // statements, and global-scope shadowing (`x` reassigned at top
        // level multiple times, and again inside a function via `global`).
        let src = r#"
def add(x, y):
    print("x is {} and y is {}".format(x, y))
    return x + y  # Return values with a return statement

# Calling functions with parameters
add(5, 6)  # => prints out "x is 5 and y is 6" and returns 11

# Another way to call functions is with keyword arguments
add(y=6, x=5)  # Keyword arguments can arrive in any order.

# You can define functions that take a variable number of
# positional arguments
def varargs(*args):
    return args

varargs(1, 2, 3)  # => (1, 2, 3)

# You can define functions that take a variable number of
# keyword arguments, as well
def keyword_args(**kwargs):
    return kwargs

# Let's call it to see what happens
keyword_args(big="foot", loch="ness")  # => {"big": "foot", "loch": "ness"}


# You can do both at once, if you like
def all_the_args(*args, **kwargs):
    print(args)
    print(kwargs)
"""
all_the_args(1, 2, a=3, b=4) prints:
    (1, 2)
    {"a": 3, "b": 4}
"""

# When calling functions, you can do the opposite of args/kwargs!
# Use * to expand args (tuples) and use ** to expand kwargs (dictionaries).
args = (1, 2, 3, 4)
kwargs = {"a": 3, "b": 4}
all_the_args(*args)            # equivalent: all_the_args(1, 2, 3, 4)
all_the_args(**kwargs)         # equivalent: all_the_args(a=3, b=4)
all_the_args(*args, **kwargs)  # equivalent: all_the_args(1, 2, 3, 4, a=3, b=4)

# Returning multiple values (with tuple assignments)
def swap(x, y):
    return y, x  # Return multiple values as a tuple without the parenthesis.
                 # (Note: parenthesis have been excluded but can be included)

x = 1
y = 2
x, y = swap(x, y)     # => x = 2, y = 1
# (x, y) = swap(x,y)  # Again the use of parenthesis is optional.

# global scope
x = 5

def set_x(num):
    # local scope begins here
    # local var x not the same as global var x
    x = num    # => 43
    print(x)   # => 43

def set_global_x(num):
    # global indicates that particular var lives in the global scope
    global x
    print(x)   # => 5
    x = num    # global var x is now set to 6
    print(x)   # => 6

set_x(43)
set_global_x(6)
"#;
        let symbols = extract_exports(src);
        for expected in [
            "add",
            "varargs",
            "keyword_args",
            "all_the_args",
            "swap",
            "set_x",
            "set_global_x",
        ] {
            assert!(
                symbols.contains(&expected.to_string()),
                "expected {expected:?} in {symbols:?}"
            );
        }
        assert!(symbols.contains(&"args".to_string()));
        assert!(symbols.contains(&"kwargs".to_string()));
        assert!(symbols.contains(&"y".to_string()));
        assert_eq!(symbols.iter().filter(|s| *s == "x").count(), 2);
        // Tuple-unpacking assignment (`x, y = swap(x, y)`) must not
        // contribute a spurious symbol, and nothing from inside function
        // bodies (nested defs, `global`-qualified locals) should leak out.
        assert_eq!(symbols.iter().filter(|s| *s == "swap").count(), 1);
    }

    #[test]
    fn test_python_learnxinyminutes_human_class_excerpt() {
        // Real excerpt (learnxinyminutes.com/python): a class body with a
        // dunder __init__, plain instance methods, @classmethod,
        // @staticmethod, and a full @property/@x.setter/@x.deleter trio.
        // Only the class itself is top-level API - every member is indented
        // and must not leak out, regardless of its decorator.
        let src = r#"
class Human:

    # A class attribute. It is shared by all instances of this class
    species = "H. sapiens"

    # Basic initializer, this is called when this class is instantiated.
    # Note that the double leading and trailing underscores denote objects
    # or attributes that are used by Python but that live in user-controlled
    # namespaces. Methods(or objects or attributes) like: __init__, __str__,
    # __repr__ etc. are called special methods (or sometimes called dunder
    # methods). You should not invent such names on your own.
    def __init__(self, name):
        # Assign the argument to the instance's name attribute
        self.name = name

        # Initialize property
        self._age = 0   # the leading underscore indicates the "age" property is
                        # intended to be used internally
                        # do not rely on this to be enforced: it's a hint to other devs

    # An instance method. All methods take "self" as the first argument
    def say(self, msg):
        print("{name}: {message}".format(name=self.name, message=msg))

    # Another instance method
    def sing(self):
        return "yo... yo... microphone check... one two... one two..."

    # A class method is shared among all instances
    # They are called with the calling class as the first argument
    @classmethod
    def get_species(cls):
        return cls.species

    # A static method is called without a class or instance reference
    @staticmethod
    def grunt():
        return "*grunt*"

    # A property is just like a getter.
    # It turns the method age() into a read-only attribute of the same name.
    # There's no need to write trivial getters and setters in Python, though.
    @property
    def age(self):
        return self._age

    # This allows the property to be set
    @age.setter
    def age(self, age):
        self._age = age

    # This allows the property to be deleted
    @age.deleter
    def age(self):
        del self._age
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Human"]);
    }

    #[test]
    fn test_python_all() {
        let src = r#"
__all__ = ["create_auth", "AuthService", "DEFAULT_TTL"]

def create_auth(config):
    pass

class AuthService:
    pass

def _internal():
    pass
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["create_auth", "AuthService", "DEFAULT_TTL"]);
    }

    #[test]
    fn test_python_no_all() {
        let src = r#"
def create_auth(config):
    pass

class AuthService:
    pass

def _internal():
    pass

async def fetch_token():
    pass
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["create_auth", "AuthService", "fetch_token"]);
    }

    #[test]
    fn test_python_nested_not_captured() {
        let src = r#"
class Outer:
    class Inner:
        pass
    def method(self):
        pass

def top_level():
    def nested():
        pass
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Outer".to_string()));
        assert!(symbols.contains(&"top_level".to_string()));
        assert!(!symbols.contains(&"Inner".to_string()));
        assert!(!symbols.contains(&"method".to_string()));
        assert!(!symbols.contains(&"nested".to_string()));
    }

    #[test]
    fn test_python_dunder_excluded() {
        let src = r#"
def __init__(self):
    pass

def __repr__(self):
    pass

def public_func():
    pass
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"public_func".to_string()));
        assert!(!symbols.contains(&"__init__".to_string()));
    }

    #[test]
    fn test_python_all_overrides() {
        let src = r#"
__all__ = ["_special", "Public"]

def _special():
    pass

class Public:
    pass

class AlsoPublicButNotInAll:
    pass
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"_special".to_string()));
        assert!(symbols.contains(&"Public".to_string()));
        assert!(!symbols.contains(&"AlsoPublicButNotInAll".to_string()));
    }

    #[test]
    fn test_decorated_functions() {
        let src = r#"
@dataclass
class Config:
    host: str

@staticmethod
def create():
    pass
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Config".to_string()));
        assert!(symbols.contains(&"create".to_string()));
    }

    #[test]
    fn test_conditional_import_init() {
        // __init__.py pattern with conditional imports
        let src = r#"
__all__ = ["Router", "middleware"]

from .router import Router
from .middleware import middleware

try:
    from .optional import OptionalFeature
except ImportError:
    pass
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Router", "middleware"]);
    }

    #[test]
    fn test_python_top_level_constants_captured() {
        // Typed and untyped top-level constants/variables without a leading
        // underscore are public API, same as every sibling-language extractor
        // (e.g. Kotlin's val/var). Tuple-unpacking assignments are still skipped.
        let src = r#"
MAX_RETRIES: int = 3
DEFAULT_TIMEOUT = 30
_internal_flag = True
a, b = 1, 2

def connect():
    pass
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["MAX_RETRIES", "DEFAULT_TIMEOUT", "connect"]);
    }

    #[test]
    fn test_dunder_all_built_via_concatenation_extracts_visible_literal() {
        // Regression test: `__all__ = ["a"] + _other` previously made
        // `find_dunder_all` return `None` outright (its RHS is a
        // `binary_operator`, not a `list`), so `extract_exports` silently
        // fell back to scanning every top-level public def/class -- which
        // leaked `not_in_all` even though the module's explicit `__all__`
        // never listed it. The visible list literal (`["a"]`) is now
        // harvested even though the `_other` operand can't be statically
        // resolved, so the export surface stays restricted to what's
        // actually spelled out in source.
        let src = r#"
_other = ["b"]
__all__ = ["a"] + _other

def a():
    pass

def b():
    pass

def not_in_all():
    pass
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["a".to_string()]);
        assert!(!symbols.contains(&"not_in_all".to_string()));
    }

    #[test]
    fn test_dunder_all_concatenation_of_two_list_literals() {
        // Both operands of the concatenation are real list literals: every
        // name from both contributes, in left-to-right order.
        let src = r#"
__all__ = ["a"] + ["b", "c"]

def a():
    pass

def b():
    pass

def c():
    pass
"#;
        let symbols = extract_exports(src);
        assert_eq!(
            symbols,
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }

    #[test]
    fn test_dunder_all_non_list_expression_falls_back_to_scan() {
        // Verified-correct (unchanged) behavior: when `__all__`'s value
        // contains no list literal at all (e.g. built entirely from a
        // function call), there is nothing to harvest, so `find_dunder_all`
        // still returns `None` and `extract_exports` falls back to the
        // normal top-level scan (which still applies the leading-underscore
        // convention).
        let src = r#"
__all__ = tuple(_registry.keys())

def real_pub():
    pass

def _hidden():
    pass
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["real_pub".to_string()]);
    }

    #[test]
    fn test_multiline_decorator_with_arguments_still_captures_definition() {
        // A decorator whose call arguments span several lines
        // (`@app.route(\n    "/api",\n    methods=["GET"],\n)`) must not
        // interfere with resolving the decorated function's name -- only
        // the sibling `function_definition`/`class_definition` field is
        // inspected, regardless of how complex the decorator expression is.
        let src = r#"
@app.route(
    "/api",
    methods=["GET"],
)
def handler():
    pass
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["handler".to_string()]);
    }

    #[test]
    fn test_conditional_top_level_def_under_if_else_not_captured() {
        // Verified-correct (documented) current behavior: a `def` nested
        // inside an `if`/`else` block (e.g. version-gated compatibility
        // shims) is not a direct child of the module root in the AST, so
        // neither branch's definition is captured -- consistent with the
        // module doc comment's stated policy of not exporting from
        // conditional blocks (originally written with `TYPE_CHECKING`
        // guards in mind), and consistent with the regex backend, whose
        // `^`-anchored, column-0 regex also can't match an indented `def`.
        // Statically choosing one branch as "the" export would require
        // resolving `sys.version_info` at analysis time, which neither
        // backend attempts.
        let src = r#"
import sys

if sys.version_info >= (3, 10):
    def foo():
        return 1
else:
    def foo():
        return 2

def real_top_level():
    pass
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["real_top_level".to_string()]);
        assert!(!symbols.contains(&"foo".to_string()));
    }

    #[test]
    fn test_def_inside_main_guard_not_captured() {
        // A `def` inside `if __name__ == "__main__":` is indented source,
        // and (like every other `if`-nested def) is not a direct child of
        // the module root, so it is excluded the same way -- this matches
        // real-world intent, since such defs are script entry points, not
        // public API.
        let src = r#"
def real_public():
    pass

if __name__ == "__main__":
    def helper():
        pass
    real_public()
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["real_public".to_string()]);
        assert!(!symbols.contains(&"helper".to_string()));
    }

    #[test]
    fn test_triple_quoted_string_with_fake_def_not_captured() {
        // AST-native: unlike the regex backend (which must explicitly strip
        // triple-quoted strings before scanning), a module-level string
        // literal is just an opaque `string` node to tree-sitter -- text
        // inside it that merely looks like `def fake():`/`class Fake:` is
        // never visited as a sibling declaration in the first place.
        let src = r#"
"""
def fake():
    pass

class FakeClass:
    pass
"""

def real_func():
    pass
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["real_func".to_string()]);
    }

    #[test]
    fn test_match_statement_and_walrus_do_not_crash_or_leak() {
        // Adversarial parser-robustness check: `match`/`case` and the
        // walrus operator (`:=`) are newer syntax the grammar must still
        // parse cleanly (no panics), and neither should leak a spurious
        // top-level symbol (the walrus target lives inside a
        // `parenthesized_expression` condition, not a top-level
        // `expression_statement` assignment).
        let src = r#"
def handle(x):
    match x:
        case 1:
            return "one"
        case _:
            return "other"

if (n := 10) > 5:
    pass

class Real:
    pass
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["handle".to_string(), "Real".to_string()]);
        assert!(!symbols.contains(&"n".to_string()));
    }
}
