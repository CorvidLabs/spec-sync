use regex::Regex;
use std::sync::LazyLock;

/// Single-line # comments
static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)#.*$").unwrap());

/// Multi-line =begin/=end comments
static COMMENT_MULTI: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?ms)^=begin.*?^=end").unwrap());

/// Class declarations: class Name or class Name < Parent
static RUBY_CLASS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[^\S\n]*class\s+([A-Z]\w*)").unwrap());

/// Module declarations: module Name
static RUBY_MODULE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[^\S\n]*module\s+([A-Z]\w*)").unwrap());

/// Top-level method definitions (at zero indentation, considered public)
static RUBY_DEF: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^def\s+(?:self\.)?(\w+)").unwrap());

/// Instance method definitions (indented, inside a class — public by default)
static RUBY_METHOD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[^\S\n]+def\s+(?:self\.)?(\w+)").unwrap());

/// Constant assignments: NAME = value (uppercase first letter)
static RUBY_CONST: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[^\S\n]*([A-Z][A-Z0-9_]+)\s*=").unwrap());

/// attr_accessor / attr_reader / attr_writer declarations
static RUBY_ATTR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^[^\S\n]*attr_(?:accessor|reader|writer)\s+(.+)$").unwrap()
});

/// Symbol literal :name
static RUBY_SYMBOL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r":(\w+)").unwrap());

/// private / protected visibility markers
static VISIBILITY_PRIVATE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\s*(?:private|protected)\s*$").unwrap());

static VISIBILITY_PUBLIC: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\s*public\s*$").unwrap());

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
            symbols.push(name.as_str().to_string());
        }
    }

    for caps in RUBY_MODULE.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str().to_string();
            if !symbols.contains(&n) {
                symbols.push(n);
            }
        }
    }

    // Top-level defs (zero indentation) are always public
    for caps in RUBY_DEF.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str();
            if !n.starts_with('_') && !symbols.contains(&n.to_string()) {
                symbols.push(n.to_string());
            }
        }
    }

    // Track visibility for indented methods (inside classes)
    // Walk lines, toggle visibility state
    let mut public = true;
    for line in stripped.lines() {
        if VISIBILITY_PRIVATE.is_match(line) {
            public = false;
            continue;
        }
        if VISIBILITY_PUBLIC.is_match(line) {
            public = true;
            continue;
        }

        if public {
            if let Some(caps) = RUBY_METHOD.captures(line) {
                if let Some(name) = caps.get(1) {
                    let n = name.as_str();
                    if !n.starts_with('_')
                        && n != "initialize"
                        && !symbols.contains(&n.to_string())
                    {
                        symbols.push(n.to_string());
                    }
                }
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
}
