use super::*;

#[test]
fn test_parse_toml_string_quoted() {
    assert_eq!(parse_toml_string("\"hello\""), "hello");
}

#[test]
fn test_parse_toml_string_unquoted() {
    assert_eq!(parse_toml_string("bare_value"), "bare_value");
}

#[test]
fn test_parse_toml_string_empty_quotes() {
    assert_eq!(parse_toml_string("\"\""), "");
}

#[test]
fn test_parse_toml_string_with_whitespace() {
    assert_eq!(parse_toml_string("  \"trimmed\"  "), "trimmed");
}

#[test]
fn test_parse_toml_string_array_basic() {
    let result = parse_toml_string_array("[\"a\", \"b\", \"c\"]");
    assert_eq!(result, vec!["a", "b", "c"]);
}

#[test]
fn test_parse_toml_string_array_single() {
    let result = parse_toml_string_array("[\"only\"]");
    assert_eq!(result, vec!["only"]);
}

#[test]
fn test_parse_toml_string_array_empty() {
    let result = parse_toml_string_array("[]");
    assert!(result.is_empty());
}

#[test]
fn test_parse_toml_string_array_bare_value() {
    // When no brackets, treats as single-element array
    let result = parse_toml_string_array("\"single\"");
    assert_eq!(result, vec!["single"]);
}

#[test]
fn test_parse_toml_string_array_trailing_comment() {
    // A trailing comment after the closing bracket must not corrupt the array.
    let result = parse_toml_string_array("[\"a\", \"b\"]  # note");
    assert_eq!(result, vec!["a", "b"]);
}

#[test]
fn test_parse_toml_string_array_glob_with_brackets() {
    // A quoted glob containing `]` must survive (the close scan skips the
    // in-string bracket and stops at the array's real close).
    let result = parse_toml_string_array("[\"**/[abc]/**\", \"src\"]");
    assert_eq!(result, vec!["**/[abc]/**", "src"]);
}

#[test]
fn test_parse_toml_string_single_quoted_literal() {
    // TOML literal (single-quoted) strings are taken verbatim.
    assert_eq!(parse_toml_string("'hello'"), "hello");
    assert_eq!(parse_toml_string("  'trimmed'  "), "trimmed");
    assert_eq!(parse_toml_string("''"), "");
}

#[test]
fn test_parse_toml_string_literal_preserves_backslashes() {
    // A literal string performs NO escape processing (unlike a basic string),
    // so backslashes in a Windows path or regex survive intact.
    assert_eq!(parse_toml_string("'C:\\temp\\new'"), "C:\\temp\\new");
    assert_eq!(parse_toml_string("'\\d+'"), "\\d+");
}

#[test]
fn test_parse_toml_string_array_single_quoted() {
    assert_eq!(
        parse_toml_string_array("['a', 'b', 'c']"),
        vec!["a", "b", "c"]
    );
}

#[test]
fn test_parse_toml_string_array_mixed_quotes() {
    // Basic and literal strings can be mixed within one array.
    assert_eq!(
        parse_toml_string_array("[\"src\", 'gen']"),
        vec!["src", "gen"]
    );
}

#[test]
fn test_parse_toml_string_array_literal_glob_with_brackets() {
    // A literal-quoted glob containing `]` must not close the array early.
    assert_eq!(
        parse_toml_string_array("['**/[abc]/**', 'src']"),
        vec!["**/[abc]/**", "src"]
    );
}

#[test]
fn test_parse_toml_string_array_literal_comma_and_bracket_intact() {
    // A `,` or `]` inside a literal element must not split or truncate the array.
    assert_eq!(
        parse_toml_string_array("['a]b', 'c,d']"),
        vec!["a]b", "c,d"]
    );
}

#[test]
fn test_strip_inline_comment_hash_inside_literal() {
    // A `#` inside a string (basic or literal) is content; only an out-of-string
    // `#` starts a comment.
    assert_eq!(strip_inline_comment("'a#b'"), "'a#b'");
    assert_eq!(strip_inline_comment("'a#b' # real comment"), "'a#b'");
    assert_eq!(strip_inline_comment("\"a#b\" # real comment"), "\"a#b\"");
}

#[test]
fn test_find_toml_array_close_quote_aware() {
    // Close is the bracket that balances the first `[`, ignoring quoted `]`.
    assert_eq!(find_toml_array_close("[\"a\"]"), Some(4));
    assert_eq!(find_toml_array_close("[\"a]b\", \"c\"]"), Some(11));
    assert_eq!(find_toml_array_close("[\"a\","), None); // unterminated
    assert_eq!(find_toml_array_close("[\"a\\\"]\"]"), Some(7)); // escaped quote
    assert_eq!(find_toml_array_close("no brackets"), None);
}

#[test]
fn test_parse_toml_string_array_escaped_quote() {
    // An escaped quote inside a basic string must not toggle string state and
    // false-detect the closing bracket (2 items parsed), and the `\"` decodes
    // to a literal `"` in the value (see `toml_unescape`).
    assert_eq!(
        parse_toml_string_array("[\"a\\\"]b\", \"c\"]"),
        vec!["a\"]b", "c"]
    );
}

#[test]
fn test_parse_toml_bool_true_variants() {
    assert!(parse_toml_bool("true"));
    assert!(parse_toml_bool("yes"));
    assert!(parse_toml_bool("1"));
    assert!(parse_toml_bool("  true  "));
}

#[test]
fn test_parse_toml_bool_false_variants() {
    assert!(!parse_toml_bool("false"));
    assert!(!parse_toml_bool("no"));
    assert!(!parse_toml_bool("0"));
    assert!(!parse_toml_bool("anything_else"));
}

#[test]
fn test_strip_inline_comment_preserves_hash_in_quotes() {
    assert_eq!(
        strip_inline_comment(r#""has # inside""#),
        r#""has # inside""#
    );
    assert_eq!(strip_inline_comment(r#""ollama" # comment"#), r#""ollama""#);
    assert_eq!(strip_inline_comment("120 # seconds"), "120");
    assert_eq!(strip_inline_comment("plain_value"), "plain_value");
}

#[test]
fn test_toml_unescape_inverts_toml_escape() {
    for original in [
        "plain",
        "back\\slash",
        "quote\"inside",
        "tab\tand\nnewline",
        "windows\\path\\file.ts",
        "regex \\s+ (\\w+)",
    ] {
        assert_eq!(
            toml_unescape(&toml_escape(original)),
            original,
            "escape→unescape must round-trip: {original:?}"
        );
    }
    // An unrecognized escape is preserved byte-for-byte (a hand-written config
    // with a stray backslash must not be mangled).
    assert_eq!(toml_unescape("C:\\Users"), "C:\\Users");
}

#[test]
fn test_split_toml_array_items_is_quote_aware() {
    // A comma inside a quoted item must NOT split it.
    let items = split_toml_array_items(r#""a, b", "c""#);
    assert_eq!(items.len(), 2);
    assert_eq!(parse_toml_string(items[0].trim()), "a, b");
    assert_eq!(parse_toml_string(items[1].trim()), "c");
}

#[test]
fn test_strip_inline_comment_is_escape_aware() {
    // An escaped quote must not toggle string state and let a later `#`
    // truncate the value.
    assert_eq!(strip_inline_comment(r#""a\"b#c""#), r#""a\"b#c""#);
    // A real comment after a closed string is still stripped.
    assert_eq!(strip_inline_comment(r#""ollama" # local"#), r#""ollama""#);
}
