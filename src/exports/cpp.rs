use regex::Regex;
use std::sync::LazyLock;

// `(?m)` is required so `$` anchors to each line's end, not just the end of
// the whole input: without it, a `//` comment followed by more source (e.g.
// `void f() // note\n{`) is never stripped, and a same-line trailing comment
// sitting between a signature and its `{`/`;` terminator can make
// `CPP_FUNCTION` fail to match, silently dropping a real declaration.
static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)//.*$").unwrap());
static COMMENT_MULTI: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/").unwrap());

/// Matches a line that is (or starts with) a bare access-specifier label,
/// e.g. `public:` or `  protected:  // notes`. Anchored to the start of the
/// line so `class Foo : public Bar {` (an inheritance specifier, not a
/// label) never matches: the keyword there isn't the line's first token.
static ACCESS_LABEL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[^\S\n]*(public|private|protected)[^\S\n]*:(.*)$").unwrap());

/// Matches a `class`/`struct`/`union` header line, capturing the keyword so
/// the caller can pick the section's default visibility (`class` is
/// private by default, `struct`/`union` are public). Anchored to the line's
/// first token so `enum class Channel { ... }` doesn't match (its first
/// token is `enum`, handled separately by `CPP_TYPE`).
static CLASS_HEADER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[^\S\n]*(class|struct|union)\s+\w").unwrap());

/// C++ types: class, struct, union, enum, namespace. `enum class`/`enum
/// struct` (scoped enums) are also recognized, capturing the enum's own
/// name rather than the `class`/`struct` keyword that follows `enum`.
static CPP_TYPE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*(?:class|struct|union|namespace|enum(?:\s+(?:class|struct))?)\s+(\w+)",
    )
    .unwrap()
});

/// C++ top-level and method functions (which could include keywords, filtered in rust).
/// Allows the trailing member-function qualifiers (`const`, `noexcept`,
/// `override`, `final`) and/or a pure-virtual `= 0` specifier to appear
/// between the closing `)` and the terminating `{`/`;`, in the order the
/// C++ grammar requires them.
///
/// The return-type/qualifier prefix uses `[^\S\n]+` (not `\s+`) between
/// tokens so the match cannot bridge a newline: with a plain `\s+`, a
/// `goto` label (e.g. `failure:`) immediately followed on the next line by
/// an ordinary function *call* statement (e.g. `fclose(fh);`) would have
/// its label text absorbed as a bogus "return type", making the regex
/// misidentify the call as a declaration and leak the called function's
/// name (e.g. `fclose`) as an exported symbol.
static CPP_FUNCTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*(?:[\w:*&<>~]+[^\S\n]+)+\*?(\w+)\s*\([^)]*\)\s*(?:const\s*)?(?:noexcept\s*)?(?:override\s*)?(?:final\s*)?(?:=\s*0\s*)?[{;]",
    )
    .unwrap()
});

/// `CPP_FUNCTION`'s "one-or-more word-then-space" prefix is meant to match a
/// return-type (and qualifiers like `static`/`const`) before a function name,
/// but a bare `return someFunc(a, b);` or `throw makeError(a, b);` statement
/// has exactly the same shape — `return`/`throw` satisfies the prefix, and
/// `someFunc`/`makeError` gets captured as if it were a declared function
/// name, leaking whatever it calls (even a `private:`/`static` function) as
/// an export. Unlike the keyword filter on the *captured name*, this checks
/// whether `return`/`throw` is what's actually filling the prefix slot.
fn starts_with_return_or_throw(full_match: &str) -> bool {
    let trimmed = full_match.trim_start();
    for kw in ["return", "throw"] {
        if let Some(rest) = trimmed.strip_prefix(kw)
            && rest.starts_with(|c: char| c.is_whitespace())
        {
            return true;
        }
    }
    false
}

#[derive(Clone, Copy, PartialEq)]
enum Visibility {
    Public,
    Hidden,
}

/// Extract public symbols from C++ source code.
///
/// Type names (class/struct/union/enum/namespace) are captured regardless
/// of the enclosing section's visibility, matching the AST backend's
/// documented behavior: the regex extractor only line-matches type
/// keywords, without access context. Methods/functions, however, are only
/// exported when the line they're declared on falls under a `public:`
/// section (or no section at all, e.g. free functions and `struct`/`union`
/// members before any explicit label). Access-specifier state is tracked
/// per class body across a brace-depth stack, so a `private:`/`protected:`
/// label on its own line (the overwhelmingly common style) correctly hides
/// every member declared after it until the next label or the class's
/// closing brace — unlike a same-line-only text scan, which never sees the
/// label at all.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = COMMENT_SINGLE.replace_all(content, "");
    let stripped = COMMENT_MULTI.replace_all(&stripped, "");

    let keywords: std::collections::HashSet<&str> = [
        "static",
        "private",
        "protected",
        "return",
        "if",
        "while",
        "for",
        "switch",
        "typedef",
        "struct",
        "class",
        "union",
        "enum",
        "namespace",
        "using",
        "friend",
        "inline",
        "virtual",
        "const",
        "constexpr",
        "extern",
        "void",
        "int",
        "char",
        "float",
        "double",
    ]
    .iter()
    .cloned()
    .collect();

    let mut symbols = Vec::new();
    // `Some(visibility)` for a brace opened by a class/struct/union body;
    // `None` for any other brace (namespace, function/method body, block
    // statement) so depth stays balanced without perturbing the enclosing
    // class's visibility.
    let mut brace_stack: Vec<Option<Visibility>> = Vec::new();
    // Default visibility captured from a `class`/`struct`/`union` header,
    // waiting to be applied to the body's opening `{`. Cleared on `;`
    // first so a forward declaration (`class Foo;`) doesn't leak into the
    // next unrelated brace.
    let mut pending_visibility: Option<Visibility> = None;

    for line in stripped.split('\n') {
        let mut rest = line;

        if let Some(caps) = ACCESS_LABEL.captures(line) {
            let visibility = if &caps[1] == "public" {
                Visibility::Public
            } else {
                Visibility::Hidden
            };
            if let Some(Some(current)) = brace_stack.last_mut() {
                *current = visibility;
            }
            rest = caps.get(2).map_or("", |m| m.as_str());
        }

        let current_visibility = brace_stack
            .iter()
            .rev()
            .find_map(|scope| *scope)
            .unwrap_or(Visibility::Public);

        for caps in CPP_TYPE.captures_iter(rest) {
            if let Some(name) = caps.get(1) {
                let n = name.as_str().to_string();
                if !symbols.contains(&n) {
                    symbols.push(n);
                }
            }
        }

        for caps in CPP_FUNCTION.captures_iter(rest) {
            if let Some(name) = caps.get(1) {
                let n = name.as_str().to_string();
                if !keywords.contains(n.as_str()) {
                    let full_match = caps.get(0).unwrap().as_str();
                    if !full_match.contains("static")
                        && !starts_with_return_or_throw(full_match)
                        && current_visibility == Visibility::Public
                        && !symbols.contains(&n)
                    {
                        symbols.push(n);
                    }
                }
            }
        }

        if let Some(caps) = CLASS_HEADER.captures(rest) {
            pending_visibility = Some(if &caps[1] == "class" {
                Visibility::Hidden
            } else {
                Visibility::Public
            });
        }

        for ch in rest.chars() {
            match ch {
                '{' => brace_stack.push(pending_visibility.take()),
                '}' => {
                    brace_stack.pop();
                }
                ';' => pending_visibility = None,
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
    fn test_cpp_exports() {
        let src = r#"
namespace Math {
    class Calculator {
        public:
            int add(int a, int b);
    };
}
struct Point {
    double x, y;
};
void greetUser() {
}
static void localHelper() {
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Math".to_string()));
        assert!(symbols.contains(&"Calculator".to_string()));
        assert!(symbols.contains(&"Point".to_string()));
        assert!(symbols.contains(&"greetUser".to_string()));
        assert!(symbols.contains(&"add".to_string()));
        assert!(!symbols.contains(&"localHelper".to_string()));
    }

    #[test]
    fn test_private_protected_sections_excluded() {
        // The access-specifier label sits on its own line, so a same-match
        // text scan for "private:"/"protected:" (checking only the matched
        // function's own line) never sees it — every member after the
        // label used to leak as exported regardless of section.
        let src = r#"
class Foo {
private:
    void secretMethod();
    int hiddenValue();
protected:
    void protectedMethod();
public:
    void reveal();
};
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Foo".to_string()));
        assert!(symbols.contains(&"reveal".to_string()));
        assert!(!symbols.contains(&"secretMethod".to_string()));
        assert!(!symbols.contains(&"hiddenValue".to_string()));
        assert!(!symbols.contains(&"protectedMethod".to_string()));
    }

    #[test]
    fn test_class_defaults_private_struct_defaults_public() {
        // A `class` body's members before any explicit label are private
        // by default; a `struct` body's are public by default.
        let src = r#"
class Foo {
    void hiddenByDefault();
};
struct Bar {
    void visibleByDefault();
};
"#;
        let symbols = extract_exports(src);
        assert!(!symbols.contains(&"hiddenByDefault".to_string()));
        assert!(symbols.contains(&"visibleByDefault".to_string()));
    }

    #[test]
    fn test_nested_class_in_private_section_type_still_captured() {
        // Nested type names are captured regardless of the enclosing
        // section's visibility, matching the AST backend and the regex
        // extractor's original (correct) behavior for type declarations.
        let src = r#"
class Outer {
private:
    class Inner {
    public:
        void innerMethod();
    };
};
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Inner".to_string()));
        assert!(symbols.contains(&"innerMethod".to_string()));
    }

    #[test]
    fn test_return_and_throw_call_not_misparsed_as_declaration() {
        // Regression test: `CPP_FUNCTION`'s "one-or-more word-then-space" prefix (meant
        // to match a return type/qualifiers) is also satisfied by the single word
        // `return`/`throw`, so `return helperSum(a, b);` or `throw makeError(x);` was
        // misparsed as a declaration -- leaking the called function even when it's
        // `static` at namespace scope or `private:` inside a class.
        let src = r#"
static int helperSum(int a, int b) {
    return a + b;
}

int publicWrapper(int a, int b) {
    return helperSum(a, b);
}

class Widget {
private:
    void secretHelper() {}
public:
    void reveal() {
        throw secretHelper();
    }
};
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"publicWrapper".to_string()));
        assert!(!symbols.contains(&"helperSum".to_string()));
        assert!(!symbols.contains(&"secretHelper".to_string()));
    }

    #[test]
    fn test_trailing_qualifiers_do_not_hide_methods() {
        // `const`, `const noexcept`, and `const override` between the `)`
        // and the terminating `;` used to prevent CPP_FUNCTION from
        // matching at all, silently dropping every qualified method.
        let src = r#"
namespace payments {
class PaymentProcessor {
public:
    virtual bool authorize(double amount) const = 0;
    virtual void capture(const std::string& id) = 0;
    virtual double refundableAmount() const noexcept = 0;
};

class StripeProcessor : public PaymentProcessor {
public:
    bool authorize(double amount) const override;
    std::string lastError() const;
};
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"authorize".to_string()));
        assert!(symbols.contains(&"capture".to_string()));
        assert!(symbols.contains(&"refundableAmount".to_string()));
        assert!(symbols.contains(&"lastError".to_string()));
    }

    #[test]
    fn test_template_class_const_noexcept_methods_captured() {
        let src = r#"
template<typename T>
class Stack {
public:
    void push(const T& value);
    T& top();
    bool empty() const noexcept;
    size_t size() const noexcept;
};
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"push".to_string()));
        assert!(symbols.contains(&"top".to_string()));
        assert!(symbols.contains(&"empty".to_string()));
        assert!(symbols.contains(&"size".to_string()));
    }

    #[test]
    fn test_enum_class_captures_real_name_not_keyword() {
        // `enum class Name { ... }` used to capture the literal "class"
        // keyword as the symbol name instead of "Channel".
        let src = "enum class Channel { Red, Green, Blue, Alpha };";
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Channel".to_string()));
        assert!(!symbols.contains(&"class".to_string()));
    }
}
