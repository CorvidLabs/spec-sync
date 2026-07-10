use regex::Regex;
use std::sync::LazyLock;

static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"//.*$").unwrap());

static COMMENT_MULTI: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/").unwrap());

/// C# public/internal types: class, struct, interface, enum, record. The modifier group
/// is a repeated alternation (not a fixed sequence) since C# allows `static`/`partial`/
/// `sealed`/`abstract` in any order and any combination (e.g. `public sealed partial
/// class Config` — `sealed` before `partial` — was previously unmatched by a regex that
/// hardcoded `static, partial, sealed, abstract` as the only accepted order).
static CS_TYPE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*public\s+(?:(?:static|partial|sealed|abstract)\s+)*(?:class|struct|interface|enum|record)\s+(\w+)",
    )
    .unwrap()
});

/// C# public delegate declarations: `delegate <return-type> <name>(...)`. Unlike
/// class/struct/interface/enum/record, a delegate declaration has a mandatory return
/// type between the `delegate` keyword and the delegate's own name (e.g. `public
/// delegate int IncrementDelegate();`), so it can't share CS_TYPE's `keyword name`
/// pattern without wrongly capturing the return type (e.g. `int`) as the symbol name.
static CS_DELEGATE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*public\s+(?:static\s+)?(?:partial\s+)?(?:sealed\s+)?(?:abstract\s+)?delegate\s+\S+\s+(\w+)\s*[(<]",
    )
    .unwrap()
});

/// C# public methods and properties. The modifier group is a repeated alternation (not a
/// fixed sequence) since C# allows these in any order and any combination — a hardcoded
/// order (e.g. requiring `static` before `virtual`) would reject equally valid real-world
/// orderings like `public sealed override void Foo()`. Covers `required` (C# 11 required
/// auto-properties, e.g. `public required string Id { get; init; }`), `event` (field-like
/// event declarations, e.g. `public event EventHandler<EventArgs> Updated;`), and
/// `readonly` (e.g. `public static readonly Config Empty = ...;`, a common
/// constant/singleton idiom) — without any of these the member name is dropped. The
/// trailing character class covers `(` (methods), `{` (block-bodied properties), `;`
/// (fields/abstract members with no body), and a leading `=` (covering both a plain
/// field initializer, e.g. `= new Config();`, and an expression-bodied member's `=>`,
/// since matching just the arrow's first character is enough to recognize either shape)
/// — a field initializer is exactly the "readonly ... = ..." idiom this pattern exists
/// to capture, so a terminator that only accepted `=>` (and not a bare `=`) would still
/// silently drop it even with `readonly` present in the modifier list.
static CS_MEMBER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*public\s+(?:(?:static|virtual|override|sealed|abstract|async|new|required|readonly|event)\s+)*(?:\w+(?:<[^>]*>)?(?:\[\])?(?:\?)?)\s+(\w+)\s*[({;=]",
    )
    .unwrap()
});

/// Matches the start of a public interface declaration, up to (but not including)
/// any generic type parameters, base-interface list, `where` constraint clauses, or
/// the opening brace.
static CS_INTERFACE_START: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[^\S\n]*public\s+(?:partial\s+)?interface\s+\w+").unwrap());

/// C# interface member declaration (method, indexer, property, or field-like event).
/// Unlike class/struct members, interface members are implicitly public: C# forbids
/// writing an explicit `public` on ordinary interface members (a compile error), so
/// CS_MEMBER's hard requirement for a literal `public` keyword finds zero members in
/// every interface. This is scanned separately against each interface's body text,
/// with no `public` required — but an explicit `private`/`internal` (allowed on C#
/// 8+ default interface members) still excludes the member.
static CS_INTERFACE_MEMBER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*(private|internal)?\s*(?:(?:public|static|abstract|virtual|sealed|async|new|readonly|event)\s+)*(?:\w+(?:<[^>]*>)?(?:\[\])?(?:\?)?)\s+(\w+)\s*(?:[({;]|=>)",
    )
    .unwrap()
});

/// Matches ANY class/struct/interface/enum/record header, regardless of its own access
/// modifier (or lack of one), so the caller can track type-nesting visibility. Unlike
/// `CS_TYPE` (which requires a literal `public` and is used to decide what's exported),
/// this exists purely to notice when a body belongs to a non-public (default/
/// `internal`/`private`/`protected`) type, since C# has no concept of a member being
/// "more public" than its own containing type: a `public` member or nested type
/// declared inside an `internal`/`private` type is not actually reachable from outside
/// the assembly/type, no matter what its own modifier says.
static CS_TYPE_HEADER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*(?:(?P<vis>public|internal|private|protected)\s+)?(?:(?:static|partial|sealed|abstract)\s+)*(?:class|struct|interface|enum|record)\s+\w+",
    )
    .unwrap()
});

/// Returns the byte index of the `}` that matches the `{` at `open_pos`, counting
/// nested brace depth. `open_pos` must point at a `{` byte.
fn find_matching_brace(bytes: &[u8], open_pos: usize) -> Option<usize> {
    let mut depth = 0i32;
    for (i, &b) in bytes.iter().enumerate().skip(open_pos) {
        match b {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// For every line, whether declarations directly inside the type-nesting chain active
/// at that line sit within an entirely-public chain of enclosing types (or are at top
/// level, which is trivially public). A type only stays "in chain" via its own header
/// line's access modifier: anything other than a literal `public` (the default,
/// `internal`, `private`, or `protected`) breaks the chain for everything nested
/// inside, regardless of that nested content's own modifiers — a `public` method or
/// nested type declared inside an `internal`/`private` class is not actually reachable
/// from outside, no matter what its own line says.
///
/// Returns (per-line effective-public flags, byte offset each line starts at) so a
/// regex match's start offset can be mapped back to "was this line's nesting chain
/// fully public".
fn compute_line_effective_public(stripped: &str) -> (Vec<bool>, Vec<usize>) {
    let mut line_effective = Vec::new();
    let mut line_starts = Vec::new();
    let mut stack: Vec<bool> = Vec::new();
    let mut pending: Option<bool> = None;
    let mut offset = 0usize;

    for line in stripped.split_inclusive('\n') {
        let line_no_nl = line.strip_suffix('\n').unwrap_or(line);
        line_starts.push(offset);
        offset += line.len();

        let effective_now = stack.last().copied().unwrap_or(true);
        line_effective.push(effective_now);

        if let Some(caps) = CS_TYPE_HEADER.captures(line_no_nl) {
            let is_public_here = caps.name("vis").map(|m| m.as_str()) == Some("public");
            pending = Some(is_public_here && effective_now);
        }

        for ch in line_no_nl.chars() {
            match ch {
                '{' => stack.push(pending.take().unwrap_or(effective_now)),
                '}' => {
                    stack.pop();
                }
                ';' => pending = None,
                _ => {}
            }
        }
    }

    (line_effective, line_starts)
}

/// Looks up the effective-public flag for the line containing byte `offset`.
fn effective_public_at(offset: usize, line_effective: &[bool], line_starts: &[usize]) -> bool {
    let line_idx = match line_starts.binary_search(&offset) {
        Ok(i) => i,
        Err(i) => i.saturating_sub(1),
    };
    line_effective.get(line_idx).copied().unwrap_or(true)
}

/// Extract public symbols from C# source code.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = COMMENT_SINGLE.replace_all(content, "");
    let stripped = COMMENT_MULTI.replace_all(&stripped, "");

    let (line_effective, line_starts) = compute_line_effective_public(&stripped);
    let is_effective_public =
        |start: usize| effective_public_at(start, &line_effective, &line_starts);

    let mut symbols = Vec::new();

    for caps in CS_TYPE.captures_iter(&stripped) {
        if let Some(name) = caps.get(1)
            && is_effective_public(caps.get(0).unwrap().start())
        {
            symbols.push(name.as_str().to_string());
        }
    }

    for caps in CS_DELEGATE.captures_iter(&stripped) {
        if let Some(name) = caps.get(1)
            && is_effective_public(caps.get(0).unwrap().start())
        {
            let n = name.as_str().to_string();
            if !symbols.contains(&n) {
                symbols.push(n);
            }
        }
    }

    for caps in CS_MEMBER.captures_iter(&stripped) {
        if let Some(name) = caps.get(1)
            && is_effective_public(caps.get(0).unwrap().start())
        {
            let n = name.as_str().to_string();
            if !symbols.contains(&n) {
                symbols.push(n);
            }
        }
    }

    // Interface members are implicitly public and never carry their own `public`
    // keyword, so scan each interface's body separately from CS_MEMBER. Only walk
    // interfaces whose own header line is itself reachable from a fully-public chain
    // -- a `public` interface nested inside an `internal`/`private` type is no more
    // reachable than a `public` method would be in the same position.
    let bytes = stripped.as_bytes();
    for m in CS_INTERFACE_START.find_iter(&stripped) {
        if !is_effective_public(m.start()) {
            continue;
        }
        let after = m.end();
        if let Some(rel_open) = stripped[after..].find('{') {
            let open_pos = after + rel_open;
            if let Some(close_pos) = find_matching_brace(bytes, open_pos) {
                let body = &stripped[open_pos + 1..close_pos];
                for caps in CS_INTERFACE_MEMBER.captures_iter(body) {
                    if caps.get(1).is_some() {
                        continue; // explicit `private`/`internal` — not exported
                    }
                    if let Some(name) = caps.get(2) {
                        let n = name.as_str().to_string();
                        if !symbols.contains(&n) {
                            symbols.push(n);
                        }
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
    fn test_csharp_exports() {
        let src = r#"
namespace Example.Auth;

public class AuthService {
    public string Validate(string token) {}
    private void InternalCheck() {}
    public static AuthService Instance { get; }
    public int Timeout;
}

public interface IAuthenticator {}
public enum AuthStatus { Active, Expired }
public record UserProfile(string Name, int Age);
public struct Config {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"AuthService".to_string()));
        assert!(symbols.contains(&"Validate".to_string()));
        assert!(symbols.contains(&"IAuthenticator".to_string()));
        assert!(symbols.contains(&"AuthStatus".to_string()));
        assert!(symbols.contains(&"UserProfile".to_string()));
        assert!(symbols.contains(&"Config".to_string()));
        assert!(!symbols.contains(&"InternalCheck".to_string()));
    }

    #[test]
    fn test_csharp_async() {
        let src = r#"
public class Service {
    public async Task<string> FetchData() {}
    public virtual void OnUpdate() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Service".to_string()));
        assert!(symbols.contains(&"FetchData".to_string()));
        assert!(symbols.contains(&"OnUpdate".to_string()));
    }

    #[test]
    fn test_csharp_comments_stripped() {
        let src = r#"
// public class FakeClass {}
/* public struct FakeStruct {} */
/// <summary>XML doc comment</summary>
/// public enum FakeEnum {}
public class RealClass {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"RealClass".to_string()));
        assert!(!symbols.contains(&"FakeClass".to_string()));
        assert!(!symbols.contains(&"FakeStruct".to_string()));
        assert!(!symbols.contains(&"FakeEnum".to_string()));
    }

    #[test]
    fn test_csharp_static_class() {
        let src = r#"
public static class Extensions {
    public static string ToUpper(string s) {}
    public static int Parse(string s) {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Extensions".to_string()));
        assert!(symbols.contains(&"ToUpper".to_string()));
        assert!(symbols.contains(&"Parse".to_string()));
    }

    #[test]
    fn test_csharp_private_internal_excluded() {
        let src = r#"
public class Api {
    private string _secret;
    internal void Setup() {}
    protected virtual void OnInit() {}
    public string Name;
    public void Process() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Api".to_string()));
        assert!(symbols.contains(&"Name".to_string()));
        assert!(symbols.contains(&"Process".to_string()));
        assert!(!symbols.contains(&"_secret".to_string()));
        assert!(!symbols.contains(&"Setup".to_string()));
        assert!(!symbols.contains(&"OnInit".to_string()));
    }

    #[test]
    fn test_csharp_sealed_partial() {
        let src = r#"
public sealed class Singleton {
    public static Singleton Instance;
}

public partial class UserService {
    public void Create() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Singleton".to_string()));
        assert!(symbols.contains(&"Instance".to_string()));
        assert!(symbols.contains(&"UserService".to_string()));
        assert!(symbols.contains(&"Create".to_string()));
    }

    #[test]
    fn test_csharp_reversed_modifier_order_and_readonly_field() {
        // Regression test: CS_TYPE/CS_MEMBER previously hardcoded a fixed modifier
        // order (`static, partial, sealed, abstract` / `static, virtual, override,
        // abstract, async, new, required, event`), so an equally valid but differently
        // ordered real-world declaration like `public sealed partial class` failed to
        // match at all. `readonly` was also missing entirely from CS_MEMBER, silently
        // dropping the extremely common `public static readonly` constant/singleton
        // idiom.
        let src = r#"
public sealed partial class Config {
    public static readonly Config Empty = new Config();
    public sealed override string ToString() => "Config";
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Config".to_string()));
        assert!(symbols.contains(&"Empty".to_string()));
        assert!(symbols.contains(&"ToString".to_string()));
    }

    #[test]
    fn test_csharp_nullable_and_array_types() {
        let src = r#"
public class DataStore {
    public string? FindById(int id) {}
    public int[] GetIds() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"DataStore".to_string()));
        assert!(symbols.contains(&"FindById".to_string()));
        assert!(symbols.contains(&"GetIds".to_string()));
    }

    #[test]
    fn test_csharp_interface_members_exported() {
        // Interfaces forbid explicit access modifiers on ordinary members (compile
        // error to write `public` inside an interface body), so CS_MEMBER's hard
        // requirement for a literal `public` keyword must not be the only path to
        // exporting members — interface bodies are scanned separately.
        let src = r#"
public interface IRepository<T> where T : class {
    T GetById(int id);
    IEnumerable<T> GetAll();
    void Add(T item);
    bool Remove(int id);
    int Count { get; }
}

public interface IAuditable {
    DateTime CreatedAt { get; }
    DateTime? ModifiedAt { get; set; }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"IRepository".to_string()));
        assert!(symbols.contains(&"IAuditable".to_string()));
        assert!(symbols.contains(&"GetById".to_string()));
        assert!(symbols.contains(&"GetAll".to_string()));
        assert!(symbols.contains(&"Add".to_string()));
        assert!(symbols.contains(&"Remove".to_string()));
        assert!(symbols.contains(&"Count".to_string()));
        assert!(symbols.contains(&"CreatedAt".to_string()));
        assert!(symbols.contains(&"ModifiedAt".to_string()));
    }

    #[test]
    fn test_csharp_interface_default_member_visibility_excluded() {
        // C# 8+ default interface members may carry an explicit access modifier;
        // `private`/`internal` ones must still be excluded even inside an otherwise
        // implicitly-public interface body.
        let src = r#"
public interface ILogger {
    void Log(string message);
    private void WriteRaw(string message) { }
    internal void Flush() { }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"ILogger".to_string()));
        assert!(symbols.contains(&"Log".to_string()));
        assert!(!symbols.contains(&"WriteRaw".to_string()));
        assert!(!symbols.contains(&"Flush".to_string()));
    }

    #[test]
    fn test_csharp_required_property_and_event_exported() {
        // Finding #2: C# 11 `required` auto-properties must not be dropped.
        // Finding #3: field-like `event` declarations (including generic delegate
        // types like `EventHandler<EventArgs>`) must not be dropped.
        let src = r#"
public abstract class Entity {
    public required string Id { get; init; }
    public string? Description { get; set; }
    public event EventHandler<EventArgs> Updated;
    public event Action Saved;
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Entity".to_string()));
        assert!(symbols.contains(&"Id".to_string()));
        assert!(symbols.contains(&"Description".to_string()));
        assert!(symbols.contains(&"Updated".to_string()));
        assert!(symbols.contains(&"Saved".to_string()));
    }

    #[test]
    fn test_csharp_abstract_members() {
        let src = r#"
public abstract class Repository {
    public abstract void Save();
    public abstract string Name { get; }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Repository".to_string()));
        assert!(symbols.contains(&"Save".to_string()));
    }

    #[test]
    fn test_csharp_real_world_delegate_and_event() {
        // Real excerpt from learnxinyminutes.com's C# tutorial (DelegateTest class).
        // Regression test for a bug found via this exact source: `public delegate int
        // IncrementDelegate();` was captured as symbol "int" (the return type) instead
        // of "IncrementDelegate" (the delegate's actual name), because delegates —
        // unlike class/struct/interface/enum/record — carry a mandatory return type
        // between the `delegate` keyword and their own name.
        let src = r#"
    public class DelegateTest
    {
        public static int count = 0;
        public static int Increment()
        {
            // increment count then return it
            return ++count;
        }

        // A delegate is a reference to a method.
        // To reference the Increment method,
        // first declare a delegate with the same signature,
        // i.e. takes no arguments and returns an int
        public delegate int IncrementDelegate();

        // An event can also be used to trigger delegates
        // Create an event with the delegate type
        public static event IncrementDelegate MyEvent;

        static void Main(string[] args)
        {
            IncrementDelegate inc = new IncrementDelegate(Increment);
            Console.WriteLine(inc());  // => 1
        }
    }
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"DelegateTest".to_string()));
        assert!(symbols.contains(&"Increment".to_string()));
        assert!(symbols.contains(&"IncrementDelegate".to_string()));
        assert!(symbols.contains(&"MyEvent".to_string()));
        // The return type must never be mistaken for the delegate's name.
        assert!(!symbols.contains(&"int".to_string()));
        // `Main` here has no `public` modifier (defaults to private) and must be excluded.
        assert!(!symbols.contains(&"Main".to_string()));
    }

    #[test]
    fn test_csharp_real_world_bicycle_properties_and_enum() {
        // Real excerpt from learnxinyminutes.com's C# tutorial (Bicycle class): a
        // full-bodied property, a protected auto-property, an internal auto-property
        // with a private setter, a plain (non-public) field, a public field-backed
        // auto-property, an expression-bodied readonly property, a nested public enum,
        // and a `[Flags]`-decorated enum. Exercises real idiomatic C# visibility syntax
        // beyond what the hand-written tests cover.
        let src = r#"
    public class Bicycle
    {
        public int Cadence
        {
            get
            {
                return _cadence;
            }
            set
            {
                _cadence = value;
            }
        }
        private int _cadence;

        protected virtual int Gear
        {
            get;
            set;
        }

        internal int Wheels
        {
            get;
            private set;
        }

        int _speed;
        public string Name { get; set; }

        public string LongName => Name + " " + _speed + " speed";

        public enum BikeBrand
        {
            AIST,
            BMC,
            Electra = 42,
            Gitane
        }

        public BikeBrand Brand;

        [Flags]
        public enum BikeAccessories
        {
            None = 0,
            Bell = 1,
            MudGuards = 2,
            Racks = 4,
            Lights = 8,
            FullPackage = Bell | MudGuards | Racks | Lights
        }

        public BikeAccessories Accessories { get; set; }
    }
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Bicycle".to_string()));
        assert!(symbols.contains(&"Cadence".to_string()));
        assert!(symbols.contains(&"Name".to_string()));
        assert!(symbols.contains(&"LongName".to_string()));
        assert!(symbols.contains(&"BikeBrand".to_string()));
        assert!(symbols.contains(&"Brand".to_string()));
        assert!(symbols.contains(&"BikeAccessories".to_string()));
        assert!(symbols.contains(&"Accessories".to_string()));
        // `Gear` and `Wheels` are protected/internal, not public, and must be excluded.
        assert!(!symbols.contains(&"Gear".to_string()));
        assert!(!symbols.contains(&"Wheels".to_string()));
        // `_cadence` and `_speed` are private/default-visibility fields.
        assert!(!symbols.contains(&"_cadence".to_string()));
        assert!(!symbols.contains(&"_speed".to_string()));
    }

    #[test]
    fn test_csharp_members_of_internal_class_not_exported() {
        // Bug found via adversarial probing: C# had no concept of type-nesting
        // visibility at all -- CS_TYPE/CS_MEMBER only checked whether the literal
        // word `public` appeared on the *member's own* line, never whether the
        // enclosing type itself was reachable. A `public` method or nested `public`
        // enum declared inside a non-public (default/`internal`/`private`/
        // `protected`) class is not actually part of the assembly's public surface --
        // outside code can't even name the containing type to reach them -- yet both
        // leaked out as exported.
        let src = r#"
internal class Hidden {
    public void Leak() {}
    public enum Status { A, B }
}
"#;
        let symbols = extract_exports(src);
        assert!(!symbols.contains(&"Hidden".to_string()));
        assert!(
            !symbols.contains(&"Leak".to_string()),
            "method of an internal class must not be exported: {symbols:?}"
        );
        assert!(
            !symbols.contains(&"Status".to_string()),
            "nested public enum inside an internal class must not be exported: {symbols:?}"
        );
    }

    #[test]
    fn test_csharp_nested_private_class_members_not_exported() {
        // Same bug class as above, one level deeper: a `private` nested class inside
        // an otherwise fully-public outer class also breaks the chain for its own
        // members, even though every individual line involved says `public`.
        let src = r#"
public class Outer {
    private class Inner {
        public void AlsoLeak() {}
    }
    public void VisibleMethod() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Outer".to_string()));
        assert!(symbols.contains(&"VisibleMethod".to_string()));
        assert!(
            !symbols.contains(&"AlsoLeak".to_string()),
            "method of a private nested class must not be exported: {symbols:?}"
        );
    }

    #[test]
    fn test_csharp_generic_class_with_where_clause_and_attribute() {
        // Documents correct (already-working) behavior: a `[Serializable]`-style
        // attribute line immediately preceding a class header doesn't interfere with
        // CS_TYPE_HEADER's nesting tracking, and a generic class constrained with a
        // `where` clause is captured the same as a non-generic one -- exercising
        // real-world syntax breadth for the type-nesting fix above, not previously
        // covered for *classes* (only for interfaces).
        let src = r#"
[Serializable]
public class Repo<TEntity> where TEntity : class {
    public TEntity Find(int id) => default;
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Repo".to_string()));
        assert!(symbols.contains(&"Find".to_string()));
    }
}
