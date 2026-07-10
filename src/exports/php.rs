use regex::Regex;
use std::sync::LazyLock;

static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"//.*$").unwrap());

static COMMENT_MULTI: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/").unwrap());

static COMMENT_HASH: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)#[^\[].*$").unwrap());

/// PHP public type declarations: class, interface, trait, enum
static PHP_TYPE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*(?:abstract\s+|final\s+)?(?:readonly\s+)?(?:class|interface|trait|enum)\s+(\w+)",
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
static PHP_FUNCTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"(?m)^[^\S\n]*((?:{PHP_MODIFIER}\s+)*)function\s+(\w+)"
    ))
    .unwrap()
});

/// PHP const declarations at class or top level. Captures the full leading modifier
/// run (group 1) and the constant name (group 2); see `PHP_FUNCTION` for why the
/// modifier run is captured as a whole rather than matched in a fixed order.
static PHP_CONST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"(?m)^[^\S\n]*((?:{PHP_MODIFIER}\s+)*)const\s+(\w+)"
    ))
    .unwrap()
});

/// True if a captured modifier run contains `private` or `protected`.
fn has_non_public_modifier(modifiers: &str) -> bool {
    modifiers
        .split_whitespace()
        .any(|m| m == "private" || m == "protected")
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
}
