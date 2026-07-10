use regex::Regex;
use std::sync::LazyLock;

/// Double-quoted R string literals, including ones that span multiple lines
/// (e.g. a long SQL query or codegen template assigned to a constant).
/// Stripped before matching so that assignment-looking text embedded inside
/// string content isn't mistaken for a real top-level declaration.
///
/// Deliberately does NOT also strip single-quoted strings: roxygen doc
/// comments (`#'`) use a literal apostrophe as their prefix, and English
/// prose in those comments is full of unbalanced apostrophes ("isn't",
/// "don't", "it's"). Without look-behind support, a single-quote string
/// regex applied to the whole file cannot tell a comment's apostrophe from
/// a real string delimiter, and will happily "close" a string many lines
/// later inside another `#'` comment — silently eating roxygen blocks
/// (including their `@export` tags) in between. Double-quoted strings don't
/// have this problem in practice, since stray unescaped double quotes are
/// rare in prose.
static STRING_LITERAL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?s)"(?:[^"\\]|\\.)*""#).unwrap());

/// Blank R 4.0+ raw strings — `r"(...)"`, `R"[...]"`, `r"{...}"`, or with
/// any number of `-` between the quote and the bracket (`r"---(...)---"`,
/// used when the content itself contains the plain closing sequence) —
/// *before* [`STRING_LITERAL`] runs.
///
/// This must run first because `STRING_LITERAL` has no concept of the raw
/// string's real delimiter (`)"`/`]"`/`}"`, optionally with dashes); it just
/// toggles on bare `"` characters. A raw string that happens to contain an
/// *odd* number of embedded, unescaped `"` characters — completely legal
/// inside a raw string, since that's the whole point of the feature (e.g.
/// `r"(She said "hello")"`)- desyncs `STRING_LITERAL`'s naive quote-pairing:
/// consecutive quotes get paired left-to-right regardless of which raw
/// string they actually belong to, and the *gap* between two such pairs can
/// land on a real line of source, leaking it back out unstripped as if it
/// were top-level code. Recognizing the true raw-string delimiter up front
/// and blanking the whole literal (including its embedded quotes) prevents
/// `STRING_LITERAL` from ever seeing those quotes at all.
///
/// The `r`/`R` prefix immediately adjacent to a `"`/`'` with no operator
/// between is unambiguous in valid R syntax — juxtaposing an identifier and
/// a string literal with nothing between them is otherwise a syntax error —
/// so this is checked defensively (requiring the preceding character, if
/// any, not be part of a longer identifier) rather than needing lookaround.
fn strip_raw_strings(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < n {
        let is_prefix_letter = chars[i] == 'r' || chars[i] == 'R';
        let prev_is_ident =
            i > 0 && (chars[i - 1].is_alphanumeric() || chars[i - 1] == '_' || chars[i - 1] == '.');
        if is_prefix_letter
            && !prev_is_ident
            && i + 1 < n
            && (chars[i + 1] == '"' || chars[i + 1] == '\'')
        {
            let quote = chars[i + 1];
            let mut j = i + 2;
            let dash_start = j;
            while j < n && chars[j] == '-' {
                j += 1;
            }
            let dash_count = j - dash_start;
            if j < n && matches!(chars[j], '(' | '[' | '{') {
                let close_bracket = match chars[j] {
                    '(' => ')',
                    '[' => ']',
                    '{' => '}',
                    _ => unreachable!(),
                };
                let mut k = j + 1;
                let mut end = None;
                while k < n {
                    if chars[k] == close_bracket {
                        let dashes_ok = (0..dash_count).all(|d| chars.get(k + 1 + d) == Some(&'-'));
                        if dashes_ok && chars.get(k + 1 + dash_count) == Some(&quote) {
                            end = Some(k + 1 + dash_count + 1);
                            break;
                        }
                    }
                    k += 1;
                }
                let end = end.unwrap_or(n);
                for &ch in &chars[i..end] {
                    out.push(if ch == '\n' { '\n' } else { ' ' });
                }
                i = end;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// A single roxygen2 comment line whose content is (only) the `@export`
/// tag — i.e. `@export` is the very first thing after `#'` and optional
/// leading whitespace, not merely a substring appearing later in a
/// descriptive sentence. Real roxygen2 tags always begin the comment line;
/// `@export` itself takes no argument, so nothing meaningful legitimately
/// follows it on the same line. Anchoring this way avoids a false positive
/// on prose that happens to mention `@export` in passing, e.g. `#' See
/// ?other_export for docs on @export usage elsewhere.` — that line is not a
/// real export tag and must not flip the file into "tag mode" (nor count
/// toward a specific function's tagged block below).
/// Anchored at column 0: roxygen2 only ever attaches doc blocks to
/// top-level objects, so an indented `#'`-prefixed comment (e.g. sitting
/// above a nested helper inside another function's body) carries no real
/// `@export` meaning and must not flip the whole file into tag mode.
static EXPORT_TAG_LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^#'\s*@export\b").unwrap());

/// A contiguous roxygen `#'` comment block (captured in group 1, so its text
/// can be checked for `@export`) immediately followed by a top-level
/// `name <- function(...)`, `name <- \(...)` (R 4.1+ lambda shorthand), or
/// `name = function(...)` assignment. The name (group 2) may be a bare
/// identifier (dots allowed, e.g. S3 methods like `print.myclass`) or a
/// backtick-quoted operator like `` `%+%` ``.
///
/// Both the comment lines and the declaration itself are anchored at column
/// 0 (no leading whitespace permitted) so that an indented comment block
/// followed by an indented assignment — e.g. a nested helper function
/// defined inside another function's body that happens to carry its own
/// `#'`-style comment — is never mistaken for a top-level declaration.
/// roxygen2 itself only ever processes doc blocks that precede top-level
/// objects, so this mirrors real semantics, not just a style preference.
static ROXYGEN_TAGGED_DECL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^((?:#'[^\n]*\n)+)(`[^`]+`|[.\w]+)\s*(?:<-|=)\s*(?:function\s*\(|\\\()")
        .unwrap()
});

/// Same idea as `ROXYGEN_TAGGED_DECL`, but for R's S4 object system, which declares a
/// generic/class/method by *calling* `setGeneric`/`setClass`/`setRefClass` with the
/// name as its first (quoted) argument, rather than via a `name <- function(...)`
/// assignment. This is a common, idiomatic real-world pattern (the Bioconductor
/// ecosystem in particular is heavily S4-based) that the assignment-only pattern above
/// doesn't fit at all -- an S4 generic with an explicit `#' @export` tag was previously
/// silently dropped outright.
static ROXYGEN_TAGGED_S4_DECL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?m)^((?:#'[^\n]*\n)+)(?:setGeneric|setClass|setRefClass)\s*\(\s*["']([.\w]+)["']"#,
    )
    .unwrap()
});

/// Fallback (no `@export` tags anywhere in the file): top-level function
/// assignment via `<-` or `=`, to either `function(...)` or the `\(...)`
/// lambda shorthand. Anchored at the start of a line so indented (nested)
/// assignments and commented-out (`#`-prefixed) lines are never matched.
static TOP_LEVEL_FUNCTION_DECL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^(`[^`]+`|[.\w]+)\s*(?:<-|=)\s*(?:function\s*\(|\\\()").unwrap()
});

/// Extract exported symbols from R source code.
///
/// R has no visibility keywords. The real-world public API surface of a
/// documented package is determined by roxygen2 `#' @export` tags placed
/// directly above the function they apply to; that is the strongest signal
/// and takes precedence whenever any `@export` tag appears in the file. For
/// a plain script with no roxygen documentation at all, fall back to R's
/// naming convention: top-level function assignments are public unless
/// their name starts with `.`, which conventionally marks them internal.
pub fn extract_exports(content: &str) -> Vec<String> {
    let raw_stripped = strip_raw_strings(content);
    // The S4 declaration forms (`setGeneric("name", ...)` etc.) need their quoted
    // name *argument* to survive, so they're matched against `raw_stripped` --
    // before the blanket string-literal stripping below, which exists to protect
    // the *other* patterns (a `name <- function(...)` assignment never has its own
    // name inside a string) from being confused by unrelated string content
    // elsewhere in the file, but would just as surely destroy the one piece of
    // text the S4 patterns actually need to read.
    let stripped = STRING_LITERAL.replace_all(&raw_stripped, "");

    if EXPORT_TAG_LINE.is_match(&stripped) {
        let mut symbols: Vec<String> = Vec::new();
        for caps in ROXYGEN_TAGGED_DECL.captures_iter(&stripped) {
            let block = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            if !EXPORT_TAG_LINE.is_match(block) {
                continue;
            }
            if let Some(name) = caps.get(2) {
                let n = name.as_str().trim_matches('`').to_string();
                if !symbols.contains(&n) {
                    symbols.push(n);
                }
            }
        }
        for caps in ROXYGEN_TAGGED_S4_DECL.captures_iter(&raw_stripped) {
            let block = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            if !EXPORT_TAG_LINE.is_match(block) {
                continue;
            }
            if let Some(name) = caps.get(2) {
                let n = name.as_str().to_string();
                if !symbols.contains(&n) {
                    symbols.push(n);
                }
            }
        }
        return symbols;
    }

    // Fallback: top-level function assignments whose name doesn't start
    // with `.` (R's internal/private naming convention).
    let mut symbols: Vec<String> = Vec::new();
    for caps in TOP_LEVEL_FUNCTION_DECL.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str().trim_matches('`');
            if n.starts_with('.') {
                continue;
            }
            let n = n.to_string();
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
    fn test_r_export_tag_basic() {
        let src = r#"
#' Add two numbers
#'
#' @param x A number
#' @param y A number
#' @export
add_numbers <- function(x, y) {
  x + y
}

#' Internal helper, not exported
helper <- function(x) {
  x * 2
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["add_numbers"]);
    }

    #[test]
    fn test_r_export_precedence_over_dot_convention() {
        // Once @export tags exist anywhere in the file, roxygen tags are
        // authoritative — even a dot-prefixed name is exported if tagged,
        // just as Python's __all__ can list an underscore-prefixed name.
        let src = r#"
#' @export
.weirdly_named <- function() {
  TRUE
}

#' Not tagged, so not exported even though it looks public
also_public_looking <- function() {
  FALSE
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec![".weirdly_named"]);
    }

    #[test]
    fn test_r_s4_setgeneric_with_export_tag_captured() {
        // Regression test: R's S4 object system declares a generic/class/method by
        // *calling* `setGeneric`/`setClass`/`setRefClass` with the name as its first
        // quoted argument, not via a `name <- function(...)` assignment. This is a
        // common real-world pattern (heavily used in Bioconductor-style packages) that
        // was previously silently dropped entirely, even with an explicit `@export` tag.
        let src = r#"
#' My generic
#' @export
setGeneric("myMethod", function(x) standardGeneric("myMethod"))

#' My class
#' @export
setClass("MyClass", representation(x = "numeric"))

#' Not tagged
setGeneric("internalOnly", function(x) standardGeneric("internalOnly"))

#' Plain function
#' @export
myFunc <- function(x) x
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"myMethod".to_string()));
        assert!(symbols.contains(&"MyClass".to_string()));
        assert!(symbols.contains(&"myFunc".to_string()));
        assert!(!symbols.contains(&"internalOnly".to_string()));
    }

    #[test]
    fn test_r_s4_setgeneric_without_export_tag_not_exported_by_convention() {
        // Unlike a plain `name <- function(...)` assignment, an S4 declaration with
        // no `@export` tag at all (or only an unrelated tag like `@exportClass`) is
        // NOT exported by naming convention -- established behavior (see
        // `test_export_tag_variant_like_export_class_is_not_treated_as_export` above),
        // matching real-world R practice where S4 class/generic registrations are
        // often internal implementation details even when undocumented.
        let src = r#"
setGeneric("publicLooking", function(x) standardGeneric("publicLooking"))
regularFunc <- function(x) x
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["regularFunc".to_string()]);
    }

    #[test]
    fn test_r_no_export_tags_naming_convention() {
        // No @export tags anywhere -> fall back to the naming convention:
        // leading dot means internal/private.
        let src = r#"
connect <- function(host, port) {
  TRUE
}

.internal_helper <- function() {
  42
}

fetch_data = function(id) {
  NULL
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["connect", "fetch_data"]);
        assert!(!symbols.contains(&".internal_helper".to_string()));
    }

    #[test]
    fn test_r_comment_stripping_safety() {
        // Commented-out declarations (regular # comments, not roxygen) must
        // never be mistaken for real top-level functions.
        let src = r#"
# old_api <- function(x) {
#   x
# }

real_api <- function(x) {
  x + 1
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["real_api"]);
    }

    #[test]
    fn test_r_string_literal_pseudocode_not_captured() {
        // A long string constant (e.g. an embedded SQL query, common in real
        // R packages that wrap database access) can contain lines that look
        // exactly like top-level assignments and must not leak out.
        let src = r#"
sql_query <- "
SELECT *
FROM foo
bar = 5
"

real_func <- function() {
  TRUE
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["real_func"]);
        assert!(!symbols.contains(&"bar".to_string()));
    }

    #[test]
    fn test_r_lambda_shorthand_syntax() {
        // R 4.1+ backslash lambda shorthand, no roxygen anywhere.
        let src = r#"
square <- \(x) x^2

.private_square <- \(x) x^2
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["square"]);
    }

    #[test]
    fn test_r_s3_method_dot_in_name_not_excluded() {
        // S3 dispatch methods conventionally contain a dot (print.myclass)
        // but do NOT start with one, so they must still be treated as
        // public — only a *leading* dot means internal.
        let src = r#"
print.myclass <- function(x, ...) {
  cat("myclass\n")
}

summary.myclass <- function(object, ...) {
  invisible(object)
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["print.myclass", "summary.myclass"]);
    }

    #[test]
    fn test_r_backtick_operator_export() {
        // Custom infix operators are exported like any other function and
        // must have their backticks stripped from the reported name.
        let src = r#"
#' Custom infix operator
#' @export
`%+%` <- function(a, b) {
  paste0(a, b)
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["%+%"]);
    }

    #[test]
    fn test_r_nested_function_not_captured() {
        // Only top-level (zero-indentation) assignments are public API;
        // functions defined inside another function's body are implementation
        // detail and must not leak out even in the no-@export fallback.
        let src = r#"
process <- function(items) {
  transform <- function(x) {
    x * 2
  }
  lapply(items, transform)
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["process"]);
        assert!(!symbols.contains(&"transform".to_string()));
    }

    #[test]
    fn test_r_adversarial_nested_roxygen_not_exported() {
        // Adversarial probe (not a claim this is idiomatic R): a nested
        // helper inside a top-level exported function, itself preceded by
        // an indented comment that happens to look like a roxygen
        // `@export` tag. roxygen2 only ever processes top-level blocks, so
        // this indented "tag" has no real meaning and must not cause the
        // nested helper to be reported as a public export.
        let src = r#"
#' @export
wrapper <- function() {
  #' @export
  inner_helper <- function(x) {
    x + 1
  }
  inner_helper(5)
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["wrapper"]);
        assert!(!symbols.contains(&"inner_helper".to_string()));
    }

    #[test]
    fn test_r_adversarial_indented_fake_export_does_not_suppress_fallback() {
        // A nested helper carries an indented `#'`-style comment that merely
        // *looks* like an `@export` tag. Since roxygen2 never attaches doc
        // blocks to nested (non-top-level) objects, this must not count as
        // a real @export tag anywhere in the file — so the file should
        // still fall back to the naming-convention rule, and the genuine
        // top-level `wrapper` function must still be reported.
        let src = r#"
wrapper <- function() {
  #' @export
  inner_helper <- function(x) {
    x + 1
  }
  inner_helper(5)
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["wrapper"]);
        assert!(!symbols.contains(&"inner_helper".to_string()));
    }

    #[test]
    fn test_export_mentioned_in_prose_is_not_a_real_tag() {
        // Regression: `HAS_EXPORT_TAG` previously matched `@export` as a
        // substring anywhere on a `#'` line, so a doc line merely
        // *mentioning* `@export` in a sentence (not as a standalone tag)
        // incorrectly flipped the whole file into "tag mode", where a real
        // roxygen2 tag is authoritative over the naming convention. Using a
        // dot-prefixed (normally-private) name makes the bug observable:
        // under the old behavior tag-mode treated the block as tagged
        // (block text contains the substring "@export") and reported
        // `.not_tagged` as exported anyway, ignoring the leading-dot
        // convention entirely. With the fix, this file has no real tag
        // anywhere, so it correctly falls back to the naming convention and
        // excludes the dot-prefixed name.
        let src = r#"
#' Do the thing. See ?other_export for docs on @export usage elsewhere.
.not_tagged <- function() {
  TRUE
}
"#;
        let symbols = extract_exports(src);
        assert!(
            !symbols.contains(&".not_tagged".to_string()),
            "prose mention of @export wrongly treated as a real tag, got {symbols:?}"
        );
    }

    #[test]
    fn test_raw_string_with_embedded_quote_does_not_break_parsing() {
        // Verified-correct existing behavior in this shape (kept as a
        // regression guard): an R 4.0+ raw string containing an escaped-free
        // embedded quote pair.
        let src = r#"
msg <- r"(She said "hello" to me)"

real_func <- function() {
  TRUE
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["real_func"]);
    }

    #[test]
    fn test_raw_string_odd_quote_count_no_longer_leaks_fake_decl() {
        // Regression: a raw string whose embedded, unescaped quotes desync
        // the naive `STRING_LITERAL` quote-pairing (consecutive quotes
        // paired left-to-right, ignoring which raw string they belong to)
        // could leave a real-looking declaration in the *gap* between two
        // such pairs unstripped -- and that gap could start a fresh line,
        // making it look like genuine top-level code. Before
        // `strip_raw_strings` existed this produced
        // `["fake_leak", "real_func"]`.
        let src = "msg <- r\"(a \"\nfake_leak <- function(x) x\n\" b)\"\n\nreal_func <- function() {\n  TRUE\n}\n";
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["real_func"]);
        assert!(
            !symbols.contains(&"fake_leak".to_string()),
            "raw string content leaked as a fake top-level declaration, got {symbols:?}"
        );
    }

    #[test]
    fn test_export_tag_variant_like_export_class_is_not_treated_as_export() {
        // Verified-correct existing behavior (the `\b` word boundary after
        // `@export` already handled this correctly, both before and after
        // the anchoring fix above): `@exportClass`/`@exportMethod` are
        // distinct roxygen2 directives, not the plain `@export` tag, so
        // they must not flip the file into tag mode. With no real `@export`
        // tag anywhere, this file falls back to the naming convention, and
        // the undocumented (non-dot) function is correctly reported.
        let src = r#"
#' @exportClass MyClass
setClass("MyClass", representation(x = "numeric"))

undocumented <- function() {
  TRUE
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["undocumented"], "got {symbols:?}");
    }

    #[test]
    fn test_r_mixed_export_and_undocumented_functions() {
        // Realistic package file: some functions roxygen-documented and
        // exported, one roxygen-documented but explicitly internal (no
        // @export), and one with no documentation at all. Once any @export
        // tag exists in the file, only tagged functions count.
        let src = r#"
#' Fetch a record by id
#'
#' @param id character
#' @export
fetch_record <- function(id) {
  NULL
}

#' Validate configuration
#' @export
validate_config <- function(config) {
  TRUE
}

#' Internal cache lookup
#' @keywords internal
cache_lookup <- function(key) {
  NULL
}

undocumented_helper <- function() {
  NA
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["fetch_record", "validate_config"]);
    }
}
