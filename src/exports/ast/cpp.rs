use tree_sitter::{Node, Parser, Tree};

/// Parse C++ source into a tree-sitter AST.
fn parse_cpp(content: &str) -> Option<Tree> {
    let mut parser = Parser::new();
    let language = tree_sitter_cpp::LANGUAGE.into();
    parser.set_language(&language).ok()?;
    parser.parse(content, None)
}

#[derive(Clone, Copy, PartialEq)]
enum Visibility {
    Public,
    Private,
    Protected,
}

/// Extract exported symbols from C++ source using tree-sitter AST.
///
/// Superset of C: class/struct/union/enum/namespace names, plus methods
/// declared or defined anywhere (including inside a class body under a
/// `public:` section) as long as they aren't `static`/`private`/`protected`.
/// `class` bodies default to private visibility, `struct`/`union` bodies
/// default to public, matching real C++ semantics.
pub fn extract_exports(content: &str) -> Vec<String> {
    let tree = match parse_cpp(content) {
        Some(t) => t,
        None => return Vec::new(),
    };

    let root = tree.root_node();
    let src = content.as_bytes();
    let mut symbols = Vec::new();

    walk(&root, src, &mut symbols);

    symbols
}

/// Walk a node's direct children, dispatching each to `handle_node`.
/// Used for "unrestricted" scopes (top level, namespace bodies, function
/// bodies) where there is no access-specifier state to track.
fn walk(node: &Node, src: &[u8], symbols: &mut Vec<String>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        handle_node(&child, src, symbols);
    }
}

fn handle_node(node: &Node, src: &[u8], symbols: &mut Vec<String>) {
    match node.kind() {
        "namespace_definition" => {
            add_type_name(node, src, symbols);
            if let Some(body) = node.child_by_field_name("body") {
                walk(&body, src, symbols);
            }
        }
        "class_specifier" | "struct_specifier" | "union_specifier" => {
            add_type_name(node, src, symbols);
            let default_vis = if node.kind() == "class_specifier" {
                Visibility::Private
            } else {
                Visibility::Public
            };
            if let Some(body) = node.child_by_field_name("body") {
                walk_field_list(&body, src, default_vis, symbols);
            }
        }
        "enum_specifier" => {
            add_type_name(node, src, symbols);
        }
        "function_definition" | "declaration" => {
            handle_nested_type_in_declaration(node, src, symbols);
            if !has_static(node, src) {
                add_function_name(node, src, symbols);
            }
            if let Some(body) = node.child_by_field_name("body") {
                walk(&body, src, symbols);
            }
        }
        _ => {
            // Recurse generically to find nested items inside constructs we
            // don't special-case (extern "C" blocks, preproc conditionals,
            // linkage specifications, etc.) — mirrors the regex extractor
            // scanning the whole file regardless of surrounding syntax.
            walk(node, src, symbols);
        }
    }
}

/// Walk the body of a class/struct/union, tracking `public:`/`private:`/
/// `protected:` access-specifier state across siblings.
fn walk_field_list(node: &Node, src: &[u8], mut vis: Visibility, symbols: &mut Vec<String>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "access_specifier" => {
                let text = child.utf8_text(src).unwrap_or_default();
                vis = match text {
                    "public" => Visibility::Public,
                    "private" => Visibility::Private,
                    "protected" => Visibility::Protected,
                    _ => vis,
                };
            }
            "field_declaration" | "function_definition" => {
                // Nested type definitions are captured regardless of the
                // enclosing section's visibility (matches the regex, which
                // only line-matches type keywords, not access context).
                // Note: unlike a bare top-level `class Foo { ... };`, the
                // C++ grammar always wraps a member type definition (e.g.
                // `class Inner { ... };` inside a class body) in a
                // `field_declaration` node with the specifier as its `type`
                // field — it is never a direct `field_declaration_list`
                // child, so this must be unwrapped explicitly.
                handle_nested_type_in_declaration(&child, src, symbols);
                if vis == Visibility::Public && !has_static(&child, src) {
                    add_function_name(&child, src, symbols);
                }
                if let Some(body) = child.child_by_field_name("body") {
                    walk(&body, src, symbols);
                }
            }
            "class_specifier"
            | "struct_specifier"
            | "union_specifier"
            | "enum_specifier"
            | "namespace_definition" => {
                // Defensive: not currently produced as a direct child by
                // this grammar (see note above), but handle it the same way
                // if a future grammar version ever does.
                handle_node(&child, src, symbols);
            }
            "friend_declaration" => {
                // `friend std::ostream& operator<<(...);` wraps an inner
                // `declaration` node (the friended function's prototype) as
                // its only named child. Route it through the normal
                // `declaration` handling (which itself checks `static` and
                // extracts the function name) — gated on the current
                // section's visibility like any other member declaration.
                if vis == Visibility::Public {
                    walk(&child, src, symbols);
                }
            }
            "alias_declaration"
                // `using Pixel = std::uint32_t;` inside a class body is a
                // type alias member; treat it like any other public member.
                if vis == Visibility::Public => {
                    add_type_name(&child, src, symbols);
                }
            _ => {}
        }
    }
}

/// If `node`'s `type` field is itself a nested class/struct/union/enum
/// definition (e.g. the `class Inner { ... }` in a member declaration
/// `class Inner { ... };`, or a top-level `struct Point { ... } origin;`),
/// dispatch it to `handle_node` so its name — and any qualifying members —
/// are captured. This mirrors the regex extractor, which line-matches the
/// `class`/`struct`/`union`/`enum` keyword regardless of what (if anything)
/// follows the closing brace or which scope contains it.
fn handle_nested_type_in_declaration(node: &Node, src: &[u8], symbols: &mut Vec<String>) {
    if let Some(type_node) = node.child_by_field_name("type")
        && matches!(
            type_node.kind(),
            "class_specifier" | "struct_specifier" | "union_specifier" | "enum_specifier"
        )
    {
        handle_node(&type_node, src, symbols);
    }
}

/// True if `node` has a direct `storage_class_specifier` child whose text is
/// `static`.
fn has_static(node: &Node, src: &[u8]) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "storage_class_specifier"
            && child.utf8_text(src).unwrap_or_default() == "static"
        {
            return true;
        }
    }
    false
}

/// Push a class/struct/union/enum/namespace name (no dedup, matching the
/// regex extractor's type-name pass).
fn add_type_name(node: &Node, src: &[u8], symbols: &mut Vec<String>) {
    if let Some(name) = node.child_by_field_name("name") {
        symbols.push(name.utf8_text(src).unwrap_or_default().to_string());
    }
}

/// Push a function/method name, deduped (matching the regex extractor's
/// function pass). Requires a `type` field to be present, which excludes
/// constructors/destructors — the regex extractor's function pattern also
/// requires a return-type token before the name, so it never matches those.
fn add_function_name(node: &Node, src: &[u8], symbols: &mut Vec<String>) {
    if node.child_by_field_name("type").is_none() {
        return;
    }
    let Some(declarator) = node.child_by_field_name("declarator") else {
        return;
    };
    let Some(func_decl) = find_function_declarator(&declarator) else {
        return;
    };
    let Some(name_node) = func_decl.child_by_field_name("declarator") else {
        return;
    };
    if matches!(
        name_node.kind(),
        "identifier" | "field_identifier" | "operator_name"
    ) {
        let name = name_node.utf8_text(src).unwrap_or_default().to_string();
        if !symbols.contains(&name) {
            symbols.push(name);
        }
    }
}

/// Drill through wrapper declarators (`pointer_declarator`,
/// `reference_declarator`, etc.) to find the innermost `function_declarator`.
///
/// `pointer_declarator` exposes its inner declarator via a named
/// `declarator` field, but `reference_declarator` (e.g. the `int&` in
/// `int& getRef();`) does not carry a named field for its child in this
/// grammar — so a plain `child_by_field_name("declarator")` lookup misses
/// it. Fall back to scanning all children (named or not) for any wrapper
/// that leads to a `function_declarator`.
fn find_function_declarator<'tree>(node: &Node<'tree>) -> Option<Node<'tree>> {
    if node.kind() == "function_declarator" {
        return Some(*node);
    }
    if let Some(inner) = node.child_by_field_name("declarator") {
        return find_function_declarator(&inner);
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(found) = find_function_declarator(&child) {
            return Some(found);
        }
    }
    None
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
    fn test_class_defaults_private() {
        // `class` bodies default to private; a method with no explicit
        // access-specifier section should NOT be captured.
        let src = r#"
class Foo {
    int defaultPrivate();
};
"#;
        let symbols = extract_exports(src);
        assert!(!symbols.contains(&"defaultPrivate".to_string()));
        assert!(symbols.contains(&"Foo".to_string()));
    }

    #[test]
    fn test_struct_defaults_public() {
        // `struct` bodies default to public.
        let src = r#"
struct Bar {
    int defaultPublic();
};
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"defaultPublic".to_string()));
        assert!(symbols.contains(&"Bar".to_string()));
    }

    #[test]
    fn test_private_protected_sections_excluded() {
        let src = r#"
class Widget {
    public:
        int visibleMethod();
    private:
        int secretMethod();
    protected:
        int protectedMethod();
};
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"visibleMethod".to_string()));
        assert!(!symbols.contains(&"secretMethod".to_string()));
        assert!(!symbols.contains(&"protectedMethod".to_string()));
    }

    #[test]
    fn test_union_and_enum() {
        let src = r#"
union Storage {
    int a;
    float b;
};
enum Color { Red, Green, Blue };
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Storage".to_string()));
        assert!(symbols.contains(&"Color".to_string()));
    }

    #[test]
    fn test_static_method_excluded() {
        let src = r#"
class Widget {
    public:
        static int staticMethod();
        int instanceMethod();
};
"#;
        let symbols = extract_exports(src);
        assert!(!symbols.contains(&"staticMethod".to_string()));
        assert!(symbols.contains(&"instanceMethod".to_string()));
    }

    #[test]
    fn test_ignores_text_in_strings_and_comments() {
        // AST-native: text that merely looks like exported declarations
        // inside a string literal or comment must not be captured.
        let src = r#"
void realFunc() {
    const char* msg = "class FakeClass { void fakeMethod(); };";
}
// class CommentedOutClass { void commentedMethod(); };
/* struct AlsoCommented { int alsoFake(); }; */
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"realFunc".to_string()));
        assert!(!symbols.contains(&"FakeClass".to_string()));
        assert!(!symbols.contains(&"fakeMethod".to_string()));
        assert!(!symbols.contains(&"CommentedOutClass".to_string()));
        assert!(!symbols.contains(&"commentedMethod".to_string()));
        assert!(!symbols.contains(&"AlsoCommented".to_string()));
        assert!(!symbols.contains(&"alsoFake".to_string()));
    }

    #[test]
    fn test_reference_returning_function_captured() {
        // `reference_declarator` (e.g. `int&`) does not expose its inner
        // declarator via a named field the way `pointer_declarator` does;
        // this regressed without an unwrap fallback in
        // `find_function_declarator`.
        let src = r#"
int& getRef();
char* getName();
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"getRef".to_string()));
        assert!(symbols.contains(&"getName".to_string()));
    }

    #[test]
    fn test_nested_class_in_public_section_captured() {
        // A nested type definition inside a class body (e.g. `class Inner
        // { ... };`) is wrapped by the grammar in a `field_declaration`
        // node with the specifier as its `type` field — it never appears
        // as a direct `field_declaration_list` child, unlike a top-level
        // `class Foo { ... };`.
        let src = r#"
class Outer {
    public:
        class Inner {
            public:
                void innerPublic();
        };
        struct InnerStruct {
            void innerStructMethod();
        };
        enum InnerEnum { A, B };
};
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Outer".to_string()));
        assert!(symbols.contains(&"Inner".to_string()));
        assert!(symbols.contains(&"innerPublic".to_string()));
        assert!(symbols.contains(&"InnerStruct".to_string()));
        assert!(symbols.contains(&"innerStructMethod".to_string()));
        assert!(symbols.contains(&"InnerEnum".to_string()));
    }

    #[test]
    fn test_nested_class_in_private_section_type_still_captured() {
        // Nested type definitions are captured regardless of the enclosing
        // section's visibility (matches the regex, which only line-matches
        // type keywords, not access context) — but the nested type's own
        // members still respect its own default/explicit visibility.
        let src = r#"
class Outer {
    private:
        class Inner {
            public:
                void innerPublic();
        };
};
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Outer".to_string()));
        assert!(symbols.contains(&"Inner".to_string()));
        assert!(symbols.contains(&"innerPublic".to_string()));
    }

    #[test]
    fn test_top_level_type_with_trailing_declarator_captured() {
        // `struct Point2 { ... } origin;` is wrapped in a top-level
        // `declaration` node (type field = struct_specifier, declarator =
        // the `origin` instance) rather than being a bare struct_specifier.
        let src = r#"
struct Point2 {
    double x, y;
} origin;
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Point2".to_string()));
    }

    #[test]
    fn test_malformed_and_empty_input_does_not_panic() {
        assert_eq!(extract_exports(""), Vec::<String>::new());
        let _ = extract_exports("class Foo { public: int add(int a, int");
        let _ = extract_exports("}}}{{{ class ;;; struct");
        let _ = extract_exports("\0\0\0");
    }

    #[test]
    fn test_nested_namespace_and_class() {
        let src = r#"
namespace Outer {
    namespace Inner {
        class Deep {
            public:
                void deepMethod();
        };
    }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Outer".to_string()));
        assert!(symbols.contains(&"Inner".to_string()));
        assert!(symbols.contains(&"Deep".to_string()));
        assert!(symbols.contains(&"deepMethod".to_string()));
    }

    #[test]
    fn test_abstract_interface_and_template_class() {
        // Pure virtual methods under `public:` in an abstract base class
        // (the C++ interface/protocol idiom), plus a templated generic
        // class — both should surface their public members without needing
        // a per-member visibility keyword, matching the class's already-
        // tracked `public:` section state.
        let src = r#"
class PaymentProcessor {
public:
    virtual bool authorize(double amount) const = 0;
    virtual void capture(const std::string& id) = 0;
    virtual double refundableAmount() const noexcept = 0;
};

class StripeProcessor : public PaymentProcessor {
public:
    bool authorize(double amount) const override;
};

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
        assert!(symbols.contains(&"PaymentProcessor".to_string()));
        assert!(symbols.contains(&"authorize".to_string()));
        assert!(symbols.contains(&"capture".to_string()));
        assert!(symbols.contains(&"refundableAmount".to_string()));
        assert!(symbols.contains(&"Stack".to_string()));
        assert!(symbols.contains(&"push".to_string()));
        assert!(symbols.contains(&"top".to_string()));
        assert!(symbols.contains(&"empty".to_string()));
        assert!(symbols.contains(&"size".to_string()));
    }

    #[test]
    fn test_public_operator_overload_captured() {
        // `operator==` (and other operator overloads) declared as a public
        // class member is genuine public API; its declarator is an
        // `operator_name` node, not `identifier`/`field_identifier`.
        let src = r#"
class Color {
public:
    Color(std::uint8_t r, std::uint8_t g, std::uint8_t b);
    bool operator==(const Color& other) const;
    Color operator+(const Color& other) const;
};
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Color".to_string()));
        assert!(symbols.contains(&"operator==".to_string()));
        assert!(symbols.contains(&"operator+".to_string()));
    }

    #[test]
    fn test_friend_operator_in_public_section_captured() {
        // `friend std::ostream& operator<<(...)` is a very common C++
        // idiom for stream output and is genuinely part of a class's
        // public API surface when declared under `public:`.
        let src = r#"
class Color {
public:
    friend std::ostream& operator<<(std::ostream& os, const Color& c);
private:
    std::uint8_t red_;
};
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Color".to_string()));
        assert!(symbols.contains(&"operator<<".to_string()));
    }

    #[test]
    fn test_public_using_alias_in_class_captured() {
        // `using Alias = Type;` inside a class body is commonly part of the
        // class's public API surface (e.g. exposing a pixel/handle type).
        let src = r#"
class Color {
public:
    using Pixel = std::uint32_t;
    Pixel blend() const;
private:
    using Internal = std::uint8_t;
};
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Color".to_string()));
        assert!(symbols.contains(&"Pixel".to_string()));
        assert!(symbols.contains(&"blend".to_string()));
        assert!(!symbols.contains(&"Internal".to_string()));
    }
}
