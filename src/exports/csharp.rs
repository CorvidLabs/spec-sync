use regex::Regex;
use std::sync::LazyLock;

static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"//.*$").unwrap());

static COMMENT_MULTI: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/").unwrap());

/// C# public/internal types: class, struct, interface, enum, record, delegate
static CS_TYPE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*public\s+(?:static\s+)?(?:partial\s+)?(?:sealed\s+)?(?:abstract\s+)?(?:class|struct|interface|enum|record|delegate)\s+(\w+)",
    )
    .unwrap()
});

/// C# public methods and properties
static CS_MEMBER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*public\s+(?:static\s+)?(?:virtual\s+)?(?:override\s+)?(?:abstract\s+)?(?:async\s+)?(?:new\s+)?(?:\w+(?:<[^>]*>)?(?:\[\])?(?:\?)?)\s+(\w+)\s*[({;]",
    )
    .unwrap()
});

/// Extract public symbols from C# source code.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = COMMENT_SINGLE.replace_all(content, "");
    let stripped = COMMENT_MULTI.replace_all(&stripped, "");

    let mut symbols = Vec::new();

    for caps in CS_TYPE.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            symbols.push(name.as_str().to_string());
        }
    }

    for caps in CS_MEMBER.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
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
}
