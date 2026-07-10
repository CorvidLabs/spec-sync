use regex::Regex;
use std::sync::LazyLock;

static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"//.*$").unwrap());

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

/// Extract public symbols from Vala source code.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = COMMENT_SINGLE.replace_all(content, "");
    let stripped = COMMENT_MULTI.replace_all(&stripped, "");

    let mut symbols = Vec::new();

    for caps in VALA_TYPE.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str().to_string();
            if !symbols.contains(&n) {
                symbols.push(n);
            }
        }
    }

    for caps in VALA_DELEGATE.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str().to_string();
            if !symbols.contains(&n) {
                symbols.push(n);
            }
        }
    }

    for caps in VALA_MEMBER.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str().to_string();
            if !symbols.contains(&n) {
                symbols.push(n);
            }
        }
    }

    for caps in VALA_CTOR.captures_iter(&stripped) {
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

    symbols
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
