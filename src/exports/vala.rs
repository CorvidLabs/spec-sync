use regex::Regex;
use std::sync::LazyLock;

/// `(?m)` is required here (unlike a naive `$`, which without this flag only anchors
/// to the very end of the whole file) so `$` matches at the end of EVERY line, not just
/// the file's last one. Without it, a `//` comment on any line but the last is never
/// actually stripped -- its raw text (including any stray `{`/`}` it happens to
/// contain, e.g. `// this has a stray {`) survives into the brace-scanning pass below
/// and can desync the scope stack, silently dropping real members that lexically
/// follow the comment (see `test_vala_standalone_comment_with_stray_brace_does_not_...`).
static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)//.*$").unwrap());

static COMMENT_MULTI: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/").unwrap());

/// Vala public types: class, struct, interface, enum, errordomain. Delegates are
/// handled separately (`VALA_DELEGATE`) because a delegate's name follows its return
/// type rather than following the `delegate` keyword directly, so it doesn't fit this
/// "keyword immediately followed by the name" shape.
///
/// Vala requires the literal `public` keyword on every declaration that should be
/// part of a module's public surface -- the opposite direction from Kotlin/D/Groovy,
/// which default to public and require an explicit keyword to restrict. Without
/// `public`, a Vala member defaults to a more restricted (roughly internal-like)
/// visibility, so a bare `public` requirement (same direction as csharp.rs/java.rs)
/// is correct here.
static VALA_TYPE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*public\s+(?:abstract\s+)?(?:compact\s+)?(?:class|struct|interface|enum|errordomain)\s+(\w+)",
    )
    .unwrap()
});

/// Vala public delegates: `public delegate <ReturnType> <Name> (...)`, e.g.
/// `public delegate void ForeachFunc (Object obj);`. The declared name follows the
/// return type (and an optional `owned`/`unowned` ownership qualifier on it), not the
/// `delegate` keyword itself, so `VALA_TYPE`'s "keyword then name" shape does not
/// apply. An optional generic parameter list may follow the name before the
/// parameter list opens.
static VALA_DELEGATE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*public\s+delegate\s+(?:owned\s+)?(?:unowned\s+)?(?:[\w.]+(?:<[^>]*>)?(?:\[\])?(?:\?)?)\s+(\w+)(?:<[^>]*>)?\s*\(",
    )
    .unwrap()
});

/// Vala public members: methods, properties, fields, and signals. Requires the
/// literal `public` keyword (see `VALA_TYPE` doc for why that's the correct
/// direction for Vala).
///
/// `owned`/`unowned` are Vala ownership-transfer qualifiers written directly on a
/// return type and are very common on real getters, e.g. `public owned string
/// to_string ()` or `public unowned string get_name ()` -- without accounting for
/// them the type token would swallow the qualifier and the real type/name pair would
/// misalign. `signal` marks a GObject signal declaration, e.g. `public signal void
/// changed ()` -- signals are connectable/emittable from outside the declaring type
/// exactly like a method call, so they are as much a part of the public API surface
/// as an ordinary method and must not be dropped just because they end in `;` with no
/// body. The terminator class includes `=` (not just `({;`) so public fields with an
/// initializer, e.g. `public string name = "unknown";`, are still captured.
static VALA_MEMBER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*public\s+(?:static\s+)?(?:abstract\s+)?(?:virtual\s+)?(?:override\s+)?(?:async\s+)?(?:extern\s+)?(?:inline\s+)?(?:dynamic\s+)?(?:new\s+)?(?:signal\s+)?(?:owned\s+)?(?:unowned\s+)?(?:[\w.]+(?:<[^>]*>)?(?:\[\])?(?:\?)?)\s+(\w+)(?:<[^>]*>)?\s*[({;=]",
    )
    .unwrap()
});

/// Vala constructors: `public ClassName (...) { ... }` (the canonical/default
/// constructor) and `public ClassName.ctor_name (...) { ... }` (a named constructor,
/// e.g. `public Foo.from_file (string path) { ... }`, invoked by consumers as `new
/// Foo.from_file (...)`). Unlike ordinary members, constructors have no separate
/// return-type token before the name -- just `public` followed directly by a single
/// identifier (optionally dotted with a constructor name) and `(` -- so
/// `VALA_MEMBER`, which requires a distinct type-then-name pair, never matches them.
static VALA_CTOR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[^\S\n]*public\s+(\w+)(?:\.(\w+))?\s*\(").unwrap());

/// Detect a line that opens a type body (class/struct/interface). A member declared as
/// `public` only counts as part of the public surface if its *enclosing* type is itself
/// exported -- a `public` method on a non-`public` (and thus non-exported) class is not
/// reachable from outside the module, so it must not leak just because the member's own
/// line happens to say `public`.
static VALA_TYPE_BODY_OPEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*(?:public\s+)?(?:abstract\s+)?(?:compact\s+)?(?:class|struct|interface)\b",
    )
    .unwrap()
});

/// Replace the contents of Vala string literals -- regular `"..."` (including
/// interpolated `@"..."`, which uses the same `"`-delimited shape) and verbatim
/// `"""..."""` -- with spaces, while preserving delimiters, all other characters,
/// and every newline. Verbatim strings are Vala's mechanism for embedding raw
/// multi-line text and are common enough (e.g. embedded templates/queries) that a
/// literal `{`/`}` inside one must not be mistaken for a real scope-opening or
/// scope-closing brace by the line-by-line brace scan below; the same is true for
/// an ordinary single-line string that merely happens to contain a stray `{` or
/// `}` character. Masking here (rather than trying to make the brace scanner
/// itself string-aware) also stops arbitrary string content that happens to
/// resemble a declaration (e.g. doc text reading "public class Foo") from being
/// misread by the export-detection regexes, since those regexes run on this same
/// masked text.
fn mask_string_literals(content: &str) -> String {
    let chars: Vec<char> = content.chars().collect();
    let mut out = String::with_capacity(chars.len());
    let mut i = 0;
    let mut in_verbatim = false;
    let mut in_regular = false;

    while i < chars.len() {
        let ch = chars[i];

        if in_verbatim {
            if ch == '"' && chars.get(i + 1) == Some(&'"') && chars.get(i + 2) == Some(&'"') {
                out.push_str("\"\"\"");
                i += 3;
                in_verbatim = false;
            } else {
                out.push(if ch == '\n' { '\n' } else { ' ' });
                i += 1;
            }
            continue;
        }

        if in_regular {
            if ch == '\\' && i + 1 < chars.len() {
                // Blank the whole escape sequence (`\"`, `\\`, ...) as a pair so an
                // escaped quote can never be misread as the string's closing quote.
                out.push(' ');
                out.push(' ');
                i += 2;
                continue;
            }
            if ch == '"' {
                out.push('"');
                in_regular = false;
                i += 1;
                continue;
            }
            if ch == '\n' {
                // Regular strings don't span lines; bail out defensively instead of
                // swallowing the rest of the file if one is unterminated.
                in_regular = false;
                out.push('\n');
                i += 1;
                continue;
            }
            out.push(' ');
            i += 1;
            continue;
        }

        if ch == '"' && chars.get(i + 1) == Some(&'"') && chars.get(i + 2) == Some(&'"') {
            out.push_str("\"\"\"");
            i += 3;
            in_verbatim = true;
            continue;
        }
        if ch == '"' {
            out.push('"');
            in_regular = true;
            i += 1;
            continue;
        }

        out.push(ch);
        i += 1;
    }

    out
}

/// Extract public symbols from Vala source code.
pub fn extract_exports(content: &str) -> Vec<String> {
    // String literals are masked FIRST, before either comment regex runs. Both
    // `COMMENT_SINGLE` (`//...`) and `COMMENT_MULTI` (`/*...*/`) operate on raw text with
    // no notion of string literals; if they ran first, a `//` or `/*` sequence occurring
    // inside a string (e.g. a URL like `"http://example.com"`) would be misread as a
    // real comment start. For `COMMENT_SINGLE` that silently truncates the rest of the
    // physical line -- dropping any real trailing declaration and, worse, half of a
    // brace pair if the truncated text contained one. For `COMMENT_MULTI`, which is
    // non-greedy but spans lines via `(?s)`, a stray `/*`-like sequence inside a string
    // could swallow arbitrarily much genuine code up to the next real `*/`. Masking
    // string interiors to spaces first (while preserving every newline, so line numbers
    // and line-based scanning below stay aligned) removes any `//`/`/*`-shaped text that
    // was only ever string data, so the comment regexes only ever see real comments.
    let stripped = mask_string_literals(content);
    let stripped = COMMENT_SINGLE.replace_all(&stripped, "");
    let stripped = COMMENT_MULTI.replace_all(&stripped, "");

    let mut symbols = Vec::new();
    // Tracks nested `{ }` scopes: true = a `public` class/struct/interface body (member
    // declarations are legitimate export candidates), false = a non-public type body, a
    // method body, or any other block. An empty stack means top level (namespace scope),
    // which is exportable.
    let mut scope_stack: Vec<bool> = Vec::new();
    // Set when a `public class/struct/interface` header is seen but its line has no `{`
    // (a multi-line header: base-type list, generic constraints, or a standalone brace on
    // a later line). Carried across lines -- untouched by lines that don't themselves open
    // a fresh header -- until the header's own opening `{` is reached, at which point it is
    // consumed. Without this, a header whose brace isn't on the same physical line as the
    // `class`/`struct`/`interface` keyword would look, to the per-brace check below, like it
    // opens on an empty (or base-type-list-only) segment with no keyword at all, and the
    // whole body would be wrongly treated as non-exportable.
    let mut pending_public_header = false;

    for line in stripped.lines() {
        let in_exportable_scope = scope_stack.last().copied().unwrap_or(true);

        if in_exportable_scope {
            if let Some(caps) = VALA_TYPE.captures(line)
                && let Some(name) = caps.get(1)
            {
                let n = name.as_str().to_string();
                if !symbols.contains(&n) {
                    symbols.push(n);
                }
            }

            if let Some(caps) = VALA_DELEGATE.captures(line)
                && let Some(name) = caps.get(1)
            {
                let n = name.as_str().to_string();
                if !symbols.contains(&n) {
                    symbols.push(n);
                }
            }

            if let Some(caps) = VALA_MEMBER.captures(line)
                && let Some(name) = caps.get(1)
            {
                let n = name.as_str().to_string();
                if !symbols.contains(&n) {
                    symbols.push(n);
                }
            }

            if let Some(caps) = VALA_CTOR.captures(line) {
                // Named constructor (`Foo.bar`) captures the constructor name in group 2;
                // otherwise fall back to group 1, the canonical constructor's identifier
                // (which is just the class name again -- redundant with `VALA_TYPE`, but
                // deduped below).
                let name = caps.get(2).or_else(|| caps.get(1));
                if let Some(name) = name {
                    let n = name.as_str().to_string();
                    if !symbols.contains(&n) {
                        symbols.push(n);
                    }
                }
            }
        }

        // A header line (`public class Foo<G>` with no `{` yet) sets the pending flag so a
        // later line's brace can still recognize the body as exportable. This must be
        // checked against `in_exportable_scope`, matching the per-brace check below, so a
        // header nested inside non-exported territory can't set a pending flag that would
        // wrongly grant its body exportability once its `{` is reached.
        if in_exportable_scope
            && VALA_TYPE_BODY_OPEN.is_match(line)
            && line.trim_start().starts_with("public")
            && !line.contains('{')
        {
            pending_public_header = true;
        }

        // Walk the line brace-by-brace (rather than computing one push-value for the whole
        // line) because more than one `{` can appear on a single physical line -- e.g. two
        // one-liner/compact declarations opened back-to-back, or a namespace and a type
        // opened together as `namespace Foo { class Bar {` -- and each brace can belong to a
        // different declaration with different visibility. Reusing a single line-wide value
        // for every brace on the line previously let a later, non-public declaration on the
        // same line inherit an earlier, unrelated declaration's (or the namespace's) visibility.
        let mut seg_start = 0usize;
        for (idx, ch) in line.char_indices() {
            match ch {
                '{' => {
                    // Only the text since the previous brace on this line (or the start of
                    // the line, for the first brace) can describe what this particular brace
                    // opens; text belonging to an earlier segment must not leak in.
                    let segment = &line[seg_start..idx];
                    let cur_scope = scope_stack.last().copied().unwrap_or(true);
                    // Only treat a freshly-opened type body as an exportable scope if the
                    // type itself is in exported territory here -- otherwise a member of a
                    // non-`public` class (or one nested inside a method body) would still be
                    // treated as sitting directly in an exportable scope, leaking it even
                    // though its enclosing type is correctly excluded above.
                    let segment_opens_type = VALA_TYPE_BODY_OPEN.is_match(segment)
                        && segment.trim_start().starts_with("public");
                    let opens_type_body =
                        cur_scope && (segment_opens_type || pending_public_header);
                    // `namespace` bodies don't carry their own visibility -- they're purely
                    // organizational -- so a namespace's `{` inherits whatever scope was
                    // already in effect instead of unconditionally pushing `false` (which
                    // would otherwise wall off every declaration inside a top-level
                    // `namespace { ... }` block).
                    let opens_namespace = segment.trim_start().starts_with("namespace");
                    let push_value = if opens_namespace {
                        cur_scope
                    } else {
                        opens_type_body
                    };
                    scope_stack.push(push_value);
                    pending_public_header = false;
                    seg_start = idx + ch.len_utf8();
                }
                '}' => {
                    scope_stack.pop();
                    seg_start = idx + ch.len_utf8();
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
    fn test_vala_two_scopes_opened_on_one_line_dont_share_visibility() {
        // Regression test: before the fix, brace-push visibility was computed ONCE per
        // physical line and reused for every `{` on that line. Here `public class Outer`
        // and the nested, non-public `class Inner` both open their `{` on the same line --
        // the old code used `Outer`'s (exportable) push-value for *both* braces, so
        // `Inner`'s body was wrongly treated as exportable and `leak` leaked out even
        // though `Inner` itself has no `public` keyword. Each brace must be evaluated
        // against only the text since the previous brace on that line.
        let src = r#"
public class Outer { class Inner {
    public void leak () { }
}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Outer".to_string()));
        assert!(!symbols.contains(&"Inner".to_string()));
        assert!(!symbols.contains(&"leak".to_string()));
    }

    #[test]
    fn test_vala_namespace_and_class_opened_on_one_line_dont_share_visibility() {
        // Regression test: same root cause as the sibling test above, but via the
        // `namespace`-inherits-current-scope path instead of two type declarations. Before
        // the fix, `namespace Foo { class Bar {` computed a single line-wide push-value
        // (top-level "true", since a namespace inherits whatever scope was already in
        // effect) and applied it to *both* braces, so the non-public `class Bar`'s body was
        // wrongly marked exportable and `leak` leaked out.
        let src = r#"
namespace Foo { class Bar {
    public void leak () { }
}
}
"#;
        let symbols = extract_exports(src);
        assert!(!symbols.contains(&"Bar".to_string()));
        assert!(!symbols.contains(&"leak".to_string()));
    }

    #[test]
    fn test_vala_brace_inside_string_literal_does_not_desync_scope_stack() {
        // Regression test: the brace scanner previously walked every character of a line
        // looking for `{`/`}`, including characters inside string literals. A stray `}`
        // inside a string (`"}"`) was read as a real scope-closing brace, popping the
        // non-public `Hidden` class's `false` entry off the stack one line early. The
        // stack then defaulted back to top-level (exportable), so the very next line's
        // `public void leak ()` -- still lexically inside `Hidden`'s body -- was wrongly
        // treated as top-level and exported. String literal contents must now be masked
        // out before brace scanning.
        let src = r#"
class Hidden : Object {
    public string weird = "}";
    public void leak () { }
}
"#;
        let symbols = extract_exports(src);
        assert!(!symbols.contains(&"Hidden".to_string()));
        assert!(!symbols.contains(&"leak".to_string()));
    }

    #[test]
    fn test_vala_verbatim_string_braces_and_fake_declaration_text_ignored() {
        // Regression test: Vala's `"""..."""` verbatim strings can span multiple lines and
        // are commonly used to embed raw multi-line text (templates, queries, etc). A
        // verbatim string containing `{`/`}` characters, and even text that reads like a
        // real declaration ("public class Fake {}"), must not desync the scope stack or be
        // misread by the export-detection regexes -- both operate on the same masked text,
        // so string interiors must be blanked out (not just brace-scanned around).
        let src = r#"
public class Templates : Object {
    public string body = """
    { this looks like a brace block }
    public class Fake {}
    """;
    public void real_method () { }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Templates".to_string()));
        assert!(symbols.contains(&"body".to_string()));
        assert!(symbols.contains(&"real_method".to_string()));
        assert!(!symbols.contains(&"Fake".to_string()));
    }

    #[test]
    fn test_vala_multiline_class_header_with_brace_on_its_own_line() {
        // Regression test: when a `public class`/`interface` header's `{` is on a later
        // line than the `class`/`struct`/`interface` keyword (e.g. a multi-line base-type
        // or generic-constraint list), the per-brace check previously only looked at the
        // text since the last brace on the SAME line, saw no `class` keyword on a line
        // containing only `{`, and treated the whole body as non-exportable -- silently
        // dropping every member. A pending-header flag now carries the header's
        // exportability forward across intervening lines to the brace that finally opens it.
        let src = r#"
public class Wide<G, H>
    : Object
{
    public G left;
    public H right;
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Wide".to_string()));
        assert!(symbols.contains(&"left".to_string()));
        assert!(symbols.contains(&"right".to_string()));
    }

    #[test]
    fn test_vala_generic_class_with_generic_base_types() {
        // Verified-correct: `<G>` on the class name and `<G>` on each base type in the
        // implements/extends list must not be mistaken for scope-opening/closing braces,
        // and the generic parameter list must not swallow the following name (`get_item`)
        // or leak the private field.
        let src = r#"
public class Repo<G> : Collection<G>, Iterable<G> {
    public G get_item () { return this.item; }
    private G item;
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Repo".to_string()));
        assert!(symbols.contains(&"get_item".to_string()));
        assert!(!symbols.contains(&"item".to_string()));
    }

    #[test]
    fn test_vala_oneliner_stubs_do_not_corrupt_subsequent_top_level_scope() {
        // Verified-correct: `public class Empty {}` and `public interface Marker {}` each
        // open and close their `{`/`}` pair on the same line, so the scope stack must
        // return exactly to the top-level baseline after each -- if the push/pop didn't
        // net out to zero, the following `Next` class (and its member) would be
        // misclassified.
        let src = r#"
public class Empty {}
public interface Marker {}
public class Next : Object {
    public void after_stub () { }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Empty".to_string()));
        assert!(symbols.contains(&"Marker".to_string()));
        assert!(symbols.contains(&"Next".to_string()));
        assert!(symbols.contains(&"after_stub".to_string()));
    }

    #[test]
    fn test_vala_private_nested_class_with_public_looking_method_not_leaked() {
        // Verified-correct: a `private` nested class's `public` method must not leak just
        // because the method's own line says `public` -- its enclosing `Secret` class is
        // not itself exported, so nothing inside it is reachable from outside the module.
        let src = r#"
public class Container : Object {
    private class Secret : Object {
        public void reveal_looking_name () { }
    }
    public void real_public_method () { }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Container".to_string()));
        assert!(symbols.contains(&"real_public_method".to_string()));
        assert!(!symbols.contains(&"Secret".to_string()));
        assert!(!symbols.contains(&"reveal_looking_name".to_string()));
    }

    #[test]
    fn test_vala_mixed_class_namespace_class_nesting_cascades_correctly() {
        // Verified-correct: mixed namespace/class nesting must cascade exportability in
        // both directions. A `public class` nested inside a `namespace` nested inside a
        // `public class` is still exportable (positive case); the same shape rooted in a
        // non-public outer class must exclude everything inside it, even a `public`-looking
        // inner class and method (negative case), matching the documented "enclosing type
        // must itself be exported" rule.
        let positive = r#"
public class Outer : Object {
    namespace Inner {
        public class Deep : Object {
            public void surface () { }
        }
    }
}
"#;
        let positive_symbols = extract_exports(positive);
        assert!(positive_symbols.contains(&"Outer".to_string()));
        assert!(positive_symbols.contains(&"Deep".to_string()));
        assert!(positive_symbols.contains(&"surface".to_string()));

        let negative = r#"
class HiddenOuter : Object {
    namespace Inner {
        public class ShouldNotLeak : Object {
            public void method () { }
        }
    }
}
"#;
        let negative_symbols = extract_exports(negative);
        assert!(negative_symbols.is_empty());
    }

    #[test]
    fn test_vala_escaped_quote_inside_string_does_not_end_string_early() {
        // Verified-correct: a backslash-escaped quote (`\"`) inside a regular string must
        // not be read as the string's closing delimiter -- if it were, the following `{`
        // in the same string would be treated as a real, unterminated scope-opening brace
        // and everything after it (including `after_escape`) would be misclassified.
        let src = r#"
public class Quoter : Object {
    public string s = "a \" b { c";
    public void after_escape () { }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Quoter".to_string()));
        assert!(symbols.contains(&"s".to_string()));
        assert!(symbols.contains(&"after_escape".to_string()));
    }

    #[test]
    fn test_vala_exports() {
        let src = r#"
namespace Example.Auth {
    public class AuthService : Object {
        public string validate (string token) { return token; }
        private void internal_check () { }
        public static AuthService instance;
        public int timeout;
    }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"AuthService".to_string()));
        assert!(symbols.contains(&"validate".to_string()));
        assert!(symbols.contains(&"instance".to_string()));
        assert!(symbols.contains(&"timeout".to_string()));
        assert!(!symbols.contains(&"internal_check".to_string()));
    }

    #[test]
    fn test_vala_public_member_in_non_public_class_not_leaked() {
        // Regression test: a `public` method is only part of the module's public surface
        // if its *enclosing* class is itself exported. A `public` method on a class with
        // no `public` keyword (thus not exported) must not leak just because the member's
        // own line says `public`.
        let src = r#"
class Hidden : Object {
    public void method () { }
}

public class Visible : Object {
    public void reveal () { }
}
"#;
        let symbols = extract_exports(src);
        assert!(!symbols.contains(&"Hidden".to_string()));
        assert!(!symbols.contains(&"method".to_string()));
        assert!(symbols.contains(&"Visible".to_string()));
        assert!(symbols.contains(&"reveal".to_string()));
    }

    #[test]
    fn test_vala_private_protected_internal_and_unmarked_excluded() {
        // The core exclusion case: Vala members default to a restricted visibility
        // without an explicit `public` keyword -- unmarked, `private`, `protected`,
        // and `internal` members must all be excluded.
        let src = r#"
public class Api : Object {
    private string secret;
    protected void on_init () { }
    internal void setup () { }
    void unmarked_helper () { }
    public string name;
    public void process () { }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Api".to_string()));
        assert!(symbols.contains(&"name".to_string()));
        assert!(symbols.contains(&"process".to_string()));
        assert!(!symbols.contains(&"secret".to_string()));
        assert!(!symbols.contains(&"on_init".to_string()));
        assert!(!symbols.contains(&"setup".to_string()));
        assert!(!symbols.contains(&"unmarked_helper".to_string()));
    }

    #[test]
    fn test_vala_comments_stripped() {
        let src = r#"
// public class FakeClass : Object {}
/* public struct FakeStruct {} */
/**
 * public enum FakeEnum { A, B }
 */
public class RealClass : Object {
    public void real_method () { }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"RealClass".to_string()));
        assert!(symbols.contains(&"real_method".to_string()));
        assert!(!symbols.contains(&"FakeClass".to_string()));
        assert!(!symbols.contains(&"FakeStruct".to_string()));
        assert!(!symbols.contains(&"FakeEnum".to_string()));
    }

    #[test]
    fn test_vala_interface_members_require_explicit_public() {
        // Unlike C#, Vala interface members are NOT implicitly public -- writing
        // `public abstract` explicitly on every interface member is required and
        // idiomatic (e.g. Gee.Iterable's `public abstract Iterator<G> iterator ()`).
        // A bare `abstract` line with no `public` must be excluded.
        let src = r#"
public interface Reversible : Object {
    public abstract void reverse ();
    public abstract int length { get; }
    abstract void not_exported ();
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Reversible".to_string()));
        assert!(symbols.contains(&"reverse".to_string()));
        assert!(symbols.contains(&"length".to_string()));
        assert!(!symbols.contains(&"not_exported".to_string()));
    }

    #[test]
    fn test_vala_signals() {
        // Signals are a Vala/GObject-specific construct: connectable/emittable from
        // outside the declaring type, so they are public API surface just like a
        // method, even though the declaration has no body.
        let src = r#"
public class Button : Object {
    public signal void clicked ();
    public virtual signal void state_changed (int old_state, int new_state);
    private signal void internal_tick ();
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Button".to_string()));
        assert!(symbols.contains(&"clicked".to_string()));
        assert!(symbols.contains(&"state_changed".to_string()));
        assert!(!symbols.contains(&"internal_tick".to_string()));
    }

    #[test]
    fn test_vala_errordomain() {
        // `errordomain` is Vala's exception-type declaration (used pervasively in
        // GLib/GObject-based APIs, e.g. GLib.FileError, GLib.IOError).
        let src = r#"
public errordomain FileError {
    NOT_FOUND,
    PERMISSION_DENIED
}

public class FileLoader : Object {
    public string load (string path) throws FileError { return ""; }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"FileError".to_string()));
        assert!(symbols.contains(&"FileLoader".to_string()));
        assert!(symbols.contains(&"load".to_string()));
    }

    #[test]
    fn test_vala_named_and_default_constructors() {
        // Named constructors (`ClassName.ctor_name`) are a common Vala idiom for
        // alternate ways to build an instance, e.g. `new Config.from_file (path)`.
        let src = r#"
public class Config : Object {
    public Config () { }
    public Config.from_file (string path) { }
    public Config.with_defaults () { }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Config".to_string()));
        assert!(symbols.contains(&"from_file".to_string()));
        assert!(symbols.contains(&"with_defaults".to_string()));
    }

    #[test]
    fn test_vala_owned_unowned_and_properties() {
        // `owned`/`unowned` ownership-transfer qualifiers directly precede the
        // return type on real Vala getters and must not cause the name to be
        // misdetected as the qualifier or dropped entirely.
        let src = r#"
public class Node : Object {
    public unowned string get_name () { return this._name; }
    public owned string to_string () { return this._name; }
    public int count { get; set; default = 0; }
    public string label { get; construct; }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Node".to_string()));
        assert!(symbols.contains(&"get_name".to_string()));
        assert!(symbols.contains(&"to_string".to_string()));
        assert!(symbols.contains(&"count".to_string()));
        assert!(symbols.contains(&"label".to_string()));
    }

    #[test]
    fn test_vala_delegate() {
        let src = r#"
public delegate void ForeachFunc (Object obj);
public delegate bool Predicate<G> (G item);

public class Collection : Object {
    public void foreach (ForeachFunc func) { }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"ForeachFunc".to_string()));
        assert!(symbols.contains(&"Predicate".to_string()));
        assert!(symbols.contains(&"foreach".to_string()));
    }

    #[test]
    fn test_vala_compact_class_struct_and_generics() {
        let src = r#"
public compact class Point {
    public double x;
    public double y;
}

public struct Vector {
    public double dx;
    public double dy;
}

public class Container<G> : Object {
    public G item;
    public G get_item () { return this.item; }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Point".to_string()));
        assert!(symbols.contains(&"x".to_string()));
        assert!(symbols.contains(&"y".to_string()));
        assert!(symbols.contains(&"Vector".to_string()));
        assert!(symbols.contains(&"dx".to_string()));
        assert!(symbols.contains(&"Container".to_string()));
        assert!(symbols.contains(&"item".to_string()));
        assert!(symbols.contains(&"get_item".to_string()));
    }

    #[test]
    fn test_vala_async_and_static_methods() {
        let src = r#"
public class Downloader : Object {
    public static Downloader instance;
    public async string fetch (string url) throws Error { return ""; }
    public static void main (string[] args) { }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Downloader".to_string()));
        assert!(symbols.contains(&"instance".to_string()));
        assert!(symbols.contains(&"fetch".to_string()));
        assert!(symbols.contains(&"main".to_string()));
    }

    #[test]
    fn test_vala_two_full_declarations_packed_on_one_line_documented_limitation() {
        // Documented, out-of-scope limitation (not a scope-tracking bug): each
        // export-detection regex is run with `.captures()` (first match only), not
        // `.captures_iter()`, so at most one declaration per physical line/segment is
        // ever extracted. Two full declarations on separate lines are both captured
        // (first case below). Two full declarations packed onto the SAME physical line
        // -- `public class A {} public class B {}` -- only yield the first (`A`); `B`
        // is silently missed as a *symbol*. This is purely a regex-extraction gap: the
        // scope stack itself still balances correctly (each `{}` pair nets to zero), as
        // proven by the third class parsing correctly afterward.
        let src = "public class A {}\npublic class B {}\n";
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"A".to_string()));
        assert!(symbols.contains(&"B".to_string()));

        let packed = "public class A {} public class B {}\npublic class C : Object {\n    public void after_packed () { }\n}\n";
        let packed_symbols = extract_exports(packed);
        assert!(packed_symbols.contains(&"A".to_string()));
        assert!(
            !packed_symbols.contains(&"B".to_string()),
            "known limitation: only the first declaration per line is captured as a symbol"
        );
        // Scope integrity is unaffected by the missed second symbol -- the class after
        // the packed line is still correctly recognized as exportable.
        assert!(packed_symbols.contains(&"C".to_string()));
        assert!(packed_symbols.contains(&"after_packed".to_string()));
    }

    #[test]
    fn test_vala_trailing_same_line_comment_with_stray_brace_does_not_desync_scope() {
        // Verified-correct: a `//` comment trailing on the SAME line as a real
        // scope-opening `{` (e.g. `public class Foo : Object { // ... }`) must have its
        // stray, comment-only `}` stripped before brace-scanning, or it would
        // prematurely pop `Foo`'s just-pushed scope entry and misclassify everything
        // that lexically follows.
        let src = r#"
public class Foo : Object { // trailing comment with a stray brace }
    public void bar () { }
}
public class After : Object {
    public void after_method () { }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Foo".to_string()));
        assert!(symbols.contains(&"bar".to_string()));
        assert!(symbols.contains(&"After".to_string()));
        assert!(symbols.contains(&"after_method".to_string()));
    }

    #[test]
    fn test_vala_standalone_comment_with_stray_brace_does_not_desync_scope() {
        // Regression test: `COMMENT_SINGLE` (`//.*$`) previously had no `(?m)` flag, so
        // in the `regex` crate `$` only anchors at the very end of the WHOLE input, not
        // at each line's end -- meaning a `//` comment on any line but the file's last
        // was never actually stripped. A standalone comment line containing a stray,
        // unbalanced `{` (`// this has a stray {`) then survived into the brace-scanning
        // pass, pushed a bogus extra scope layer, and that layer (not `Foo`'s real one)
        // got popped by `Foo`'s real closing `}` -- leaving `Foo`'s own `true` entry
        // stranded on the stack. The concrete, observable symptom: `bar`, a legitimate
        // public method lexically inside `Foo`, was evaluated while the stray layer was
        // on top and silently dropped from the results. Fixed by adding `(?m)` to
        // `COMMENT_SINGLE` so `$` anchors per-line.
        let src = r#"
public class Foo : Object {
    // this has a stray {
    public void bar () { }
}
public class After : Object {
    public void after_method () { }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Foo".to_string()));
        assert!(
            symbols.contains(&"bar".to_string()),
            "bar must not be dropped just because a preceding comment contains a stray {{"
        );
        assert!(symbols.contains(&"After".to_string()));
        assert!(symbols.contains(&"after_method".to_string()));
    }

    #[test]
    fn test_vala_comment_stray_brace_does_not_leak_subsequent_non_public_class() {
        // Sharper companion to the regression test above: confirms the stray-brace
        // desync (now fixed) doesn't leave a stranded scope-stack entry that would
        // *wrongly grant* exportability to a later, genuinely non-public class either
        // -- i.e. the fix doesn't just stop under-exporting, it must also not cause
        // over-exporting as a side effect.
        let src = r#"
public class Foo : Object {
    // this has a stray {
    public void bar () { }
}
class Hidden : Object {
    public void should_not_leak () { }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Foo".to_string()));
        assert!(symbols.contains(&"bar".to_string()));
        assert!(!symbols.contains(&"Hidden".to_string()));
        assert!(!symbols.contains(&"should_not_leak".to_string()));
    }

    #[test]
    fn test_vala_interface_abstract_members_mixed_with_nested_class_with_body() {
        // Verified-correct: an interface can mix brace-less members (abstract method
        // signatures and property declarations ending in `;`) with a nested type that
        // DOES have a real `{ ... }` body. The depth counter must not be confused by
        // this mix -- the nested class's own members must be attributed to it (not to
        // the interface or top level), and parsing must land back at the correct depth
        // afterward so the interface's own remaining member and the next top-level
        // class are still recognized correctly.
        let src = r#"
public interface Shape : Object {
    public abstract double area ();
    public abstract double perimeter ();

    public class Origin : Object {
        public double x;
        public double y;
    }

    public abstract string label { get; }
}

public class AfterInterface : Object {
    public void after_method () { }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Shape".to_string()));
        assert!(symbols.contains(&"area".to_string()));
        assert!(symbols.contains(&"perimeter".to_string()));
        assert!(symbols.contains(&"Origin".to_string()));
        assert!(symbols.contains(&"x".to_string()));
        assert!(symbols.contains(&"y".to_string()));
        assert!(symbols.contains(&"label".to_string()));
        assert!(symbols.contains(&"AfterInterface".to_string()));
        assert!(symbols.contains(&"after_method".to_string()));
    }

    #[test]
    fn test_vala_generic_method_with_requires_constraint_clause() {
        // Verified-correct: a generic method's `requires (T : GLib.Object)` constraint
        // clause sits between the parameter list and the method's real opening `{`. The
        // parens in the constraint must not be mistaken for the parameter list's own
        // terminator, and the clause must not introduce any brace-scanning confusion
        // before the method's actual body opens.
        let src = r#"
public class Box : Object {
    public T foo<T> (T x) requires (T : GLib.Object) {
        return x;
    }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Box".to_string()));
        assert!(symbols.contains(&"foo".to_string()));
    }

    #[test]
    fn test_vala_enum_nested_inside_class_values_not_leaked() {
        // Verified-correct: a `public enum` nested inside a `public class` is itself a
        // legitimate exported type (captured via `VALA_TYPE`, same as class/struct/
        // interface). Its own `{ ... }` body is comma-separated values, not
        // declarations, and must be treated as a non-exportable scope -- so the value
        // names themselves (`RED`, `YELLOW`, `GREEN`) are never mistaken for separate
        // public symbols -- while the enum's own brace pair still balances correctly so
        // sibling and subsequent top-level members are unaffected.
        let src = r#"
public class Traffic : Object {
    public enum Light { RED, YELLOW, GREEN }
    public void go () { }
}
public class AfterEnum : Object {
    public void after_method () { }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Traffic".to_string()));
        assert!(symbols.contains(&"Light".to_string()));
        assert!(!symbols.contains(&"RED".to_string()));
        assert!(!symbols.contains(&"YELLOW".to_string()));
        assert!(!symbols.contains(&"GREEN".to_string()));
        assert!(symbols.contains(&"go".to_string()));
        assert!(symbols.contains(&"AfterEnum".to_string()));
        assert!(symbols.contains(&"after_method".to_string()));
    }

    #[test]
    fn test_vala_namespace_two_siblings_then_third_class_after_double_close() {
        // Verified-correct: a namespace containing two sibling classes -- one public,
        // one not -- followed by a third public class back at namespace level after
        // BOTH inner classes' braces close (two pops) and the namespace's own brace
        // closes (a third pop). The depth counter must return exactly to top level, not
        // over- or under-pop, so the third class is still recognized as exportable.
        let src = r#"
namespace Foo {
    public class SiblingA : Object {
        public void method_a () { }
    }
    class SiblingB : Object {
        public void method_b () { }
    }
}
public class ThirdAtNamespaceLevel : Object {
    public void method_c () { }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"SiblingA".to_string()));
        assert!(symbols.contains(&"method_a".to_string()));
        assert!(!symbols.contains(&"SiblingB".to_string()));
        assert!(!symbols.contains(&"method_b".to_string()));
        assert!(symbols.contains(&"ThirdAtNamespaceLevel".to_string()));
        assert!(symbols.contains(&"method_c".to_string()));
    }

    #[test]
    fn test_vala_url_like_string_with_double_slash_not_mistaken_for_comment() {
        // Regression test: comment-stripping previously ran on raw, unmasked text, so a
        // `//` inside a string literal -- e.g. a URL like `"http://example.com"` -- was
        // indistinguishable from a real `//` comment start. `COMMENT_SINGLE` would match
        // from that `//` to the end of the line and erase the rest of it. In this
        // benign case the erased suffix happens to be only the remaining string
        // content, so the field itself still parses (its name/terminator precede the
        // `//`) -- but the very next member on its own line, `connect_to_server`, must
        // also still be recognized, proving the corruption didn't bleed into subsequent
        // lines. Fixed by masking string literals before either comment regex runs.
        let src = r#"
public class Client : Object {
    public string url = "http://example.com";
    public void connect_to_server () { }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Client".to_string()));
        assert!(symbols.contains(&"url".to_string()));
        assert!(symbols.contains(&"connect_to_server".to_string()));
    }

    #[test]
    fn test_vala_double_slash_in_string_followed_by_code_on_same_line_known_limitation() {
        // Companion to the regression test above, with a second declaration packed
        // after the string on the SAME physical line: `public string url = "http://x";
        // public void trailing () { }`. Now that string-masking runs before
        // comment-stripping, the `//` inside the string no longer truncates the line, so
        // `trailing`'s `{ }` reaches the brace scanner intact and does not desync
        // anything afterward (`After`/`after_method` still parse correctly). `trailing`
        // itself is not captured as a *symbol*, but that is the same pre-existing,
        // documented, out-of-scope "first declaration per line only" limitation covered
        // by `test_vala_two_full_declarations_packed_on_one_line_documented_limitation`
        // -- not a new scope-corruption bug.
        let src = r#"
public class Client : Object {
    public string url = "http://x"; public void trailing () { }
}
public class After : Object {
    public void after_method () { }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Client".to_string()));
        assert!(symbols.contains(&"url".to_string()));
        assert!(symbols.contains(&"After".to_string()));
        assert!(symbols.contains(&"after_method".to_string()));
    }

    #[test]
    fn test_vala_double_slash_in_string_does_not_swallow_opening_brace_of_later_closed_block() {
        // Most severe variant of the same root cause: if comment-stripping ran on raw
        // (unmasked) text and matched a `//` inside a string, it would erase everything
        // after it on that physical line -- including a trailing declaration's OPENING
        // `{` -- while the declaration's matching closing `}` (on a later, separate
        // line) survives untouched. That stray, unmatched `}` would then pop one scope
        // entry too many, permanently desyncing depth-tracking for the rest of the file
        // (verified by temporarily reverting the fix: this exact input then failed to
        // recognize `After`/`after_method`). With string-masking now running before
        // either comment regex, the `//` inside the string is gone (masked) before
        // comment-stripping ever runs, so this failure mode cannot trigger.
        let src = r#"
public class Client : Object {
    public string url = "http://x"; public void trailing () {
        do_something ();
    }
}
public class After : Object {
    public void after_method () { }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Client".to_string()));
        assert!(symbols.contains(&"url".to_string()));
        assert!(symbols.contains(&"After".to_string()));
        assert!(symbols.contains(&"after_method".to_string()));
    }
}
