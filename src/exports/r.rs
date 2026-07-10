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

/// Any roxygen2 `#'` doc comment anywhere in the file containing an
/// `@export` tag. Presence of even one of these means the file is a
/// documented package and roxygen tags — not the naming convention — are
/// the authoritative signal for public API (mirrors Python's `__all__`
/// precedence rule).
///
/// Anchored at column 0: roxygen2 only ever attaches doc blocks to
/// top-level objects, so an indented `#'`-prefixed comment (e.g. sitting
/// above a nested helper inside another function's body) carries no real
/// `@export` meaning and must not flip the whole file into tag mode.
static HAS_EXPORT_TAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^#'.*@export\b").unwrap());

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
    let stripped = STRING_LITERAL.replace_all(content, "");

    if HAS_EXPORT_TAG.is_match(&stripped) {
        let mut symbols: Vec<String> = Vec::new();
        for caps in ROXYGEN_TAGGED_DECL.captures_iter(&stripped) {
            let block = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            if !block.contains("@export") {
                continue;
            }
            if let Some(name) = caps.get(2) {
                let n = name.as_str().trim_matches('`').to_string();
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
