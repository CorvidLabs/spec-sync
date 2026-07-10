use regex::Regex;
use std::sync::LazyLock;

/// __all__ = ["Name1", "Name2"]
static ALL_DECL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"__all__\s*=\s*\[([^\]]*)\]"#).unwrap());

/// Top-level def name( or class Name
static TOP_LEVEL_DECL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^(?:def|class|async def)\s+(\w+)").unwrap());

/// One assignment target at the start of a (possibly chained) top-level
/// assignment: `NAME = ` or `NAME: Type = `. Used by `chained_assignment_targets`
/// to walk `x = y = z = value` one target at a time — a single anchored,
/// non-repeating regex can only ever capture the first target (`x`), silently
/// dropping every other name in the chain.
static ASSIGN_TARGET: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\w+)\s*(?::[^=\n]+)?=").unwrap());

/// Walks a top-level (column-0) line for every `NAME =` assignment target,
/// left to right, so a chained assignment (`x = y = z = 5`) yields all of
/// `x`, `y`, `z` instead of just the first. Rejects `==` comparisons: the
/// character immediately after the matched `=` must not itself be `=` (the
/// `regex` crate has no look-around, so this is checked manually instead of
/// via a lookahead). Returns an empty vec for any line that isn't a bare
/// column-0 assignment at all (e.g. indented code, a `def`/`class` line, or a
/// plain expression statement).
fn chained_assignment_targets(line: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut rest = line;
    while let Some(caps) = ASSIGN_TARGET.captures(rest) {
        let whole = caps.get(0).unwrap();
        if rest[whole.end()..].starts_with('=') {
            break;
        }
        names.push(caps[1].to_string());
        rest = rest[whole.end()..].trim_start();
    }
    names
}

/// Quoted string in __all__
static QUOTED: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"["'](\w+)["']"#).unwrap());

/// Triple-quoted string literal (docstrings, template/codegen strings). Stripped
/// before scanning for declarations so that unindented `def`/`class`/assignment
/// text embedded inside a docstring or string constant isn't mistaken for real
/// top-level code.
static TRIPLE_QUOTED_STRING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?s)"""(?:.*?)"""|'''(?:.*?)'''"#).unwrap());

/// Extract exported symbols from Python source code.
/// If `__all__` is defined, use that. Otherwise, all top-level
/// functions, classes, and constants/variables that don't start with `_`
/// are considered public.
pub fn extract_exports(content: &str) -> Vec<String> {
    // Strip triple-quoted string literals first so that example code inside
    // module docstrings or def/class-looking text inside template strings
    // doesn't get mistaken for real top-level declarations.
    let stripped = TRIPLE_QUOTED_STRING.replace_all(content, "");

    // Check for __all__ first
    if let Some(caps) = ALL_DECL.captures(&stripped)
        && let Some(list) = caps.get(1)
    {
        let mut symbols = Vec::new();
        for name_cap in QUOTED.captures_iter(list.as_str()) {
            if let Some(name) = name_cap.get(1) {
                symbols.push(name.as_str().to_string());
            }
        }
        return symbols;
    }

    // Fallback: top-level def/class/assignment that don't start with _,
    // collected in source order.
    let mut hits: Vec<(usize, String)> = Vec::new();
    for caps in TOP_LEVEL_DECL.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str();
            if !n.starts_with('_') {
                hits.push((name.start(), n.to_string()));
            }
        }
    }
    let mut offset = 0usize;
    for line in stripped.split_inclusive('\n') {
        let bare = line.trim_end_matches(['\n', '\r']);
        for n in chained_assignment_targets(bare) {
            if !n.starts_with('_') {
                hits.push((offset, n));
            }
        }
        offset += line.len();
    }
    hits.sort_by_key(|(pos, _)| *pos);

    hits.into_iter().map(|(_, n)| n).collect()
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
        // Top-level defs, in source order.
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
        // Top-level scalar assignments (`args`, `kwargs`, `x`, `y`) are
        // captured, including repeated rebindings of `x`.
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

DEFAULT_TTL = 3600
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
    fn test_python_all_single_quotes() {
        let src = r#"
__all__ = ['Foo', 'bar_func']

class Foo:
    pass

def bar_func():
    pass

class _Hidden:
    pass
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Foo", "bar_func"]);
    }

    #[test]
    fn test_python_all_overrides_conventions() {
        // When __all__ is present, even underscore-prefixed names are exported if listed
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
    fn test_python_decorators_ignored() {
        let src = r#"
@dataclass
class Config:
    host: str

@staticmethod
def create():
    pass

@app.route("/")
async def index():
    pass
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Config".to_string()));
        assert!(symbols.contains(&"create".to_string()));
        assert!(symbols.contains(&"index".to_string()));
    }

    #[test]
    fn test_python_nested_not_captured() {
        // Only top-level (no indentation) defs/classes should be captured
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
        // Inner and method are indented, should not be captured
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
        assert!(!symbols.contains(&"__repr__".to_string()));
    }

    #[test]
    fn test_python_protocol_and_property_methods_not_leaked() {
        // typing.Protocol / ABC pattern with @abstractmethod and @property/@x.setter
        // decorated methods: only the class itself is top-level API, the methods
        // are indented and must not leak out even though Python has no per-member
        // visibility keyword to filter on.
        let src = r#"
from typing import Protocol
from abc import abstractmethod


class Authenticator(Protocol):
    @abstractmethod
    def validate(self) -> bool: ...

    @property
    def name(self) -> str:
        return self._name

    @name.setter
    def name(self, value: str) -> None:
        self._name = value
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Authenticator"]);
    }

    #[test]
    fn test_python_module_docstring_example_code_not_captured() {
        // A module docstring containing an unindented usage example with
        // def/class lines must not produce false-positive public API entries.
        let src = r#"
"""
auth_service.py

Example:

def helper_snippet():
    return True

class ExampleUsage:
    pass
"""

import os


def real_public_func():
    pass


class RealService:
    pass
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["real_public_func", "RealService"]);
    }

    #[test]
    fn test_python_template_string_constant_not_captured() {
        // Top-level triple-quoted string constants (codegen templates, SQL
        // snippets, fixtures) can contain unindented def/class-looking text
        // that must not be mistaken for real declarations.
        let src = r#"
CODEGEN_TEMPLATE = """
class NotARealClass:
    pass

def not_a_real_function():
    pass
"""

def real_func():
    pass
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["CODEGEN_TEMPLATE", "real_func"]);
    }

    #[test]
    fn test_python_chained_assignment_captures_all_targets() {
        // Regression test: `TOP_LEVEL_VAR` was a single anchored (non-repeating) regex,
        // so a chained assignment (`x = y = 5`) only ever captured the first target --
        // `y`'s module-level exposure was silently missed.
        let src = r#"
x = y = z = 5
MAX = MIN = 10

def connect():
    pass
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"x".to_string()));
        assert!(symbols.contains(&"y".to_string()));
        assert!(symbols.contains(&"z".to_string()));
        assert!(symbols.contains(&"MAX".to_string()));
        assert!(symbols.contains(&"MIN".to_string()));
        assert!(symbols.contains(&"connect".to_string()));
    }

    #[test]
    fn test_python_top_level_constants_captured() {
        // Typed and untyped top-level constants/variables without a leading
        // underscore are public API, same as every sibling-language extractor.
        let src = r#"
MAX_RETRIES: int = 3
DEFAULT_TIMEOUT = 30
_internal_flag = True

def connect():
    pass
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["MAX_RETRIES", "DEFAULT_TIMEOUT", "connect"]);
    }
}
