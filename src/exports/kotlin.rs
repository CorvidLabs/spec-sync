use regex::Regex;
use std::sync::LazyLock;

static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"//.*$").unwrap());

static COMMENT_MULTI: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/").unwrap());

/// Kotlin top-level declarations (everything not marked private/internal/protected is public by default).
/// We match: fun, class, object, interface, typealias, val, var, enum class, data class, sealed class, annotation class
/// Also recognizes the `override` modifier (extremely common on interface/abstract-class members —
/// without it the whole line fails to match at all, since the leading modifier chain is anchored to
/// line start) and extension function/property receivers (`fun ReceiverType.name()`, `val Receiver.name`),
/// where the captured group must be the real declared symbol, not the receiver type name.
/// Also recognizes `infix` and `operator` (idiomatic on operator-overload members like `plus`,
/// `plusAssign`, `invoke`, `contains`, `set`, and on infix member functions) — without them the
/// whole line previously failed to match, same class of bug as a missing `override`.
/// Then exclude lines that start with private/internal/protected.
static KT_DECL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*(?:(?:public|internal|private|protected)\s+)?(?:override\s+)?(?:suspend\s+)?(?:inline\s+)?(?:data\s+|sealed\s+|enum\s+|annotation\s+|abstract\s+|open\s+)?(?:const\s+)?(?:infix\s+)?(?:operator\s+)?(?:fun|class|object|interface|typealias|val|var)\s+(?:<[^>]+>\s+)?(?:[A-Za-z_]\w*(?:<[^>]*>)?\??\.)?(\w+)",
    )
    .unwrap()
});

/// Detect visibility — private/internal/protected lines should be excluded
static PRIVATE_LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[^\S\n]*(?:private|internal|protected)\s+").unwrap());

/// Detect a line that opens a "type body" scope (class/object/interface, optionally a companion
/// object), as opposed to a function body or any other block. Declarations directly inside a type
/// body are real export candidates (e.g. interface/class members); declarations nested inside a
/// function body — or any other block, like a `for`/`if`/lambda — are local and are never exported,
/// regardless of whether they happen to use `val`/`var`/`fun`.
static TYPE_BODY_OPEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*(?:companion\s+)?(?:(?:public|internal|private|protected)\s+)?(?:(?:data|sealed|enum|annotation|abstract|open)\s+)*(?:class|object|interface)\b",
    )
    .unwrap()
});

/// Extract exported symbols from Kotlin source code.
/// In Kotlin, everything is public by default unless marked private/internal/protected.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = COMMENT_SINGLE.replace_all(content, "");
    let stripped = COMMENT_MULTI.replace_all(&stripped, "");

    let mut symbols = Vec::new();
    // Tracks nested `{ }` scopes: true = a class/object/interface body (member declarations are
    // legitimate export candidates), false = a function body or any other block (declarations
    // inside are local and never exported). An empty stack means top level.
    let mut scope_stack: Vec<bool> = Vec::new();

    for line in stripped.lines() {
        let trimmed = line.trim();
        let in_exportable_scope = scope_stack.last().copied().unwrap_or(true);

        // Skip private/internal/protected declarations, and anything nested inside a
        // non-type-body scope (function bodies, control-flow blocks, lambdas, ...).
        if in_exportable_scope
            && !PRIVATE_LINE.is_match(line)
            && let Some(caps) = KT_DECL.captures(line)
            && let Some(name) = caps.get(1)
        {
            // Skip companion objects (they're not standalone exports)
            if !trimmed.starts_with("companion") {
                symbols.push(name.as_str().to_string());
            }
        }

        // A class/object/interface only opens an *exportable* scope for its members if it is
        // itself reachable from the top level. A class declared locally inside a function body
        // (a valid, if unusual, Kotlin construct) is not accessible from outside that function,
        // so its members must not leak out as if they were module-level exports even though the
        // line syntactically looks like a type-body opener.
        let opens_type_body = in_exportable_scope && TYPE_BODY_OPEN.is_match(line);
        for ch in line.chars() {
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
    fn test_kotlin_exports() {
        let src = r#"
package com.example.auth

fun createAuth(config: Config): Auth {}
private fun internalHelper() {}
class AuthService {}
data class UserProfile(val name: String)
sealed class AuthState {}
object AuthManager {}
interface Authenticator {}
typealias Token = String
val DEFAULT_TTL = 3600
var globalInstance: Auth? = null
enum class Status { ACTIVE, EXPIRED }
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"createAuth".to_string()));
        assert!(symbols.contains(&"AuthService".to_string()));
        assert!(symbols.contains(&"UserProfile".to_string()));
        assert!(symbols.contains(&"AuthState".to_string()));
        assert!(symbols.contains(&"AuthManager".to_string()));
        assert!(symbols.contains(&"Authenticator".to_string()));
        assert!(symbols.contains(&"Token".to_string()));
        assert!(symbols.contains(&"DEFAULT_TTL".to_string()));
        assert!(symbols.contains(&"globalInstance".to_string()));
        assert!(symbols.contains(&"Status".to_string()));
        assert!(!symbols.contains(&"internalHelper".to_string()));
    }

    #[test]
    fn test_kotlin_visibility() {
        let src = r#"
public fun publicFun() {}
internal fun internalFun() {}
protected fun protectedFun() {}
private fun privateFun() {}
fun defaultPublicFun() {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"publicFun".to_string()));
        assert!(symbols.contains(&"defaultPublicFun".to_string()));
        assert!(!symbols.contains(&"internalFun".to_string()));
        assert!(!symbols.contains(&"protectedFun".to_string()));
        assert!(!symbols.contains(&"privateFun".to_string()));
    }

    #[test]
    fn test_kotlin_suspend() {
        let src = r#"
suspend fun fetchData(): Data {}
public suspend fun loadProfile(): Profile {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"fetchData".to_string()));
        assert!(symbols.contains(&"loadProfile".to_string()));
    }

    #[test]
    fn test_kotlin_const_val_exported() {
        // Finding #9: top-level `const val` is a public compile-time constant and
        // must be captured; the `const` modifier previously blocked the match.
        let src = r#"
const val MAX_SIZE = 100
public const val VISIBLE = 2
internal const val HIDDEN = 3
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"MAX_SIZE".to_string()), "const val");
        assert!(symbols.contains(&"VISIBLE".to_string()), "public const val");
        assert!(
            !symbols.contains(&"HIDDEN".to_string()),
            "internal const val is not exported"
        );
    }

    #[test]
    fn test_kotlin_override_members_exported() {
        // Finding: `override val` / `override fun` members were silently dropped because
        // `override` wasn't part of the recognized modifier chain, and the line-anchored
        // regex has no way to skip an unrecognized leading token — the whole line failed to
        // match, so the entire concrete public surface of a class implementing an interface
        // (extremely common/idiomatic Kotlin) disappeared.
        let src = r#"
interface Authenticator {
    val name: String
    fun validate(token: String): Boolean
}

class TokenAuthenticator(private val secret: String) : Authenticator {
    override val name: String = "token"
    override val priority: Int = 5
    override fun validate(token: String): Boolean {
        val trimmed = token.trim()
        val isValid = trimmed == secret
        return isValid
    }
    override fun describe(): String {
        val base = super.describe()
        return "$base [token]"
    }
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Authenticator".to_string()));
        assert!(symbols.contains(&"TokenAuthenticator".to_string()));
        assert!(symbols.contains(&"name".to_string()));
        assert!(symbols.contains(&"priority".to_string()));
        assert!(symbols.contains(&"validate".to_string()));
        assert!(symbols.contains(&"describe".to_string()));
        assert!(
            !symbols.contains(&"trimmed".to_string()),
            "local var inside override fn body"
        );
        assert!(
            !symbols.contains(&"isValid".to_string()),
            "local var inside override fn body"
        );
        assert!(
            !symbols.contains(&"base".to_string()),
            "local var inside override fn body"
        );
    }

    #[test]
    fn test_kotlin_extension_function_captures_real_name() {
        // Finding: extension functions/properties captured the receiver type name instead of
        // the actual declared symbol — `fun String.toSlug()` produced "String" (not a real
        // declared symbol at all) and never produced "toSlug".
        let src = r#"
fun String.toSlug(): String {
    val lowered = this.lowercase()
    return lowered.replace(" ", "-")
}

val String.wordCount: Int
    get() = this.split(" ").size
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"toSlug".to_string()));
        assert!(symbols.contains(&"wordCount".to_string()));
        assert!(
            !symbols.contains(&"String".to_string()),
            "receiver type must not be captured as a symbol"
        );
        assert!(
            !symbols.contains(&"lowered".to_string()),
            "local var inside extension fn body"
        );
    }

    #[test]
    fn test_kotlin_locals_not_exported() {
        // Finding: with no brace/scope tracking, any `val`/`var` declared on its own line
        // anywhere in the file — including local variables deep inside function bodies, even
        // inside functions already marked `private` — was emitted as a top-level public export.
        let src = r#"
fun computeTotal(items: List<Int>): Int {
    val subtotal = items.sum()
    val tax = subtotal * 0.08
    var total = subtotal + tax
    for (bonus in listOf(1, 2, 3)) {
        val adjusted = bonus * 2
        total += adjusted
    }
    return total.toInt()
}

private fun helper(): Int {
    val secretLocal = 42
    return secretLocal
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["computeTotal".to_string()]);
    }

    #[test]
    fn test_kotlin_learnxinyminutes_operator_overloads() {
        // Real excerpt from learnxinyminutes.com/kotlin (the "Counter" operator-overloading
        // section). Finding: `operator` (and `infix`, covered by the sibling regression test
        // below) was not part of the recognized modifier chain, so every `operator fun` member
        // — an extremely common, idiomatic Kotlin pattern for overloading `+=`, `++`, `+`, `*`,
        // `in`, `[]`, and invocation — silently failed to match and was dropped entirely,
        // including the top-level extension-function overload of unary `-`.
        let src = r#"
data class Counter(var value: Int) {
    // overload Counter += Int
    operator fun plusAssign(increment: Int) {
        this.value += increment
    }

    // overload Counter++ and ++Counter
    operator fun inc() = Counter(value + 1)

    // overload Counter + Counter
    operator fun plus(other: Counter) = Counter(this.value + other.value)

    // overload Counter * Counter
    operator fun times(other: Counter) = Counter(this.value * other.value)

    // overload Counter * Int
    operator fun times(value: Int) = Counter(this.value * value)

    // overload Counter in Counter
    operator fun contains(other: Counter) = other.value == this.value

    // overload Counter[Int] = Int
    operator fun set(index: Int, value: Int) {
        this.value = index + value
    }

    // overload Counter instance invocation
    operator fun invoke() = println("The value of the counter is $value")

}
/* You can also overload operators through extension methods */
// overload -Counter
operator fun Counter.unaryMinus() = Counter(-this.value)

fun operatorOverloadingDemo() {
    var counter1 = Counter(0)
    var counter2 = Counter(5)
    counter1 += 7
    println(counter1) // => Counter(value=7)
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Counter".to_string()));
        assert!(symbols.contains(&"plusAssign".to_string()));
        assert!(symbols.contains(&"inc".to_string()));
        assert!(symbols.contains(&"plus".to_string()));
        assert!(symbols.contains(&"times".to_string()));
        assert!(symbols.contains(&"contains".to_string()));
        assert!(symbols.contains(&"set".to_string()));
        assert!(symbols.contains(&"invoke".to_string()));
        assert!(
            symbols.contains(&"unaryMinus".to_string()),
            "top-level `operator fun Counter.unaryMinus()` extension must capture the real name"
        );
        assert!(symbols.contains(&"operatorOverloadingDemo".to_string()));
    }

    #[test]
    fn test_kotlin_learnxinyminutes_local_class_members_not_exported() {
        // Real excerpt from learnxinyminutes.com/kotlin (inside `fun main`). Finding: a class
        // declared locally inside a function body still unconditionally opened an "exportable"
        // type-body scope for its own members (the scope tracker only looked at whether the
        // *line* syntactically opened a class/object/interface, not whether that class was
        // itself reachable from the top level), so `memberFunction` leaked out as if it were a
        // module-level export even though `ExampleClass` — and everything in it — is local to
        // `main` and inaccessible from outside it. Also exercises `infix fun`.
        let src = r#"
fun main(args: Array<String>) {
    // The "class" keyword is used to declare classes.
    class ExampleClass(val x: Int) {
        fun memberFunction(y: Int): Int {
            return x + y
        }

        infix fun infixMemberFunction(y: Int): Int {
            return x * y
        }
    }
    val fooExampleClass = ExampleClass(7)
    println(fooExampleClass.memberFunction(4)) // => 11
    println(fooExampleClass infixMemberFunction 4) // => 28
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["main".to_string()]);
    }
}
