use regex::Regex;
use std::sync::LazyLock;

static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"#.*$").unwrap());

/// Crystal public declarations: class, module, struct, enum, def, alias,
/// property, getter, setter. An optional leading `abstract` modifier (as in
/// `abstract class Foo` or `abstract def bar`) is skipped so it doesn't
/// defeat the line anchor. A `def` name may be qualified with `self.` (class
/// methods, e.g. `def self.foo`) which is skipped so the captured name is
/// the actual method name rather than the literal word `self`. The name may
/// also be written as a symbol (e.g. `property :name`, `getter :label`),
/// which is Crystal's idiomatic macro-accessor shorthand, so a leading `:`
/// is likewise skipped. The captured name may carry a single trailing `?`,
/// `!`, or `=` since those are valid Crystal identifier characters (e.g.
/// `valid?`, `save!`, `x=`).
static CRYSTAL_DECL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*(?:abstract\s+)?(?:class|module|struct|enum|def|alias|property|getter|setter)\s+(?:self\.)?:?(\w+[?!=]?)",
    )
    .unwrap()
});

/// Exclude private/protected declarations
static CRYSTAL_PRIVATE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[^\S\n]*(?:private|protected)").unwrap());

/// Extract public symbols from Crystal source code.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = COMMENT_SINGLE.replace_all(content, "");

    let mut symbols = Vec::new();

    for line in stripped.lines() {
        if CRYSTAL_PRIVATE.is_match(line) {
            continue;
        }
        if let Some(caps) = CRYSTAL_DECL.captures(line)
            && let Some(name) = caps.get(1)
        {
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
    fn test_crystal_exports() {
        let src = r#"
# Crystal program
module MyLib
  class User
    def initialize(@name : String)
    end
    def get_name
      @name
    end
  end
  private class Helper
  end
end
def global_func
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"MyLib".to_string()));
        assert!(symbols.contains(&"User".to_string()));
        assert!(symbols.contains(&"get_name".to_string()));
        assert!(symbols.contains(&"global_func".to_string()));
        assert!(!symbols.contains(&"Helper".to_string()));
    }

    #[test]
    fn test_crystal_abstract_module_interface() {
        // `abstract def` inside a module previously vanished entirely because
        // the leading `abstract` token defeated the line-anchored regex.
        let src = r#"
module Authenticatable
  abstract def validate : Bool
  abstract def user_id : Int64

  def authenticated? : Bool
    validate
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Authenticatable".to_string()));
        assert!(symbols.contains(&"validate".to_string()));
        assert!(symbols.contains(&"user_id".to_string()));
        // Method names ending in `?` must be captured in full, not truncated.
        assert!(symbols.contains(&"authenticated?".to_string()));
        assert!(!symbols.contains(&"authenticated".to_string()));
    }

    #[test]
    fn test_crystal_abstract_class_hierarchy() {
        // The `abstract` modifier can prefix `class`/`struct` themselves too,
        // which used to drop the container's own name from the output.
        let src = r#"
abstract class Shape
  abstract def area : Float64
  abstract def perimeter : Float64

  def describe : String
    "Area: #{area}, Perimeter: #{perimeter}"
  end
end

abstract struct Money
  abstract def amount : Int32
  abstract def currency : String
end

class Circle < Shape
  def initialize(@radius : Float64)
  end

  def area : Float64
    Math::PI * @radius ** 2
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Shape".to_string()));
        assert!(symbols.contains(&"area".to_string()));
        assert!(symbols.contains(&"perimeter".to_string()));
        assert!(symbols.contains(&"describe".to_string()));
        assert!(symbols.contains(&"Money".to_string()));
        assert!(symbols.contains(&"amount".to_string()));
        assert!(symbols.contains(&"currency".to_string()));
        assert!(symbols.contains(&"Circle".to_string()));
    }

    #[test]
    fn test_crystal_property_getter_setter_macros() {
        // `property`/`getter`/`setter` are Crystal's idiomatic public-accessor
        // generators (like attr_accessor) and previously weren't recognized
        // as declarations at all.
        let src = r#"
class Point
  property x : Int32
  property y : Int32
  getter label : String = "point"

  def initialize(@x : Int32, @y : Int32)
  end

  def to_s(io : IO) : Nil
    io << "(#{x}, #{y})"
  end

  private def save!
    true
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Point".to_string()));
        assert!(symbols.contains(&"x".to_string()));
        assert!(symbols.contains(&"y".to_string()));
        assert!(symbols.contains(&"label".to_string()));
        assert!(symbols.contains(&"initialize".to_string()));
        assert!(symbols.contains(&"to_s".to_string()));
        // Explicit `private` on a member still excludes it.
        assert!(!symbols.contains(&"save!".to_string()));
    }

    #[test]
    fn test_crystal_learnxinyminutes_class_methods_and_accessors() {
        // Real excerpt (trimmed) from learnxinyminutes.com/crystal: a class
        // with instance/class variables, an initializer, instance
        // getter/setter defs, the `property`/`getter`/`setter` macro
        // shorthand (symbol-argument form), and a `self.`-qualified class
        // method. `def self.say` previously leaked the literal word `self`
        // into the results while dropping the real method name entirely.
        let src = r#"
class Human

  # A class variable. It is shared by all instances of this class.
  @@species = "H. sapiens"

  # An instance variable. Type of name is String
  @name : String

  # Basic initializer
  def initialize(@name, @age = 0)
  end

  # Basic setter method
  def name=(name)
    @name = name
  end

  # Basic getter method
  def name
    @name
  end

  # The above functionality can be encapsulated using the property method as follows
  property :name

  # Getter/setter methods can also be created individually like this
  getter :name
  setter :name

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
        assert!(symbols.contains(&"initialize".to_string()));
        assert!(symbols.contains(&"name=".to_string()));
        assert!(symbols.contains(&"name".to_string()));
        assert!(symbols.contains(&"species".to_string()));
        // `def self.say` must yield the real method name `say`, not the
        // literal word `self`.
        assert!(symbols.contains(&"say".to_string()));
        assert!(!symbols.contains(&"self".to_string()));
    }

    #[test]
    fn test_crystal_learnxinyminutes_class_method_setter_and_derived_class() {
        // Real excerpt from learnxinyminutes.com/crystal: class-scope
        // (`@@`) state exposed via `self.`-qualified reader and writer
        // methods, plus a derived class with an empty body. Exercises that
        // `def self.foo=(value)` resolves to `foo=` (not `self`) and that a
        // subclass declaration on its own line is still captured.
        let src = r#"
class Human
  @@foo = 0

  def self.foo
    @@foo
  end

  def self.foo=(value)
    @@foo = value
  end
end

class Worker < Human
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Human".to_string()));
        assert!(symbols.contains(&"foo".to_string()));
        assert!(symbols.contains(&"foo=".to_string()));
        assert!(symbols.contains(&"Worker".to_string()));
        assert!(!symbols.contains(&"self".to_string()));
    }
}
