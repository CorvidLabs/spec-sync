use regex::Regex;
use std::sync::LazyLock;

static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"//.*$").unwrap());

static COMMENT_MULTI: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/").unwrap());

static COMMENT_HASH: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)#[^\[].*$").unwrap());

/// PHP public type declarations: class, interface, trait, enum
///
/// `(?i)` makes the keyword literals case-insensitive to match PHP's own
/// case-insensitive keyword rules (e.g. `CLASS Foo {}` and `Class Foo {}`
/// are both valid PHP); the captured name in group 1 still preserves its
/// original source casing since only literal keyword matching is affected.
static PHP_TYPE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?im)^[^\S\n]*(?:abstract\s+|final\s+)?(?:readonly\s+)?(?:class|interface|trait|enum)\s+(\w+)",
    )
    .unwrap()
});

/// Modifier keywords that may precede `function`/`const`, in any order. PSR-12
/// mandates `abstract`/`final` before the visibility keyword (e.g. `abstract public
/// function foo()`, `final public const FOO = 1`), but member declarations may carry
/// any subset/ordering of these, so the whole run is captured and inspected rather
/// than pattern-matched position-by-position.
const PHP_MODIFIER: &str = r"(?:abstract|final|public|private|protected|static|readonly)";

/// PHP function declarations (top-level or class/interface/trait member). Captures
/// the full leading modifier run (group 1) so callers can inspect it for
/// private/protected regardless of where public/static/abstract/final sit relative
/// to it, and the function name (group 2).
///
/// `(?i)` makes the modifier and `function` keyword literals case-insensitive
/// (PHP keywords are case-insensitive, so `PUBLIC FUNCTION bar()` is valid PHP);
/// the captured modifier run and function name preserve their original casing.
static PHP_FUNCTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"(?im)^[^\S\n]*((?:{PHP_MODIFIER}\s+)*)function\s+(\w+)"
    ))
    .unwrap()
});

/// PHP const declarations at class or top level. Captures the full leading modifier
/// run (group 1) and the constant name (group 2); see `PHP_FUNCTION` for why the
/// modifier run is captured as a whole rather than matched in a fixed order, and
/// for why the keyword literals are matched case-insensitively via `(?i)`.
static PHP_CONST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"(?im)^[^\S\n]*((?:{PHP_MODIFIER}\s+)*)const\s+(\w+)"
    ))
    .unwrap()
});

/// True if a captured modifier run contains `private` or `protected`. PHP
/// modifier keywords are case-insensitive (e.g. `PRIVATE function hidden()`),
/// so each token is compared ASCII-case-insensitively rather than by exact
/// equality.
fn has_non_public_modifier(modifiers: &str) -> bool {
    modifiers
        .split_whitespace()
        .any(|m| m.eq_ignore_ascii_case("private") || m.eq_ignore_ascii_case("protected"))
}

/// Extract public symbols from PHP source code.
/// Classes, interfaces, traits, enums are always included.
/// Functions and constants are included unless marked private/protected.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = COMMENT_SINGLE.replace_all(content, "");
    let stripped = COMMENT_MULTI.replace_all(&stripped, "");
    let stripped = COMMENT_HASH.replace_all(&stripped, "");

    let mut symbols = Vec::new();

    // Types are always public in PHP (visibility is per-member, not per-type)
    for caps in PHP_TYPE.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            symbols.push(name.as_str().to_string());
        }
    }

    // Functions: include unless the modifier run contains private/protected
    for caps in PHP_FUNCTION.captures_iter(&stripped) {
        let modifiers = caps.get(1).map_or("", |m| m.as_str());
        if let Some(name) = caps.get(2) {
            let n = name.as_str();
            // Skip constructor and magic methods
            if n.starts_with("__") {
                continue;
            }
            if has_non_public_modifier(modifiers) {
                continue;
            }
            if !symbols.contains(&n.to_string()) {
                symbols.push(n.to_string());
            }
        }
    }

    // Constants
    for caps in PHP_CONST.captures_iter(&stripped) {
        let modifiers = caps.get(1).map_or("", |m| m.as_str());
        if let Some(name) = caps.get(2) {
            let n = name.as_str().to_string();
            if has_non_public_modifier(modifiers) {
                continue;
            }
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

    /// Real excerpt from learnxinyminutes.com/php.md (the `MyClass` walkthrough),
    /// exercising mixed static/instance property visibility, a constructor, an
    /// implicitly-public `final function` (no visibility keyword), magic methods,
    /// and a `public static function`, all interleaved with block comments.
    #[test]
    fn test_php_learnxinyminutes_class_members() {
        let src = r#"<?php
class MyClass
{
    const MY_CONST      = 'value'; // A constant

    static $staticVar   = 'static';

    // Static variables and their visibility
    public static $publicStaticVar = 'publicStatic';
    // Accessible within the class only
    private static $privateStaticVar = 'privateStatic';
    // Accessible from the class and subclasses
    protected static $protectedStaticVar = 'protectedStatic';

    // Properties must declare their visibility
    public $property    = 'public';
    public $instanceProp;
    protected $prot = 'protected'; // Accessible from the class and subclasses
    private $priv   = 'private';   // Accessible within the class only

    // Create a constructor with __construct
    public function __construct($instanceProp)
    {
        // Access instance variables with $this
        $this->instanceProp = $instanceProp;
    }

    // Methods are declared as functions inside a class
    public function myMethod()
    {
        print 'MyClass';
    }

    // final keyword would make a function unoverridable
    final function youCannotOverrideMe()
    {
    }

    // Magic Methods

    // what to do if Object is treated as a String
    public function __toString()
    {
        return $property;
    }

    // opposite to __construct()
    // called when object is no longer referenced
    public function __destruct()
    {
        print "Destroying";
    }

/*
 * Declaring class properties or methods as static makes them accessible without
 * needing an instantiation of the class. A property declared as static can not
 * be accessed with an instantiated class object (though a static method can).
 */

    public static function myStaticMethod()
    {
        print 'I am static';
    }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"MyClass".to_string()));
        assert!(symbols.contains(&"MY_CONST".to_string()));
        assert!(symbols.contains(&"myMethod".to_string()));
        assert!(symbols.contains(&"youCannotOverrideMe".to_string()));
        assert!(symbols.contains(&"myStaticMethod".to_string()));
        // Magic methods are excluded even when explicitly public.
        assert!(!symbols.contains(&"__construct".to_string()));
        assert!(!symbols.contains(&"__toString".to_string()));
        assert!(!symbols.contains(&"__destruct".to_string()));
        // Bare properties (no `const`/`function` keyword) must never be
        // mistaken for declarations, regardless of visibility.
        assert!(!symbols.contains(&"staticVar".to_string()));
        assert!(!symbols.contains(&"publicStaticVar".to_string()));
        assert!(!symbols.contains(&"property".to_string()));
    }

    /// Real excerpt from learnxinyminutes.com/php.md covering interfaces (including
    /// `extends`), an abstract class, a concrete class implementing multiple
    /// interfaces, and a trait consumed via `use` inside a class body.
    #[test]
    fn test_php_learnxinyminutes_interfaces_and_traits() {
        let src = r#"<?php
interface InterfaceOne
{
    public function doSomething();
}

interface InterfaceTwo
{
    public function doSomethingElse();
}

// interfaces can be extended
interface InterfaceThree extends InterfaceTwo
{
    public function doAnotherContract();
}

abstract class MyAbstractClass implements InterfaceOne
{
    public $x = 'doSomething';
}

class MyConcreteClass extends MyAbstractClass implements InterfaceTwo
{
    public function doSomething()
    {
        echo $x;
    }

    public function doSomethingElse()
    {
        echo 'doSomethingElse';
    }
}

// Classes can implement more than one interface
class SomeOtherClass implements InterfaceOne, InterfaceTwo
{
    public function doSomething()
    {
        echo 'doSomething';
    }

    public function doSomethingElse()
    {
        echo 'doSomethingElse';
    }
}

trait MyTrait
{
    public function myTraitMethod()
    {
        print 'I have MyTrait';
    }
}

class MyTraitfulClass
{
    use MyTrait;
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"InterfaceOne".to_string()));
        assert!(symbols.contains(&"InterfaceTwo".to_string()));
        assert!(symbols.contains(&"InterfaceThree".to_string()));
        assert!(symbols.contains(&"doAnotherContract".to_string()));
        assert!(symbols.contains(&"MyAbstractClass".to_string()));
        assert!(symbols.contains(&"MyConcreteClass".to_string()));
        assert!(symbols.contains(&"SomeOtherClass".to_string()));
        assert!(symbols.contains(&"doSomething".to_string()));
        assert!(symbols.contains(&"doSomethingElse".to_string()));
        assert!(symbols.contains(&"MyTrait".to_string()));
        assert!(symbols.contains(&"myTraitMethod".to_string()));
        assert!(symbols.contains(&"MyTraitfulClass".to_string()));
    }

    #[test]
    fn test_php_class_and_methods() {
        let src = r#"<?php

namespace App\Auth;

class AuthService {
    public const DEFAULT_TTL = 3600;
    private const INTERNAL_KEY = "secret";

    public function validate(string $token): bool {}
    private function internalCheck(): void {}
    public static function create(): self {}
    protected function helper(): void {}
}

interface Authenticator {
    public function authenticate(): bool;
}

abstract class BaseController {}

enum Status {
    case Active;
    case Expired;
}

trait Loggable {
    public function log(): void {}
}

function standalone_helper(): void {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"AuthService".to_string()));
        assert!(symbols.contains(&"Authenticator".to_string()));
        assert!(symbols.contains(&"BaseController".to_string()));
        assert!(symbols.contains(&"Status".to_string()));
        assert!(symbols.contains(&"Loggable".to_string()));
        assert!(symbols.contains(&"validate".to_string()));
        assert!(symbols.contains(&"create".to_string()));
        assert!(symbols.contains(&"DEFAULT_TTL".to_string()));
        assert!(symbols.contains(&"standalone_helper".to_string()));
        assert!(!symbols.contains(&"internalCheck".to_string()));
        assert!(!symbols.contains(&"helper".to_string()));
        assert!(!symbols.contains(&"INTERNAL_KEY".to_string()));
    }

    #[test]
    fn test_php_final_readonly() {
        let src = r#"<?php
final class Config {}
readonly class ValueObject {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Config".to_string()));
        assert!(symbols.contains(&"ValueObject".to_string()));
    }

    #[test]
    fn test_php_skips_magic_methods() {
        let src = r#"<?php
class Foo {
    public function __construct() {}
    public function __toString(): string {}
    public function getName(): string {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Foo".to_string()));
        assert!(symbols.contains(&"getName".to_string()));
        assert!(!symbols.contains(&"__construct".to_string()));
        assert!(!symbols.contains(&"__toString".to_string()));
    }

    #[test]
    fn test_php_abstract_final_modifier_order() {
        let src = r#"<?php
abstract class BaseRepository {
    abstract public function find(int $id): ?Model;
    abstract public function save(Model $model): bool;
    final public function getConnectionCount(): int { return 0; }
    public static function make(): static { return new static(); }
}
final class OrderService {
    final public const MAX_ITEMS = 100;
    final public function checkout(Cart $cart): Order { return new Order($cart); }
}
trait Cacheable {
    abstract public function getCacheKey(): string;
    final public static function ttl(): int { return 3600; }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"BaseRepository".to_string()));
        assert!(symbols.contains(&"find".to_string()));
        assert!(symbols.contains(&"save".to_string()));
        assert!(symbols.contains(&"getConnectionCount".to_string()));
        assert!(symbols.contains(&"make".to_string()));
        assert!(symbols.contains(&"OrderService".to_string()));
        assert!(symbols.contains(&"MAX_ITEMS".to_string()));
        assert!(symbols.contains(&"checkout".to_string()));
        assert!(symbols.contains(&"Cacheable".to_string()));
        assert!(symbols.contains(&"getCacheKey".to_string()));
        assert!(symbols.contains(&"ttl".to_string()));
    }

    #[test]
    fn test_php_abstract_final_still_excludes_private_protected() {
        let src = r#"<?php
class Repository {
    final private function hydrate(array $row): Model { return new Model($row); }
    abstract protected function tableName(): string;
    final protected const CACHE_TTL = 60;
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Repository".to_string()));
        assert!(!symbols.contains(&"hydrate".to_string()));
        assert!(!symbols.contains(&"tableName".to_string()));
        assert!(!symbols.contains(&"CACHE_TTL".to_string()));
    }

    #[test]
    fn test_php_enum_methods_and_readonly_promoted_properties() {
        let src = r#"<?php
enum Suit: string {
    case Hearts = 'hearts';
    case Spades = 'spades';

    public function label(): string {
        return match ($this) {
            self::Hearts => 'Hearts',
            self::Spades => 'Spades',
        };
    }

    private function rawValue(): string {
        return $this->value;
    }
}

final class Point {
    public function __construct(
        public readonly float $x,
        public readonly float $y,
        private readonly string $label = '',
    ) {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Suit".to_string()));
        assert!(symbols.contains(&"label".to_string()));
        assert!(!symbols.contains(&"rawValue".to_string()));
        assert!(symbols.contains(&"Point".to_string()));
        // Promoted constructor properties are not function/const declarations and
        // must not be spuriously picked up as top-level symbols.
        assert!(!symbols.contains(&"x".to_string()));
        assert!(!symbols.contains(&"y".to_string()));
    }

    /// Regression test for a bug where PHP_TYPE/PHP_FUNCTION/PHP_CONST only
    /// matched lowercase keyword literals even though PHP keywords are
    /// case-insensitive. Before the `(?i)` fix, `CLASS Foo { PUBLIC FUNCTION
    /// bar() {} PRIVATE FUNCTION hidden() {} }` produced an empty symbol
    /// list: neither "CLASS" nor "FUNCTION" matched the all-lowercase
    /// regexes, so every declaration silently vanished (not just failed to
    /// be excluded, but was dropped from the output entirely). This test
    /// would have failed outright (empty/missing results) on the buggy
    /// code and catches any future regression that reintroduces
    /// case-sensitive keyword matching.
    #[test]
    fn test_php_case_insensitive_keywords() {
        let src = r#"<?php
CLASS Foo {
    PUBLIC FUNCTION bar() {}
    PRIVATE FUNCTION hidden() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Foo".to_string()));
        assert!(symbols.contains(&"bar".to_string()));
        assert!(!symbols.contains(&"hidden".to_string()));
    }

    /// Companion to `test_php_case_insensitive_keywords`, exercising mixed
    /// (not just all-caps) casing on `const` and modifier keywords, plus a
    /// case-varied `readonly`/`final` type declaration. Also guards the
    /// `has_non_public_modifier` string comparison: before the fix it used
    /// exact `==` against lowercase literals, so `Protected` or `PRIVATE`
    /// modifiers would fail to be recognized as non-public and the member
    /// would have leaked into the export list instead of being excluded
    /// (the opposite failure mode from the dropped-entirely case above).
    #[test]
    fn test_php_case_insensitive_const_and_modifiers() {
        let src = r#"<?php
Final Readonly Class Money {
    Public Const CURRENCY = 'USD';
    Protected Const INTERNAL_RATE = 1;
    PRIVATE CONST SECRET = 'x';
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Money".to_string()));
        assert!(symbols.contains(&"CURRENCY".to_string()));
        assert!(!symbols.contains(&"INTERNAL_RATE".to_string()));
        assert!(!symbols.contains(&"SECRET".to_string()));
    }

    /// Anonymous classes (`new class extends Foo implements Bar {}`) must
    /// never leak `extends`/`implements` targets, or the `class`/`extends`
    /// keywords themselves, as bogus type names. The `class` keyword here
    /// is preceded by `new` on the same line, so it never lands at the
    /// line-start position PHP_TYPE requires and is correctly skipped
    /// entirely; only the real named function inside the anonymous class
    /// body should surface.
    #[test]
    fn test_php_anonymous_class_no_bogus_type_name() {
        let src = r#"<?php
function makeLogger(): LoggerInterface {
    return new class extends BaseLogger implements LoggerInterface {
        public function log(string $msg): void {
            echo $msg;
        }
    };
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"makeLogger".to_string()));
        assert!(symbols.contains(&"log".to_string()));
        assert!(!symbols.contains(&"extends".to_string()));
        assert!(!symbols.contains(&"implements".to_string()));
        assert!(!symbols.contains(&"BaseLogger".to_string()));
        assert!(!symbols.contains(&"LoggerInterface".to_string()));
    }

    /// Inline closures (`function ($x) {...}`) and arrow functions
    /// (`fn($x) => ...`) assigned to a variable must not be treated as
    /// named public declarations: PHP_FUNCTION requires `function` (or,
    /// deliberately, never matches `fn`) to be followed by whitespace and
    /// then an identifier, and an anonymous `function (` has no identifier
    /// there, so it cannot match. The assigned variable names must not
    /// leak in either.
    #[test]
    fn test_php_closures_and_arrow_functions_not_captured() {
        let src = r#"<?php
class Calculator {
    public function build(): callable {
        $add = function ($x, $y) {
            return $x + $y;
        };
        $inc = fn($x) => $x + 1;
        return $add;
    }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Calculator".to_string()));
        assert!(symbols.contains(&"build".to_string()));
        assert!(!symbols.contains(&"add".to_string()));
        assert!(!symbols.contains(&"inc".to_string()));
        assert!(!symbols.contains(&"x".to_string()));
        assert!(!symbols.contains(&"y".to_string()));
        assert!(!symbols.contains(&"fn".to_string()));
    }

    /// PHP 8.2 allows combining `final` and `readonly` on a class
    /// declaration (`final readonly class Foo`). PHP_TYPE's optional
    /// `abstract|final` then optional `readonly` prefix groups must both
    /// apply together, not just individually, for the class name to be
    /// captured.
    #[test]
    fn test_php_final_readonly_class_combo() {
        let src = r#"<?php
final readonly class ImmutablePoint {
    public function __construct(
        public float $x,
        public float $y,
    ) {}

    public function distanceFromOrigin(): float {
        return sqrt($this->x ** 2 + $this->y ** 2);
    }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"ImmutablePoint".to_string()));
        assert!(symbols.contains(&"distanceFromOrigin".to_string()));
        assert!(!symbols.contains(&"__construct".to_string()));
    }

    /// `define('FOO', 1)` is a runtime function call, not a `const`
    /// declaration, and must intentionally not be captured as one:
    /// PHP_CONST only matches the literal `const` keyword, and `define`
    /// is a different identifier entirely, so no match should ever occur
    /// regardless of arguments or casing.
    #[test]
    fn test_php_define_call_not_captured_as_const() {
        let src = r#"<?php
define('FOO', 1);
DEFINE('BAR', 2);

class Config {
    public const REAL_CONST = 3;
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Config".to_string()));
        assert!(symbols.contains(&"REAL_CONST".to_string()));
        assert!(!symbols.contains(&"FOO".to_string()));
        assert!(!symbols.contains(&"BAR".to_string()));
        assert!(!symbols.contains(&"define".to_string()));
    }
}
