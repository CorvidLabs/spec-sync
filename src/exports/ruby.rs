use regex::Regex;
use std::sync::LazyLock;

/// Single-line # comments
static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)#.*$").unwrap());

/// Multi-line =begin/=end comments
static COMMENT_MULTI: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?ms)^=begin.*?^=end").unwrap());

/// Class declarations: class Name, class Name < Parent, or compact namespaced
/// form class Api::V1::Name — the whole `A::B::C` path is captured so the
/// real (last-segment) class name can be recovered instead of truncating at
/// the first `::`.
static RUBY_CLASS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[^\S\n]*class\s+((?:[A-Z]\w*::)*[A-Z]\w*)").unwrap());

/// Module declarations: module Name, including compact namespaced form
/// module Api::V1::Name.
static RUBY_MODULE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[^\S\n]*module\s+((?:[A-Z]\w*::)*[A-Z]\w*)").unwrap());

/// Top-level method definitions (at zero indentation, considered public).
/// Captures a trailing `?`/`!` since predicate/bang methods (`valid?`,
/// `reset!`) are idiomatic Ruby and that punctuation is part of the name.
static RUBY_DEF: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^def\s+(?:self\.)?(\w+[?!]?)").unwrap());

/// Instance method definitions (indented, inside a class — public by default).
/// Captures a trailing `?`/`!` — see RUBY_DEF.
static RUBY_METHOD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[^\S\n]+def\s+(?:self\.)?(\w+[?!]?)").unwrap());

/// Constant assignments: NAME = value. Ruby constants are any identifier
/// starting with an uppercase letter — not just SCREAMING_SNAKE_CASE, but
/// also single-letter (`X = 5`) and PascalCase (`Var = "..."`,
/// `DefaultLogger = Logger.new`) forms. The `=` must immediately (mod
/// whitespace) follow the whole identifier, not a `.` continuation of it —
/// this keeps the pattern from matching a bare prefix of a longer dotted
/// expression like `Human.foo = 2` (a setter call, not a constant
/// declaration).
static RUBY_CONST: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[^\S\n]*([A-Z]\w*)[^\S\n]*=").unwrap());

/// attr_accessor / attr_reader / attr_writer declarations
static RUBY_ATTR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[^\S\n]*attr_(?:accessor|reader|writer)\s+(.+)$").unwrap());

/// Symbol literal :name
static RUBY_SYMBOL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r":(\w+[?!]?)").unwrap());

/// private / protected visibility markers (bare toggle — nothing else on the line)
static VISIBILITY_PRIVATE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\s*(?:private|protected)\s*$").unwrap());

static VISIBILITY_PUBLIC: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\s*public\s*$").unwrap());

/// Post-hoc symbol-based visibility: `private :name`, `private :a, :b`,
/// `private(:name)`. This marks already-defined methods private by name
/// rather than toggling visibility for subsequent defs.
static VISIBILITY_PRIVATE_SYMBOLS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[^\S\n]*(?:private|protected)[^\S\n]*\(?[^\S\n]*(:[\w?!]+(?:[^\S\n]*,[^\S\n]*:[\w?!]+)*)[^\S\n]*\)?[^\S\n]*$",
    )
    .unwrap()
});

/// A singleton-class body (`class << self`, `class << other`). It opens a
/// block that a later `end` closes, and — like `class`/`module` — it carries
/// its own visibility state (a `private` inside `class << self` applies to
/// the singleton methods, not to the enclosing class body). `RUBY_CLASS`
/// deliberately does not match it (there's no constant being declared), so
/// it needs its own pattern or its `end` pops the enclosing class's
/// visibility-restore entry.
static SINGLETON_CLASS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[^\S\n]*class[^\S\n]*<<").unwrap());

/// A Ruby 3.0+ "endless method" (`def name(...) = expr` or `def name = expr`)
/// has no body block and needs no matching `end` at all. Two ordinary defs
/// that also carry an `=` must NOT be mistaken for it, since they do own an
/// `end`:
///   * a default-parameter def (`def foo(x = 1)`) — the endless form's `=`
///     sits *after* the parameter list closes, not inside it;
///   * a setter def (`def name=(value)`) — Ruby requires the `=` of a setter
///     name to be attached to the name, so an endless def without a
///     parameter list always has whitespace before its `=`.
static ENDLESS_DEF: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^[^\S\n]*def\s+[\w.?!]+(?:[^\S\n]*\([^)]*\)[^\S\n]*=[^=]|[^\S\n]+=[^=])")
        .unwrap()
});

/// A bare `end` keyword closing the innermost open block.
static BLOCK_END: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^[^\S\n]*end\b").unwrap());

/// Heredoc start: `<<~TAG`, `<<-TAG`, `<<TAG`, or quoted-tag variants
/// (`<<~"TAG"`, `<<~'TAG'`). Requires no whitespace immediately before `<<`
/// (checked by the caller, not the regex) so a bitshift expression like
/// `x << SOME_CONST` is never mistaken for a heredoc opener. The body lines
/// of a heredoc are arbitrary text -- they may contain a line that is just
/// `end` (or `class Foo`, etc.) which must never reach the block-nesting or
/// visibility scan below, or the scan desyncs.
static HEREDOC_START: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<<([-~]?)["']?([A-Za-z_]\w*)["']?"#).unwrap());

/// Compact namespaced declarations (`class Api::V1::UsersController`) capture
/// the whole `A::B::C` path, but only the last segment is the type actually
/// being declared here — the outer segments are references to an
/// already-existing namespace, not new declarations.
fn last_segment(name: &str) -> String {
    name.rsplit("::").next().unwrap_or(name).to_string()
}

/// Whether `character` can end a Ruby expression. A `/` or `%` that follows
/// one of these is the division or modulo operator, not the opening
/// delimiter of a regex or `%`-literal.
fn ends_expression(character: char) -> bool {
    character.is_alphanumeric() || matches!(character, ')' | ']' | '}' | '_')
}

/// The trailing identifier of `text` (`""` when it doesn't end in one), used
/// to ask what keyword — if any — immediately precedes a position.
fn trailing_word(text: &str) -> &str {
    let start = text
        .char_indices()
        .rev()
        .take_while(|(_, character)| character.is_alphanumeric() || *character == '_')
        .last()
        .map(|(index, _)| index);
    match start {
        Some(index) => &text[index..],
        None => "",
    }
}

/// Whether a regex or `%`-literal can begin right after `prefix` — i.e.
/// whether `prefix` leaves us in expression position rather than having just
/// completed an operand (in which case `/` is division and `%` is modulo).
fn literal_can_start_here(prefix: &str) -> bool {
    let head = prefix.trim_end();
    match head.chars().last() {
        None => true,
        Some(character) if !ends_expression(character) => true,
        Some(_) => matches!(
            trailing_word(head),
            "and"
                | "or"
                | "not"
                | "when"
                | "in"
                | "if"
                | "unless"
                | "while"
                | "until"
                | "then"
                | "else"
                | "elsif"
                | "case"
                | "return"
                | "rescue"
                | "do"
        ),
    }
}

/// Scans forward from the opening quote at `open` and returns the index just
/// past the closing quote (or the end of the line, for a quote that never
/// closes on this line).
fn skip_quoted(chars: &[char], open: usize, quote: char) -> usize {
    let mut cursor = open + 1;
    while cursor < chars.len() {
        if chars[cursor] == '\\' {
            cursor += 2;
            continue;
        }
        if chars[cursor] == quote {
            return cursor + 1;
        }
        cursor += 1;
    }
    chars.len()
}

/// If a `%`-literal (`%w[...]`, `%i(...)`, `%q{...}`, `%{...}`) starts at
/// `open`, returns the index just past its closing delimiter. Returns `None`
/// for the modulo operator, for an unknown delimiter, and for a literal that
/// does not close on this line.
fn percent_literal_end(chars: &[char], open: usize, prefix: &str) -> Option<usize> {
    if !literal_can_start_here(prefix) {
        return None;
    }
    let mut cursor = open + 1;
    if matches!(
        chars.get(cursor),
        Some('q' | 'Q' | 'w' | 'W' | 'i' | 'I' | 'r' | 's')
    ) {
        cursor += 1;
    }
    let close = match chars.get(cursor)? {
        '(' => ')',
        '[' => ']',
        '{' => '}',
        '<' => '>',
        '|' => '|',
        '!' => '!',
        '/' => '/',
        _ => return None,
    };
    cursor += 1;
    while cursor < chars.len() {
        if chars[cursor] == '\\' {
            cursor += 2;
            continue;
        }
        if chars[cursor] == close {
            return Some(cursor + 1);
        }
        cursor += 1;
    }
    None
}

/// If a regex literal starts at the `/` at `open`, returns the index just
/// past its closing `/`. Returns `None` for the division operator and for a
/// literal that does not close on this line.
fn regex_literal_end(chars: &[char], open: usize, prefix: &str) -> Option<usize> {
    if !literal_can_start_here(prefix) {
        return None;
    }
    let mut cursor = open + 1;
    while cursor < chars.len() {
        if chars[cursor] == '\\' {
            cursor += 2;
            continue;
        }
        if chars[cursor] == '/' {
            return Some(cursor + 1);
        }
        cursor += 1;
    }
    None
}

/// Replaces every string, backtick, regex and `%`-literal on a line with a
/// single identifier-shaped placeholder, so a keyword that merely appears
/// *inside* a literal (`raise "unexpected end"`, `line =~ /end/`,
/// `%w[begin end]`) is never read as a real block opener or closer by
/// `scan_line_blocks`. The placeholder is an identifier character rather
/// than whitespace on purpose: the text before a keyword decides whether it
/// opens a block, and `x = "text" if flag` must keep reading as "an operand
/// already completed here" (a statement modifier), exactly like `x = 1 if
/// flag`.
fn mask_literals(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut masked = String::with_capacity(line.len());
    let mut index = 0;
    while index < chars.len() {
        let current = chars[index];
        match current {
            '"' | '\'' | '`' => {
                index = skip_quoted(&chars, index, current);
                masked.push('s');
            }
            '%' => match percent_literal_end(&chars, index, &masked) {
                Some(next) => {
                    index = next;
                    masked.push('s');
                }
                None => {
                    masked.push(current);
                    index += 1;
                }
            },
            '/' => match regex_literal_end(&chars, index, &masked) {
                Some(next) => {
                    index = next;
                    masked.push('s');
                }
                None => {
                    masked.push(current);
                    index += 1;
                }
            },
            _ => {
                masked.push(current);
                index += 1;
            }
        }
    }
    masked
}

/// Whether an `if`/`unless`/`while`/`until` appearing after `prefix` opens a
/// multi-line block (and therefore owns an `end`) rather than being a
/// statement modifier (which owns nothing). It opens when nothing precedes
/// it on the line — the classic `if cond` header — and also when what
/// precedes it cannot stand as a complete expression: after an assignment
/// (`x = if cond`, `@memo ||= while ...`), after `(`, `;`, `&&`, `||`, or
/// after a keyword that introduces a fresh expression (`then`, `else`,
/// `do`, `and`, `or`, `not`). Anything else — `return if x`, `next if x`,
/// `log "msg" if x` — is a modifier and must not be counted, or the block
/// stack gains an entry that no `end` will ever pop.
fn keyword_opens_block_here(prefix: &str) -> bool {
    let head = prefix.trim_end();
    if head.is_empty() {
        return true;
    }
    if let Some(before_equals) = head.strip_suffix('=') {
        // `==`, `!=`, `<=`, `>=`, `=~` compare — they don't assign, so what
        // follows them is the right-hand side of a completed expression.
        if !matches!(
            before_equals.chars().last(),
            Some('=' | '!' | '<' | '>' | '~')
        ) {
            return true;
        }
    }
    if head.ends_with("&&") || head.ends_with("||") || head.ends_with('(') || head.ends_with(';') {
        return true;
    }
    matches!(
        trailing_word(head),
        "then" | "else" | "do" | "and" | "or" | "not"
    )
}

/// What one source line does to Ruby's block nesting.
struct LineBlocks {
    /// A block that needs a matching `end` opens on this line (`def`, a
    /// `do` block, `begin`/`case`/`for`, or a non-modifier
    /// `if`/`unless`/`while`/`until`). `class`/`module`/`class <<` are
    /// reported by their own patterns instead, since they also carry
    /// visibility state.
    opens: bool,
    /// An `end` keyword appears somewhere on this line.
    has_end: bool,
}

/// Reads the block-nesting effect of a single (literal-masked) line.
///
/// Everything here is deliberately token-based rather than anchored to the
/// line's first word: a block opener does not have to start its line.
/// `x = if cond`, `@memo ||= begin`, `items.each do |item|` and
/// `class << self` all open a block whose `end` arrives later, and an
/// extractor that misses the opener lets that `end` pop the enclosing
/// class's visibility-restore entry — which republishes every `private`
/// method that follows as public API.
fn scan_line_blocks(masked: &str, endless_def: bool) -> LineBlocks {
    let bytes = masked.as_bytes();
    let mut blocks = LineBlocks {
        opens: false,
        has_end: false,
    };
    let mut index = 0;
    while index < bytes.len() {
        if !(bytes[index].is_ascii_alphabetic() || bytes[index] == b'_') {
            index += 1;
            continue;
        }
        let start = index;
        while index < bytes.len() && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
        {
            index += 1;
        }
        let word = &masked[start..index];
        if !matches!(
            word,
            "def" | "do" | "begin" | "case" | "for" | "if" | "unless" | "while" | "until" | "end"
        ) {
            continue;
        }
        let prefix = &masked[..start];
        // `record.end`, `:end`, `@if`, `$begin` — a qualified name, a symbol
        // or a variable that merely spells a keyword.
        if matches!(
            prefix
                .chars()
                .rev()
                .find(|character| !character.is_whitespace()),
            Some('.' | ':' | '@' | '$')
        ) {
            continue;
        }
        // A hash label (`validates :name, if: :admin?`, `{ end: 3 }`) is a
        // key, not a keyword. `::` is a namespace separator, not a label.
        let rest = &masked[index..];
        if rest.starts_with(':') && !rest.starts_with("::") {
            continue;
        }
        match word {
            "end" => blocks.has_end = true,
            "def" => blocks.opens |= !endless_def,
            // `begin`/`case`/`for` are never statement modifiers, and a `do`
            // is either a block opener (`items.each do |item|`) or the
            // redundant body marker of a `while`/`until`/`for` header on
            // this same line -- which reports the very same single open.
            "do" | "begin" | "case" | "for" => blocks.opens = true,
            // `if` / `unless` / `while` / `until`: block header or modifier.
            _ => blocks.opens |= keyword_opens_block_here(prefix),
        }
    }
    blocks
}

/// Extract public symbols from Ruby source code.
/// Ruby defaults to public visibility. We track visibility state changes
/// (private/protected/public) to determine which methods are public.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = COMMENT_MULTI.replace_all(content, "");
    let stripped = COMMENT_SINGLE.replace_all(&stripped, "");

    let mut symbols = Vec::new();

    // Classes and modules are always "public" at the namespace level
    for caps in RUBY_CLASS.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = last_segment(name.as_str());
            if !symbols.contains(&n) {
                symbols.push(n);
            }
        }
    }

    for caps in RUBY_MODULE.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = last_segment(name.as_str());
            if !symbols.contains(&n) {
                symbols.push(n);
            }
        }
    }

    // Methods explicitly privatized post-hoc via `private :name` / `protected
    // :a, :b` (as opposed to the bare `private` line, which toggles
    // visibility for defs that follow it). Collected up front over the whole
    // file since the post-hoc call commonly comes *after* the method it
    // privatizes.
    let mut privatized: std::collections::HashSet<String> = std::collections::HashSet::new();
    for caps in VISIBILITY_PRIVATE_SYMBOLS.captures_iter(&stripped) {
        if let Some(list) = caps.get(1) {
            for sym in RUBY_SYMBOL.captures_iter(list.as_str()) {
                if let Some(name) = sym.get(1) {
                    privatized.insert(name.as_str().to_string());
                }
            }
        }
    }

    // Top-level defs (zero indentation) are always public
    for caps in RUBY_DEF.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str();
            if !n.starts_with('_') && !privatized.contains(n) && !symbols.contains(&n.to_string()) {
                symbols.push(n.to_string());
            }
        }
    }

    // Track visibility for indented methods (inside classes).
    // Walk lines, toggle visibility state. Visibility is scoped to the
    // innermost class/module: entering a new `class`/`module` line resets
    // the state back to public, so a `private` toggle inside one
    // class/module does not leak into sibling classes/modules later in the
    // same file. `scope_stack` remembers, for each currently-open block,
    // what to restore `public` to when that block's `end` is reached:
    // `Some(prev)` for a class/module body (restore the enclosing scope's
    // visibility), `None` for any other block (def/if/case/do/... — these
    // don't carry their own visibility state, so a `private` toggle set
    // before entering one still applies to a `def` that happens to sit
    // inside a nested `if`/`case`). Without this stack, a `private` toggle
    // set before a nested class/module would incorrectly stay in effect
    // forever after that nested scope's `end` closes it (visibility was
    // reset to public on entry but never restored on exit).
    //
    // The stack is only as good as the opener detection feeding it: every
    // `end` in the file pops something, so an opener that goes unnoticed
    // makes its `end` pop the enclosing class's `Some(prev)` entry instead,
    // restoring `public` in the middle of a `private` region and publishing
    // the rest of the class as public API. `scan_line_blocks` therefore
    // recognises openers wherever they occur on a line — `x = if cond`,
    // `items.each do |item|`, `class << self` — not just as the line's
    // first token.
    let mut public = true;
    let mut scope_stack: Vec<Option<bool>> = Vec::new();
    // Tag of a currently-open heredoc, if any. While this is `Some`, every
    // line is heredoc body content -- arbitrary text that must be skipped
    // entirely (not scanned for `end`, `class`, `private`, etc.) until the
    // line that terminates it is found.
    let mut heredoc_terminator: Option<String> = None;
    for line in stripped.lines() {
        if let Some(tag) = &heredoc_terminator {
            // A `<<~`/`<<-` heredoc terminator may be indented; a bare
            // `<<TAG` terminator must start at column 0. Accepting either by
            // trimming is safe here since we're only trying to find *where
            // the body ends*, not reformat it.
            if line.trim() == tag.as_str() {
                heredoc_terminator = None;
            }
            continue;
        }

        // String/regex/`%`-literal contents are masked out before any
        // block-nesting question is asked of the line, so `raise "the end"`
        // and `text =~ /end/` can't be read as block structure.
        let masked = mask_literals(line);
        let is_singleton_class = SINGLETON_CLASS.is_match(line);
        let is_class_or_module =
            is_singleton_class || RUBY_CLASS.is_match(line) || RUBY_MODULE.is_match(line);
        let blocks = scan_line_blocks(&masked, ENDLESS_DEF.is_match(line));

        if is_class_or_module {
            if !blocks.has_end {
                scope_stack.push(Some(public));
                public = true;
            }
        } else if blocks.opens && !blocks.has_end {
            scope_stack.push(None);
        }

        if BLOCK_END.is_match(line)
            && let Some(Some(prev_public)) = scope_stack.pop()
        {
            public = prev_public;
        }

        if VISIBILITY_PRIVATE.is_match(line) {
            public = false;
            continue;
        }
        if VISIBILITY_PUBLIC.is_match(line) {
            public = true;
            continue;
        }
        if VISIBILITY_PRIVATE_SYMBOLS.is_match(line) {
            // Post-hoc `private :name` only affects the named method(s)
            // (handled via `privatized` above) — it doesn't change
            // visibility for defs that follow it.
            continue;
        }

        if public
            && let Some(caps) = RUBY_METHOD.captures(line)
            && let Some(name) = caps.get(1)
        {
            let n = name.as_str();
            if !n.starts_with('_')
                && n != "initialize"
                && !privatized.contains(n)
                && !symbols.contains(&n.to_string())
            {
                symbols.push(n.to_string());
            }
        }

        // A heredoc opener on this line means every subsequent line is
        // opaque body content until the terminator is found -- start
        // skipping from the *next* line onward (this line has already been
        // fully scanned above). Bitshift expressions like `x << SOME_CONST`
        // never match: Ruby heredocs require no whitespace between `<<` and
        // the optional `~`/`-` modifier/tag, which `HEREDOC_START` mirrors
        // by having no whitespace token in that position, whereas a bitshift
        // operator is conventionally (and here, structurally) followed by a
        // space before its right-hand operand.
        // `class <<self` (no space) is a singleton-class header, not a
        // heredoc opener -- reading it as one would swallow the whole rest
        // of the file as opaque body text while hunting for a terminator
        // line that says `self`.
        if !is_singleton_class
            && let Some(caps) = HEREDOC_START.captures(line)
            && let Some(tag) = caps.get(2)
        {
            heredoc_terminator = Some(tag.as_str().to_string());
        }
    }

    // Constants
    for caps in RUBY_CONST.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str().to_string();
            if !symbols.contains(&n) {
                symbols.push(n);
            }
        }
    }

    // attr_accessor / attr_reader / attr_writer (public attributes)
    for caps in RUBY_ATTR.captures_iter(&stripped) {
        if let Some(attrs) = caps.get(1) {
            for sym in RUBY_SYMBOL.captures_iter(attrs.as_str()) {
                if let Some(name) = sym.get(1) {
                    let n = name.as_str().to_string();
                    if !symbols.contains(&n) {
                        symbols.push(n);
                    }
                }
            }
        }
    }

    symbols
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ruby_class_and_methods() {
        let src = r#"
module Authentication
  class AuthService
    DEFAULT_TTL = 3600

    attr_reader :token, :expires_at

    def validate(token)
      # ...
    end

    def self.create(config)
      # ...
    end

    private

    def internal_check
      # ...
    end
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Authentication".to_string()));
        assert!(symbols.contains(&"AuthService".to_string()));
        assert!(symbols.contains(&"validate".to_string()));
        assert!(symbols.contains(&"create".to_string()));
        assert!(symbols.contains(&"DEFAULT_TTL".to_string()));
        assert!(symbols.contains(&"token".to_string()));
        assert!(symbols.contains(&"expires_at".to_string()));
        assert!(!symbols.contains(&"internal_check".to_string()));
    }

    #[test]
    fn test_ruby_top_level_functions() {
        let src = r#"
def process_data(input)
  # ...
end

def _private_helper
  # ...
end

class DataProcessor
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"process_data".to_string()));
        assert!(symbols.contains(&"DataProcessor".to_string()));
        assert!(!symbols.contains(&"_private_helper".to_string()));
    }

    #[test]
    fn test_ruby_visibility_toggle() {
        let src = r#"
class Foo
  def public_one
  end

  def public_two
  end

  private

  def secret_one
  end

  public

  def public_again
  end

  protected

  def also_hidden
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Foo".to_string()));
        assert!(symbols.contains(&"public_one".to_string()));
        assert!(symbols.contains(&"public_two".to_string()));
        assert!(symbols.contains(&"public_again".to_string()));
        assert!(!symbols.contains(&"secret_one".to_string()));
        assert!(!symbols.contains(&"also_hidden".to_string()));
    }

    #[test]
    fn test_ruby_visibility_restored_after_nested_module_closes() {
        // Regression test: a `private` toggle set in an outer class body must resume
        // after a nested `module`/`class`'s `end` closes it -- previously, entering the
        // nested type unconditionally reset visibility to public and nothing ever
        // restored it, so `secret` (declared after `Inner` closes) incorrectly leaked.
        let src = r#"
class Outer
  private

  module Inner
    def pub
    end
  end

  def secret
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Outer".to_string()));
        assert!(symbols.contains(&"Inner".to_string()));
        assert!(symbols.contains(&"pub".to_string()));
        assert!(!symbols.contains(&"secret".to_string()));
    }

    #[test]
    fn test_ruby_visibility_persists_across_nested_if_block() {
        // A `private` toggle must still apply to a `def` that happens to sit inside a
        // nested `if`/`case`/`begin` block within the class body -- these blocks don't
        // carry their own visibility state, unlike `class`/`module`.
        let src = r#"
class Config
  private

  if RUBY_VERSION >= "3.0"
    def conditional_method
    end
  end

  def another_secret
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Config".to_string()));
        assert!(!symbols.contains(&"conditional_method".to_string()));
        assert!(!symbols.contains(&"another_secret".to_string()));
    }

    #[test]
    fn test_ruby_skips_initialize() {
        let src = r#"
class Bar
  def initialize(name)
    @name = name
  end

  def name
    @name
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Bar".to_string()));
        assert!(symbols.contains(&"name".to_string()));
        assert!(!symbols.contains(&"initialize".to_string()));
    }

    #[test]
    fn test_ruby_visibility_scoped_per_class() {
        // A `private` toggle inside one class must not leak into a sibling
        // class/module later in the same file — visibility resets at each
        // new class/module boundary.
        let src = r#"
module Billing
  class Invoice
    def total
    end

    private

    def apply_discount
    end
  end

  class Receipt
    def summary
    end
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Invoice".to_string()));
        assert!(symbols.contains(&"Receipt".to_string()));
        assert!(symbols.contains(&"total".to_string()));
        assert!(symbols.contains(&"summary".to_string()));
        assert!(!symbols.contains(&"apply_discount".to_string()));
    }

    #[test]
    fn test_ruby_namespaced_class_declaration() {
        // Compact namespaced class declarations (common for Rails
        // controllers/models) must resolve to the real class name, not a
        // truncated leading namespace segment.
        let src = r#"
class Api::V1::UsersController
  def index
  end

  def create
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"UsersController".to_string()));
        assert!(symbols.contains(&"index".to_string()));
        assert!(symbols.contains(&"create".to_string()));
        assert!(!symbols.contains(&"Api".to_string()));
    }

    #[test]
    fn test_ruby_posthoc_private_symbol_and_predicate_methods() {
        // Covers: `private :name` post-hoc visibility (as opposed to a bare
        // `private` toggle), predicate (`?`) and bang (`!`) method names
        // kept intact, and a realistic mixin module using `module_function`
        // plus a `self.` singleton method.
        let src = r#"
module Authenticatable
  module_function

  def logged_in?
    !!current_session
  end

  def self.reset!
    @session = nil
  end

  def internal_token
    @session
  end

  private :internal_token
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Authenticatable".to_string()));
        assert!(symbols.contains(&"logged_in?".to_string()));
        assert!(symbols.contains(&"reset!".to_string()));
        assert!(!symbols.contains(&"internal_token".to_string()));
        assert!(!symbols.contains(&"logged_in".to_string()));
        assert!(!symbols.contains(&"reset".to_string()));
    }

    #[test]
    fn test_ruby_learnxinyminutes_human_class() {
        // Real excerpt from learnxinyminutes.com/ruby (the "Classes" section):
        // a class variable, `initialize`, a `name=` setter, a plain getter,
        // attr_accessor/attr_reader/attr_writer on top of the hand-written
        // getter/setter, a `self.` class method, and a plain instance method
        // reading the class variable.
        let src = r#"
# You can define a class with the 'class' keyword.
class Human

  # A class variable. It is shared by all instances of this class.
  @@species = 'H. sapiens'

  # Basic initializer
  def initialize(name, age = 0)
    # Assign the argument to the 'name' instance variable for the instance.
    @name = name
    # If no age given, we will fall back to the default in the arguments list.
    @age = age
  end

  # Basic setter method
  def name=(name)
    @name = name
  end

  # Basic getter method
  def name
    @name
  end

  # The above functionality can be encapsulated using the attr_accessor method
  # as follows.
  attr_accessor :name

  # Getter/setter methods can also be created individually like this.
  attr_reader :name
  attr_writer :name

  # A class method uses self to distinguish from instance methods.
  # It can only be called on the class, not an instance.
  def self.say(msg)
    puts msg
  end

  def species
    @@species
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Human".to_string()));
        assert!(symbols.contains(&"name".to_string()));
        assert!(symbols.contains(&"say".to_string()));
        assert!(symbols.contains(&"species".to_string()));
        assert!(!symbols.contains(&"initialize".to_string()));
    }

    #[test]
    fn test_ruby_learnxinyminutes_concern_module() {
        // Real excerpt from learnxinyminutes.com/ruby (the module callbacks
        // section): a module with nested modules, a `self.included` hook
        // method, and a class that mixes the outer module in.
        let src = r#"
# Callbacks are executed when including and extending a module
module ConcernExample
  def self.included(base)
    base.extend(ClassMethods)
    base.send(:include, InstanceMethods)
  end

  module ClassMethods
    def bar
      'bar'
    end
  end

  module InstanceMethods
    def qux
      'qux'
    end
  end
end

class Something
  include ConcernExample
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"ConcernExample".to_string()));
        assert!(symbols.contains(&"ClassMethods".to_string()));
        assert!(symbols.contains(&"InstanceMethods".to_string()));
        assert!(symbols.contains(&"Something".to_string()));
        assert!(symbols.contains(&"bar".to_string()));
        assert!(symbols.contains(&"qux".to_string()));
        assert!(symbols.contains(&"included".to_string()));
    }

    #[test]
    fn test_ruby_heredoc_body_line_matching_end_does_not_desync_visibility() {
        // Regression test: a heredoc body line that is just `end` (a very common
        // thing to see in a heredoc holding e.g. generated Ruby/SQL/HTML source)
        // used to be scanned as a real block-closer. That popped the wrong
        // entry off the block-nesting stack, so the *real* `end` a few lines
        // later incorrectly popped `class Foo`'s visibility-restore entry
        // instead, flipping `public` back to `true` while still inside the
        // class body and leaking `secret` even though it's declared under
        // `private`.
        let src = r#"
class Foo
  def public_one
  end

  private

  SQL_TEXT = <<~SQL
    end
  SQL

  def secret
  end
end

class Bar
  def also_public
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Foo".to_string()));
        assert!(symbols.contains(&"Bar".to_string()));
        assert!(symbols.contains(&"public_one".to_string()));
        assert!(symbols.contains(&"SQL_TEXT".to_string()));
        assert!(symbols.contains(&"also_public".to_string()));
        assert!(!symbols.contains(&"secret".to_string()));
    }

    #[test]
    fn test_ruby_heredoc_variants_with_end_in_body_do_not_desync_visibility() {
        // Same regression as above but exercising the other heredoc opener
        // spellings: bare `<<TAG`, dash `<<-TAG`, and a quoted tag `<<~"TAG"`.
        // All three must have their body lines (including a literal `end`)
        // skipped by the block-nesting scan.
        let src = r#"
class Baz
  private

  A = <<TAGA
end
TAGA

  B = <<-TAGB
    end
    TAGB

  C = <<~"TAGC"
    end
  TAGC

  def secret
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Baz".to_string()));
        assert!(!symbols.contains(&"secret".to_string()));
    }

    #[test]
    fn test_ruby_oneliner_class_does_not_leak_public_after_private() {
        // Regression test: a self-contained one-liner namespace declaration
        // (`class Inline; end`) never pushes a block-nesting stack entry (it
        // has nothing to restore), but the code used to unconditionally reset
        // `public = true` anyway. That stomped on a `private` toggle set
        // earlier in the enclosing class body, so `secret` (declared right
        // after the one-liner) incorrectly came back as public.
        let src = r#"
class Foo
  private

  class Inline; end

  def secret
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Foo".to_string()));
        assert!(symbols.contains(&"Inline".to_string()));
        assert!(!symbols.contains(&"secret".to_string()));
    }

    #[test]
    fn test_ruby_oneliner_module_does_not_leak_public_after_private() {
        // Same regression as above, but for a one-liner `module` instead of
        // a one-liner `class`.
        let src = r#"
class Foo
  private

  module Inline; end

  def secret
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Foo".to_string()));
        assert!(symbols.contains(&"Inline".to_string()));
        assert!(!symbols.contains(&"secret".to_string()));
    }

    #[test]
    fn test_ruby_begin_rescue_ensure_does_not_desync_visibility() {
        // A `begin`/`rescue`/`ensure`/`end` block doesn't carry its own
        // visibility state (unlike class/module), so a `private` toggle set
        // before it must still apply to a `def` that follows the block's
        // `end`. This also checks that `begin` itself pushes exactly one
        // stack entry regardless of how many `rescue`/`ensure` clauses it has.
        let src = r#"
class Config
  private

  begin
    require "optional_dep"
  rescue LoadError
    nil
  ensure
    nil
  end

  def another_secret
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Config".to_string()));
        assert!(!symbols.contains(&"another_secret".to_string()));
    }

    #[test]
    fn test_ruby_case_when_else_does_not_desync_visibility() {
        // A `case`/`when`/`else`/`end` block, like `if`/`begin`, doesn't carry
        // its own visibility state -- a `private` toggle set before it must
        // still apply after the block's `end` closes it.
        let src = r#"
class Config
  private

  case RUBY_VERSION
  when "3.0"
    nil
  else
    nil
  end

  def another_secret
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Config".to_string()));
        assert!(!symbols.contains(&"another_secret".to_string()));
    }

    #[test]
    fn test_ruby_oneliner_def_does_not_desync_visibility() {
        // A one-liner `def quick; end` is self-contained (its `end` is on the
        // same line) and must never push a block-nesting stack entry --
        // otherwise a later unrelated `end` would incorrectly pop it.
        let src = r#"
class Foo
  def quick; end

  private

  def secret
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Foo".to_string()));
        assert!(symbols.contains(&"quick".to_string()));
        assert!(!symbols.contains(&"secret".to_string()));
    }

    #[test]
    fn test_ruby_assignment_form_if_does_not_leak_private_methods() {
        // Regression test (#479): an `if` used as an *expression* (`x = if
        // cond`) opens a block just like a leading `if` does, but the opener
        // is not the line's first token. Missing it meant the block's `end`
        // popped `class WatchCommand`'s visibility-restore entry, flipping
        // `public` back on inside the `private` region -- so every method
        // after the construct was published as public API.
        let src = r#"
class WatchCommand
  def public_one
    1
  end

  private

  def legit_private
    2
  end

  x = if true
    :nested
  end

  def should_be_private
    3
  end

  def also_private
    4
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"WatchCommand".to_string()));
        assert!(symbols.contains(&"public_one".to_string()));
        assert!(!symbols.contains(&"legit_private".to_string()));
        assert!(!symbols.contains(&"should_be_private".to_string()));
        assert!(!symbols.contains(&"also_private".to_string()));
    }

    #[test]
    fn test_ruby_public_method_after_assignment_form_if_still_extracts() {
        // Control for the regression above: bounding the private region must
        // not be achieved by muting the scan. A public method that follows an
        // assignment-form `if` is still a real export.
        let src = r#"
class WatchCommand
  def public_one
    1
  end

  x = if true
    :nested
  end

  def public_after
    2
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"WatchCommand".to_string()));
        assert!(symbols.contains(&"public_one".to_string()));
        assert!(symbols.contains(&"public_after".to_string()));
    }

    #[test]
    fn test_ruby_trailing_do_block_does_not_leak_private_methods() {
        // A `do ... end` block almost never starts its line (`items.each do
        // |item|`), so anchoring opener detection to the first token missed
        // it and its `end` closed the class's visibility region early.
        let src = r#"
class Registry
  def public_one
  end

  private

  ITEMS.each do |item|
    define_method(item) { item }
  end

  def secret
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Registry".to_string()));
        assert!(symbols.contains(&"public_one".to_string()));
        assert!(!symbols.contains(&"secret".to_string()));
    }

    #[test]
    fn test_ruby_modifier_if_is_not_a_block_opener() {
        // The mirror-image error: a statement modifier (`handle if flag`)
        // owns no `end`, so counting it as an opener pushes an entry nothing
        // ever pops -- and the *next* real `end` then restores the wrong
        // visibility. Here `module Inner`'s `end` must restore the outer
        // class's `private`, keeping `secret` off the contract.
        let src = r#"
class Outer
  private

  module Inner
    def pub
    end

    handle if flag
  end

  def secret
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"pub".to_string()));
        assert!(!symbols.contains(&"secret".to_string()));
    }

    #[test]
    fn test_ruby_keyword_inside_a_literal_is_not_block_structure() {
        // `end` inside a string, a regex, or a `%w` list is text, not a
        // block closer. Reading one as a closer suppressed the push for the
        // block that line opens, leaving the real `end` to pop the class's
        // visibility-restore entry instead.
        let src = r#"
class Foo
  private

  limit = if config.fetch("end")
    1
  end

  pattern = if source =~ /end/
    2
  end

  tokens = if list == %w[end]
    3
  end

  def secret
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Foo".to_string()));
        assert!(!symbols.contains(&"secret".to_string()));
    }

    #[test]
    fn test_ruby_hash_label_spelled_like_a_keyword_is_not_block_structure() {
        // `end:` / `if:` in a keyword-argument list are hash keys. Counting
        // the `end:` label as a block closer suppressed the push for the
        // `if` that opens on the same line, so the construct's real `end`
        // ended the private region early.
        let src = r#"
class Timeline
  private

  span = if window?(start: 1, end: 2)
    3
  end

  def secret
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Timeline".to_string()));
        assert!(!symbols.contains(&"secret".to_string()));
    }

    #[test]
    fn test_ruby_singleton_class_body_carries_its_own_visibility() {
        // `class << self` opens a block with its own visibility state, but
        // it declares no constant so the class-declaration pattern doesn't
        // see it. Treating it as invisible let its `end` close the enclosing
        // class's `private` region.
        let src = r#"
class Foo
  private

  class << self
    def build
    end
  end

  def secret
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Foo".to_string()));
        assert!(symbols.contains(&"build".to_string()));
        assert!(!symbols.contains(&"secret".to_string()));
    }

    #[test]
    fn test_ruby_singleton_class_without_a_space_is_not_a_heredoc() {
        // `class <<self` (no space) looks exactly like a `<<TAG` heredoc
        // opener. Reading it as one swallows the entire rest of the file as
        // heredoc body while hunting for a terminator line saying `self`,
        // dropping every remaining declaration.
        let src = r#"
class Foo
  private

  class <<self
    def build
    end
  end

  def secret
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"build".to_string()));
        assert!(!symbols.contains(&"secret".to_string()));
    }

    #[test]
    fn test_ruby_setter_def_is_not_an_endless_method() {
        // `def name=(value)` is a setter with a body and an `end`, not a
        // Ruby 3 endless method (`def name = value`, which needs whitespace
        // before the `=`). Treating the setter as endless skipped its push,
        // so its `end` closed the enclosing `private` region.
        let src = r#"
class Person
  def public_one
  end

  private

  def name=(value)
    @name = value
  end

  def secret
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Person".to_string()));
        assert!(symbols.contains(&"public_one".to_string()));
        assert!(!symbols.contains(&"name".to_string()));
        assert!(!symbols.contains(&"secret".to_string()));
    }

    #[test]
    fn test_ruby_endless_method_does_not_desync_visibility() {
        // A Ruby 3.0+ endless method (`def square(x) = x * x`) has no `end`
        // at all -- it must not push a block-nesting stack entry, or a later
        // unrelated `end` elsewhere in the class would incorrectly pop it and
        // desync the visibility-restore chain.
        let src = r#"
class MathHelper
  def square(x) = x * x

  private

  def secret
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"MathHelper".to_string()));
        assert!(symbols.contains(&"square".to_string()));
        assert!(!symbols.contains(&"secret".to_string()));
    }
}
