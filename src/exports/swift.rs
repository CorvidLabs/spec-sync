use regex::Regex;
use std::sync::LazyLock;

static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"//.*$").unwrap());

static COMMENT_MULTI: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/").unwrap());

/// Swift public/open declarations:
/// public/open func, class, struct, enum, protocol, typealias, var, let, actor.
/// The modifier group allows any run of the common member modifiers between the
/// access keyword and the declaration keyword (finding #4: `public final class` was
/// missed — as were the even more common `public override func` / `public mutating
/// func` / `public lazy var` / `public weak var`).
/// The `prefix`/`postfix` modifiers and `operator` keyword, plus the symbol-character
/// name alternative, let this also capture custom operator declarations like
/// `public prefix operator !!!` and `public prefix func !!! (...)` — real, idiomatic
/// Swift syntax whose "name" is punctuation rather than an identifier, so `\w+` alone
/// silently dropped them. `<`/`>` are deliberately excluded from the symbol class since
/// they're ambiguous with a directly-adjacent generic parameter list (e.g. `func <-><T:
/// Equatable>(...)`); operators built only from `<`/`>` are a rarer, accepted gap.
static SWIFT_DECL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)(?:public|open)\s+(?:(?:final|static|class|override|mutating|nonmutating|lazy|weak|unowned|dynamic|nonisolated|prefix|postfix)\s+)*(?:func|class|struct|enum|protocol|typealias|var|let|actor|init|operator)\s+(\w+|[-+*/=!&|^~%.?]+)",
    )
    .unwrap()
});

/// Swift init doesn't have a name — detect public init separately
static SWIFT_INIT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)(?:public|open)\s+(?:required\s+)?(?:convenience\s+)?init\s*\(").unwrap()
});

/// Swift subscript doesn't have a name either — same story as init.
static SWIFT_SUBSCRIPT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)(?:public|open)\s+(?:static\s+)?subscript\s*\(").unwrap());

/// Header of a `public`/`open` `protocol` or `extension` declaration, up to (but not
/// including) its opening brace. Protocol requirements structurally can't repeat the
/// access keyword (Swift forbids access modifiers on protocol requirements) and
/// extension members inherit the extension's own access level unless marked otherwise,
/// so neither is visible to SWIFT_DECL. Their bodies are scanned separately below.
static SWIFT_PROTOCOL_OR_EXTENSION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)(?:public|open)\s+(?:protocol|extension)\s+\w+").unwrap());

/// Header of a `public`/`open` `enum` declaration. Enum cases can never carry their own
/// access modifier (Swift forbids it), so case identifiers must be pulled from the body.
static SWIFT_ENUM: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)(?:public|open)\s+enum\s+\w+").unwrap());

/// A member `var`/`let` at the top of a container body. Allows the same run of member
/// modifiers as SWIFT_DECL, plus an optional `private(set)`/`fileprivate(set)`/
/// `internal(set)` — that only restricts the setter, the property itself is still
/// exported at the container's level.
static MEMBER_VAR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(?:(?:private|fileprivate|internal)\(set\)\s+)?(?:(?:static|class|final|override|mutating|nonmutating|lazy|weak|unowned|dynamic|nonisolated)\s+)*(?:var|let)\s+(\w+)",
    )
    .unwrap()
});

/// A member `func` at the top of a container body (no access keyword needed).
/// Also covers `prefix`/`postfix` operator overloads (e.g. `static prefix func - (...)`
/// inside a `public extension`) via the same symbol-character name alternative used by
/// `SWIFT_DECL` — see its doc comment for why `<`/`>` are excluded.
static MEMBER_FUNC: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(?:(?:static|class|final|override|mutating|nonmutating|dynamic|nonisolated|prefix|postfix)\s+)*func\s+(\w+|[-+*/=!&|^~%.?]+)",
    )
    .unwrap()
});

/// A member `subscript` at the top of a container body — like init, it has no name.
static MEMBER_SUBSCRIPT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?:static\s+)?subscript\s*\(").unwrap());

/// An `associatedtype` requirement at the top of a protocol body.
static MEMBER_ASSOCIATEDTYPE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^associatedtype\s+(\w+)").unwrap());

/// Extract exported (public/open) symbols from Swift source code.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = COMMENT_SINGLE.replace_all(content, "");
    let stripped = COMMENT_MULTI.replace_all(&stripped, "");

    let mut symbols = Vec::new();

    for caps in SWIFT_DECL.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            symbols.push(name.as_str().to_string());
        }
    }

    // Count public inits (they don't have a standalone name)
    if SWIFT_INIT.is_match(&stripped) {
        symbols.push("init".to_string());
    }

    // Public subscripts also don't have a standalone name.
    if SWIFT_SUBSCRIPT.is_match(&stripped) {
        symbols.push("subscript".to_string());
    }

    // Protocol requirements and extension members: neither repeats the container's own
    // access keyword, so pull them straight out of the body.
    for m in SWIFT_PROTOCOL_OR_EXTENSION.find_iter(&stripped) {
        if let Some(body) = find_body(&stripped, m.end()) {
            symbols.extend(scan_container_members(body, true, false));
        }
    }

    // Enum cases can never carry their own access modifier.
    for m in SWIFT_ENUM.find_iter(&stripped) {
        if let Some(body) = find_body(&stripped, m.end()) {
            symbols.extend(scan_container_members(body, false, true));
        }
    }

    symbols
}

/// Returns the text between the next `{` at/after `from` and its brace-depth-matched
/// closing `}`, so nested blocks (getter bodies, nested types) don't terminate the body
/// early.
fn find_body(text: &str, from: usize) -> Option<&str> {
    let bytes = text.as_bytes();
    let mut i = from;
    while i < bytes.len() && bytes[i] != b'{' {
        i += 1;
    }
    if i >= bytes.len() {
        return None;
    }
    let start = i + 1;
    let mut depth = 1;
    let mut j = start;
    while j < bytes.len() {
        match bytes[j] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&text[start..j]);
                }
            }
            _ => {}
        }
        j += 1;
    }
    None
}

/// Scans the top brace-depth level of a container body (protocol/extension or enum) for
/// members that inherit the container's own visibility instead of repeating it.
/// `allow_members` covers var/let/func/subscript/associatedtype (protocol & extension
/// bodies); `allow_case` covers enum `case` identifiers. Skips into nested blocks
/// (getter/function bodies, nested types) so local declarations and `switch case` labels
/// inside method bodies aren't mistaken for members, and skips members explicitly marked
/// private/fileprivate/internal (extension members can carry these; protocol
/// requirements never can).
fn scan_container_members(body: &str, allow_members: bool, allow_case: bool) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth: i32 = 0;

    for raw_line in body.lines() {
        let line = raw_line.trim();
        if depth == 0 && !line.is_empty() && !starts_with_excluded_modifier(line) {
            if allow_case && line.starts_with("case ") {
                out.extend(extract_case_names(&line["case ".len()..]));
            } else if allow_members {
                if let Some(caps) = MEMBER_ASSOCIATEDTYPE.captures(line) {
                    out.push(caps[1].to_string());
                } else if MEMBER_SUBSCRIPT.is_match(line) {
                    out.push("subscript".to_string());
                } else if let Some(caps) = MEMBER_FUNC.captures(line) {
                    out.push(caps[1].to_string());
                } else if let Some(caps) = MEMBER_VAR.captures(line) {
                    out.push(caps[1].to_string());
                }
            }
        }

        for ch in raw_line.chars() {
            match ch {
                '{' => depth += 1,
                '}' => depth = (depth - 1).max(0),
                _ => {}
            }
        }
    }

    out
}

/// True if `line` (already trimmed) opens with an explicit `private`/`fileprivate`/
/// `internal` modifier that narrows a member below the container's own access level.
/// `private(set)`/`fileprivate(set)`/`internal(set)` are deliberately NOT matched here —
/// those only restrict the setter, so the member itself is still exported.
fn starts_with_excluded_modifier(line: &str) -> bool {
    for kw in ["private", "fileprivate", "internal"] {
        if let Some(rest) = line.strip_prefix(kw)
            && rest.starts_with(char::is_whitespace)
        {
            return true;
        }
    }
    false
}

/// Splits an enum `case` payload on top-level commas (ignoring commas nested inside
/// associated-value parens/generics) and extracts each case's leading identifier,
/// discarding associated-value lists and raw-value assignments.
fn extract_case_names(rest: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth: i32 = 0;
    for ch in rest.chars() {
        match ch {
            '(' | '<' | '[' => {
                depth += 1;
                current.push(ch);
            }
            ')' | '>' | ']' => {
                depth -= 1;
                current.push(ch);
            }
            ',' if depth == 0 => parts.push(std::mem::take(&mut current)),
            _ => current.push(ch),
        }
    }
    if !current.trim().is_empty() {
        parts.push(current);
    }

    parts
        .into_iter()
        .filter_map(|part| {
            let name: String = part
                .trim()
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if name.is_empty() { None } else { Some(name) }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swift_exports() {
        let src = r#"
public class AuthService {
    public var token: String
    public let apiVersion: Int
    public func validate() -> Bool {}
    private func internalCheck() {}
    public static func shared() -> AuthService {}
}
public struct Config {}
public enum AuthStatus { case active, expired }
public protocol Authenticator {}
public typealias Token = String
open class BaseController {}
public actor SessionManager {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"AuthService".to_string()));
        assert!(symbols.contains(&"token".to_string()));
        assert!(symbols.contains(&"apiVersion".to_string()));
        assert!(symbols.contains(&"validate".to_string()));
        assert!(symbols.contains(&"shared".to_string()));
        assert!(symbols.contains(&"Config".to_string()));
        assert!(symbols.contains(&"AuthStatus".to_string()));
        assert!(symbols.contains(&"Authenticator".to_string()));
        assert!(symbols.contains(&"Token".to_string()));
        assert!(symbols.contains(&"BaseController".to_string()));
        assert!(symbols.contains(&"SessionManager".to_string()));
        assert!(!symbols.contains(&"internalCheck".to_string()));
    }

    #[test]
    fn test_swift_init() {
        let src = r#"
public class Foo {
    public init(name: String) {}
    public convenience init() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Foo".to_string()));
        assert!(symbols.contains(&"init".to_string()));
    }

    #[test]
    fn test_swift_open() {
        let src = r#"
open class BaseView {
    open func layoutSubviews() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"BaseView".to_string()));
        assert!(symbols.contains(&"layoutSubviews".to_string()));
    }

    #[test]
    fn test_swift_final_modifiers_exported() {
        // Finding #4: `final` (and combinations with static/class) between the
        // access keyword and the declaration keyword must not hide the export.
        let src = r#"
public final class Repository {}
open final class BaseService {}
public final func recompute() -> Int {}
public static let shared = Repository()
public class RegularClass {}
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"Repository".to_string()),
            "public final class"
        );
        assert!(
            symbols.contains(&"BaseService".to_string()),
            "open final class"
        );
        assert!(
            symbols.contains(&"recompute".to_string()),
            "public final func"
        );
        assert!(symbols.contains(&"shared".to_string()), "public static let");
        assert!(
            symbols.contains(&"RegularClass".to_string()),
            "plain public class still works"
        );
    }

    #[test]
    fn test_swift_common_member_modifiers_exported() {
        // The common member modifiers between the access keyword and the declaration
        // keyword must not hide the export (override/mutating are more common than
        // `final func`).
        let src = r#"
public override func recompute() -> Int {}
public mutating func append(_ x: Int) {}
public lazy var cache = Cache()
public weak var delegate: Foo?
open dynamic func draw() {}
public final override func layout() {}
public nonisolated func now() -> Date {}
"#;
        let symbols = extract_exports(src);
        for name in [
            "recompute",
            "append",
            "cache",
            "delegate",
            "draw",
            "layout",
            "now",
        ] {
            assert!(
                symbols.contains(&name.to_string()),
                "missing {name}: {symbols:?}"
            );
        }
    }

    #[test]
    fn test_swift_protocol_requirements_exported() {
        // Findings #1 and #3: protocol requirements (var/func/static var) and
        // associatedtype requirements can never carry their own `public`/`open`
        // keyword (Swift forbids access modifiers on protocol requirements), so they
        // must be treated as exported whenever the enclosing protocol itself is
        // public/open.
        let src = r#"
public protocol Repository {
    associatedtype Model: Identifiable
    associatedtype Query
    var count: Int { get }
    static var shared: Self { get }
    func fetch(_ query: Query) async throws -> [Model]
    func save(_ model: Model) async throws
    func delete(id: Model.ID) async throws
}
"#;
        let symbols = extract_exports(src);
        for name in [
            "Repository",
            "Model",
            "Query",
            "count",
            "shared",
            "fetch",
            "save",
            "delete",
        ] {
            assert!(
                symbols.contains(&name.to_string()),
                "missing {name}: {symbols:?}"
            );
        }
    }

    #[test]
    fn test_swift_extension_members_exported() {
        // Finding #2: `public extension Foo { ... }` members inherit the extension's
        // own access level and don't need to repeat `public` — they must still be
        // captured. A member explicitly marked `private` inside the extension must
        // still be excluded.
        let src = r#"
public extension String {
    var isBlank: Bool { trimmingCharacters(in: .whitespacesAndNewlines).isEmpty }
    func truncated(to length: Int) -> String { count > length ? String(prefix(length)) + "\u{2026}" : self }
    private func helper() -> Int { 0 }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"isBlank".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"truncated".to_string()), "{symbols:?}");
        assert!(
            !symbols.contains(&"helper".to_string()),
            "private extension member must stay excluded: {symbols:?}"
        );
    }

    #[test]
    fn test_swift_enum_cases_and_subscript_exported() {
        // Finding #4: enum cases can never carry an access modifier by language
        // design, so cases of a public/open enum must be treated as exported.
        // Finding #5: `public subscript(...)` has no standalone name (like `init`)
        // but is real public API and must be captured too.
        let src = r#"
public enum NetworkError: Error {
    case invalidResponse
    case unauthorized
    case timeout(seconds: Int)
}

public struct OrderedDictionary<Key: Hashable, Value> {
    private var storage: [Key: Value] = [:]
    public subscript(key: Key) -> Value? {
        get { storage[key] }
        set { storage[key] = newValue }
    }
}
"#;
        let symbols = extract_exports(src);
        for name in [
            "NetworkError",
            "invalidResponse",
            "unauthorized",
            "timeout",
            "OrderedDictionary",
            "subscript",
        ] {
            assert!(
                symbols.contains(&name.to_string()),
                "missing {name}: {symbols:?}"
            );
        }
        assert!(!symbols.contains(&"storage".to_string()));
    }

    #[test]
    fn test_swift_learnxinyminutes_raw_tutorial_no_panic() {
        // Verbatim excerpt from learnxinyminutes.com/swift — real, syntactically valid
        // source from an annotated REPL-style walkthrough of the language, not a
        // library, so it (accurately) declares nothing `public`/`open`. This is a pure
        // robustness check: unicode/emoji identifiers, raw strings (`#"..."#`), nested
        // multi-line comments, and multi-line triple-quoted strings must all be scanned
        // without panicking or hanging, and a snippet with zero public API must yield
        // zero symbols rather than false positives.
        let src = r####"
/* Nested multiline comments
 /* ARE */
 allowed
 */

let theAnswer = 42
var theQuestion = "What is the Answer?"

let aString: String = "A string"
let aDouble: Double = 0

let descriptionString = "The value of aDouble is \(aDouble)"
let equation = "Six by nine is \(6 * 9), not 42!"
let explanationString = #"The string I used was "The value of aDouble is \(aDouble)" and the result was \#(descriptionString)"#

let multiLineString = """
    This is a multi-line string.
    It's called that because it takes up multiple lines (wow!)
        Any indentation beyond the closing quotation marks is kept, the rest is discarded.
    You can include " or "" in multi-line strings because the delimiter is three "s.
    """

let shoppingList = ["catfish", "water", "tulips",] //commas are allowed after the last element
let secondElement = shoppingList[1] // Arrays are 0-indexed

var occupations = [
    "Malcolm": "Captain",
    "Kaylee": "Mechanic"
]
occupations["Jayne"] = "Public Relations"

// MARK: Other variables
let øπΩ = "value" // unicode variable names
let 🤯 = "wow" // emoji variable names

// Keywords can be used as variable names
let convenience = "keyword"
let weak = "another keyword"
let override = "another keyword"
let `class` = "keyword"
"####;
        let symbols = extract_exports(src);
        assert!(
            symbols.is_empty(),
            "tutorial excerpt declares nothing public/open: {symbols:?}"
        );
    }

    #[test]
    fn test_swift_learnxinyminutes_real_declarations() {
        // Adapted (added `public`/`open`) from learnxinyminutes.com/swift's Enums and
        // Structures & Classes sections — real idiomatic Swift: a plain enum with a
        // computed property, a struct with a custom subscript, and a class hierarchy
        // using computed get/set properties, `willSet`, and `override`.
        let src = r#"
public enum Suit {
    case spades, hearts, diamonds, clubs
    var icon: Character {
        switch self {
        case .spades:
            return "♤"
        case .hearts:
            return "♡"
        case .diamonds:
            return "♢"
        case .clubs:
            return "♧"
        }
    }
}

public struct NamesTable {
    public let names: [String]

    // Custom subscript
    public subscript(index: Int) -> String {
        return names[index]
    }
}

open class Shape {
    open func getArea() -> Int {
        return 0
    }
}

public class Rect: Shape {
    public var sideLength: Int = 1

    // Custom getter and setter property
    public var perimeter: Int {
        get {
            return 4 * sideLength
        }
        set {
            sideLength = newValue / 4
        }
    }

    public var identifier: String = "defaultID" {
        willSet(someIdentifier) {
            print(someIdentifier)
        }
    }

    public init(sideLength: Int) {
        self.sideLength = sideLength
        super.init()
    }

    public override func getArea() -> Int {
        return sideLength * sideLength
    }
}
"#;
        let symbols = extract_exports(src);
        for name in [
            "Suit",
            "NamesTable",
            "names",
            "subscript",
            "Shape",
            "getArea",
            "Rect",
            "sideLength",
            "perimeter",
            "identifier",
            "init",
        ] {
            assert!(
                symbols.contains(&name.to_string()),
                "missing {name}: {symbols:?}"
            );
        }
    }

    #[test]
    fn test_swift_learnxinyminutes_custom_operators() {
        // Real excerpt (adapted with `public`) from learnxinyminutes.com/swift's
        // Operators section — custom prefix operator declaration and definition.
        // Regression test: custom operator "names" are symbol characters, not `\w+`
        // identifiers, so a bug here silently dropped every public operator export.
        let src = r#"
// Custom operators can start with the characters:
//      / = - + * % < > ! & | ^ . ~
public prefix operator !!!

// A prefix operator that triples the side length when used
public prefix func !!! (shape: inout Square) -> Square {
    shape.sideLength *= 3
    return shape
}
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.iter().filter(|s| *s == "!!!").count() >= 2,
            "expected the operator declaration and its definition both captured: {symbols:?}"
        );
    }
}
