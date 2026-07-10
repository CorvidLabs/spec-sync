use regex::Regex;
use std::sync::LazyLock;

/// Single-line # comments
static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)#.*$").unwrap());

/// Multi-line =begin/=end comments
static COMMENT_MULTI: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?ms)^=begin.*?^=end").unwrap());

/// Class declarations: class Name, class Name < Parent, or compact namespaced
/// form class Api::V1::Name — the whole `A::B::C` path is captured so the
/// real (last-segment) class name can be recovered instead of truncating at
/// the first `::`.
static RUBY_CLASS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[^\S\n]*class\s+((?:[A-Z]\w*::)*[A-Z]\w*)").unwrap());

/// Module declarations: module Name, including compact namespaced form
/// module Api::V1::Name.
static RUBY_MODULE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[^\S\n]*module\s+((?:[A-Z]\w*::)*[A-Z]\w*)").unwrap());

/// Top-level method definitions (at zero indentation, considered public).
/// Captures a trailing `?`/`!` since predicate/bang methods (`valid?`,
/// `reset!`) are idiomatic Ruby and that punctuation is part of the name.
static RUBY_DEF: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^def\s+(?:self\.)?(\w+[?!]?)").unwrap());

/// Instance method definitions (indented, inside a class — public by default).
/// Captures a trailing `?`/`!` — see RUBY_DEF.
static RUBY_METHOD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[^\S\n]+def\s+(?:self\.)?(\w+[?!]?)").unwrap());

/// Constant assignments: NAME = value. Ruby constants are any identifier
/// starting with an uppercase letter — not just SCREAMING_SNAKE_CASE, but
/// also single-letter (`X = 5`) and PascalCase (`Var = "..."`,
/// `DefaultLogger = Logger.new`) forms. The `=` must immediately (mod
/// whitespace) follow the whole identifier, not a `.` continuation of it —
/// this keeps the pattern from matching a bare prefix of a longer dotted
/// expression like `Human.foo = 2` (a setter call, not a constant
/// declaration).
static RUBY_CONST: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[^\S\n]*([A-Z]\w*)[^\S\n]*=").unwrap());

/// attr_accessor / attr_reader / attr_writer declarations
static RUBY_ATTR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[^\S\n]*attr_(?:accessor|reader|writer)\s+(.+)$").unwrap());

/// Symbol literal :name
static RUBY_SYMBOL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r":(\w+[?!]?)").unwrap());

/// private / protected visibility markers (bare toggle — nothing else on the line)
static VISIBILITY_PRIVATE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\s*(?:private|protected)\s*$").unwrap());

static VISIBILITY_PUBLIC: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\s*public\s*$").unwrap());

/// Post-hoc symbol-based visibility: `private :name`, `private :a, :b`,
/// `private(:name)`. This marks already-defined methods private by name
/// rather than toggling visibility for subsequent defs.
static VISIBILITY_PRIVATE_SYMBOLS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*(?:private|protected)[^\S\n]*\(?[^\S\n]*(:[\w?!]+(?:[^\S\n]*,[^\S\n]*:[\w?!]+)*)[^\S\n]*\)?[^\S\n]*$",
    )
    .unwrap()
});

/// Compact namespaced declarations (`class Api::V1::UsersController`) capture
/// the whole `A::B::C` path, but only the last segment is the type actually
/// being declared here — the outer segments are references to an
/// already-existing namespace, not new declarations.
fn last_segment(name: &str) -> String {
    name.rsplit("::").next().unwrap_or(name).to_string()
}

/// Extract public symbols from Ruby source code.
/// Ruby defaults to public visibility. We track visibility state changes
/// (private/protected/public) to determine which methods are public.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = COMMENT_MULTI.replace_all(content, "");
    let stripped = COMMENT_SINGLE.replace_all(&stripped, "");

    let mut symbols = Vec::new();

    // Classes and modules are always "public" at the namespace level
    for caps in RUBY_CLASS.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = last_segment(name.as_str());
            if !symbols.contains(&n) {
                symbols.push(n);
            }
        }
    }

    for caps in RUBY_MODULE.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = last_segment(name.as_str());
            if !symbols.contains(&n) {
                symbols.push(n);
            }
        }
    }

    // Methods explicitly privatized post-hoc via `private :name` / `protected
    // :a, :b` (as opposed to the bare `private` line, which toggles
    // visibility for defs that follow it). Collected up front over the whole
    // file since the post-hoc call commonly comes *after* the method it
    // privatizes.
    let mut privatized: std::collections::HashSet<String> = std::collections::HashSet::new();
    for caps in VISIBILITY_PRIVATE_SYMBOLS.captures_iter(&stripped) {
        if let Some(list) = caps.get(1) {
            for sym in RUBY_SYMBOL.captures_iter(list.as_str()) {
                if let Some(name) = sym.get(1) {
                    privatized.insert(name.as_str().to_string());
                }
            }
        }
    }

    // Top-level defs (zero indentation) are always public
    for caps in RUBY_DEF.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str();
            if !n.starts_with('_') && !privatized.contains(n) && !symbols.contains(&n.to_string()) {
                symbols.push(n.to_string());
            }
        }
    }

    // Track visibility for indented methods (inside classes).
    // Walk lines, toggle visibility state. Visibility is scoped to the
    // innermost class/module: entering a new `class`/`module` line resets
    // the state back to public, so a `private` toggle inside one
    // class/module does not leak into sibling classes/modules later in the
    // same file.
    let mut public = true;
    for line in stripped.lines() {
        if RUBY_CLASS.is_match(line) || RUBY_MODULE.is_match(line) {
            public = true;
        }

        if VISIBILITY_PRIVATE.is_match(line) {
            public = false;
            continue;
        }
        if VISIBILITY_PUBLIC.is_match(line) {
            public = true;
            continue;
        }
        if VISIBILITY_PRIVATE_SYMBOLS.is_match(line) {
            // Post-hoc `private :name` only affects the named method(s)
            // (handled via `privatized` above) — it doesn't change
            // visibility for defs that follow it.
            continue;
        }

        if public
            && let Some(caps) = RUBY_METHOD.captures(line)
            && let Some(name) = caps.get(1)
        {
            let n = name.as_str();
            if !n.starts_with('_')
                && n != "initialize"
                && !privatized.contains(n)
                && !symbols.contains(&n.to_string())
            {
                symbols.push(n.to_string());
            }
        }
    }

    // Constants
    for caps in RUBY_CONST.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str().to_string();
            if !symbols.contains(&n) {
                symbols.push(n);
            }
        }
    }

    // attr_accessor / attr_reader / attr_writer (public attributes)
    for caps in RUBY_ATTR.captures_iter(&stripped) {
        if let Some(attrs) = caps.get(1) {
            for sym in RUBY_SYMBOL.captures_iter(attrs.as_str()) {
                if let Some(name) = sym.get(1) {
                    let n = name.as_str().to_string();
                    if !symbols.contains(&n) {
                        symbols.push(n);
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
    fn test_ruby_class_and_methods() {
        let src = r#"
module Authentication
  class AuthService
    DEFAULT_TTL = 3600

    attr_reader :token, :expires_at

    def validate(token)
      # ...
    end

    def self.create(config)
      # ...
    end

    private

    def internal_check
      # ...
    end
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Authentication".to_string()));
        assert!(symbols.contains(&"AuthService".to_string()));
        assert!(symbols.contains(&"validate".to_string()));
        assert!(symbols.contains(&"create".to_string()));
        assert!(symbols.contains(&"DEFAULT_TTL".to_string()));
        assert!(symbols.contains(&"token".to_string()));
        assert!(symbols.contains(&"expires_at".to_string()));
        assert!(!symbols.contains(&"internal_check".to_string()));
    }

    #[test]
    fn test_ruby_top_level_functions() {
        let src = r#"
def process_data(input)
  # ...
end

def _private_helper
  # ...
end

class DataProcessor
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"process_data".to_string()));
        assert!(symbols.contains(&"DataProcessor".to_string()));
        assert!(!symbols.contains(&"_private_helper".to_string()));
    }

    #[test]
    fn test_ruby_visibility_toggle() {
        let src = r#"
class Foo
  def public_one
  end

  def public_two
  end

  private

  def secret_one
  end

  public

  def public_again
  end

  protected

  def also_hidden
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Foo".to_string()));
        assert!(symbols.contains(&"public_one".to_string()));
        assert!(symbols.contains(&"public_two".to_string()));
        assert!(symbols.contains(&"public_again".to_string()));
        assert!(!symbols.contains(&"secret_one".to_string()));
        assert!(!symbols.contains(&"also_hidden".to_string()));
    }

    #[test]
    fn test_ruby_skips_initialize() {
        let src = r#"
class Bar
  def initialize(name)
    @name = name
  end

  def name
    @name
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Bar".to_string()));
        assert!(symbols.contains(&"name".to_string()));
        assert!(!symbols.contains(&"initialize".to_string()));
    }

    #[test]
    fn test_ruby_visibility_scoped_per_class() {
        // A `private` toggle inside one class must not leak into a sibling
        // class/module later in the same file — visibility resets at each
        // new class/module boundary.
        let src = r#"
module Billing
  class Invoice
    def total
    end

    private

    def apply_discount
    end
  end

  class Receipt
    def summary
    end
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Invoice".to_string()));
        assert!(symbols.contains(&"Receipt".to_string()));
        assert!(symbols.contains(&"total".to_string()));
        assert!(symbols.contains(&"summary".to_string()));
        assert!(!symbols.contains(&"apply_discount".to_string()));
    }

    #[test]
    fn test_ruby_namespaced_class_declaration() {
        // Compact namespaced class declarations (common for Rails
        // controllers/models) must resolve to the real class name, not a
        // truncated leading namespace segment.
        let src = r#"
class Api::V1::UsersController
  def index
  end

  def create
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"UsersController".to_string()));
        assert!(symbols.contains(&"index".to_string()));
        assert!(symbols.contains(&"create".to_string()));
        assert!(!symbols.contains(&"Api".to_string()));
    }

    #[test]
    fn test_ruby_posthoc_private_symbol_and_predicate_methods() {
        // Covers: `private :name` post-hoc visibility (as opposed to a bare
        // `private` toggle), predicate (`?`) and bang (`!`) method names
        // kept intact, and a realistic mixin module using `module_function`
        // plus a `self.` singleton method.
        let src = r#"
module Authenticatable
  module_function

  def logged_in?
    !!current_session
  end

  def self.reset!
    @session = nil
  end

  def internal_token
    @session
  end

  private :internal_token
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Authenticatable".to_string()));
        assert!(symbols.contains(&"logged_in?".to_string()));
        assert!(symbols.contains(&"reset!".to_string()));
        assert!(!symbols.contains(&"internal_token".to_string()));
        assert!(!symbols.contains(&"logged_in".to_string()));
        assert!(!symbols.contains(&"reset".to_string()));
    }

    #[test]
    fn test_ruby_learnxinyminutes_human_class() {
        // Real excerpt from learnxinyminutes.com/ruby (the "Classes" section):
        // a class variable, `initialize`, a `name=` setter, a plain getter,
        // attr_accessor/attr_reader/attr_writer on top of the hand-written
        // getter/setter, a `self.` class method, and a plain instance method
        // reading the class variable.
        let src = r#"
# You can define a class with the 'class' keyword.
class Human

  # A class variable. It is shared by all instances of this class.
  @@species = 'H. sapiens'

  # Basic initializer
  def initialize(name, age = 0)
    # Assign the argument to the 'name' instance variable for the instance.
    @name = name
    # If no age given, we will fall back to the default in the arguments list.
    @age = age
  end

  # Basic setter method
  def name=(name)
    @name = name
  end

  # Basic getter method
  def name
    @name
  end

  # The above functionality can be encapsulated using the attr_accessor method
  # as follows.
  attr_accessor :name

  # Getter/setter methods can also be created individually like this.
  attr_reader :name
  attr_writer :name

  # A class method uses self to distinguish from instance methods.
  # It can only be called on the class, not an instance.
  def self.say(msg)
    puts msg
  end

  def species
    @@species
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Human".to_string()));
        assert!(symbols.contains(&"name".to_string()));
        assert!(symbols.contains(&"say".to_string()));
        assert!(symbols.contains(&"species".to_string()));
        assert!(!symbols.contains(&"initialize".to_string()));
    }

    #[test]
    fn test_ruby_learnxinyminutes_concern_module() {
        // Real excerpt from learnxinyminutes.com/ruby (the module callbacks
        // section): a module with nested modules, a `self.included` hook
        // method, and a class that mixes the outer module in.
        let src = r#"
# Callbacks are executed when including and extending a module
module ConcernExample
  def self.included(base)
    base.extend(ClassMethods)
    base.send(:include, InstanceMethods)
  end

  module ClassMethods
    def bar
      'bar'
    end
  end

  module InstanceMethods
    def qux
      'qux'
    end
  end
end

class Something
  include ConcernExample
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"ConcernExample".to_string()));
        assert!(symbols.contains(&"ClassMethods".to_string()));
        assert!(symbols.contains(&"InstanceMethods".to_string()));
        assert!(symbols.contains(&"Something".to_string()));
        assert!(symbols.contains(&"bar".to_string()));
        assert!(symbols.contains(&"qux".to_string()));
        assert!(symbols.contains(&"included".to_string()));
    }
}
