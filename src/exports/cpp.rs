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

/// Matches a `class`/`struct`/`union` header line, capturing the keyword (group 1) so
/// the caller can pick the section's default visibility (`class` is private by
/// default, `struct`/`union` are public) and the type's own name (group 2) so
/// constructor/destructor declarations inside its body can be recognized. Anchored to
/// the line's first token so `enum class Channel { ... }` doesn't match (its first
/// token is `enum`, handled separately by `CPP_TYPE`).
static CLASS_HEADER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[^\S\n]*(class|struct|union)\s+(\w+)").unwrap());

/// An anonymous namespace (`namespace { ... }` or `namespace` with the brace on the
/// next line): its own name is absent, and — unlike a *named* namespace — every
/// entity defined inside one has internal linkage, i.e. is private to this
/// translation unit and never part of the public API, even though the members
/// themselves may be written with no visibility label at all (namespaces don't have
/// `public:`/`private:` sections). Anchored so a named namespace (`namespace Foo {`)
/// never matches: the name token between `namespace` and `{`/EOL breaks the match.
static ANON_NAMESPACE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[^\S\n]*namespace(?:[^\S\n]*\{)?[^\S\n]*$").unwrap());

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
/// `override`, `final`) and/or a pure-virtual/defaulted/deleted specifier (`= 0`,
/// `= default`, `= delete`) to appear between the closing `)` and the terminating
/// `{`/`;`, in the order the C++ grammar requires them. Also allows a trailing
/// return type (`auto name(...) -> Type`) in the same trailing position.
///
/// The parameter list allows one level of nested parens (`(?:[^()]|\([^()]*\))*`)
/// so a callback parameter of function-pointer type (e.g. `int register_callback(int
/// (*cb)(int), void *ctx);`) doesn't break the match: a flat `[^)]*` parameter list
/// stops at the callback's own closing `)`, leaving the real terminator unreachable
/// from that position and silently dropping the entire declaration.
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
        r"(?m)^[^\S\n]*(?:[\w:*&<>~]+[^\S\n]+)+\*?(\w+)\s*\((?:[^()]|\([^()]*\))*\)\s*(?:const\s*)?(?:noexcept\s*)?(?:override\s*)?(?:final\s*)?(?:=\s*(?:0|default|delete)\s*)?(?:->\s*[\w:*&<>,\s]+)?[{;]",
    )
    .unwrap()
});

/// Matches a constructor or destructor declaration/definition for a known enclosing
/// class name, e.g. `Widget(int x);`, `~Widget();`, `Widget() = default;`. Unlike
/// ordinary methods, special member functions have no return-type prefix at all — the
/// class name itself fills that role — so `CPP_FUNCTION`'s "one-or-more
/// word-then-space prefix before the name" shape structurally cannot match them
/// (and deliberately shouldn't be loosened to allow a bare `name(args);` shape, since
/// that would also start matching ordinary function-call statements with no return
/// value, e.g. `doWork(x, y);`, as if they were declarations). Built as a dynamic
/// regex keyed on the specific class name so only that class's own constructors and
/// destructor are recognized, never an unrelated call to a same-named free function.
fn ctor_dtor_name(line: &str, class_name: &str) -> Option<String> {
    let pattern = format!(
        r"^[^\S\n]*(~)?{}\s*\((?:[^()]|\([^()]*\))*\)\s*(?:const\s*)?(?:noexcept\s*)?(?:=\s*(?:default|delete)\s*)?[{{;]",
        regex::escape(class_name)
    );
    let re = Regex::new(&pattern).ok()?;
    let caps = re.captures(line)?;
    Some(format!("{}{}", caps.get(1).map_or("", |_| "~"), class_name))
}

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
    // Parallel stack to `brace_stack`: `Some(name)` for a brace opened by a
    // class/struct/union header (so constructor/destructor declarations inside can be
    // recognized against the enclosing type's own name), `None` for any other brace.
    let mut class_name_stack: Vec<Option<String>> = Vec::new();
    let mut pending_class_name: Option<String> = None;

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
        let current_class_name = class_name_stack.iter().rev().find_map(|n| n.clone());

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

        if current_visibility == Visibility::Public
            && let Some(class_name) = &current_class_name
            && let Some(n) = ctor_dtor_name(rest, class_name)
            && !symbols.contains(&n)
        {
            symbols.push(n);
        }

        if let Some(caps) = CLASS_HEADER.captures(rest) {
            pending_visibility = Some(if &caps[1] == "class" {
                Visibility::Hidden
            } else {
                Visibility::Public
            });
            pending_class_name = Some(caps[2].to_string());
        } else if ANON_NAMESPACE.is_match(rest) {
            // Internal-linkage scope: members defined inside have no visibility
            // label of their own but must not be treated as part of the public API.
            pending_visibility = Some(Visibility::Hidden);
            pending_class_name = None;
        }

        for ch in rest.chars() {
            match ch {
                '{' => {
                    brace_stack.push(pending_visibility.take());
                    class_name_stack.push(pending_class_name.take());
                }
                '}' => {
                    brace_stack.pop();
                    class_name_stack.pop();
                }
                ';' => {
                    pending_visibility = None;
                    pending_class_name = None;
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

    #[test]
    fn test_constructor_and_destructor_captured() {
        // Bug found via adversarial probing: constructors/destructors have no
        // return-type prefix at all (the class name fills that role), so
        // `CPP_FUNCTION`'s "one-or-more word-then-space prefix before the name" shape
        // structurally could never match `Widget(int x);` or `~Widget();` -- every
        // constructor and destructor in every public class silently vanished from the
        // extracted API surface. Also covers `= default`/`= delete` special-member
        // specifiers, which the old trailing-qualifier group didn't recognize either.
        let src = r#"
class Widget {
public:
    Widget(int x);
    Widget();
    ~Widget();
    Widget(const Widget&) = default;
    Widget& assignSelf() { return *this; }
private:
    Widget(double hidden);
};
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Widget".to_string()));
        assert!(
            symbols.contains(&"~Widget".to_string()),
            "expected destructor ~Widget in {symbols:?}"
        );
        assert!(symbols.contains(&"assignSelf".to_string()));
    }

    #[test]
    fn test_trailing_return_type_arrow_syntax_captured() {
        // Bug found via adversarial probing: trailing return type syntax (`auto
        // name(...) -> Type`, common in modern C++11+ code, especially templates) put
        // extra text after the closing `)` that none of the recognized trailing
        // qualifiers (`const`/`noexcept`/`override`/`final`/`= 0`) matched, so the
        // whole declaration failed and `compute` was dropped.
        let src = r#"
class Calculator {
public:
    auto compute(int a, int b) -> int;
    auto name() const -> std::string;
};
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"compute".to_string()),
            "expected compute in {symbols:?}"
        );
        assert!(
            symbols.contains(&"name".to_string()),
            "expected name in {symbols:?}"
        );
    }

    #[test]
    fn test_function_pointer_callback_parameter_not_dropped() {
        // Bug found via adversarial probing, same class of bug as the C backend:
        // `CPP_FUNCTION`'s parameter-list group was a flat `[^)]*`, which can never
        // contain a `)`. A callback parameter of function-pointer type has its own
        // closing `)` before the real parameter list ends, so the flat pattern
        // stopped there and the terminator was unreachable -- the entire declaration
        // silently failed to match.
        let src = r#"
int register_callback(int (*cb)(int), void *ctx);
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"register_callback".to_string()),
            "expected register_callback in {symbols:?}"
        );
        assert!(!symbols.contains(&"cb".to_string()));
    }

    #[test]
    fn test_anonymous_namespace_members_not_exported() {
        // Bug found via adversarial probing: entities defined inside an anonymous
        // namespace have internal linkage (private to the translation unit) even
        // though the members carry no `private`/`protected` label of their own --
        // namespaces don't have access-specifier sections. Previously an anonymous
        // namespace's brace pushed `None` onto the visibility stack (the same as any
        // other non-class brace), which inherits the *enclosing* visibility rather
        // than hiding its own members, so a free function defined only for internal
        // use inside `namespace { ... }` at file scope leaked out as if it were a
        // public top-level function.
        let src = r#"
namespace {
    void hiddenLinkage() {}
}

namespace utils {
    void visibleHelper() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(
            !symbols.contains(&"hiddenLinkage".to_string()),
            "anonymous namespace member must not be exported: {symbols:?}"
        );
        assert!(
            symbols.contains(&"visibleHelper".to_string()),
            "named namespace member must still be exported: {symbols:?}"
        );
    }
}
