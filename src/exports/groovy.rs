use regex::Regex;
use std::sync::LazyLock;

/// Single-line `//` comments. Requires `(?m)` -- without it, `$` anchors only to the absolute
/// end of the *whole file* (regex crate default), not the end of each line, so a `//` comment
/// anywhere but the file's literal last line would never actually match and never get
/// stripped. That silently left any brace characters inside such a comment (e.g. a stray
/// unbalanced `{` in prose) as raw, unstripped text for the scope tracker to scan, desyncing
/// `scope_stack`. This mostly went unnoticed because ordinary comment text rarely has the
/// shape of a declaration and stray comment braces are usually balanced -- an *unbalanced*
/// brace in a non-last-line comment is what actually surfaces the bug.
static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)//.*$").unwrap());

static COMMENT_MULTI: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/").unwrap());

/// Triple-quoted GStrings (`"""..."""`), which -- unlike the single-quote form below -- are
/// explicitly allowed to span multiple lines. Stripped with `(?s)` (dot matches newline)
/// *before* `STRING_DOUBLE` so the single-line regex never gets a chance to partially match
/// into a triple-quote's leading/trailing pair of quote characters (which, left to
/// `STRING_DOUBLE` alone, matches an empty `""` off one end and leaves the rest of the triple
/// -quoted literal -- including any brace characters in it -- as unstripped raw text). A
/// non-greedy body (`.*?`) stops at the first closing `"""`.
static STRING_TRIPLE_DOUBLE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("(?s)\"\"\".*?\"\"\"").unwrap());

/// Triple-quoted plain strings (`'''...'''`), Groovy's non-interpolating multi-line literal.
/// See `STRING_TRIPLE_DOUBLE` for why this must run before the single-line `STRING_SINGLE`.
static STRING_TRIPLE_SINGLE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)'''.*?'''").unwrap());

/// Single-line double-quoted strings, including GStrings (`"Hello ${name}"`). Brace
/// characters inside a string literal -- whether literal text like `"{ not code }"` or a
/// GString interpolation `${...}` -- are not real scope-opening/closing braces, but the
/// scope tracker below scans raw characters with no string awareness. Left unstripped, a
/// single `{` inside a string (unbalanced within that literal, e.g. `"Hello ${name} {
/// unbalanced"`) desyncs `scope_stack` for the rest of the file and can silently drop later
/// legitimate exports. Matching is confined to one line (no `(?s)`); multi-line triple-quoted
/// strings are handled separately above by `STRING_TRIPLE_DOUBLE`/`STRING_TRIPLE_SINGLE`
/// before this regex ever runs.
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
    let stripped = STRING_TRIPLE_DOUBLE.replace_all(&stripped, "");
    let stripped = STRING_TRIPLE_SINGLE.replace_all(&stripped, "");
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
    // Set when a line matches `TYPE_BODY_OPEN` (a class/interface/trait/enum keyword) but has
    // no `{` of its own -- a multi-line header, e.g. `class Foo\n    extends Bar\n
    // implements Baz {`. Carries that declaration line's own exportability forward across any
    // header-continuation lines until the `{` that actually opens the body is seen (see
    // `opens_type_body` below).
    let mut pending_type_body_open: Option<bool> = None;

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
        // Whether *this* declaration line itself sits in exported territory. A type body
        // that opens here should only be treated as an export-worthy scope (see
        // `opens_type_body` below) when this line itself qualifies -- otherwise members of
        // a `private`/`protected`/`@PackageScope`-narrowed type would incorrectly leak as
        // exports even though the type itself was correctly excluded.
        let is_exportable_here =
            in_exportable_scope && !is_package_scoped && !PRIVATE_LINE.is_match(decl_line);

        if is_exportable_here {
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

        // Only treat a freshly-opened type body as an "exportable scope" (and thus a
        // candidate for its members to be captured) if the opening declaration itself was
        // in exported territory -- otherwise a `private class Hidden { ... }` (or one
        // narrowed via `@PackageScope`) would still push `true`, leaking its members as
        // exports even though the type itself is correctly excluded above.
        //
        // Finding: a multi-line class/interface/trait/enum header -- e.g.
        // `class Foo\n        extends Bar\n        implements Baz {` -- puts the type
        // keyword on an earlier line than the `{` that actually opens the body. Deriving
        // `opens_type_body` from only the line containing `{` meant a continuation line like
        // `implements Baz {` (which itself matches none of `class|interface|trait|enum`)
        // looked like an ordinary block open, wrongly pushing `false` and hiding every real
        // member of an otherwise fully public class as "local". `pending_type_body_open`
        // carries the type-decl line's own exportability forward across any header
        // continuation lines until the `{` that consumes it is finally seen.
        //
        // Finding (second adversarial pass): this classification is computed ONCE per line
        // and then applied to *every* `{` encountered while scanning `decl_line` below. That
        // is correct for the common case of a single brace per line, but when a line packs
        // multiple unmatched `{` characters together -- e.g. `class Outer { private class
        // Hidden {` opening both `Outer`'s and the nested (private!) `Hidden`'s bodies on one
        // physical line -- every brace on that line gets the *same* classification, even
        // though only the first `{` should inherit it; deeper braces on the same line need
        // their own re-evaluation against the scope_stack state and text as of that specific
        // brace. To keep this fix surgical (a full per-character recursive-descent scan is a
        // much larger rewrite than this bug warrants), braces are processed with a running
        // per-segment re-check: each `{` re-derives its own `opens_type_body` from the text
        // since the previous brace (or line start) rather than reusing one line-wide value,
        // and consults the *current* top of `scope_stack` (which reflects any same-line
        // pushes/pops already applied) for its own exportability gate. See the per-brace loop
        // below.
        let starts_type_decl = TYPE_BODY_OPEN.is_match(decl_line);
        let has_open_brace = decl_line.contains('{');

        if starts_type_decl && !has_open_brace {
            pending_type_body_open = Some(is_exportable_here);
        }

        let opens_type_body = if !has_open_brace {
            false
        } else if starts_type_decl {
            is_exportable_here
        } else {
            pending_type_body_open.take().unwrap_or(false)
        };

        // Per-brace re-evaluation: `segment_start` tracks the byte offset (within `decl_line`)
        // of the text since the last brace (or line start). For the *first* `{` on the line,
        // the segment is the whole line-leading text and `opens_type_body` (computed above,
        // including the multi-line-header carry-forward) is authoritative. For every
        // subsequent `{` on the same line, the segment is just the text between the previous
        // brace and this one -- if *that* segment itself opens a type body (e.g. `private
        // class Hidden` sitting between an outer `{` and this `{`), its own modifiers are
        // checked directly rather than inheriting the first brace's classification.
        let mut segment_start = 0usize;
        let mut first_brace_seen = false;

        for (idx, ch) in decl_line.char_indices() {
            match ch {
                '{' => {
                    let push_value = if !first_brace_seen {
                        first_brace_seen = true;
                        opens_type_body
                    } else {
                        let segment = decl_line[segment_start..idx].trim();
                        if TYPE_BODY_OPEN.is_match(segment) {
                            // This inner segment itself opens a type body -- gate its
                            // exportability on both the *current* enclosing scope (top of
                            // `scope_stack`, reflecting same-line pushes so far) and whether
                            // this segment's own modifiers narrow it.
                            let enclosing_exportable = scope_stack.last().copied().unwrap_or(true);
                            enclosing_exportable && !PRIVATE_LINE.is_match(segment)
                        } else {
                            // An ordinary block open (method body, closure, control-flow
                            // block, ...) nested on the same line -- never a type body.
                            false
                        }
                    };
                    scope_stack.push(push_value);
                    segment_start = idx + 1;
                }
                '}' => {
                    scope_stack.pop();
                    segment_start = idx + 1;
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
    fn test_groovy_private_class_members_not_leaked() {
        // Regression test: members of a `private` class must not be treated as exported
        // just because they sit inside a "type body" scope. Only the outer declaration's
        // own visibility should gate whether its body counts as exportable.
        let src = r#"
private class Hidden {
    void method() {}
    int secretField
}

class Visible {
    void publicMethod() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(!symbols.contains(&"Hidden".to_string()));
        assert!(!symbols.contains(&"method".to_string()));
        assert!(!symbols.contains(&"secretField".to_string()));
        assert!(symbols.contains(&"Visible".to_string()));
        assert!(symbols.contains(&"publicMethod".to_string()));
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

    #[test]
    fn test_groovy_three_level_nested_classes_all_public() {
        // Verified-correct: a class nested inside a class nested inside a class, all with
        // zero modifiers, are each public by default -- every level's name and every
        // level's members should surface, since the whole chain of enclosing scopes is
        // exportable.
        let src = r#"
class Outer {
    class Middle {
        class Inner {
            void innerMethod() {}
        }
        void middleMethod() {}
    }
    void outerMethod() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Outer".to_string()));
        assert!(symbols.contains(&"Middle".to_string()));
        assert!(symbols.contains(&"Inner".to_string()));
        assert!(symbols.contains(&"innerMethod".to_string()));
        assert!(symbols.contains(&"middleMethod".to_string()));
        assert!(symbols.contains(&"outerMethod".to_string()));
    }

    #[test]
    fn test_groovy_private_middle_class_hides_deeper_nesting() {
        // Verified-correct: when the *middle* of a 3-level nesting chain is `private`, both
        // the private class itself and everything nested inside it (including a class that
        // itself has zero modifiers) must be excluded -- visibility narrowing at any level
        // must propagate down through all deeper scopes, not just the immediate children.
        let src = r#"
class Outer {
    private class Middle {
        class Inner {
            void innerMethod() {}
        }
        void middleMethod() {}
    }
    void outerMethod() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Outer".to_string()));
        assert!(symbols.contains(&"outerMethod".to_string()));
        assert!(!symbols.contains(&"Middle".to_string()));
        assert!(!symbols.contains(&"Inner".to_string()));
        assert!(!symbols.contains(&"innerMethod".to_string()));
        assert!(!symbols.contains(&"middleMethod".to_string()));
    }

    #[test]
    fn test_groovy_multiline_class_header_still_exports_members() {
        // Real bug found via adversarial testing: a multi-line class header --
        // `class Foo` on one line, `extends Bar` / `implements Baz {` on following lines --
        // puts the `class` keyword on an earlier line than the `{` that actually opens the
        // body. `opens_type_body` used to be derived only from the line containing `{`, so
        // the continuation line `implements Baz {` (which itself matches none of
        // `class|interface|trait|enum`) looked like an ordinary block open and wrongly
        // pushed `false` onto `scope_stack`, hiding every member of `Foo` -- `method` and
        // `anotherMethod` -- as if they were locals, even though `Foo` itself was correctly
        // captured. Fixed via `pending_type_body_open`, which carries the type-decl line's
        // own exportability forward across header-continuation lines until the `{` that
        // consumes it is finally seen.
        let src = r#"
class Foo
        extends Bar
        implements Baz {
    void method() {}
    def anotherMethod() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Foo".to_string()));
        assert!(
            symbols.contains(&"method".to_string()),
            "regression: multi-line class header desynced scope tracking and hid this member"
        );
        assert!(
            symbols.contains(&"anotherMethod".to_string()),
            "regression: multi-line class header desynced scope tracking and hid this member"
        );
    }

    #[test]
    fn test_groovy_generic_bounds_do_not_confuse_scope_tracking() {
        // Verified-correct: a bounded generic parameter (`<T extends Comparable<T>>`) written
        // on the same line as the class keyword must not confuse the type-decl match (which
        // only looks for the class name immediately after `class`) or the brace/scope
        // tracker -- the class and its public member should surface, its private field
        // should not.
        let src = r#"
class Container<T extends Comparable<T>> {
    private List<T> items = []

    void add(T item) {
        items.add(item)
    }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Container".to_string()));
        assert!(symbols.contains(&"add".to_string()));
        assert!(!symbols.contains(&"items".to_string()));
    }

    #[test]
    fn test_groovy_gstring_with_nested_braces_in_interpolation() {
        // Verified-correct: a GString interpolation that itself contains a closure with
        // braces (`"${foo { it }}"`) is still a single-line double-quoted literal that
        // STRING_DOUBLE strips wholesale before scope tracking ever sees its characters --
        // the closure's `{`/`}` must not be counted as real scope braces, which would
        // otherwise desync `scope_stack` and hide the next member (`other`).
        let src = r#"
class Greeter {
    String greet(String name) {
        return "${foo { it }}"
    }

    void other() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Greeter".to_string()));
        assert!(symbols.contains(&"greet".to_string()));
        assert!(
            symbols.contains(&"other".to_string()),
            "nested braces inside a GString interpolation must not desync scope tracking"
        );
    }

    #[test]
    fn test_groovy_triple_quoted_single_line_string_braces() {
        // Verified-correct: a triple-quoted GString (`"""..."""`) written entirely on one
        // line, containing literal braces, must not leak those braces into scope tracking.
        let src = r#"
class Templater {
    def render() {
        return """Hello { world }"""
    }

    void other() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Templater".to_string()));
        assert!(symbols.contains(&"render".to_string()));
        assert!(symbols.contains(&"other".to_string()));
    }

    #[test]
    fn test_groovy_triple_quoted_multiline_string_with_balanced_braces() {
        // Verified-correct (now handled by design, not by luck): a triple-quoted GString
        // spanning multiple physical lines, containing braces that happen to balance across
        // the split, must not corrupt scope tracking. `STRING_TRIPLE_DOUBLE` now strips the
        // whole literal (dot-matches-newline) before the line-based scope tracker ever runs.
        let src = "
class Templater {
    def render() {
        return \"\"\"Hello {
        world }\"\"\"
    }

    void other() {}
}
";
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Templater".to_string()));
        assert!(symbols.contains(&"render".to_string()));
        assert!(symbols.contains(&"other".to_string()));
    }

    #[test]
    fn test_groovy_triple_quoted_multiline_string_with_unbalanced_braces() {
        // Real bug found via adversarial testing: a triple-quoted GString spanning multiple
        // physical lines with an *unbalanced* brace count inside it (three `{` and zero `}`
        // on the opening line) used to desync `scope_stack`, because `STRING_DOUBLE` only
        // matches single-line strings -- the leftover, unstripped `{` characters were
        // scanned as real scope-opening braces and hid `other` (the next real member) as if
        // it were a local. Fixed by adding `STRING_TRIPLE_DOUBLE`/`STRING_TRIPLE_SINGLE`,
        // dot-matches-newline regexes that strip triple-quoted literals in full (dropping any
        // brace characters inside them) before the line-based scope tracker ever runs.
        let src = "
class Templater {
    def render() {
        return \"\"\"Hello { { {
        world\"\"\"
    }

    void other() {}
}
";
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Templater".to_string()));
        assert!(symbols.contains(&"render".to_string()));
        assert!(
            symbols.contains(&"other".to_string()),
            "regression: unbalanced braces inside a multi-line triple-quoted string desynced \
             scope tracking and hid this legitimate public method"
        );
    }

    #[test]
    fn test_groovy_one_liner_class_and_method_stubs() {
        // Verified-correct: fully one-line stubs (`class Foo {}`, `def baz() {}`) push and
        // pop their scope on the very same line -- the net effect on `scope_stack` must be
        // zero so later declarations are scoped correctly, and a `private` one-liner method
        // must still be excluded.
        let src = r#"
class Foo {}

class Bar {
    def baz() {}
    private def hidden() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Foo".to_string()));
        assert!(symbols.contains(&"Bar".to_string()));
        assert!(symbols.contains(&"baz".to_string()));
        assert!(!symbols.contains(&"hidden".to_string()));
    }

    #[test]
    fn test_groovy_class_nested_in_interface_mixed_nesting() {
        // Verified-correct: a class nested directly inside an interface body (mixed
        // type-body nesting, not just class-in-class) is still a real export candidate when
        // public, and a `private` class nested in the same interface -- along with its own
        // member -- must still be excluded.
        let src = r#"
interface Container {
    class Impl {
        void run() {}
    }
    private class HiddenImpl {
        void secret() {}
    }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Container".to_string()));
        assert!(symbols.contains(&"Impl".to_string()));
        assert!(symbols.contains(&"run".to_string()));
        assert!(!symbols.contains(&"HiddenImpl".to_string()));
        assert!(!symbols.contains(&"secret".to_string()));
    }

    #[test]
    fn test_groovy_private_closure_field_hides_its_body_locals() {
        // Verified-correct: a `private` closure assigned to a `def` field, with a body
        // spanning multiple lines (`{ input -> ... }`), must have both the field itself and
        // every local declared inside its body excluded -- the closure's own braces still
        // need to be tracked as a (non-exportable) scope so a later public member isn't
        // affected either way.
        let src = r#"
class Worker {
    private def handler = { input ->
        def scratch = input * 2
        scratch
    }

    def publicOne() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Worker".to_string()));
        assert!(symbols.contains(&"publicOne".to_string()));
        assert!(!symbols.contains(&"handler".to_string()));
        assert!(!symbols.contains(&"scratch".to_string()));
    }

    #[test]
    fn test_groovy_packed_same_line_private_nested_class_not_leaked() {
        // Real bug found via SECOND adversarial pass: when a line packs multiple unmatched
        // `{` characters together -- e.g. a type declaration opening its body on the same
        // physical line as a *nested* declaration that also opens its body, such as
        // `class Outer { private class Hidden {` -- the old code computed `opens_type_body`
        // ONCE for the whole line and pushed that single value for every `{` encountered.
        // Since the outer `class Outer` half of the line is itself exportable, every brace on
        // the line -- including the one opening the nested `private class Hidden` -- was
        // pushed as `true`, wrongly treating `Hidden`'s body as an exportable type scope and
        // leaking its member `secret` as a public export even though `Hidden` is private.
        // Fixed by re-deriving each brace's own classification from the text since the
        // previous brace on the same line (and the current top of `scope_stack`, reflecting
        // any same-line pushes already applied), rather than reusing one line-wide value.
        let src = r#"
class Outer { private class Hidden {
    void secret() {}
}
    void outerMethod() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Outer".to_string()));
        assert!(symbols.contains(&"outerMethod".to_string()));
        assert!(!symbols.contains(&"Hidden".to_string()));
        assert!(
            !symbols.contains(&"secret".to_string()),
            "regression: packing a private nested class's opening brace onto the same line as \
             its enclosing public class's opening brace desynced per-brace classification and \
             leaked this member of the private class as a public export"
        );
    }

    #[test]
    fn test_groovy_packed_same_line_class_and_method_open_locals_not_leaked() {
        // Real bug found via SECOND adversarial pass: a common idiomatic style packs a type
        // declaration's brace and its very first member's opening brace onto the same line,
        // e.g. `class Config { static Map defaults() {`. The old single-value-per-line logic
        // pushed `opens_type_body` (true, since `Config` is exportable) for BOTH the class's
        // `{` and the method's `{`, so a local variable declared on the next line inside
        // `defaults()`'s body (`result`) was wrongly treated as sitting directly in the class
        // body and leaked as a public export. Fixed by the same per-brace re-derivation as
        // above: only the first brace on a line inherits the line-wide `opens_type_body`
        // value; subsequent braces re-check whether their own preceding segment text is
        // itself a type-body opener.
        //
        // Note: `defaults` itself is NOT asserted here. Capturing it hits a separate,
        // pre-existing, unrelated limitation -- the declaration matcher only ever looks for
        // (and captures) a single declaration per physical line (an `if`/`else if` chain over
        // GROOVY_TYPE_DECL / GROOVY_DEF_DECL / GROOVY_TYPED_MEMBER), so when `class Config {`
        // and `static Map defaults() {` share one physical line, only the first ("Config")
        // is ever matched. That is out of scope for this scope-tracking-focused pass; what
        // this test actually targets is whether packing the two `{`s together corrupts the
        // *scope classification* used to gate `result`.
        let src = r#"
class Config { static Map defaults() {
    def result = [:]
    return result
} }
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Config".to_string()));
        assert!(
            !symbols.contains(&"result".to_string()),
            "regression: packing a class's opening brace and its method's opening brace onto \
             the same line desynced per-brace classification and leaked this method-local \
             variable as a public export"
        );
    }

    #[test]
    fn test_groovy_one_liner_triple_nested_classes_all_public() {
        // Verified-correct: three levels of class nesting packed entirely onto one physical
        // line (`class A { class B { class C {} } }`) must still surface every level's name,
        // and the net brace depth must return to baseline (zero) so a script-level
        // declaration on the very next line is correctly seen as top-level, not still nested.
        //
        // Note: `B` and `C` are NOT asserted here -- capturing them hits the same
        // pre-existing, unrelated "one declaration per physical line" limitation noted in
        // `test_groovy_packed_same_line_class_and_method_open_locals_not_leaked` above (three
        // type declarations sharing one line means only the first, "A", is ever matched).
        // What this test actually targets is brace *depth* bookkeeping: does scanning three
        // opens and three closes packed on one line net back to exactly zero, so the
        // following top-level declaration is correctly seen as top-level rather than still
        // nested (which would silently swallow it, or worse, ripple a sign error through the
        // rest of the file).
        let src = r#"
class A { class B { class C {} } }
def topLevelHelper() {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"A".to_string()));
        assert!(
            symbols.contains(&"topLevelHelper".to_string()),
            "brace depth must return exactly to baseline after the one-liner nesting so this \
             following top-level declaration is not wrongly treated as still nested"
        );
    }

    #[test]
    fn test_groovy_gstring_with_embedded_apostrophe_does_not_desync_single_quotes() {
        // Verified-correct: a double-quoted GString containing an unescaped single quote
        // (`"it's a ${value} test"`) must be consumed in full by STRING_DOUBLE -- which runs
        // before STRING_SINGLE -- so the embedded apostrophe is removed along with the rest
        // of the literal and never has a chance to make STRING_SINGLE falsely start a
        // single-quoted-string match from that apostrophe onward (which would otherwise eat
        // everything up to the next real `'` in the file, including subsequent code).
        let src = r#"
class Notes {
    String describe(String value) {
        return "it's a ${value} test"
    }

    void other() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Notes".to_string()));
        assert!(symbols.contains(&"describe".to_string()));
        assert!(
            symbols.contains(&"other".to_string()),
            "an unescaped apostrophe inside a double-quoted GString must not desync \
             single-quoted-string stripping and hide this following member"
        );
    }

    #[test]
    fn test_groovy_slashy_string_with_balanced_braces_does_not_desync() {
        // Documents a real, accepted pre-existing limitation: Groovy's slashy-string regex
        // literals (`/pattern/`) are not recognized or stripped by this file at all (there is
        // no SLASHY_STRING regex). When the brace characters embedded in one happen to
        // balance -- e.g. `/\{.*\}/`, exactly one literal `{` and one literal `}` -- the
        // naive char-by-char scope scan still nets to zero for that line by coincidence, so
        // this common case (a regex matching a single brace pair) is not actually broken in
        // practice.
        let src = r#"
class Matcher {
    def re = /\{.*\}/

    void other() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Matcher".to_string()));
        assert!(symbols.contains(&"re".to_string()));
        assert!(symbols.contains(&"other".to_string()));
    }

    #[test]
    fn test_groovy_slashy_string_with_unbalanced_braces_is_a_known_limitation() {
        // Documents a real, accepted pre-existing limitation (not fixed by this pass, scoped
        // as future work): because slashy strings (`/like this/`) have no dedicated stripping
        // regex, a slashy string whose *unescaped* brace count is unbalanced -- e.g.
        // `/\{\{\{/,` three literal `{` and zero `}` -- desyncs `scope_stack` exactly like an
        // unstripped triple-quoted string used to. This is asserted here as documentation of
        // current (imperfect) behavior, not as a requirement: `other` is expected to be
        // dropped by the existing desync, matching triple-quoted strings before
        // STRING_TRIPLE_DOUBLE/STRING_TRIPLE_SINGLE were added. Slashy-string support would
        // need its own regex (tricky: `/` also means division) and is out of scope here.
        let src = r#"
class Matcher {
    def re = /\{\{\{/

    void other() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Matcher".to_string()));
        assert!(symbols.contains(&"re".to_string()));
        assert!(
            !symbols.contains(&"other".to_string()),
            "known limitation: unbalanced literal braces inside an unrecognized slashy string \
             still desync scope tracking (no SLASHY_STRING stripping regex exists); if this \
             ever starts passing because slashy-string support was added, this assertion \
             should be flipped to `assert!(symbols.contains(...))`"
        );
    }

    #[test]
    fn test_groovy_comment_with_stray_brace_at_deepest_nesting_level() {
        // Real bug found via SECOND adversarial pass: `COMMENT_SINGLE` (`//.*$`) was missing
        // the `(?m)` flag. Without it, `$` only anchors to the absolute end of the *whole
        // file* (the regex crate's non-multiline default), not the end of each line -- so a
        // `//` comment anywhere but the file's literal last line never actually matched and
        // was never stripped. That went unnoticed in earlier tests because ordinary comment
        // text rarely looks like a declaration and stray comment braces were usually
        // balanced. Here, a `//` comment with a genuinely *unbalanced* brace sitting deep
        // inside a 3-level class nesting (and followed by more file content, so not on the
        // last line) desynced `scope_stack` and hid `outerMethod` -- a shallower, unrelated
        // member -- as if it were still nested. Fixed by adding `(?m)` to `COMMENT_SINGLE`.
        let src = r#"
class Outer {
    class Middle {
        class Inner {
            // stray unbalanced brace in a comment: {
            void innerMethod() {}
        }
    }
    void outerMethod() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Outer".to_string()));
        assert!(symbols.contains(&"Middle".to_string()));
        assert!(symbols.contains(&"Inner".to_string()));
        assert!(symbols.contains(&"innerMethod".to_string()));
        assert!(
            symbols.contains(&"outerMethod".to_string()),
            "a stray unbalanced brace inside a comment, even at the deepest nesting level, \
             must not desync scope tracking for shallower levels"
        );
    }
}
