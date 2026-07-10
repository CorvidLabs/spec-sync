use regex::Regex;
use std::sync::LazyLock;

static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"//.*$").unwrap());

static COMMENT_MULTI: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/").unwrap());

/// Single-line double-quoted strings, including GStrings (`"Hello ${name}"`). Brace
/// characters inside a string literal -- whether literal text like `"{ not code }"` or a
/// GString interpolation `${...}` -- are not real scope-opening/closing braces, but the
/// scope tracker below scans raw characters with no string awareness. Left unstripped, a
/// single `{` inside a string (unbalanced within that literal, e.g. `"Hello ${name} {
/// unbalanced"`) desyncs `scope_stack` for the rest of the file and can silently drop later
/// legitimate exports. Matching is confined to one line (no `(?s)`) so multi-line
/// triple-quoted strings are intentionally left to the caller's line-based scanning, same as
/// this file's existing multi-line comment handling.
static STRING_DOUBLE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#""(?:\\.|[^"\\\n])*""#).unwrap());

/// Single-line single-quoted strings (Groovy's non-interpolating string literal).
static STRING_SINGLE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"'(?:\\.|[^'\\\n])*'").unwrap());

/// Groovy type declarations: class, interface, trait, enum, and Java-style `@interface`
/// annotation types. Unlike Java, Groovy has NO package-private default -- a bare
/// `class Foo { def bar() {} }` with zero modifiers is fully public. `public` is legal but
/// redundant/rare in idiomatic Groovy; only an explicit `private`/`protected` (or the
/// `@PackageScope` annotation, handled separately below) narrows visibility.
static GROOVY_TYPE_DECL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*(?:(?:public|private|protected)\s+)?(?:static\s+)?(?:(?:final|abstract|sealed)\s+)*(?:@interface|class|interface|trait|enum)\s+(\w+)",
    )
    .unwrap()
});

/// `def`-declared members -- Groovy's dynamically-typed method/property/field marker.
/// Covers `def greet(name) {}`, `def total = 0`, and a bare `def helper`.
static GROOVY_DEF_DECL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*(?:(?:public|private|protected)\s+)?(?:static\s+)?(?:final\s+)?def\s+(\w+)",
    )
    .unwrap()
});

/// Statically-typed member declarations: `String greet(String name) {}`, `int count = 0`,
/// `void run() {}`, `List<String> items`. Requires two bare words in a row (TYPE then NAME),
/// which is specifically the shape of a declaration rather than a call or expression -- this
/// keeps the pattern from matching ordinary statements like `foo.bar()` or `x = 5`. The regex
/// crate has no look-around, so the first word is captured (group 1) and rejected in Rust via
/// `GROOVY_KEYWORDS` when it's a control-flow/reserved keyword whose line could otherwise look
/// like TYPE NAME (e.g. `return foo()`, `throw new Bar()`); the real name is group 2.
static GROOVY_TYPED_MEMBER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*(?:(?:public|private|protected)\s+)?(?:static\s+)?(?:(?:final|abstract|synchronized|native)\s+)*(?:<[^>]+>\s+)?([\w.]+(?:<[^<>]*>)?(?:\[\])*)\s+(\w+)",
    )
    .unwrap()
});

/// Reserved/control-flow words that must never be treated as a TYPE token by
/// `GROOVY_TYPED_MEMBER` -- lines like `return foo()` or `throw new Bar()` otherwise have the
/// same "word word(" shape as a real declaration.
const GROOVY_KEYWORDS: &[&str] = &[
    "if",
    "for",
    "while",
    "return",
    "new",
    "throw",
    "switch",
    "catch",
    "try",
    "else",
    "do",
    "assert",
    "synchronized",
    "instanceof",
    "package",
    "import",
    "extends",
    "implements",
    "throws",
    "case",
    "break",
    "continue",
    "finally",
    "in",
    "as",
    "def",
    "class",
    "interface",
    "trait",
    "enum",
    "super",
    "this",
];

/// Explicit narrowing -- only `private`/`protected` reduce Groovy's public-by-default (there
/// is no `internal` keyword in Groovy).
static PRIVATE_LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[^\S\n]*(?:private|protected)\s+").unwrap());

/// Matches one leading `@AnnotationName` (optionally with a `(...)` argument list, e.g.
/// `@PackageScope(Visibility.PROTECTED)`) at the start of a line/remainder, capturing the
/// annotation name. Used both for annotation-only lines (`@PackageScope` alone above a
/// declaration) and annotations written inline with the declaration they modify
/// (`@PackageScope def foo() {}`, `@CompileStatic class Foo {`) -- the latter must NOT be
/// treated as a no-op skip line, since the declaration (and its opening brace, if any) still
/// needs normal processing.
static ANNOTATION_PREFIX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^@(\w+)(?:\([^)]*\))?\s*").unwrap());

/// Detect a line that opens a "type body" scope (class/interface/trait/enum), as opposed to a
/// method body or any other block. Declarations directly inside a type body are real export
/// candidates; declarations nested inside a method body -- or any other block, like a
/// `for`/`if`/closure -- are local and are never exported, regardless of whether they happen
/// to use `def` or a type name.
static TYPE_BODY_OPEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*(?:(?:public|private|protected)\s+)?(?:static\s+)?(?:(?:final|abstract|sealed)\s+)*(?:class|interface|trait|enum)\b",
    )
    .unwrap()
});

/// Extract exported symbols from Groovy source code.
/// Groovy is public-by-default: a declaration is part of the public surface unless it
/// explicitly narrows itself with `private`/`protected` or the `@PackageScope` annotation.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = COMMENT_SINGLE.replace_all(content, "");
    let stripped = COMMENT_MULTI.replace_all(&stripped, "");
    let stripped = STRING_DOUBLE.replace_all(&stripped, "");
    let stripped = STRING_SINGLE.replace_all(&stripped, "");

    let mut symbols: Vec<String> = Vec::new();
    // Tracks nested `{ }` scopes: true = a class/interface/trait/enum body (member
    // declarations are legitimate export candidates), false = a method body or any other
    // block (declarations inside are local and never exported). An empty stack means top
    // level (script-level declarations are also part of the public surface).
    let mut scope_stack: Vec<bool> = Vec::new();
    // Set by a preceding `@PackageScope` annotation line; consumed by the next non-blank,
    // non-annotation line (the declaration it modifies).
    let mut package_scope_pending = false;

    for line in stripped.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        let mut is_package_scoped = package_scope_pending;
        package_scope_pending = false;

        // `decl_line` is what the declaration-matching and brace-tracking passes below
        // operate on. For an ordinary line it's just `line`; when the line opens with one or
        // more annotations (`@Foo`, `@PackageScope(...)`, ...) the annotation prefix is
        // stripped off first so it can't interfere with either pass.
        //
        // `@interface Foo` is a Java-style annotation *type declaration*, not a usage of an
        // annotation on the following line -- it must fall through to normal matching rather
        // than being treated like `@PackageScope`/`@CompileStatic` and stripped/skipped.
        let mut decl_line = line;

        if trimmed.starts_with('@') && !trimmed.starts_with("@interface") {
            let mut rest = trimmed;
            let mut saw_package_scope = false;

            while rest.starts_with('@') && !rest.starts_with("@interface") {
                match ANNOTATION_PREFIX.captures(rest) {
                    Some(caps) => {
                        if &caps[1] == "PackageScope" {
                            saw_package_scope = true;
                        }
                        rest = rest[caps.get(0).unwrap().end()..].trim_start();
                    }
                    None => break,
                }
            }

            if rest.is_empty() {
                // Pure annotation line(s) with nothing else -- e.g. `@PackageScope` sitting
                // on its own line above the declaration it modifies. Nothing to declare and
                // no braces to track here; carry the pending narrowing to the next line.
                if saw_package_scope {
                    package_scope_pending = true;
                }
                continue;
            }

            // Annotation(s) inline with the declaration they modify, e.g.
            // `@PackageScope def foo() {}` or `@CompileStatic class Foo {`. The declaration
            // (and its opening brace, if any) still needs normal processing below -- skipping
            // it here would both drop the declaration from the export scan entirely and skip
            // brace-tracking for its `{`, desyncing `scope_stack` for the rest of the file.
            if saw_package_scope {
                is_package_scoped = true;
            }
            decl_line = rest;
        }

        let in_exportable_scope = scope_stack.last().copied().unwrap_or(true);

        if in_exportable_scope && !is_package_scoped && !PRIVATE_LINE.is_match(decl_line) {
            if let Some(caps) = GROOVY_TYPE_DECL.captures(decl_line)
                && let Some(name) = caps.get(1)
            {
                let n = name.as_str().to_string();
                if !symbols.contains(&n) {
                    symbols.push(n);
                }
            } else if let Some(caps) = GROOVY_DEF_DECL.captures(decl_line)
                && let Some(name) = caps.get(1)
            {
                let n = name.as_str().to_string();
                if !symbols.contains(&n) {
                    symbols.push(n);
                }
            } else if let Some(caps) = GROOVY_TYPED_MEMBER.captures(decl_line)
                && let Some(type_tok) = caps.get(1)
                && let Some(name) = caps.get(2)
                && !GROOVY_KEYWORDS.contains(&type_tok.as_str())
            {
                let n = name.as_str().to_string();
                if !symbols.contains(&n) {
                    symbols.push(n);
                }
            }
        }

        let opens_type_body = TYPE_BODY_OPEN.is_match(decl_line);
        for ch in decl_line.chars() {
            match ch {
                '{' => scope_stack.push(opens_type_body),
                '}' => {
                    scope_stack.pop();
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
    fn test_groovy_public_by_default() {
        // Finding-shaped case: Groovy defaults to PUBLIC with zero modifiers, unlike Java's
        // package-private default. A bare `class`/`def` with no keyword at all must still be
        // captured as exported.
        let src = r#"
class UserService {
    def findUser(String id) {
        return repository.lookup(id)
    }

    String describe() {
        "user service"
    }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"UserService".to_string()));
        assert!(symbols.contains(&"findUser".to_string()));
        assert!(symbols.contains(&"describe".to_string()));
    }

    #[test]
    fn test_groovy_private_protected_excluded() {
        let src = r#"
class AuthService {
    def login(String user) {}
    private def hashPassword(String raw) {}
    protected void auditLog(String msg) {}
    private String secretKey = "abc123"
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"AuthService".to_string()));
        assert!(symbols.contains(&"login".to_string()));
        assert!(!symbols.contains(&"hashPassword".to_string()));
        assert!(!symbols.contains(&"auditLog".to_string()));
        assert!(!symbols.contains(&"secretKey".to_string()));
    }

    #[test]
    fn test_groovy_comments_stripped() {
        let src = r#"
// class FakeClass {}
/* class AlsoFake {
    def alsoFake() {}
} */
/**
 * Javadoc-style comment
 * def notReal() {}
 */
class RealClass {
    def realMethod() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"RealClass".to_string()));
        assert!(symbols.contains(&"realMethod".to_string()));
        assert!(!symbols.contains(&"FakeClass".to_string()));
        assert!(!symbols.contains(&"AlsoFake".to_string()));
        assert!(!symbols.contains(&"alsoFake".to_string()));
        assert!(!symbols.contains(&"notReal".to_string()));
    }

    #[test]
    fn test_groovy_package_scope_annotation_excluded() {
        // Groovy-specific edge case: `@PackageScope` narrows visibility without any keyword
        // on the declaration line itself -- easy to miss if you only look for
        // private/protected tokens.
        let src = r#"
class OrderProcessor {
    def submit(Order order) {}

    @PackageScope
    def validateInternal(Order order) {}

    @PackageScope(Visibility.PROTECTED)
    String internalNote
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"OrderProcessor".to_string()));
        assert!(symbols.contains(&"submit".to_string()));
        assert!(!symbols.contains(&"validateInternal".to_string()));
        assert!(!symbols.contains(&"internalNote".to_string()));
    }

    #[test]
    fn test_groovy_trait_members_exported() {
        let src = r#"
trait Flyable {
    boolean airborne = false

    void fly() {
        airborne = true
    }

    private void groundCheck() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Flyable".to_string()));
        assert!(symbols.contains(&"airborne".to_string()));
        assert!(symbols.contains(&"fly".to_string()));
        assert!(!symbols.contains(&"groundCheck".to_string()));
    }

    #[test]
    fn test_groovy_locals_not_exported() {
        // Declarations nested inside a method body -- even `def`/typed locals inside a
        // public method -- are not part of the public surface.
        let src = r#"
class ReportBuilder {
    def build(List<String> rows) {
        def subtotal = 0
        int count = rows.size()
        for (row in rows) {
            def trimmed = row.trim()
            subtotal += trimmed.length()
        }
        return subtotal
    }

    private def helper() {
        def secretLocal = 42
        return secretLocal
    }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"ReportBuilder".to_string()));
        assert!(symbols.contains(&"build".to_string()));
        assert!(!symbols.contains(&"subtotal".to_string()));
        assert!(!symbols.contains(&"count".to_string()));
        assert!(!symbols.contains(&"trimmed".to_string()));
        assert!(!symbols.contains(&"helper".to_string()));
        assert!(!symbols.contains(&"secretLocal".to_string()));
    }

    #[test]
    fn test_groovy_static_members_and_enum() {
        let src = r#"
class MathUtils {
    static final int MAX_RETRIES = 3
    static int square(int n) {
        return n * n
    }
    private static int cachedValue = 0
}

enum Status {
    ACTIVE, INACTIVE, PENDING
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"MathUtils".to_string()));
        assert!(symbols.contains(&"MAX_RETRIES".to_string()));
        assert!(symbols.contains(&"square".to_string()));
        assert!(!symbols.contains(&"cachedValue".to_string()));
        assert!(symbols.contains(&"Status".to_string()));
    }

    #[test]
    fn test_groovy_script_level_def_and_closure() {
        // Groovy scripts (no enclosing class) commonly declare top-level `def` functions and
        // closures assigned to variables -- both are part of the script's exposed surface.
        let src = r#"
def greet(String name) {
    "Hello, ${name}!"
}

def multiply = { int a, int b -> a * b }

private def internalHelper() {
    return 42
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"greet".to_string()));
        assert!(symbols.contains(&"multiply".to_string()));
        assert!(!symbols.contains(&"internalHelper".to_string()));
    }

    #[test]
    fn test_groovy_annotated_class_with_generics() {
        let src = r#"
@CompileStatic
class Repository<T> {
    private List<T> items = []

    void add(T item) {
        items.add(item)
    }

    List<T> findAll() {
        return items
    }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Repository".to_string()));
        assert!(symbols.contains(&"add".to_string()));
        assert!(symbols.contains(&"findAll".to_string()));
        assert!(!symbols.contains(&"items".to_string()));
    }

    #[test]
    fn test_groovy_interface_and_annotation_type() {
        let src = r#"
interface Authenticator {
    boolean authenticate(String token)
}

@interface Cacheable {
    int ttl() default 300
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Authenticator".to_string()));
        assert!(symbols.contains(&"authenticate".to_string()));
        assert!(symbols.contains(&"Cacheable".to_string()));
    }

    #[test]
    fn test_groovy_inline_package_scope_annotation_does_not_leak_locals() {
        // Finding: `@PackageScope` written inline with the declaration it modifies (rather
        // than on its own line above it) hit the same skip path as a standalone annotation
        // line, which did `continue` *before* the line's opening `{` was pushed onto
        // `scope_stack`. That desynced the scope tracker for the rest of the class body, so a
        // local variable a couple of lines into the annotated method's body (`taxAmount`) was
        // wrongly treated as sitting directly in the class body and leaked out as a public
        // export -- while `validateOrder` itself was correctly excluded.
        let src = r#"
class OrderProcessor {
    def submitOrder(Order order) {
        return repository.save(order)
    }

    @PackageScope def validateOrder(Order order) {
        def subtotal = order.items.sum { it.price }
        def taxAmount = subtotal * 0.08
        return subtotal + taxAmount
    }

    def cancelOrder(Order order) {
        order.status = "CANCELLED"
    }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"OrderProcessor".to_string()));
        assert!(symbols.contains(&"submitOrder".to_string()));
        assert!(symbols.contains(&"cancelOrder".to_string()));
        assert!(
            !symbols.contains(&"validateOrder".to_string()),
            "@PackageScope narrows visibility even written inline with the declaration"
        );
        assert!(
            !symbols.contains(&"subtotal".to_string()),
            "local var inside the @PackageScope method body must not leak"
        );
        assert!(
            !symbols.contains(&"taxAmount".to_string()),
            "local var two lines into the @PackageScope method body must not leak \
             (regression: inline annotation desynced scope_stack and exposed this as public)"
        );
    }

    #[test]
    fn test_groovy_inline_annotation_on_class_still_exports_class_and_members() {
        // Finding: any annotation line (not just `@PackageScope`) written inline with the
        // declaration it decorates -- e.g. `@CompileStatic class Repository {` all on one
        // line, a common idiomatic Groovy style -- was treated identically to a standalone
        // annotation-only line and `continue`d away entirely. That dropped the class
        // declaration itself from the export scan and skipped brace-tracking for its opening
        // `{`, corrupting scope depth for every member that followed.
        let src = r#"
@CompileStatic class Repository<T> {
    private List<T> items = []

    void add(T item) {
        items.add(item)
    }

    List<T> findAll() {
        return items
    }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Repository".to_string()));
        assert!(symbols.contains(&"add".to_string()));
        assert!(symbols.contains(&"findAll".to_string()));
        assert!(!symbols.contains(&"items".to_string()));
    }

    #[test]
    fn test_groovy_gstring_braces_do_not_corrupt_scope_tracking() {
        // Finding: brace characters inside a string/GString literal (e.g. the `{`/`}` from a
        // `${...}` interpolation, or stray literal braces in template text) were scanned as
        // real scope-opening/closing characters, since the scope tracker has no string
        // awareness. An unbalanced brace count within a single string literal desynced
        // `scope_stack` and silently dropped every real declaration for the rest of the class
        // -- here, `renderInvoice` (the very next member) disappeared from the export list
        // even though it has zero modifiers and is fully public.
        let src = r#"
class InvoiceRenderer {
    String header = "Invoice for ${customer.name} { unbalanced brace in template text"

    String renderInvoice(Invoice invoice) {
        return "${header}\n${invoice.total}"
    }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"InvoiceRenderer".to_string()));
        assert!(symbols.contains(&"header".to_string()));
        assert!(
            symbols.contains(&"renderInvoice".to_string()),
            "regression: unbalanced brace inside a string literal desynced scope tracking \
             and dropped this legitimate public method"
        );
    }
}
