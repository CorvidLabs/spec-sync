use regex::Regex;
use std::sync::LazyLock;

// NOTE: `(?m)` is required here (unlike the naive `//.*$` copied by some sibling
// extractors) — without it, `$` anchors only to the very end of the whole haystack, not
// to each line ending, so a `//` comment on any line but the last would never be
// stripped at all. Verified empirically: `//.*$` alone left a multi-line string
// completely untouched, while `(?m)//.*$` correctly stripped every line's comment.
static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)//.*$").unwrap());

static COMMENT_MULTI: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/").unwrap());

/// A double-quoted string literal, with or without the `@` prefix that makes it an
/// `NSString` literal (`@"..."`) rather than a plain C string (`"..."`). Handles escaped
/// characters (`\"`, `\\`, ...) so an escaped quote doesn't end the match early, and
/// deliberately never matches across a newline — Objective-C string literals can't
/// legitimately span multiple lines, and bounding the match to one line means a stray,
/// unterminated `"` (e.g. inside an apostrophe-free English comment) can't make this
/// regex run away and swallow real code looking for a closing quote further down the
/// file.
///
/// This is stripped (like comments) before any directive/method regex runs, because
/// unlike comments, string *contents* are otherwise indistinguishable from real source
/// text to a plain-text regex. Two concrete failure modes this prevents, both realistic
/// in a `.m` file that logs or documents things:
///   - A log message like `@"Tip: define your class with @interface MyClass"` would
///     otherwise be misread as an actual `@interface` declaration, fabricating a class
///     that doesn't exist in this file.
///   - A log message like `@"Reached @end of initialization"` would otherwise be misread
///     as the block's real `@end` terminator, truncating the
///     `@implementation`/`@protocol` block scan early and silently dropping any method
///     defined after it (but before the real `@end`).
/// Stripped before comment removal (rather than after) so a URL-shaped string like
/// `@"http://example.com"` — which contains `//` — can't be misread as starting a `//`
/// comment either.
#[allow(clippy::doc_lazy_continuation)]
static STRING_LITERAL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"@?"(?:[^"\\\n]|\\.)*""#).unwrap());

/// `@interface ClassName`, `@interface ClassName : SuperClass`, or the category/class-
/// extension form `@interface ClassName (CategoryName)` / `@interface ClassName ()` — in
/// every case the class name is the first identifier after the directive, so we only
/// need to capture that. Note: this deliberately never scans *inside* an `@interface`
/// body for methods (see module docs) — including the anonymous class-extension form,
/// which is the idiomatic way Objective-C code hides "private" methods/properties from
/// a header. Since we never look inside these bodies, that convention falls out for
/// free rather than needing special-case exclusion logic.
static INTERFACE_NAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)@interface\s+(\w+)").unwrap());

/// `@implementation ClassName` or `@implementation ClassName (CategoryName)` — same
/// name-extraction shape as `INTERFACE_NAME`. Every method defined inside the block that
/// follows is treated as part of the class's exported surface (see module docs for why
/// this is a deliberate over-approximation).
static IMPLEMENTATION_NAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)@implementation\s+(\w+)").unwrap());

/// `@protocol ProtocolName` header. Deliberately does NOT match the unrelated expression
/// form `@protocol(SomeProtocol)` / `@protocol (SomeProtocol)` (used to obtain a
/// `Protocol *` object at the call site, not to declare one) — that form has no
/// whitespace-then-bare-identifier immediately after the keyword the way a declaration
/// does, so `\s+(\w+)` never matches it. Bare forward declarations (`@protocol Foo;`,
/// with no body) are filtered out separately below since they don't declare the
/// protocol's actual contract in this file.
static PROTOCOL_NAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)@protocol\s+(\w+)").unwrap());

/// Matches the tail of a *forward-declaration list* immediately after a `@protocol NAME`
/// match: either nothing but a semicolon (`@protocol Foo;`) or one or more additional
/// comma-separated names before the semicolon (`@protocol Foo, Bar;` /
/// `@protocol Foo, Bar, Baz;`) — Objective-C allows forward-declaring several protocols
/// in a single statement, exactly like `@class Foo, Bar;` does for classes. Anchored with
/// `^` so it only matches when this exact tail shape appears *immediately* after the
/// captured name (mirrors how `after.starts_with(';')` used to be checked before this
/// regex replaced it). Critically, this must NOT match a real declaration's tail — a real
/// `@protocol Foo <Bar>` or `@protocol Foo\n...\n@end` never starts with `,` or `;` here,
/// so neither is mistaken for a forward declaration.
static PROTOCOL_FORWARD_DECL_TAIL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?:,\s*\w+\s*)*;").unwrap());

/// The `(ReturnType)` header of an instance (`-`) or class (`+`) method at the start of a
/// line, e.g. `- (void)` or `+ (instancetype)`. Everything after the closing paren up to
/// the next `{` (a definition, inside `@implementation`) or `;` (a requirement, inside
/// `@protocol`) is the selector text, parsed out separately by `parse_selector` since it
/// can span multiple lines for multi-keyword selectors.
static METHOD_HEADER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[ \t]*[-+]\s*\([^()]*\)").unwrap());

/// A single non-nested `(Type)` parameter-type annotation inside a selector's tail.
/// Applied repeatedly (see `parse_selector`) so nested parens — like block-typed
/// parameters, e.g. `(void (^)(NSInteger))` — fully unwind from the inside out.
static PAREN_GROUP: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\([^()]*\)").unwrap());

/// Extract Objective-C's public-surface-by-convention symbols from a `.m`/`.mm` file.
///
/// Objective-C has no `public`/`private` keyword for methods or classes — real
/// visibility is an architectural split between the `.h` file (the public contract) and
/// the `.m` file (the implementation), and this tool only ever sees the `.m` side. There
/// is no reliable way, from a `.m` file alone, to tell a method that's declared in the
/// public `.h` apart from a helper that's only ever declared in a private
/// `@interface ClassName () ... @end` class-continuation category living in the same
/// `.m` file — both are implemented identically inside `@implementation ... @end`. So
/// this is a deliberate best-effort over-approximation, not a precise visibility check:
///   - `@interface`/`@implementation` names (including categories/extensions) become
///     exported type names,
///   - `@protocol NAME ... @end` declarations (Objective-C's closest analog to a public
///     contract, and to Swift's `protocol`) become exported type names — bare forward
///     declarations (`@protocol Foo;`, no body) are excluded since they don't declare
///     the contract itself,
///   - every instance/class method *defined* inside an `@implementation ... @end` block
///     becomes an exported member,
///   - every method *requirement* inside a `@protocol ... @end` block becomes an
///     exported member.
/// Method bodies are only ever pulled out of `@implementation`/`@protocol` blocks, never
/// out of `@interface` blocks (bare, category, or anonymous extension) — so the common
/// "hide private helpers in a class-extension category" idiom is naturally excluded
/// without needing special-case logic, though a helper declared *only* via a comment or
/// implemented with no interface declaration at all is still (unavoidably) captured.
/// The one true "private" signal Objective-C .m files do have — a file-scope `static` C
/// function — is correctly left uncaptured, since it never appears inside an
/// `@implementation` block in the first place (see the C export extractor for that
/// case).
///
/// Multi-keyword selectors (`- (void)setName:(NSString *)name age:(int)age`) are
/// captured as their full colon-joined selector string (`setName:age:`), matching how
/// Objective-C itself names such a method (as in `@selector(setName:age:)`), rather than
/// just the first keyword (`setName`) — the first keyword alone is ambiguous between two
/// methods that share a first keyword but differ afterward (`setName:age:` vs.
/// `setName:completion:`), so it's the less useful choice for a "what does a consumer
/// actually reference" export list.
///
/// String literals (`@"..."` and `"..."`) are stripped up front, alongside comments —
/// see `STRING_LITERAL` for why: log/documentation text that happens to mention
/// `@interface`/`@end` as plain words would otherwise be misread as real directives.
#[allow(clippy::doc_lazy_continuation)]
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = STRING_LITERAL.replace_all(content, "");
    let stripped = COMMENT_SINGLE.replace_all(&stripped, "");
    let stripped = COMMENT_MULTI.replace_all(&stripped, "");

    let mut symbols = Vec::new();

    for caps in INTERFACE_NAME.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str().to_string();
            if !symbols.contains(&n) {
                symbols.push(n);
            }
        }
    }

    for caps in IMPLEMENTATION_NAME.captures_iter(&stripped) {
        let full = caps.get(0).unwrap();
        if let Some(name) = caps.get(1) {
            let n = name.as_str().to_string();
            if !symbols.contains(&n) {
                symbols.push(n);
            }
        }
        if let Some(rel_end) = stripped[full.end()..].find("@end") {
            let block = &stripped[full.end()..full.end() + rel_end];
            for method in extract_block_methods(block) {
                if !symbols.contains(&method) {
                    symbols.push(method);
                }
            }
        }
    }

    for caps in PROTOCOL_NAME.captures_iter(&stripped) {
        let full = caps.get(0).unwrap();
        // Skip bare forward declarations (`@protocol Foo;`) *and* comma-separated
        // forward-declaration lists (`@protocol Foo, Bar;`) — neither declares the
        // protocol's actual contract in this file, just references a name defined
        // elsewhere. The comma-separated form used to slip past a plain
        // `after.starts_with(';')` check (since the very next character after `Foo` is
        // `,`, not `;`), which fabricated a "Foo" protocol export and then searched for
        // the next literal `@end` in the file to harvest as its "requirements" — greedily
        // spanning into and leaking methods from a completely unrelated, later block
        // (including declaration-only methods from a private class-extension interface
        // that are never otherwise exported).
        let after = stripped[full.end()..].trim_start();
        if PROTOCOL_FORWARD_DECL_TAIL.is_match(after) {
            continue;
        }
        if let Some(name) = caps.get(1) {
            let n = name.as_str().to_string();
            if !symbols.contains(&n) {
                symbols.push(n);
            }
        }
        if let Some(rel_end) = stripped[full.end()..].find("@end") {
            let block = &stripped[full.end()..full.end() + rel_end];
            for method in extract_block_methods(block) {
                if !symbols.contains(&method) {
                    symbols.push(method);
                }
            }
        }
    }

    symbols
}

/// Scans a block body (the text strictly between an `@implementation`/`@protocol`
/// header and its matching `@end`) for method definitions/requirements — lines shaped
/// like `- (Type)selector ...` or `+ (Type)selector ...` — returning each as its full
/// colon-joined selector string.
fn extract_block_methods(block: &str) -> Vec<String> {
    let mut out = Vec::new();
    for m in METHOD_HEADER.find_iter(block) {
        let rest = &block[m.end()..];
        let sig_end = rest.find(['{', ';']).unwrap_or(rest.len());
        let selector = parse_selector(&rest[..sig_end]);
        if !selector.is_empty() {
            out.push(selector);
        }
    }
    out
}

/// Turns the raw text between a method's `(ReturnType)` and its `{`/`;` into a selector
/// string: strips out every `(Type)` parameter-type annotation (repeatedly, so nested
/// parens like block-typed parameters `(void (^)(NSInteger))` fully unwind), then
/// either returns the bare method name (no-argument methods, e.g. `dealloc`) or the
/// colon-joined keyword parts (`setName:age:` for
/// `setName:(NSString *)name age:(int)age`).
fn parse_selector(sig: &str) -> String {
    let mut cleaned = sig.to_string();
    loop {
        let replaced = PAREN_GROUP.replace_all(&cleaned, "").to_string();
        if replaced == cleaned {
            break;
        }
        cleaned = replaced;
    }

    if !cleaned.contains(':') {
        return cleaned
            .trim()
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
    }

    let mut selector = String::new();
    for part in cleaned.split_whitespace() {
        if let Some(idx) = part.find(':') {
            let keyword = &part[..idx];
            if !keyword.is_empty() && keyword.chars().all(|c| c.is_alphanumeric() || c == '_') {
                selector.push_str(keyword);
                selector.push(':');
            }
        }
    }
    selector
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_objc_class_names_from_interface_and_implementation() {
        let src = r#"
@interface Greeter : NSObject
- (void)sayHello;
@end

@implementation Greeter
- (void)sayHello {
    NSLog(@"hello");
}
@end

@interface NSString (URLEncoding)
- (NSString *)urlEncoded;
@end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Greeter".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"NSString".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"sayHello".to_string()), "{symbols:?}");
    }

    #[test]
    fn test_objc_instance_and_class_methods_from_implementation() {
        let src = r#"
@implementation Counter

+ (instancetype)sharedInstance {
    static Counter *instance = nil;
    return instance;
}

- (void)increment {
    self.value += 1;
}

- (NSInteger)value {
    return _value;
}

@end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Counter".to_string()));
        assert!(
            symbols.contains(&"sharedInstance".to_string()),
            "{symbols:?}"
        );
        assert!(symbols.contains(&"increment".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"value".to_string()), "{symbols:?}");
    }

    #[test]
    fn test_objc_multi_keyword_selector_captured_as_full_selector() {
        // The chosen convention (documented in the module docs): the FULL colon-joined
        // selector is captured, not just the first keyword, since the first keyword
        // alone is ambiguous between overloaded-looking selectors.
        let src = r#"
@implementation Person

- (void)setName:(NSString *)name age:(NSInteger)age {
    _name = name;
    _age = age;
}

@end
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"setName:age:".to_string()),
            "expected full selector setName:age: in {symbols:?}"
        );
        assert!(!symbols.contains(&"setName".to_string()));
        assert!(!symbols.contains(&"age".to_string()));
    }

    #[test]
    fn test_objc_protocol_and_requirements_captured() {
        let src = r#"
@protocol GreeterDelegate <NSObject>

@required
- (void)greeter:(id)greeter didGreet:(NSString *)name;

@optional
- (void)greeterDidFinish;

@end
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"GreeterDelegate".to_string()),
            "{symbols:?}"
        );
        assert!(
            symbols.contains(&"greeter:didGreet:".to_string()),
            "{symbols:?}"
        );
        assert!(
            symbols.contains(&"greeterDidFinish".to_string()),
            "{symbols:?}"
        );
    }

    #[test]
    fn test_objc_protocol_forward_declaration_excluded() {
        // A realistic idiom: forward-declare a protocol so it can be used as a property
        // type before its real definition (often in another file entirely). The bare
        // forward reference must not be mistaken for the protocol's contract, and must
        // not swallow an unrelated later block when searching for `@end`.
        let src = r#"
@protocol ExternalDelegate;

@interface Widget : NSObject
@property (nonatomic, weak) id<ExternalDelegate> delegate;
@end

@implementation Widget
- (void)refresh {
}
@end
"#;
        let symbols = extract_exports(src);
        assert!(
            !symbols.contains(&"ExternalDelegate".to_string()),
            "bare forward declaration must not be captured as a protocol contract: {symbols:?}"
        );
        assert!(symbols.contains(&"Widget".to_string()));
        assert!(symbols.contains(&"refresh".to_string()));
    }

    #[test]
    fn test_objc_class_forward_declaration_and_static_helper_excluded() {
        // `@class Foo;` only forward-references a class, it never defines it — and a
        // file-scope `static` C function is Objective-C's one genuine "private" signal
        // in a .m file (it's private to the translation unit). Neither should appear.
        let src = r#"
@class Helper;

static NSString *formatLabel(NSInteger value) {
    return [NSString stringWithFormat:@"%ld", (long)value];
}

@implementation Widget

- (void)refresh {
    NSString *label = formatLabel(self.count);
    NSLog(@"%@", label);
}

@end
"#;
        let symbols = extract_exports(src);
        assert!(
            !symbols.contains(&"Helper".to_string()),
            "@class forward declaration must not be captured: {symbols:?}"
        );
        assert!(
            !symbols.contains(&"formatLabel".to_string()),
            "static C helper function must not be captured: {symbols:?}"
        );
        assert!(symbols.contains(&"Widget".to_string()));
        assert!(symbols.contains(&"refresh".to_string()));
    }

    #[test]
    fn test_objc_comments_stripped_before_matching() {
        let src = r#"
// @interface FakeClass
// - (void)fakeMethod;
/*
@implementation AnotherFake
- (void)alsoFake {}
@end
*/
@implementation RealClass
- (void)realMethod {
    // - (void)notAMethod
}
@end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"RealClass".to_string()));
        assert!(symbols.contains(&"realMethod".to_string()));
        assert!(!symbols.contains(&"FakeClass".to_string()));
        assert!(!symbols.contains(&"fakeMethod".to_string()));
        assert!(!symbols.contains(&"AnotherFake".to_string()));
        assert!(!symbols.contains(&"alsoFake".to_string()));
        assert!(!symbols.contains(&"notAMethod".to_string()));
    }

    #[test]
    fn test_objc_block_typed_completion_handler_parameter() {
        // Idiomatic real-world pattern: an async method taking a block (closure)
        // parameter whose own type contains nested parens. Nested parens must not
        // break selector extraction.
        let src = r#"
@implementation NetworkClient

- (void)fetchDataWithCompletion:(void (^)(NSData * _Nullable data, NSError * _Nullable error))completion {
    completion(nil, nil);
}

@end
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"fetchDataWithCompletion:".to_string()),
            "expected fetchDataWithCompletion: in {symbols:?}"
        );
    }

    #[test]
    fn test_objc_class_extension_private_methods_not_scanned_from_interface() {
        // The idiomatic Objective-C way to hide "private" methods from a public header
        // is a class-extension category declared in the .m file itself. We never scan
        // *inside* any @interface body for methods (only for the class name), so a
        // method that's only ever *declared* here (with no body) and never separately
        // implemented is correctly invisible.
        let src = r#"
@interface Widget ()
- (void)privateHelperDeclaredOnly;
@end

@implementation Widget
- (void)publicMethod {
}
@end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Widget".to_string()));
        assert!(symbols.contains(&"publicMethod".to_string()));
        assert!(
            !symbols.contains(&"privateHelperDeclaredOnly".to_string()),
            "a method only declared (never defined) in a class-extension body must not \
             be captured: {symbols:?}"
        );
    }

    #[test]
    fn test_objc_string_literal_containing_at_end_does_not_truncate_block_scan() {
        // Regression test: a log message that happens to mention "@end" as plain text
        // inside a string literal must not be mistaken for the block's real `@end`
        // terminator. Before string literals were stripped, the block-end search was a
        // naive `.find("@end")` over the (comment-stripped but still-stringed) source,
        // so this fake "@end" truncated the implementation block scan early and
        // silently dropped `logFinish`, which is defined after it but before the real
        // `@end`.
        let src = r#"
@implementation Logger

- (void)logStart {
    NSLog(@"Reached @end of initialization");
}

- (void)logFinish {
    NSLog(@"done");
}

@end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"logStart".to_string()), "{symbols:?}");
        assert!(
            symbols.contains(&"logFinish".to_string()),
            "logFinish dropped due to a fake @end inside a string literal: {symbols:?}"
        );
    }

    #[test]
    fn test_objc_string_literal_containing_fake_directive_not_captured() {
        // Regression test: a help/log string that mentions "@interface" syntax as
        // documentation text must not be mistaken for a real class declaration. Before
        // string literals were stripped, this fabricated a "MyClass" export that
        // doesn't correspond to any real declaration in the file.
        let src = r#"
@implementation Logger

- (void)printHelp {
    NSLog(@"Tip: define your class with @interface MyClass : NSObject");
}

@end
"#;
        let symbols = extract_exports(src);
        assert!(
            !symbols.contains(&"MyClass".to_string()),
            "fabricated class name from string literal text: {symbols:?}"
        );
        assert!(symbols.contains(&"Logger".to_string()));
        assert!(symbols.contains(&"printHelp".to_string()));
    }

    #[test]
    fn test_objc_comma_separated_protocol_forward_declaration_excluded() {
        // REGRESSION (real bug found via adversarial testing): `@protocol Foo, Bar;` is
        // the multi-name forward-declaration form (like `@class Foo, Bar;` for classes).
        // The exclusion check used to be a plain `after.starts_with(';')`, but the text
        // immediately following "Foo" here is ", Bar;" — not ";" — so the check failed to
        // recognize this as a forward declaration and fabricated a "Foo" protocol export
        // that doesn't exist in this file.
        let src = r#"
@protocol Foo, Bar;

@implementation Unrelated
- (void)someMethod {
}
@end
"#;
        let symbols = extract_exports(src);
        assert!(
            !symbols.contains(&"Foo".to_string()),
            "comma-separated forward declaration must not fabricate a protocol: {symbols:?}"
        );
        assert!(
            !symbols.contains(&"Bar".to_string()),
            "comma-separated forward declaration must not fabricate a protocol: {symbols:?}"
        );
        assert!(symbols.contains(&"Unrelated".to_string()));
        assert!(symbols.contains(&"someMethod".to_string()));
    }

    #[test]
    fn test_objc_comma_separated_protocol_forward_decl_does_not_leak_private_declaration() {
        // REGRESSION (real bug, more severe consequence of the same root cause): once
        // "Foo" was fabricated as a protocol name, the code searched for the next literal
        // "@end" in the whole file to harvest as its "requirements" — greedily spanning
        // into and scanning a completely unrelated, later block. Here that block is a
        // private class-extension interface (`@interface Widget ()`), whose
        // declaration-only method is never otherwise exported (see
        // `test_objc_class_extension_private_methods_not_scanned_from_interface`). Before
        // the fix, that private method leaked into the export list anyway, as a fake
        // "requirement" of the phantom "Foo" protocol.
        let src = r#"
@protocol Foo, Bar;

@interface Widget ()
- (void)privateHelperDeclaredOnly;
@end

@implementation Widget
- (void)publicMethod {
}
@end
"#;
        let symbols = extract_exports(src);
        assert!(!symbols.contains(&"Foo".to_string()), "{symbols:?}");
        assert!(
            !symbols.contains(&"privateHelperDeclaredOnly".to_string()),
            "private declaration-only method must not leak via a fabricated protocol \
             block scan: {symbols:?}"
        );
        assert!(symbols.contains(&"Widget".to_string()));
        assert!(symbols.contains(&"publicMethod".to_string()));
    }

    #[test]
    fn test_objc_category_implementation_exports_base_class_name_and_methods() {
        // `@implementation ClassName (CategoryName)` names the category, not a new type —
        // the exported symbol should be the base class name, and methods defined in the
        // category should be attributed to it (matching how `@interface NSString
        // (URLEncoding)` already behaves for the interface form).
        let src = r#"
@implementation NSString (URLEncoding)

- (NSString *)urlEncoded {
    return self;
}

@end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"NSString".to_string()), "{symbols:?}");
        assert!(
            !symbols.contains(&"URLEncoding".to_string()),
            "category name itself must not be exported as a separate type: {symbols:?}"
        );
        assert!(symbols.contains(&"urlEncoded".to_string()), "{symbols:?}");
    }

    #[test]
    fn test_objc_protocol_with_comma_separated_conformance_list() {
        // A protocol declaring conformance to *multiple* other protocols
        // (`<NSObject, UITableViewDataSource>`) is a real declaration, not a forward
        // reference, even though its tail also contains commas — it must not be
        // mistaken for the comma-separated forward-declaration form fixed above (the
        // distinguishing feature is the `<` immediately after the name instead of `,`
        // or `;`).
        let src = r#"
@protocol MultiDelegate <NSObject, UITableViewDataSource>

@required
- (void)didSelectRow:(NSInteger)row;

@optional
- (void)willReload;

@end
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"MultiDelegate".to_string()),
            "{symbols:?}"
        );
        assert!(
            symbols.contains(&"didSelectRow:".to_string()),
            "{symbols:?}"
        );
        assert!(symbols.contains(&"willReload".to_string()), "{symbols:?}");
        // The conformance list names are referenced protocols, not declared here.
        assert!(!symbols.contains(&"NSObject".to_string()));
        assert!(!symbols.contains(&"UITableViewDataSource".to_string()));
    }

    #[test]
    fn test_objc_multiline_interface_header_with_protocol_list_before_first_member() {
        // A common real-world formatting style: the superclass and protocol-conformance
        // list wrap across multiple lines before the first `@property`/method. The class
        // name must still be captured correctly regardless of what follows on later
        // lines.
        let src = r#"
@interface MyView : UIView <
    UITableViewDelegate,
    UITableViewDataSource
>

- (void)reload;

@end

@implementation MyView
- (void)reload {
}
@end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"MyView".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"reload".to_string()), "{symbols:?}");
        assert!(!symbols.contains(&"UITableViewDelegate".to_string()));
        assert!(!symbols.contains(&"UITableViewDataSource".to_string()));
    }

    #[test]
    fn test_objc_lightweight_generics_and_protocol_typed_params_do_not_break_selectors() {
        // Lightweight generics (`NSArray<NSString *> *`) and the `id<Protocol>` existential
        // syntax both use angle brackets, which must not be confused with the parens that
        // `parse_selector` strips, nor with a protocol conformance list. Both appear as
        // ordinary `(Type)` parameter annotations here and should be stripped as a single
        // non-nested paren group each.
        let src = r#"
@implementation Container

- (void)setItems:(NSArray<NSString *> *)items delegate:(id<ContainerDelegate>)delegate {
    _items = items;
    _delegate = delegate;
}

- (NSArray<NSString *> *)items {
    return _items;
}

@end
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"setItems:delegate:".to_string()),
            "{symbols:?}"
        );
        assert!(symbols.contains(&"items".to_string()), "{symbols:?}");
        // Sanity: nothing derived from inside the angle brackets should leak out as a
        // bogus separate selector/keyword.
        assert!(!symbols.contains(&"NSString".to_string()));
        assert!(!symbols.contains(&"ContainerDelegate".to_string()));
    }

    #[test]
    fn test_objc_multiline_keyword_selector_across_lines() {
        // Multi-keyword selectors are often wrapped across lines for readability. The
        // full colon-joined selector must still be reconstructed correctly since
        // `parse_selector` splits on any whitespace (including newlines), not just
        // spaces.
        let src = r#"
@implementation Requester

- (void)sendRequestWithPath:(NSString *)path
                      query:(NSDictionary *)query
                 completion:(void (^)(id response))completion {
    completion(nil);
}

@end
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"sendRequestWithPath:query:completion:".to_string()),
            "expected full multi-line selector in {symbols:?}"
        );
    }

    #[test]
    fn test_objc_two_class_extension_implementation_pairs_do_not_cross_contaminate() {
        // Two entirely independent class-extension + implementation pairs in the same
        // file: each class's private, declaration-only helper must stay invisible, and
        // each class's real public method must be attributed correctly — with no bleed
        // between the two blocks' `@end` scans.
        let src = r#"
@interface Foo ()
- (void)fooPrivateHelper;
@end

@implementation Foo
- (void)fooPublicMethod {
}
@end

@interface Bar ()
- (void)barPrivateHelper;
@end

@implementation Bar
- (void)barPublicMethod {
}
@end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Foo".to_string()));
        assert!(symbols.contains(&"Bar".to_string()));
        assert!(symbols.contains(&"fooPublicMethod".to_string()));
        assert!(symbols.contains(&"barPublicMethod".to_string()));
        assert!(
            !symbols.contains(&"fooPrivateHelper".to_string()),
            "{symbols:?}"
        );
        assert!(
            !symbols.contains(&"barPrivateHelper".to_string()),
            "{symbols:?}"
        );
    }

    #[test]
    fn test_objc_same_class_multiple_category_implementations_merge_methods() {
        // Documents the deliberate over-approximation (see module docs): methods defined
        // across *multiple* `@implementation ClassName (Category) ... @end` blocks for
        // the same base class are all attributed to that one class name, and the class
        // name itself is only listed once (deduplicated).
        let src = r#"
@implementation Widget (Networking)
- (void)fetchData {
}
@end

@implementation Widget (Persistence)
- (void)saveData {
}
@end
"#;
        let symbols = extract_exports(src);
        let widget_count = symbols.iter().filter(|s| *s == "Widget").count();
        assert_eq!(
            widget_count, 1,
            "class name must be deduplicated: {symbols:?}"
        );
        assert!(symbols.contains(&"fetchData".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"saveData".to_string()), "{symbols:?}");
    }

    #[test]
    fn test_objc_no_arg_methods_and_dealloc() {
        let src = r#"
@implementation Session

- (void)dealloc {
    [self invalidate];
}

- (BOOL)isActive {
    return YES;
}

@end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"dealloc".to_string()));
        assert!(symbols.contains(&"isActive".to_string()));
    }
}
