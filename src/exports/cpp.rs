use regex::Regex;
use std::sync::LazyLock;

// `(?m)` is required so `$` anchors to each line's end, not just the end of
// the whole input: without it, a `//` comment followed by more source (e.g.
// `void f() // note\n{`) is never stripped, and a same-line trailing comment
// sitting between a signature and its `{`/`;` terminator can make
// `CPP_FUNCTION` fail to match, silently dropping a real declaration.
static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)//.*$").unwrap());
static COMMENT_MULTI: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/").unwrap());

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

/// Extract public symbols from C++ source code.
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

    for caps in CPP_TYPE.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            symbols.push(name.as_str().to_string());
        }
    }

    for caps in CPP_FUNCTION.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str().to_string();
            if !keywords.contains(n.as_str()) {
                let full_match = caps.get(0).unwrap().as_str();
                if !full_match.contains("static")
                    && !full_match.contains("private:")
                    && !full_match.contains("protected:")
                {
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
