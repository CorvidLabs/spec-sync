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
/// `Get-User`, or quoted `'Get-User'` / `"Get-User"`. Also captures the `*`
/// wildcard character so glob tokens like `Get-*` or a bare `*` are captured
/// whole (rather than being truncated right before the `*`); wildcard
/// expansion into real function names happens separately in
/// `expand_wildcard`.
static FUNCTION_NAME_ITEM: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"["']?([A-Za-z0-9_*][A-Za-z0-9_*-]*)["']?"#).unwrap());

/// Marks the start of a subsequent `-Flag` (e.g. `-Alias`, `-Variable`,
/// `-Cmdlet`) on the same `Export-ModuleMember` line, which ends the
/// `-Function` name list. Flags are always preceded by whitespace, whereas
/// the hyphen inside a `Verb-Noun` function name never has whitespace before
/// it, so this distinguishes "another flag starts here" from "just a hyphen
/// inside a name".
static FLAG_BOUNDARY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s-[A-Za-z]").unwrap());

/// Top-level (unindented) `function Verb-Noun { ... }` /
/// `function Verb-Noun([params]) { ... }` declaration, or the two other
/// PowerShell keywords that declare a function-like, callable unit: `filter`
/// (a function implicitly wrapped for pipeline input, e.g.
/// `filter Get-EvenNumber { ... }`) and `workflow` (a PowerShell Workflow,
/// e.g. `workflow Invoke-Deployment { ... }`). PowerShell's naming
/// convention is Verb-Noun (e.g. `Get-User`, `Set-Config`), though a bare name
/// like `function Helper { ... }` is also legal so plain identifiers are
/// accepted too. Anchored at column 0 (no leading whitespace) so that helper
/// functions/filters/workflows nested inside another function's body aren't
/// mistaken for top-level declarations, mirroring the other regex-based
/// extractors.
static FUNCTION_DECL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?mi)^(?:function|filter|workflow)\s+([A-Za-z_][A-Za-z0-9_-]*)").unwrap()
});

/// Backtick (`` ` ``) immediately followed by a newline is PowerShell's
/// line-continuation operator: the following line is logically a
/// continuation of the current one (most commonly seen splitting a long
/// `-Function` list across lines). Applied as a preprocessing step before
/// any other regex runs, since every other regex here is line-anchored
/// (`^`/`[^\r\n]`) and would otherwise stop dead at the embedded newline,
/// silently truncating whatever statement was split across the continued
/// lines.
static LINE_CONTINUATION: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"`[ \t]*\r?\n").unwrap());

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

/// Joins backtick-continued lines into a single logical line by replacing
/// each trailing-backtick-then-newline with a single space. A space (rather
/// than deleting the continuation outright) keeps whatever token preceded
/// the backtick from being glued onto whatever token starts the next line.
/// Must run before every other regex in this module, all of which are
/// line-anchored and would otherwise treat a continued statement as if it
/// ended at the backtick.
fn join_line_continuations(content: &str) -> String {
    LINE_CONTINUATION.replace_all(content, " ").to_string()
}

/// Converts a PowerShell wildcard token such as `Get-*`, `*-Widget`, or a
/// bare `*` into an anchored, case-insensitive `Regex` (PowerShell's
/// wildcard matching, like the rest of the language, is case-insensitive).
/// Only `*` (match any run of characters, including none) is supported,
/// which covers the overwhelming majority of real-world `-Function`
/// wildcard usage; any other characters in the token are matched literally.
fn wildcard_to_regex(pattern: &str) -> Regex {
    let mut regex_str = String::from("(?i)^");
    let segments: Vec<&str> = pattern.split('*').collect();
    for (i, segment) in segments.iter().enumerate() {
        regex_str.push_str(&regex::escape(segment));
        if i + 1 != segments.len() {
            regex_str.push_str(".*");
        }
    }
    regex_str.push('$');
    // Built entirely from a fixed prefix/suffix and `regex::escape`d
    // segments, so this can never fail to compile.
    Regex::new(&regex_str).expect("wildcard_to_regex always builds a valid pattern")
}

/// Expands a wildcard `-Function` token (e.g. `Get-*`, or a bare `*`) into
/// the subset of `candidates` (the file's actual top-level function-like
/// names, in source order) that it matches. A literal token with no `*` in
/// it is not a wildcard at all, so callers should only invoke this once
/// they've confirmed the token actually contains `*`.
fn expand_wildcard(pattern: &str, candidates: &[String]) -> Vec<String> {
    let re = wildcard_to_regex(pattern);
    candidates
        .iter()
        .filter(|name| re.is_match(name))
        .cloned()
        .collect()
}

/// Every top-level (column-zero) `function`/`filter`/`workflow` declaration
/// in `stripped` source, in source order, deduplicated. Shared by the
/// no-`Export-ModuleMember` fallback path and by wildcard expansion (which
/// needs to know the real function names a glob like `Get-*` could match).
fn top_level_function_names(stripped: &str) -> Vec<String> {
    let mut names = Vec::new();
    for caps in FUNCTION_DECL.captures_iter(stripped) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str().to_string();
            if !names.contains(&n) {
                names.push(n);
            }
        }
    }
    names
}

/// Extract exported symbols from PowerShell source code.
///
/// PowerShell's real public-API mechanism for a module is an explicit
/// `Export-ModuleMember -Function Foo-Bar, Baz-Qux` statement. If one or more
/// such statements are present, the union of all named functions across them
/// is the authoritative export list (mirrors Python's `__all__`). Entries may
/// use `*` wildcards (e.g. `Get-*`, or a bare `*`), which are expanded
/// against the file's real top-level function names. A trailing backtick at
/// the end of a line continues the statement onto the next line, so a
/// `-Function` list may be split across multiple lines.
///
/// If no `Export-ModuleMember` statement exists (common in standalone `.ps1`
/// scripts that aren't formal modules), fall back to every top-level
/// `function`/`filter`/`workflow` declaration in source order.
pub fn extract_exports(content: &str) -> Vec<String> {
    let joined = join_line_continuations(content);
    let stripped = strip_comments(&joined);
    let top_level_names = top_level_function_names(&stripped);

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
                    let token = name.as_str();
                    if token.contains('*') {
                        // Glob token (e.g. `Get-*`, or a bare `*`): expand
                        // against the file's real top-level function names
                        // rather than emitting the literal glob text.
                        for matched in expand_wildcard(token, &top_level_names) {
                            export_hits.push((base + name.start(), matched));
                        }
                    } else {
                        export_hits.push((base + name.start(), token.to_string()));
                    }
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

    // Fallback: every top-level function/filter/workflow declaration, in
    // source order.
    top_level_names
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

    #[test]
    fn test_powershell_backtick_line_continuation_in_export_list() {
        // Regression test for bug #1: a `-Function` list split across lines
        // via a trailing backtick line-continuation must be treated as one
        // logical line. Before the fix, every regex here was line-anchored
        // and stopped dead at the embedded newline, so this returned only
        // ["Foo"] -- silently dropping "Bar" (and anything else past the
        // continuation) from the export list.
        let src =
            "function Foo {}\nfunction Bar {}\n\nExport-ModuleMember -Function Foo, `\n    Bar\n";
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Foo", "Bar"]);
    }

    #[test]
    fn test_powershell_backtick_line_continuation_spans_three_lines() {
        // Extends the line-continuation regression to a chain of two
        // continuations, confirming the join isn't a one-shot fix that only
        // handles a single wrapped line.
        let src = "function Foo {}\nfunction Bar {}\nfunction Baz {}\n\nExport-ModuleMember -Function Foo, `\n    Bar, `\n    Baz\n";
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Foo", "Bar", "Baz"]);
    }

    #[test]
    fn test_powershell_wildcard_function_export() {
        // Regression test for bug #2: `-Function Get-*` must expand to the
        // real matching top-level function names. Before the fix, the name
        // regex stopped right before `*`, producing a single bogus literal
        // entry ["Get-"] instead of the actual matching functions, and
        // Set-Baz (which doesn't match the `Get-*` glob) must still be
        // excluded.
        let src = r#"
function Get-Foo { }
function Get-Bar { }
function Set-Baz { }

Export-ModuleMember -Function Get-*
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Get-Foo", "Get-Bar"]);
        assert!(!symbols.contains(&"Get-".to_string()));
        assert!(!symbols.contains(&"Set-Baz".to_string()));
    }

    #[test]
    fn test_powershell_wildcard_mixed_with_literal_name_in_same_list() {
        // A single -Function list combining a literal name and a wildcard
        // token together, exercising both code paths (literal passthrough
        // and expand_wildcard) within one FUNCTION_NAME_ITEM scan.
        let src = r#"
function Get-Foo { }
function Get-Bar { }
function Set-Baz { }
function Remove-Qux { }

Export-ModuleMember -Function Set-Baz, Get-*
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Set-Baz", "Get-Foo", "Get-Bar"]);
        assert!(!symbols.contains(&"Remove-Qux".to_string()));
    }

    #[test]
    fn test_powershell_bare_wildcard_function_export() {
        // Bare `-Function *` (no prefix/suffix) must export every top-level
        // function. This previously "worked" already, but only by accident:
        // the old FUNCTION_NAME_ITEM regex couldn't match a standalone `*`
        // at all, so export_hits stayed empty and the code fell all the way
        // through to the unrelated no-Export-ModuleMember fallback path,
        // which happened to produce the same-looking answer. Now it is
        // handled directly through the same wildcard-expansion path used
        // for `Get-*`, so this pins the (unchanged) observable behavior
        // against a real, intentional code path rather than a coincidence.
        let src = r#"
function Get-Foo { }
function Set-Baz { }

Export-ModuleMember -Function *
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Get-Foo", "Set-Baz"]);
    }

    #[test]
    fn test_powershell_filter_and_workflow_fallback() {
        // Regression test for bug #3: `filter` and `workflow` are
        // PowerShell's other function-like declaration keywords (besides
        // `function`) and must be picked up by the no-Export-ModuleMember
        // fallback path. Before the fix, FUNCTION_DECL only matched
        // `function`, so a script using solely `filter`/`workflow` exported
        // nothing at all.
        let src = r#"
filter Get-Filtered {
}

workflow Invoke-Workflow {
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Get-Filtered", "Invoke-Workflow"]);
    }

    #[test]
    fn test_powershell_nested_filter_and_workflow_excluded() {
        // Mirrors test_powershell_nested_helper_function_not_captured_in_fallback,
        // but for the newly recognized filter/workflow keywords: a filter
        // and a workflow nested (indented) inside another function's body
        // are implementation details and must stay excluded from the
        // column-zero-anchored fallback, exactly like a nested `function`
        // already was.
        let src = r#"
function Invoke-Deployment {
    filter Get-NestedFilter {
        param($Item)
        $Item
    }

    workflow Invoke-NestedWorkflow {
    }

    Get-NestedFilter "value"
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Invoke-Deployment"]);
        assert!(!symbols.contains(&"Get-NestedFilter".to_string()));
        assert!(!symbols.contains(&"Invoke-NestedWorkflow".to_string()));
    }

    #[test]
    fn test_powershell_filter_export_module_member_explicit_list() {
        // A `filter` declaration named explicitly (non-wildcard) in an
        // Export-ModuleMember -Function list must be exported like any
        // other function-like name, confirming filter/workflow support
        // isn't limited to the fallback path.
        let src = r#"
filter Get-Evens {
    param($Item)
    if ($Item % 2 -eq 0) { $Item }
}

filter Get-Odds {
    param($Item)
    if ($Item % 2 -ne 0) { $Item }
}

Export-ModuleMember -Function Get-Evens
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Get-Evens"]);
        assert!(!symbols.contains(&"Get-Odds".to_string()));
    }
}
