use regex::Regex;
use std::sync::LazyLock;

static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"//.*$").unwrap());

static COMMENT_MULTI: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/").unwrap());

/// Go exports: func Name, type Name, var Name, const Name
/// In Go, anything starting with uppercase is exported.
static GO_DECL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^(?:func|type|var|const)\s+(?:\([^)]*\)\s+)?([A-Z]\w*)").unwrap()
});

/// Go method: func (receiver) Name(...)
static GO_METHOD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^func\s+\([^)]+\)\s+([A-Z]\w*)").unwrap());

/// Opening of a grouped declaration: `const (`, `var (`, `type (` (the `(` at
/// end of the trimmed line, items following on their own lines).
static GO_GROUP_OPEN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?:const|var|type)\s*\($").unwrap());

/// Leading identifier of a grouped item line (`Name = ...`, `Name Type`, ...).
static GO_GROUP_ITEM: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^([A-Za-z_]\w*)").unwrap());

/// Extract exported symbols from Go source code.
/// In Go, any top-level identifier starting with an uppercase letter is exported.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = COMMENT_SINGLE.replace_all(content, "");
    let stripped = COMMENT_MULTI.replace_all(&stripped, "");

    let mut symbols = Vec::new();

    for caps in GO_DECL.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            symbols.push(name.as_str().to_string());
        }
    }

    // Also capture exported methods
    for caps in GO_METHOD.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str().to_string();
            if !symbols.contains(&n) {
                symbols.push(n);
            }
        }
    }

    // Grouped declarations: `const (` / `var (` / `type (` blocks, whose items sit
    // on their own lines with no keyword prefix (missed by the line-anchored
    // regexes above). Capture the leading identifier of each item at brace-depth 0
    // so struct/interface field lines inside a grouped `type` are not mistaken for
    // top-level exports.
    let mut in_group = false;
    let mut brace_depth: i32 = 0;
    for line in stripped.lines() {
        let t = line.trim();
        if in_group {
            if brace_depth == 0 && t.starts_with(')') {
                in_group = false;
                continue;
            }
            if brace_depth == 0
                && let Some(caps) = GO_GROUP_ITEM.captures(t)
                && let Some(name) = caps.get(1)
            {
                let n = name.as_str();
                if matches!(n.chars().next(), Some(c) if c.is_ascii_uppercase())
                    && !symbols.contains(&n.to_string())
                {
                    symbols.push(n.to_string());
                }
            }
            brace_depth += t.matches('{').count() as i32 - t.matches('}').count() as i32;
            if brace_depth < 0 {
                brace_depth = 0;
            }
            continue;
        }
        if GO_GROUP_OPEN.is_match(t) {
            in_group = true;
            brace_depth = 0;
        }
    }

    symbols
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_go_exports() {
        let src = r#"
package auth

func CreateAuth(config Config) Auth {}
func privateFunc() {}
type AuthService struct {}
type authInternal struct {}
const DefaultTTL = 3600
var GlobalInstance *Auth
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"CreateAuth".to_string()));
        assert!(symbols.contains(&"AuthService".to_string()));
        assert!(symbols.contains(&"DefaultTTL".to_string()));
        assert!(symbols.contains(&"GlobalInstance".to_string()));
        assert!(!symbols.contains(&"privateFunc".to_string()));
        assert!(!symbols.contains(&"authInternal".to_string()));
    }

    #[test]
    fn test_go_methods() {
        let src = r#"
package auth

func (a *Auth) Validate(token string) bool {}
func (a *Auth) internal() {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Validate".to_string()));
        assert!(!symbols.contains(&"internal".to_string()));
    }

    #[test]
    fn test_go_comments_stripped() {
        let src = r#"
package main

// func FakeExport() {}
/* func AlsoFake() {} */
func RealExport() {}
/*
func MultiLineFake() {}
type FakeType struct {}
*/
type RealType struct {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"RealExport".to_string()));
        assert!(symbols.contains(&"RealType".to_string()));
        assert!(!symbols.contains(&"FakeExport".to_string()));
        assert!(!symbols.contains(&"AlsoFake".to_string()));
        assert!(!symbols.contains(&"MultiLineFake".to_string()));
        assert!(!symbols.contains(&"FakeType".to_string()));
    }

    #[test]
    fn test_go_interface_declarations() {
        let src = r#"
package service

type Reader interface {
    Read(p []byte) (n int, err error)
}

type Writer interface {
    Write(p []byte) (n int, err error)
}

type internalHelper interface {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Reader".to_string()));
        assert!(symbols.contains(&"Writer".to_string()));
        assert!(!symbols.contains(&"internalHelper".to_string()));
    }

    #[test]
    fn test_go_const_var_groups() {
        let src = r#"
package config

const MaxRetries = 3
const minTimeout = 100

var DefaultClient *Client
var debugMode = false

type Config struct {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"MaxRetries".to_string()));
        assert!(symbols.contains(&"DefaultClient".to_string()));
        assert!(symbols.contains(&"Config".to_string()));
        assert!(!symbols.contains(&"minTimeout".to_string()));
        assert!(!symbols.contains(&"debugMode".to_string()));
    }

    #[test]
    fn test_go_value_receiver() {
        let src = r#"
package auth

func (a Auth) String() string {}
func (a Auth) serialize() string {}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"String".to_string()));
        assert!(!symbols.contains(&"serialize".to_string()));
    }

    #[test]
    fn test_go_empty_file() {
        let src = "package main\n";
        let symbols = extract_exports(src);
        assert!(symbols.is_empty());
    }

    #[test]
    fn test_go_grouped_declarations() {
        // Finding #5: items inside grouped const/var/type blocks sit on their own
        // lines with no keyword prefix and were missed by the line-anchored regex.
        let src = r#"
package config

const (
    MaxRetries = 3
    Timeout    = 30
    internalX  = 5
)

var (
    GlobalState *State
    localState  int
)

type (
    Point struct {
        X int
        Y int
    }
    Named    int
    private2 string
)
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"MaxRetries".to_string()));
        assert!(symbols.contains(&"Timeout".to_string()));
        assert!(symbols.contains(&"GlobalState".to_string()));
        assert!(symbols.contains(&"Point".to_string()));
        assert!(symbols.contains(&"Named".to_string()));
        // Struct fields inside a grouped `type` must NOT be captured as exports.
        assert!(!symbols.contains(&"X".to_string()), "struct field X");
        assert!(!symbols.contains(&"Y".to_string()), "struct field Y");
        // Unexported (lowercase) grouped items are skipped.
        assert!(!symbols.contains(&"internalX".to_string()));
        assert!(!symbols.contains(&"localState".to_string()));
        assert!(!symbols.contains(&"private2".to_string()));
    }
}
