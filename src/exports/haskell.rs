use regex::Regex;
use std::sync::LazyLock;

/// `module Name (` — everything up to (but not including) the opening paren of an
/// explicit export list, if there is one. Requires `[\w.]+` for the (possibly
/// dot-qualified) module name, e.g. `Data.List.NonEmpty`.
static MODULE_HEADER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\s*module\s+[\w.]+\s*").unwrap());

/// Leading identifier of a single export-list entry: `Name` in `Name`, `Name(..)`,
/// or `Name(Ctor1, Ctor2)`. Allows a trailing prime, which is legal in Haskell
/// identifiers (`map'`).
static ENTRY_NAME: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^([A-Za-z_][\w']*)").unwrap());

/// Top-level `data`/`newtype` declaration (module column 0). `class` is
/// deliberately excluded here — a class header can carry a superclass context
/// (`class Eq a => Ord2 a where`), so its name is resolved separately by
/// `CLASS_HEADER` instead of naively taking the first word after the keyword.
static TOP_LEVEL_DATA_CLASS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^(?:data|newtype)\s+(\w[\w']*)").unwrap());

/// Top-level `type` synonym / type family declaration. `family` is optional
/// (`type family Foo where`), and the captured name is filtered in code to drop
/// the `type instance` case (associated type instance, not a new export).
static TOP_LEVEL_TYPE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^type\s+(?:family\s+)?(\w[\w']*)").unwrap());

/// The rest of a `class` declaration's first line, used to correctly locate the
/// class name itself rather than a superclass: `class Eq a => Ord a where` names
/// `Ord`, not `Eq`.
static CLASS_HEADER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^class\s+([^\n]*)").unwrap());

/// First capitalized type/class-shaped token in a string (class and type names
/// always start with an uppercase letter in Haskell).
static FIRST_TYPE_TOKEN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[A-Z][\w']*").unwrap());

/// Top-level type signature `name ::` — used only to notice a name exists; the
/// binding itself may be defined a few lines below via equations.
static TOP_LEVEL_SIG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^([a-zA-Z_][\w']*)\s*::").unwrap());

/// Top-level value/function binding `name args = ...` (zero or more args). The
/// trailing `(?:[^=]|$)` guard (same trick as the Python extractor) keeps `==`,
/// `/=`, `<=`, `>=` inside the body from being misread as the defining `=`.
static TOP_LEVEL_BINDING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^([a-zA-Z_][\w']*)\s*[^=\n]*=(?:[^=]|$)").unwrap());

/// Top-level operator definition written in prefix (parenthesized) form, e.g.
/// `(<+>) :: Vector -> Vector -> Vector` or `(<+>) a b = ...`.
static TOP_LEVEL_OPERATOR_SIG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\(([^)\s]+)\)\s*::").unwrap());
static TOP_LEVEL_OPERATOR_BINDING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\(([^)\s]+)\)\s*[^=\n]*=(?:[^=]|$)").unwrap());

/// Haskell keywords that can appear at column 0 and would otherwise be
/// misidentified as a binding/signature name by the lowercase-leading regexes.
const KEYWORDS: &[&str] = &[
    "type", "data", "newtype", "class", "instance", "import", "module", "deriving", "where", "let",
    "in", "if", "then", "else", "case", "of", "do", "foreign", "infix", "infixl", "infixr",
    "default",
];

fn is_keyword(name: &str) -> bool {
    KEYWORDS.contains(&name)
}

/// Extract exported symbols from Haskell source code.
///
/// Haskell's real public API is the module's export list:
/// `module Foo (bar, baz, Qux(..)) where`. When present, that list *is* the
/// entire public API. When the module declaration has no parenthesized list at
/// all (`module Foo where`) — or there is no module header at all, as in a
/// headerless `Main` — everything top-level is exported by the language's own
/// rules, so we fall back to scanning top-level declarations.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = strip_comments_and_strings(content);

    if let Some(list) = find_export_list(&stripped) {
        return parse_export_list(&list);
    }

    let mut hits: Vec<(usize, String)> = Vec::new();

    for caps in TOP_LEVEL_DATA_CLASS.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str();
            if !is_keyword(n) {
                hits.push((name.start(), n.to_string()));
            }
        }
    }
    for caps in TOP_LEVEL_TYPE.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str();
            if !is_keyword(n) {
                hits.push((name.start(), n.to_string()));
            }
        }
    }
    for caps in CLASS_HEADER.captures_iter(&stripped) {
        if let Some(header) = caps.get(1) {
            let h = header.as_str();
            let h = h.split("where").next().unwrap_or(h);
            let decl_part = h.rsplit("=>").next().unwrap_or(h);
            if let Some(m) = FIRST_TYPE_TOKEN.find(decl_part) {
                hits.push((header.start(), m.as_str().to_string()));
            }
        }
    }
    for caps in TOP_LEVEL_SIG.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str();
            if !is_keyword(n) {
                hits.push((name.start(), n.to_string()));
            }
        }
    }
    for caps in TOP_LEVEL_BINDING.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str();
            if !is_keyword(n) {
                hits.push((name.start(), n.to_string()));
            }
        }
    }
    for caps in TOP_LEVEL_OPERATOR_SIG.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            hits.push((name.start(), name.as_str().to_string()));
        }
    }
    for caps in TOP_LEVEL_OPERATOR_BINDING.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            hits.push((name.start(), name.as_str().to_string()));
        }
    }

    hits.sort_by_key(|(pos, _)| *pos);

    let mut symbols = Vec::new();
    for (_, n) in hits {
        if !symbols.contains(&n) {
            symbols.push(n);
        }
    }
    symbols
}

/// Locates the module's explicit export list and returns its inner text (the
/// part between the outer parens), if one exists. Returns `None` when there is
/// no `module` header, or when the header has no parenthesized list at all
/// (`module Foo where`) — both cases mean "export everything top-level".
fn find_export_list(s: &str) -> Option<String> {
    let m = MODULE_HEADER.find(s)?;
    let rest = &s[m.end()..];
    let chars: Vec<char> = rest.chars().collect();
    if chars.first() != Some(&'(') {
        return None;
    }
    let mut depth = 0i32;
    for (idx, ch) in chars.iter().enumerate() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    let content: String = chars[1..idx].iter().collect();
                    return Some(content);
                }
            }
            _ => {}
        }
    }
    None
}

/// Splits export-list text on top-level commas (commas nested inside a `(...)`
/// constructor/method sublist do not split).
fn split_top_level(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth: i32 = 0;
    for ch in s.chars() {
        match ch {
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth -= 1;
                current.push(ch);
            }
            ',' if depth == 0 => parts.push(std::mem::take(&mut current)),
            _ => current.push(ch),
        }
    }
    if !current.trim().is_empty() {
        parts.push(current);
    }
    parts
}

/// Parses the inner text of a module's export list into exported symbol names.
fn parse_export_list(list: &str) -> Vec<String> {
    let mut symbols = Vec::new();
    for raw in split_top_level(list) {
        for name in parse_export_entry(raw.trim()) {
            if !symbols.contains(&name) {
                symbols.push(name);
            }
        }
    }
    symbols
}

/// Parses a single top-level export-list entry, e.g. `bar`, `Qux(..)`,
/// `Corge(Ctor1, Ctor2)`, `(<>)`, `pattern Foo`, `type (+)`, or `module Data.Map`
/// (whole-module re-export — contributes no leaf symbol of *this* module).
fn parse_export_entry(raw: &str) -> Vec<String> {
    let mut part = raw.trim();
    if part.is_empty() {
        return Vec::new();
    }
    if part.starts_with("module ") {
        return Vec::new();
    }
    if let Some(rest) = part.strip_prefix("pattern ") {
        part = rest.trim();
    }
    if let Some(rest) = part.strip_prefix("type ") {
        part = rest.trim();
    }

    // A bare parenthesized operator with no leading name, e.g. `(<>)`.
    if part.starts_with('(') && part.ends_with(')') && part.len() >= 2 {
        let inner = part[1..part.len() - 1].trim();
        if !inner.is_empty() {
            return vec![inner.to_string()];
        }
        return Vec::new();
    }

    let Some(caps) = ENTRY_NAME.captures(part) else {
        return Vec::new();
    };
    let name = caps[1].to_string();
    let mut out = vec![name];

    if let Some(paren_start) = part.find('(')
        && part.ends_with(')')
    {
        let inner = part[paren_start + 1..part.len() - 1].trim();
        if inner != ".." {
            for ctor in split_top_level(inner) {
                let c = ctor.trim();
                if !c.is_empty() {
                    out.push(c.to_string());
                }
            }
        }
    }

    out
}

/// True if `c` is one of Haskell's "symbol" characters — used to tell an operator
/// lexeme like `-->` apart from a comment-opening run of dashes.
fn is_symbol_char(c: char) -> bool {
    matches!(
        c,
        '!' | '#'
            | '$'
            | '%'
            | '&'
            | '*'
            | '+'
            | '.'
            | '/'
            | '<'
            | '='
            | '>'
            | '?'
            | '@'
            | '\\'
            | '^'
            | '|'
            | '~'
            | ':'
            | '-'
    )
}

/// True if `c` can continue a Haskell identifier (including the trailing
/// prime that's legal in identifiers, e.g. `map'`). Used to tell a real
/// char-literal-opening `'` apart from the prime at the end of an identifier.
fn is_ident_continue(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '\''
}

/// If a Haskell character literal opens at `chars[start]` (`chars[start] ==
/// '\''`), returns the index just past its closing quote — otherwise `None`.
/// Handles a plain character (`'c'`), a simple backslash escape (`'\"'`,
/// `'\\'`, `'\''`, `'\n'`, ...), and bounded named/numeric escapes (`'\NUL'`,
/// `'\955'`, `'\x3BB'`). Deliberately returns `None` for `''Foo` / `'foo`
/// (Template Haskell / DataKinds quotes, not literals), so callers must also
/// confirm the opening `'` isn't itself a trailing identifier prime before
/// treating this as a literal.
fn char_literal_end(chars: &[char], start: usize) -> Option<usize> {
    let n = chars.len();
    let pos = start + 1;
    if pos >= n || chars[pos] == '\'' {
        return None;
    }
    if chars[pos] == '\\' {
        let esc = pos + 1;
        if esc >= n {
            return None;
        }
        // Simple one-character escape, e.g. '\"', '\\', '\'', '\n'.
        if esc + 1 < n && chars[esc + 1] == '\'' {
            return Some(esc + 2);
        }
        // Bounded named/numeric escape, e.g. '\NUL', '\955', '\x3BB'.
        let mut j = esc;
        while j < n && j - esc < 8 && chars[j].is_alphanumeric() {
            j += 1;
        }
        if j > esc && j < n && chars[j] == '\'' {
            return Some(j + 1);
        }
        return None;
    }
    // Plain single character, e.g. 'c', '"', '-', '{'.
    if pos + 1 < n && chars[pos + 1] == '\'' {
        return Some(pos + 2);
    }
    None
}

/// Strips comments and string literals in one linear pass so neither is misread
/// as code:
/// - `{- ... -}` block comments, which nest (including `{-# ... #-}` pragmas);
/// - `--` line comments, EXCEPT when the dash run is immediately followed by
///   another symbol character, per the Haskell report — `-->` is an operator
///   lexeme, not a comment, but `--foo` is a comment;
/// - `"..."` string literals, honoring `\` escapes, so quotes/dashes/braces
///   inside string content can't be misread as comment or paren delimiters.
///
/// Newlines are preserved so the `(?m)^`-anchored declaration regexes stay
/// aligned to real source lines.
fn strip_comments_and_strings(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < n {
        let c = chars[i];

        // Block comment (nests), also covers `{-# LANGUAGE ... #-}` pragmas.
        if c == '{' && i + 1 < n && chars[i + 1] == '-' {
            i += 2;
            let mut depth = 1;
            while i < n && depth > 0 {
                if chars[i] == '{' && i + 1 < n && chars[i + 1] == '-' {
                    depth += 1;
                    i += 2;
                } else if chars[i] == '-' && i + 1 < n && chars[i + 1] == '}' {
                    depth -= 1;
                    i += 2;
                } else {
                    if chars[i] == '\n' {
                        out.push('\n');
                    }
                    i += 1;
                }
            }
            continue;
        }

        // Line comment: a run of 2+ dashes, unless the run is itself part of a
        // longer operator lexeme (e.g. `-->`).
        if c == '-' && i + 1 < n && chars[i + 1] == '-' {
            let mut j = i + 2;
            while j < n && chars[j] == '-' {
                j += 1;
            }
            let is_operator_lexeme = j < n && is_symbol_char(chars[j]);
            if !is_operator_lexeme {
                while i < n && chars[i] != '\n' {
                    i += 1;
                }
                continue;
            }
        }

        // Character literal ('c', '\n', '\"', '\NUL', ...). Guarded so a
        // trailing prime in an identifier (`map'`) or a Template Haskell /
        // DataKinds quote (`'foo`, `''Foo`) is never mistaken for one — a
        // real char literal never immediately follows an identifier
        // character. Without this, a literal like `'"'` (very common in
        // parsing/serialization code) would desync the string scanner below:
        // its embedded `"` would be read as opening a string literal, and
        // everything until the next stray `"` in the file — potentially the
        // rest of the module — would be silently swallowed as "string".
        if c == '\''
            && (i == 0 || !is_ident_continue(chars[i - 1]))
            && let Some(end) = char_literal_end(&chars, i)
        {
            for &lc in &chars[i..end] {
                out.push(lc);
            }
            i = end;
            continue;
        }

        // String literal, honoring backslash escapes.
        if c == '"' {
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
            continue;
        }

        out.push(c);
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haskell_explicit_export_list() {
        let src = r#"
module Data.Auth
    ( createAuth
    , AuthService
    , DEFAULT_TTL
    ) where

createAuth :: Config -> Auth
createAuth config = undefined

data AuthService = AuthService

internalHelper :: Int -> Int
internalHelper x = x + 1

DEFAULT_TTL :: Int
DEFAULT_TTL = 3600
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["createAuth", "AuthService", "DEFAULT_TTL"]);
    }

    #[test]
    fn test_haskell_export_list_excludes_unlisted() {
        // Everything not named in the export list is private, regardless of
        // whether it's a top-level binding, even if it "looks public" by having
        // no leading underscore (Haskell has no naming-convention visibility).
        let src = r#"
module Service (publicApi) where

publicApi :: Int -> Int
publicApi = privateHelper

privateHelper :: Int -> Int
privateHelper x = x * 2

privateConfig :: String
privateConfig = "secret"
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["publicApi"]);
        assert!(!symbols.contains(&"privateHelper".to_string()));
        assert!(!symbols.contains(&"privateConfig".to_string()));
    }

    #[test]
    fn test_haskell_type_dotdot_exports_type_only() {
        let src = r#"
module Shape
    ( Shape(..)
    , area
    ) where

data Shape = Circle Double | Square Double

area :: Shape -> Double
area (Circle r) = pi * r * r
area (Square s) = s * s
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Shape", "area"]);
        assert!(!symbols.contains(&"Circle".to_string()));
        assert!(!symbols.contains(&"Square".to_string()));
    }

    #[test]
    fn test_haskell_type_named_constructors_exported() {
        let src = r#"
module Data.Maybe.Custom
    ( Option(Some, None)
    ) where

data Option a = Some a | None
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Option", "Some", "None"]);
    }

    #[test]
    fn test_haskell_no_export_list_exports_everything() {
        // `module Foo where` with no parenthesized list: every top-level
        // declaration is public by Haskell's own rule.
        let src = r#"
module Foo where

data Widget = Widget

createWidget :: Int -> Widget
createWidget _ = Widget

class Renderable a where
    render :: a -> String
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Widget".to_string()));
        assert!(symbols.contains(&"createWidget".to_string()));
        assert!(symbols.contains(&"Renderable".to_string()));
    }

    #[test]
    fn test_haskell_comments_stripped() {
        // Both comment forms can contain declaration-shaped text and must not
        // leak into the extracted symbols, whether or not an export list exists.
        let src = r#"
module Foo (realFunc) where

-- | createAuth :: Config -> Auth
-- createAuth config = undefined

{- data FakeType = FakeType
   fakeFunc :: Int -> Int
   fakeFunc x = x
-}

realFunc :: Int -> Int
realFunc x = x + 1
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["realFunc"]);
    }

    #[test]
    fn test_haskell_nested_block_comments() {
        // Haskell block comments nest, unlike C's `/* */`.
        let src = r#"
module Foo where

{- outer {- inner data Hidden = Hidden -} still commented
   moreHidden :: Int
-}

visible :: Int
visible = 1
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["visible"]);
    }

    #[test]
    fn test_haskell_class_with_superclass_context() {
        // The class being *defined* is the name after `=>`, not the superclass
        // constraint before it.
        let src = r#"
module Data.Ord.Custom (Ord2(..)) where

class Eq a => Ord2 a where
    compare2 :: a -> a -> Ordering
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["Ord2"]);
    }

    #[test]
    fn test_haskell_class_no_export_list() {
        let src = r#"
module Foo where

class Eq a => Container f a where
    empty :: f a
    insert :: a -> f a -> f a
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Container".to_string()));
        assert!(!symbols.contains(&"Eq".to_string()));
    }

    #[test]
    fn test_haskell_operator_export() {
        // Real Haskell libraries commonly export custom infix operators using
        // the parenthesized prefix form in the export list.
        let src = r#"
module Data.Vector.Custom
    ( (<+>)
    , magnitude
    ) where

(<+>) :: Vector -> Vector -> Vector
(<+>) a b = undefined

magnitude :: Vector -> Double
magnitude = undefined

(<->) :: Vector -> Vector -> Vector
(<->) a b = undefined
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["<+>", "magnitude"]);
        assert!(!symbols.contains(&"<->".to_string()));
    }

    #[test]
    fn test_haskell_string_literal_with_dashes_and_braces() {
        // A string constant containing `--` or `{-`-shaped text must not be
        // misread as starting a comment (which would swallow real code after it).
        let src = r#"
module Foo (banner, realFunc) where

banner :: String
banner = "-- not a comment {- also not a comment -}"

realFunc :: Int -> Int
realFunc x = x + 1
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["banner", "realFunc"]);
    }

    #[test]
    fn test_haskell_operator_lexeme_not_mistaken_for_comment() {
        // `-->` is a legal operator lexeme (Haskell report: a dash run followed
        // by another symbol char does not start a comment), so a definition
        // using it must survive comment stripping.
        let src = r#"
module Foo ((-->), value) where

(-->) :: Int -> Int -> Bool
(-->) a b = a <= b

value :: Int
value = 42
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"-->".to_string()));
        assert!(symbols.contains(&"value".to_string()));
    }

    #[test]
    fn test_haskell_module_reexport_not_a_leaf_symbol() {
        let src = r#"
module Foo (module Data.Map, localFunc) where

localFunc :: Int -> Int
localFunc x = x
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["localFunc"]);
    }

    #[test]
    fn test_haskell_adversarial_char_literal_quote() {
        // ADVERSARIAL PROBE (temporary): a `Char` literal containing a double
        // quote, e.g. `'"'`, is extremely common in real parsing/serialization
        // code (CSV/JSON escaping). If the stripper doesn't know about Haskell
        // character literals, it will mistake the `"` inside `'"'` for the
        // start of a string literal and swallow the rest of the file.
        let src = r#"
module Csv where

quoteChar :: Char
quoteChar = '"'

parseCsv :: String -> [String]
parseCsv = undefined
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"parseCsv".to_string()),
            "got: {symbols:?}"
        );
    }

    #[test]
    fn test_haskell_adversarial_escaped_char_literal_quote() {
        // ADVERSARIAL PROBE (temporary): escaped-quote char literal `'\"'`.
        let src = r#"
module Csv where

escapeQuote :: Char -> String
escapeQuote c = if c == '\"' then "\\\"" else [c]

parseCsv :: String -> [String]
parseCsv = undefined
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"parseCsv".to_string()),
            "got: {symbols:?}"
        );
    }

    #[test]
    fn test_haskell_adversarial_where_indented_helper_no_export_list() {
        // ADVERSARIAL PROBE (temporary): a `where`-indented local helper must
        // NOT be treated as a top-level export in the no-export-list fallback.
        let src = r#"
module Foo where

processData :: [Int] -> [Int]
processData xs = map helper xs
  where
    helper :: Int -> Int
    helper x = x * 2
"#;
        let symbols = extract_exports(src);
        assert!(!symbols.contains(&"helper".to_string()), "got: {symbols:?}");
    }

    #[test]
    fn test_haskell_empty_export_list() {
        // `module Foo () where` is valid Haskell and means the module exports
        // nothing directly (used purely for its instances).
        let src = r#"
module Foo () where

hiddenFunc :: Int -> Int
hiddenFunc x = x
"#;
        let symbols = extract_exports(src);
        assert!(symbols.is_empty());
    }
}
