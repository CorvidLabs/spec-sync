use regex::Regex;
use std::sync::LazyLock;

static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)//.*$").unwrap());

static COMMENT_MULTI: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/").unwrap());

/// Java public declarations: public class, interface, enum, record, @interface (annotation)
static JAVA_TYPE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*public\s+(?:static\s+)?(?:final\s+)?(?:abstract\s+)?(?:sealed\s+)?(?:class|interface|enum|record|@interface)\s+(\w+)",
    )
    .unwrap()
});

/// Type/interface header used to track brace-nesting context (not to collect names --
/// `JAVA_TYPE` already does that for the `public` case). Captures whether the header
/// itself carries an explicit `public` and whether it is an interface/@interface, so
/// callers can tell when a body's direct members are implicitly public.
static JAVA_TYPE_HEADER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*(?P<pub>public\s+)?(?:static\s+)?(?:final\s+)?(?:abstract\s+)?(?:sealed\s+)?(?:strictfp\s+)?(?P<kind>@interface|interface|class|enum|record)\s+\w+",
    )
    .unwrap()
});

/// Shared "modifiers + type + name + terminator" tail for member declarations. The
/// modifier group is a repeated alternation (not a fixed sequence), since Java allows
/// `static`/`final`/`default`/`synchronized`/`abstract`/`native`/`strictfp` in any order
/// and any combination (e.g. `public synchronized static void bar()` — `synchronized`
/// before `static` — is equally valid to the canonical `public static synchronized`, but
/// a hardcoded fixed order previously rejected anything other than the one sequence it
/// hardcoded).
///
/// The leading generic-method type-parameter list allows one level of nested angle
/// brackets (`<(?:[^<>]|<[^<>]*>)*>`), not a flat `<[^>]+>`: a bounded type parameter
/// like `<T extends Comparable<T>>` (an extremely common generic-method idiom) has its
/// own nested `<T>` closing before the parameter list's real `>`, so a flat pattern
/// stopped at that inner `>` and left the trailing mandatory `\s+` unable to match
/// (the next character is the *outer* `>`, not whitespace) -- backtracking then
/// dropped the optional group entirely, but the following return-type group can't
/// start with `<` either, so the whole member declaration silently failed to match.
const JAVA_MEMBER_TAIL: &str = r"(?:(?:static|final|default|synchronized|abstract|native|strictfp)\s+)*(?:<(?:[^<>]|<[^<>]*>)*>\s+)?(?:\w+(?:<[^>]*>)?(?:\[\])*)\s+(\w+)\s*[({;=]";

/// Java public methods and fields (explicit `public` keyword required).
static JAVA_MEMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"(?m)^[^\S\n]*public\s+{JAVA_MEMBER_TAIL}")).unwrap());

/// Java interface/@interface members: same shape as `JAVA_MEMBER` but `public` is
/// optional, since interface (and annotation type) constants and methods are
/// implicitly public and idiomatically never carry the keyword. Only applied while
/// walking bodies known to be directly inside an exported interface/@interface --
/// members with an explicit `private` (legal for interface methods since Java 9) are
/// filtered out by the caller before this is tried.
static JAVA_IMPLICIT_MEMBER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(r"(?m)^[^\S\n]*(?:public\s+)?{JAVA_MEMBER_TAIL}")).unwrap()
});

/// Matches a member line that explicitly opts back out of the implicit-public default
/// with `private` (optionally preceded by one modifier, e.g. `private static`).
static JAVA_PRIVATE_MODIFIER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[^\S\n]*(?:\w+\s+)?private\b").unwrap());

/// Extract public symbols from Java source code.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = COMMENT_SINGLE.replace_all(content, "");
    let stripped = COMMENT_MULTI.replace_all(&stripped, "");

    let mut symbols = Vec::new();

    for caps in JAVA_TYPE.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            symbols.push(name.as_str().to_string());
        }
    }

    for caps in JAVA_MEMBER.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str().to_string();
            if !symbols.contains(&n) {
                symbols.push(n);
            }
        }
    }

    // Interface (and annotation type) members have no visibility keyword of their own
    // -- they're implicitly public. Walk line-by-line tracking brace depth so we know
    // when we're directly inside the body of an interface/@interface that is itself
    // part of the public surface (explicitly `public`, or nested inside another such
    // body), and pick up member declarations there even without `public`.
    let mut interface_body_stack: Vec<bool> = Vec::new();
    for line in stripped.lines() {
        let auto_public = *interface_body_stack.last().unwrap_or(&false);

        if let Some(caps) = JAVA_TYPE_HEADER.captures(line) {
            let is_interface = matches!(
                caps.name("kind").map(|m| m.as_str()),
                Some("interface") | Some("@interface")
            );
            let is_public = caps.name("pub").is_some();
            let child_auto_public = is_interface && (is_public || auto_public);
            if line.contains('{') {
                interface_body_stack.push(child_auto_public);
            }
            continue;
        }

        if auto_public
            && !JAVA_PRIVATE_MODIFIER.is_match(line)
            && let Some(caps) = JAVA_IMPLICIT_MEMBER.captures(line)
            && let Some(name) = caps.get(1)
        {
            let n = name.as_str().to_string();
            if !symbols.contains(&n) {
                symbols.push(n);
            }
        }

        for ch in line.chars() {
            match ch {
                '{' => interface_body_stack.push(false),
                '}' => {
                    interface_body_stack.pop();
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
    fn test_java_exports() {
        let src = r#"
package com.example.auth;

public class AuthService {
    public static final String DEFAULT_TOKEN = "abc";
    public String validate(String token) {}
    private void internalCheck() {}
    public int getTimeout() {}
}

public interface Authenticator {}
public enum AuthStatus { ACTIVE, EXPIRED }
public record UserProfile(String name, int age) {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"AuthService".to_string()));
        assert!(symbols.contains(&"Authenticator".to_string()));
        assert!(symbols.contains(&"AuthStatus".to_string()));
        assert!(symbols.contains(&"UserProfile".to_string()));
        assert!(symbols.contains(&"DEFAULT_TOKEN".to_string()));
        assert!(symbols.contains(&"validate".to_string()));
        assert!(symbols.contains(&"getTimeout".to_string()));
        assert!(!symbols.contains(&"internalCheck".to_string()));
    }

    #[test]
    fn test_java_abstract() {
        let src = r#"
public abstract class BaseController {
    public abstract void handle();
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"BaseController".to_string()));
        assert!(symbols.contains(&"handle".to_string()));
    }

    #[test]
    fn test_java_comments_stripped() {
        let src = r#"
// public class FakeClass {}
/* public interface FakeInterface {} */
/**
 * Javadoc comment
 * public enum FakeEnum {}
 */
public class RealClass {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"RealClass".to_string()));
        assert!(!symbols.contains(&"FakeClass".to_string()));
        assert!(!symbols.contains(&"FakeInterface".to_string()));
        assert!(!symbols.contains(&"FakeEnum".to_string()));
    }

    #[test]
    fn test_java_generics() {
        let src = r#"
public class Repository<T extends Entity> {
    public <E> List<E> findAll() {}
    public Map<String, Object> getMetadata() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Repository".to_string()));
        assert!(symbols.contains(&"findAll".to_string()));
        assert!(symbols.contains(&"getMetadata".to_string()));
    }

    #[test]
    fn test_java_annotation_type() {
        let src = r#"
public @interface Cacheable {
    int ttl() default 300;
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Cacheable".to_string()));
    }

    #[test]
    fn test_java_private_and_protected_excluded() {
        let src = r#"
public class Service {
    private String secret;
    protected void onInit() {}
    void packagePrivate() {}
    public String getName() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Service".to_string()));
        assert!(symbols.contains(&"getName".to_string()));
        assert!(!symbols.contains(&"secret".to_string()));
        assert!(!symbols.contains(&"onInit".to_string()));
        assert!(!symbols.contains(&"packagePrivate".to_string()));
    }

    #[test]
    fn test_java_sealed_class() {
        let src = r#"
public sealed class Shape permits Circle, Rectangle {
    public String describe() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Shape".to_string()));
        assert!(symbols.contains(&"describe".to_string()));
    }

    #[test]
    fn test_java_static_final_combined() {
        let src = r#"
public class Constants {
    public static final int MAX_SIZE = 100;
    public static final String VERSION = "1.0";
    public static void init() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Constants".to_string()));
        assert!(symbols.contains(&"MAX_SIZE".to_string()));
        assert!(symbols.contains(&"VERSION".to_string()));
        assert!(symbols.contains(&"init".to_string()));
    }

    #[test]
    fn test_java_interface_members_implicitly_public() {
        // Interface fields, abstract methods, default methods, and static factory
        // methods are implicitly public and idiomatically never carry the `public`
        // keyword -- the same bug class as the Swift protocol-member issue.
        let src = r#"
public interface PaymentGateway {
    int MAX_RETRIES = 3;
    String DEFAULT_CURRENCY = "USD";

    boolean authorize(String token, double amount);
    List<Transaction> history(String accountId);

    default boolean isRefundable(String transactionId) { return true; }

    static PaymentGateway createDefault() { return new DefaultPaymentGateway(); }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"PaymentGateway".to_string()));
        assert!(symbols.contains(&"MAX_RETRIES".to_string()));
        assert!(symbols.contains(&"DEFAULT_CURRENCY".to_string()));
        assert!(symbols.contains(&"authorize".to_string()));
        assert!(symbols.contains(&"history".to_string()));
        assert!(symbols.contains(&"isRefundable".to_string()));
        assert!(symbols.contains(&"createDefault".to_string()));
    }

    #[test]
    fn test_java_annotation_type_members_implicitly_public() {
        // @interface (annotation type) elements are the entire public contract of the
        // annotation and never carry `public`, same as interface members.
        let src = r#"
public @interface Cacheable {
    String key();
    boolean enabled() default true;
    int ttl() default 300;
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Cacheable".to_string()));
        assert!(symbols.contains(&"key".to_string()));
        assert!(symbols.contains(&"enabled".to_string()));
        assert!(symbols.contains(&"ttl".to_string()));
    }

    #[test]
    fn test_java_interface_private_helper_excluded() {
        // Interface private methods (legal since Java 9) are the one case where a
        // member is NOT implicitly public -- an explicit `private` must still exclude
        // it even though every other member in the body is auto-public.
        let src = r#"
public interface Validator {
    boolean isValid(String input);

    private boolean normalize(String input) { return input.trim().isEmpty(); }

    private static boolean isBlank(String input) { return input == null; }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Validator".to_string()));
        assert!(symbols.contains(&"isValid".to_string()));
        assert!(!symbols.contains(&"normalize".to_string()));
        assert!(!symbols.contains(&"isBlank".to_string()));
    }

    #[test]
    fn test_java_learnxinyminutes_bicycle_class() {
        // Real excerpt (trimmed of surrounding context only) from learnxinyminutes.com's
        // Java tutorial: a class with mixed-visibility fields (public/private/protected/
        // package-private/static), a static initializer block, two constructors, several
        // public getter/setter methods, and an `@Override` method with a trailing `//`
        // comment on the same line as the annotation -- real idiomatic syntax breadth
        // that hand-written examples might not think to combine.
        let src = r#"
class Bicycle {

    // Bicycle's Fields/Variables
    public int cadence; // Public: Can be accessed from anywhere
    private int speed;  // Private: Only accessible from within the class
    protected int gear; // Protected: Accessible from the class and subclasses
    String name; // default: Only accessible from within this package
    static String className; // Static class variable

    // Static block
    static {
        className = "Bicycle";
    }

    // Constructors are a way of creating classes
    public Bicycle() {
        gear = 1;
        cadence = 50;
        speed = 5;
        name = "Bontrager";
    }
    // This is a constructor that takes arguments
    public Bicycle(int startCadence, int startSpeed, int startGear,
        String name) {
        this.gear = startGear;
        this.cadence = startCadence;
        this.speed = startSpeed;
        this.name = name;
    }

    public int getCadence() {
        return cadence;
    }

    // void methods require no return statement
    public void setCadence(int newValue) {
        cadence = newValue;
    }
    public void setGear(int newValue) {
        gear = newValue;
    }
    public void speedUp(int increment) {
        speed += increment;
    }
    public void slowDown(int decrement) {
        speed -= decrement;
    }
    public void setName(String newName) {
        name = newName;
    }
    public String getName() {
        return name;
    }

    //Method to display the attribute values of this Object.
    @Override // Inherited from the Object class.
    public String toString() {
        return "gear: " + gear + " cadence: " + cadence + " speed: " + speed +
            " name: " + name;
    }
} // end class Bicycle
"#;
        let symbols = extract_exports(src);
        // Public members expected.
        for expected in [
            "cadence",
            "getCadence",
            "setCadence",
            "setGear",
            "speedUp",
            "slowDown",
            "setName",
            "getName",
            "toString",
        ] {
            assert!(
                symbols.contains(&expected.to_string()),
                "missing {expected}: {symbols:?}"
            );
        }
        // Non-public members must stay excluded.
        for excluded in ["speed", "gear", "name", "className"] {
            assert!(
                !symbols.contains(&excluded.to_string()),
                "leaked {excluded}: {symbols:?}"
            );
        }
    }

    #[test]
    fn test_java_learnxinyminutes_interfaces_with_pseudocode_comments() {
        // Real excerpt from learnxinyminutes.com's Java tutorial. This is a regression
        // test for a genuine bug: `COMMENT_SINGLE` was `//.*$` *without* the `(?m)` flag,
        // so in the `regex` crate `$` only anchors to the end of the whole haystack --
        // meaning `//` comments were effectively never stripped except possibly on the
        // file's last line. That was mostly invisible because `JAVA_TYPE`/`JAVA_MEMBER`
        // both require `public` to be the first non-whitespace token on the line (which a
        // `//`-prefixed line can never satisfy), but the interface-nesting brace-tracker
        // walks raw characters of every (falsely) unstripped line -- and this real file
        // has a `//`-commented pseudo-code block with an unbalanced `{` on its own line
        // right before real `public interface` declarations, exactly the kind of input
        // that could desync the nesting stack and corrupt implicit-public detection for
        // the interface members that follow.
        let src = r#"
// Interfaces
// Interface declaration syntax
// <access-level> interface <interface-name> extends <super-interfaces> {
//     // Constants
//     // Method declarations
// }

// Example - Food:
public interface Edible {
    public void eat(); // Any class that implements this interface, must
                       // implement this method.
}

public interface Digestible {
    public void digest();
    // Since Java 8, interfaces can have default method.
    public default void defaultMethod() {
        System.out.println("Hi from default method ...");
    }
}

// We can now create a class that implements both of these interfaces.
public class Fruit implements Edible, Digestible {
    @Override
    public void eat() {
        // ...
    }

    @Override
    public void digest() {
        // ...
    }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Edible".to_string()));
        assert!(symbols.contains(&"Digestible".to_string()));
        assert!(symbols.contains(&"Fruit".to_string()));
        assert!(symbols.contains(&"eat".to_string()));
        assert!(symbols.contains(&"digest".to_string()));
        assert!(symbols.contains(&"defaultMethod".to_string()));
        // The commented-out pseudo-code (e.g. "interface", "Constants", "Method
        // declarations") must never leak through as if it were a real declaration.
        assert!(!symbols.contains(&"Constants".to_string()));
        assert!(!symbols.contains(&"Declarations".to_string()));
    }

    #[test]
    fn test_java_synchronized_static_modifier_order() {
        // Regression test: `JAVA_MEMBER_TAIL` previously only special-cased
        // `static`/`final` reordering, with every other modifier
        // (synchronized/abstract/native/strictfp/default) fixed at one position in the
        // sequence -- `synchronized` before `static` (equally valid Java) failed to
        // match since the canonical order hardcoded `static` first.
        let src = r#"
public class Counter {
    public synchronized static void increment() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Counter".to_string()));
        assert!(symbols.contains(&"increment".to_string()));
    }

    #[test]
    fn test_java_public_default_method_and_modifier_order() {
        // Finding #2: an explicit `public default` interface method must still match
        // even though `default` wasn't previously in the modifier alternation.
        // Finding #3: `public final static` (reversed from the canonical
        // `public static final`) must still be recognized as a member.
        let src = r#"
public interface Widget {
    public int MAX = 5;
    public boolean render();
    public default boolean isVisible() { return true; }
}

public class Config {
    public final static int VERSION = 2;
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Widget".to_string()));
        assert!(symbols.contains(&"MAX".to_string()));
        assert!(symbols.contains(&"render".to_string()));
        assert!(
            symbols.contains(&"isVisible".to_string()),
            "public default method: {symbols:?}"
        );
        assert!(symbols.contains(&"Config".to_string()));
        assert!(
            symbols.contains(&"VERSION".to_string()),
            "public final static (reversed order): {symbols:?}"
        );
    }

    #[test]
    fn test_java_bounded_generic_method_type_parameter_captured() {
        // Bug found via adversarial probing: `JAVA_MEMBER_TAIL`'s generic-method
        // type-parameter group was a flat `<[^>]+>`, which can never contain a `>`. A
        // bounded type parameter like `<T extends Comparable<T>>` (a common generic-
        // method idiom, e.g. `Collections.max`-style signatures) has its own nested
        // `<T>` closing before the parameter list's real outer `>`, so the flat
        // pattern stopped at the inner `>`, leaving the mandatory trailing whitespace
        // unmatched at that position; the whole optional group was then dropped by
        // backtracking, but the following return-type group can't start with `<`
        // either, so the entire member declaration silently failed to match and
        // `max` was dropped.
        let src = r#"
public class Generics {
    public <T extends Comparable<T>> T max(T a, T b) { return a; }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Generics".to_string()));
        assert!(
            symbols.contains(&"max".to_string()),
            "expected max in {symbols:?}"
        );
    }

    #[test]
    fn test_java_public_method_inside_package_private_class_still_exported() {
        // Documents established, intentional behavior (matching the precedent set by
        // `test_java_learnxinyminutes_bicycle_class`, whose top-level `class Bicycle`
        // also carries no `public` modifier): this backend deliberately keys
        // exportedness off a member's *own* modifier, not full reachability through
        // every enclosing type. A package-private top-level class's `public` members
        // (and any `public` nested type within it) are still reported as exported.
        // This is a deliberate simplification, not a gap -- flipping it would
        // contradict the already-accepted Bicycle-class behavior.
        let src = r#"
class PackagePrivateOuter {
    public void doWork() {}
    public static class NestedPublic {
        public void helper() {}
    }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"doWork".to_string()));
        assert!(symbols.contains(&"NestedPublic".to_string()));
        assert!(symbols.contains(&"helper".to_string()));
    }

    #[test]
    fn test_java_public_method_inside_private_nested_class_still_exported() {
        // Same documented convention as above, one level deeper: a `private` nested
        // class's own `public` members are still reported as exported, consistent
        // with member-level modifiers being authoritative throughout this backend.
        let src = r#"
public class Outer {
    private static class Helper {
        public void helperMethod() {}
    }
    public void visibleMethod() {}
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Outer".to_string()));
        assert!(symbols.contains(&"visibleMethod".to_string()));
        assert!(symbols.contains(&"helperMethod".to_string()));
    }

    #[test]
    fn test_java_public_method_inside_enum_body_captured() {
        // Not previously exercised: a `public` method declared directly in an enum's
        // body (after the constant list) is an ordinary member declaration and must
        // be captured the same as inside a class.
        let src = r#"
public enum Status {
    ACTIVE, INACTIVE;
    public String label() { return name(); }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Status".to_string()));
        assert!(
            symbols.contains(&"label".to_string()),
            "expected label in {symbols:?}"
        );
    }
}
