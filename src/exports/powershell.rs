use regex::Regex;
use std::sync::LazyLock;

/// `Export-ModuleMember -Function Foo-Bar, Baz-Qux` (can also carry `-Variable`,
/// `-Alias`, `-Cmdlet`, etc. alongside/instead of `-Function`; we only care about
/// the `-Function` list since that's what a consuming module/spec would call).
/// Can appear multiple times in a file; the union across all occurrences is the
/// authoritative export list, exactly like Python's `__all__`.
static EXPORT_MODULE_MEMBER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?mi)^\s*Export-ModuleMember\b[^\r\n]*-Function\s+([^\r\n]+)").unwrap()
});

/// A single name (possibly quoted) inside an `-Function` list: bare
/// `Get-User`, or quoted `'Get-User'` / `"Get-User"`.
static FUNCTION_NAME_ITEM: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"["']?([A-Za-z0-9_][A-Za-z0-9_-]*)["']?"#).unwrap());

/// Marks the start of a subsequent `-Flag` (e.g. `-Alias`, `-Variable`,
/// `-Cmdlet`) on the same `Export-ModuleMember` line, which ends the
/// `-Function` name list. Flags are always preceded by whitespace, whereas
/// the hyphen inside a `Verb-Noun` function name never has whitespace before
/// it, so this distinguishes "another flag starts here" from "just a hyphen
/// inside a name".
static FLAG_BOUNDARY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s-[A-Za-z]").unwrap());

/// Top-level (unindented) `function Verb-Noun { ... }` /
/// `function Verb-Noun([params]) { ... }` declaration. PowerShell's naming
/// convention is Verb-Noun (e.g. `Get-User`, `Set-Config`), though a bare name
/// like `function Helper { ... }` is also legal so plain identifiers are
/// accepted too. Anchored at column 0 (no leading whitespace) so that helper
/// functions nested inside another function's body aren't mistaken for
/// top-level declarations, mirroring the other regex-based extractors.
static FUNCTION_DECL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?mi)^function\s+([A-Za-z_][A-Za-z0-9_-]*)").unwrap());

/// Block comment `<# ... #>`.
static BLOCK_COMMENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)<#.*?#>").unwrap());

/// Line comment `# ...` to end of line.
static LINE_COMMENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"#[^\r\n]*").unwrap());

/// Here-string literal: expandable `@" ... "@` or literal `@' ... '@`. Per
/// PowerShell syntax, the opening `@"`/`@'` must be immediately followed by a
/// newline and the closing `"@`/`'@` must be the first two characters on its
/// own line, so that's what's required here too. Real scripts routinely embed
/// example module code, doc snippets, or a templated child-module body inside
/// a here-string (e.g. to write out a generated `.psm1`, or to build a remote
/// `Invoke-Command -ScriptBlock`). Without stripping these first, text like
/// `function Foo-Bar { }` or `Export-ModuleMember -Function Baz` that only
/// exists *inside* a string literal gets mistaken for real top-level code.
static HERE_STRING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?s)@"[ \t]*\r?\n.*?\r?\n"@|@'[ \t]*\r?\n.*?\r?\n'@"#).unwrap());

/// Strip here-strings first (so `#`/`<#`/`#>`/`Export-ModuleMember` text that
/// only exists inside a string literal's body can't be misread as real code
/// or accidentally confuse the comment stripping below), then block comments
/// (so a `#` inside a `<# ... #>` block doesn't get mistaken for the start of
/// a line comment and truncate the strip early), then line comments.
fn strip_comments(content: &str) -> String {
    let no_here_strings = HERE_STRING.replace_all(content, "");
    let no_block = BLOCK_COMMENT.replace_all(&no_here_strings, "");
    LINE_COMMENT.replace_all(&no_block, "").to_string()
}

/// Extract exported symbols from PowerShell source code.
///
/// PowerShell's real public-API mechanism for a module is an explicit
/// `Export-ModuleMember -Function Foo-Bar, Baz-Qux` statement. If one or more
/// such statements are present, the union of all named functions across them
/// is the authoritative export list (mirrors Python's `__all__`).
///
/// If no `Export-ModuleMember` statement exists (common in standalone `.ps1`
/// scripts that aren't formal modules), fall back to every top-level
/// `function Verb-Noun { ... }` declaration in source order.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = strip_comments(content);

    // Check for Export-ModuleMember first.
    let mut export_hits: Vec<(usize, String)> = Vec::new();
    for caps in EXPORT_MODULE_MEMBER.captures_iter(&stripped) {
        if let Some(list) = caps.get(1) {
            let raw = list.as_str();
            let end = FLAG_BOUNDARY
                .find(raw)
                .map(|m| m.start())
                .unwrap_or(raw.len());
            let list_str = &raw[..end];
            let base = list.start();
            for name_cap in FUNCTION_NAME_ITEM.captures_iter(list_str) {
                if let Some(name) = name_cap.get(1) {
                    export_hits.push((base + name.start(), name.as_str().to_string()));
                }
            }
        }
    }
    if !export_hits.is_empty() {
        export_hits.sort_by_key(|(pos, _)| *pos);
        let mut symbols = Vec::new();
        for (_, n) in export_hits {
            if !symbols.contains(&n) {
                symbols.push(n);
            }
        }
        return symbols;
    }

    // Fallback: every top-level function declaration, in source order.
    let mut symbols = Vec::new();
    for caps in FUNCTION_DECL.captures_iter(&stripped) {
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
    fn test_powershell_export_module_member() {
        let src = r#"
function Get-User {
    param([string]$Id)
    return "user-$Id"
}

function Set-Config {
    param([hashtable]$Settings)
}

function Get-InternalHelper {
    # not exported
}

Export-ModuleMember -Function Get-User, Set-Config
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Get-User", "Set-Config"]);
    }

    #[test]
    fn test_powershell_no_export_module_member_fallback() {
        // Standalone script with no formal module boundary: every top-level
        // function is treated as public.
        let src = r#"
function Get-Widget {
    param([string]$Name)
    return $Name
}

function Remove-Widget {
    param([string]$Name)
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Get-Widget", "Remove-Widget"]);
    }

    #[test]
    fn test_powershell_multiple_export_module_member_statements() {
        // Export-ModuleMember can appear more than once; union across all of them.
        let src = r#"
function Get-Alpha { }
function Get-Beta { }
function Get-Gamma { }

Export-ModuleMember -Function Get-Alpha
Export-ModuleMember -Function Get-Beta, Get-Gamma
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Get-Alpha", "Get-Beta", "Get-Gamma"]);
    }

    #[test]
    fn test_powershell_export_module_member_excludes_unlisted_functions() {
        // Even though Get-InternalOnly is a top-level function, once
        // Export-ModuleMember exists, only the listed names count as public.
        let src = r#"
function Get-Public {
}

function Get-InternalOnly {
}

Export-ModuleMember -Function Get-Public
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Get-Public".to_string()));
        assert!(!symbols.contains(&"Get-InternalOnly".to_string()));
    }

    #[test]
    fn test_powershell_line_comments_stripped() {
        let src = r#"
# function Get-Fake { }  -- this is commented out and must not be captured
function Get-Real {
    # inline comment
    return 1
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Get-Real"]);
    }

    #[test]
    fn test_powershell_block_comments_stripped() {
        let src = r#"
<#
.SYNOPSIS
    Example module.
.DESCRIPTION
    function Get-DocExample { }
    Export-ModuleMember -Function Get-DocExample
#>

function Get-Actual {
}

Export-ModuleMember -Function Get-Actual
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Get-Actual"]);
    }

    #[test]
    fn test_powershell_export_module_member_with_trailing_alias_flag() {
        // A single Export-ModuleMember statement can carry multiple flags on
        // one line; the -Function name list ends where -Alias begins, and a
        // hyphen inside a Verb-Noun name must not be mistaken for that
        // boundary.
        let src = r#"
function Get-User {
}

function Set-Config {
}

Export-ModuleMember -Function Get-User, Set-Config -Alias gu, sc
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Get-User", "Set-Config"]);
    }

    #[test]
    fn test_powershell_quoted_names_in_export_list() {
        // Export-ModuleMember lists sometimes quote each name.
        let src = r#"
function Get-One { }
function Get-Two { }

Export-ModuleMember -Function 'Get-One', "Get-Two"
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Get-One", "Get-Two"]);
    }

    #[test]
    fn test_powershell_function_with_advanced_params_block() {
        // Realistic advanced function with CmdletBinding/param block spanning
        // multiple lines before the opening brace.
        let src = r#"
function Get-ServiceStatus
{
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string]$ServiceName
    )

    Get-Service -Name $ServiceName
}

Export-ModuleMember -Function Get-ServiceStatus
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Get-ServiceStatus"]);
    }

    #[test]
    fn test_powershell_nested_helper_function_not_captured_in_fallback() {
        // A helper function nested (and indented) inside another function's
        // body is an implementation detail, not part of the module's public
        // surface, and must not leak out in the no-Export-ModuleMember
        // fallback path.
        let src = r#"
function Invoke-Deployment {
    function Write-Log {
        param([string]$Message)
        Write-Host $Message
    }

    Write-Log "Starting deployment"
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Invoke-Deployment"]);
        assert!(!symbols.contains(&"Write-Log".to_string()));
    }

    #[test]
    fn test_powershell_verb_noun_naming_convention() {
        let src = r#"
function Get-Item2 { }
function New-Object2 { }
function ConvertTo-Json2 { }

Export-ModuleMember -Function Get-Item2, New-Object2, ConvertTo-Json2
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Get-Item2", "New-Object2", "ConvertTo-Json2"]);
    }

    #[test]
    fn test_powershell_here_string_doc_example_does_not_fake_export_directive() {
        // Regression test: a module that documents its own usage inline via
        // a here-string (very common for comment-based help / generated docs)
        // must not have that example text mistaken for a real
        // Export-ModuleMember statement. There is no real
        // Export-ModuleMember anywhere in actual code here, so this must
        // fall back to "every top-level function", picking up BOTH
        // functions -- not just the one name mentioned inside the string.
        let src = r#"
function Get-Real {
    param([string]$Name)
    return $Name
}

function Get-AlsoReal {
    param([string]$Name)
    return $Name
}

$usageDocs = @"
Example usage:
    Import-Module MyModule
    Export-ModuleMember -Function Get-Real
"@

Write-Host $usageDocs
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Get-Real", "Get-AlsoReal"]);
    }

    #[test]
    fn test_powershell_here_string_templated_module_body_not_leaked() {
        // Regression test: a packaging/codegen script that writes out a
        // child module's source as a here-string template must not have the
        // template's fake function declarations leak into this file's own
        // export list, whether or not this file has a real
        // Export-ModuleMember of its own.
        let src = r#"
function Get-Real {
}

$moduleTemplate = @"
function Get-TemplatedFunction {
    Write-Host "generated code, not part of this file's API"
}
Export-ModuleMember -Function Get-TemplatedFunction
"@

Set-Content -Path './Generated.psm1' -Value $moduleTemplate

Export-ModuleMember -Function Get-Real
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Get-Real"]);
        assert!(!symbols.contains(&"Get-TemplatedFunction".to_string()));
    }

    #[test]
    fn test_powershell_here_string_with_literal_single_quote_delimiter() {
        // The literal (non-expandable) here-string form `@' ... '@` must be
        // stripped too, not just the expandable `@" ... "@` form.
        let src = r#"
function Get-Real {
}

$template = @'
function Get-FakeFromLiteralHereString {
}
Export-ModuleMember -Function Get-FakeFromLiteralHereString
'@
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Get-Real"]);
    }
}
