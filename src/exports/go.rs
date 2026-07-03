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
    // Blank the CONTENTS of string/rune literals — including multi-line backtick raw
    // strings (SQL/GraphQL/template constants) — so a declaration-shaped token or a
    // `{`/`}`/`(`/`)` inside a literal is never read as code or as a delimiter.
    // Newlines are preserved, so all the line-anchored/line-based passes stay aligned.
    let blanked = blank_go_strings(&stripped);

    let mut symbols = Vec::new();

    for caps in GO_DECL.captures_iter(&blanked) {
        if let Some(name) = caps.get(1) {
            symbols.push(name.as_str().to_string());
        }
    }

    // Also capture exported methods
    for caps in GO_METHOD.captures_iter(&blanked) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str().to_string();
            if !symbols.contains(&n) {
                symbols.push(n);
            }
        }
    }

    // Grouped declarations: `const (` / `var (` / `type (` blocks, whose items sit
    // on their own lines with no keyword prefix (missed by the line-anchored
    // regexes above). Capture each item's leading exported (uppercase) identifier,
    // but ONLY at brace-depth 0 AND paren-depth 0 — so struct/interface fields
    // inside a grouped `type`, and the interior of a multi-line value like
    // `X = f(\n ...\n)`, are not mistaken for items or for the group's closer. The
    // scan runs on the string-blanked copy, so a `{`/`}`/`(`/`)` inside a value
    // (`X = "{"`, a struct tag, a multi-line backtick template) never corrupts the
    // depths.
    let mut in_group = false;
    let mut brace_depth: i32 = 0;
    let mut paren_depth: i32 = 0;
    for line in blanked.lines() {
        let bt = line.trim();
        if in_group {
            let at_base = brace_depth == 0 && paren_depth == 0;
            if at_base && bt.starts_with(')') {
                in_group = false;
                continue;
            }
            if at_base
                && let Some(caps) = GO_GROUP_ITEM.captures(bt)
                && let Some(name) = caps.get(1)
            {
                let n = name.as_str();
                if matches!(n.chars().next(), Some(c) if c.is_ascii_uppercase())
                    && !symbols.contains(&n.to_string())
                {
                    symbols.push(n.to_string());
                }
            }
            brace_depth += bt.matches('{').count() as i32 - bt.matches('}').count() as i32;
            paren_depth += bt.matches('(').count() as i32 - bt.matches(')').count() as i32;
            if brace_depth < 0 {
                brace_depth = 0;
            }
            if paren_depth < 0 {
                paren_depth = 0;
            }
            continue;
        }
        if GO_GROUP_OPEN.is_match(bt) {
            in_group = true;
            brace_depth = 0;
            paren_depth = 0;
        }
    }

    symbols
}

/// Return a copy of Go source with the CONTENTS of string and rune literals replaced
/// by spaces, so declaration detection and brace/paren counting are not confused by
/// code-shaped text, `{`, `}`, `(`, `)`, or `#` inside them. Handles interpreted
/// (`"..."`, with `\` escapes), raw (`` `...` `` — which may span multiple lines),
/// and rune (`'...'`) literals. Newlines are preserved throughout.
fn blank_go_strings(content: &str) -> String {
    let chars: Vec<char> = content.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(content.len());
    let mut i = 0;
    while i < n {
        match chars[i] {
            '"' => {
                out.push(' ');
                i += 1;
                while i < n {
                    if chars[i] == '\\' {
                        if i + 1 < n && chars[i + 1] == '\n' {
                            out.push('\n');
                        }
                        i += 2;
                        continue;
                    }
                    if chars[i] == '"' {
                        i += 1;
                        break;
                    }
                    if chars[i] == '\n' {
                        out.push('\n');
                    }
                    i += 1;
                }
            }
            '`' => {
                out.push(' ');
                i += 1;
                while i < n && chars[i] != '`' {
                    if chars[i] == '\n' {
                        out.push('\n');
                    }
                    i += 1;
                }
                if i < n {
                    i += 1;
                }
            }
            '\'' => {
                out.push(' ');
                i += 1;
                while i < n {
                    if chars[i] == '\\' {
                        i += 2;
                        continue;
                    }
                    if chars[i] == '\'' {
                        i += 1;
                        break;
                    }
                    if chars[i] == '\n' {
                        out.push('\n');
                    }
                    i += 1;
                }
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
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

    #[test]
    fn test_go_grouped_delimiters_in_strings() {
        // A `{` / `}` / `(` / `)` inside a grouped item's value string must not
        // corrupt brace/paren depth (which would drop later items AND leave the
        // group open, swallowing every subsequent grouped block), and a multi-line
        // function-call value must not be mistaken for the group's closer.
        let src = r#"
package config

const (
    OpenBrace = "{"
    Version   = "1.0"
    APIPath   = "/v1"
)

var (
    DefaultConfig = loadConfig(
        "path",
    )
    Alpha = 1
    Beta  = 2
)

const LaterExport = 5
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"OpenBrace".to_string()), "{symbols:?}");
        assert!(
            symbols.contains(&"Version".to_string()),
            "after brace-string"
        );
        assert!(symbols.contains(&"APIPath".to_string()));
        assert!(symbols.contains(&"DefaultConfig".to_string()));
        assert!(
            symbols.contains(&"Alpha".to_string()),
            "after multiline value"
        );
        assert!(symbols.contains(&"Beta".to_string()));
        assert!(
            symbols.contains(&"LaterExport".to_string()),
            "group closed correctly so a later const is still seen"
        );
    }

    #[test]
    fn test_go_grouped_type_struct_tag_brace() {
        // A `}` inside a struct-tag string must not drop brace_depth to 0 mid-struct
        // and leak a field as a top-level export.
        let src = "
package m

type (
    Config struct {
        Afield string `json:\"a}\"`
        Bfield int
    }
    Helper int
)
";
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Config".to_string()));
        assert!(symbols.contains(&"Helper".to_string()));
        assert!(!symbols.contains(&"Afield".to_string()), "struct field");
        assert!(!symbols.contains(&"Bfield".to_string()), "struct field");
    }

    #[test]
    fn test_go_grouped_multiline_backtick_raw_string() {
        // A multi-line backtick raw string (SQL/GraphQL/template constant) inside a
        // grouped block must be blanked across lines: its body must not leak an
        // uppercase word as a phantom export, and a `)`/`{`/`(` in the body must not
        // corrupt the group's brace/paren depth.
        let src = "
package db

const (
    schema = `
CREATE TABLE users (
    id INT
);
`
    Version = 2
)

const AfterGroup = 9
";
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Version".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"AfterGroup".to_string()), "group closed");
        // Raw-string body text is not code.
        assert!(!symbols.contains(&"CREATE".to_string()), "SQL body text");
        assert!(!symbols.contains(&"TABLE".to_string()));
    }

    #[test]
    fn test_go_grouped_backtick_with_brace_and_paren_recovers() {
        // Unbalanced `)` / `{` inside a multi-line backtick value must not close the
        // group early or leave it stuck open (which would swallow later exports and
        // subsequent grouped blocks).
        let src = "
package m

var (
    tmpl = `
) closing text and an unbalanced { brace
`
    Alpha = 1
)

const (
    Later = 5
)
";
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Alpha".to_string()), "{symbols:?}");
        assert!(
            symbols.contains(&"Later".to_string()),
            "a later separate group is still seen"
        );
    }
}
