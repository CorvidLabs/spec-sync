use regex::Regex;
use std::sync::LazyLock;

/// Lua long-form comment: `--[[ ... ]]`, or with an equals-sign fence
/// (`--[=[ ... ]=]`, `--[==[ ... ]==]`, ...). The `regex` crate has no
/// backreferences, so the opening/closing fence lengths aren't verified to
/// match each other — an approximation that's fine for a comment-stripping
/// pre-pass. Stripped before the line-comment pattern so a long comment's
/// interior lines (which may themselves start with `--`) don't leak partial
/// text once the first line is chopped off.
static LONG_COMMENT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)--\[=*\[.*?\]=*\]").unwrap());

/// Lua long-bracket string literal: `[[ ... ]]`, or with an equals-sign
/// fence (`[=[ ... ]=]`, ...). Same fence-matching approximation as
/// `LONG_COMMENT`. Applied *after* `LONG_COMMENT` so any remaining
/// `[[ ... ]]` span is a genuine string literal, not part of an
/// already-stripped `--[[ ... ]]` comment. Without this, a multi-line
/// doc/usage string (a common way to embed example code or `--help` text in
/// a Lua script) whose interior happens to contain a line starting with
/// `function foo(` or `M.foo =` at column zero would be indistinguishable
/// from real top-level code once split into lines.
static LONG_STRING: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)\[=*\[.*?\]=*\]").unwrap());

/// Lua single-line comment: `-- ...` to end of line.
static LINE_COMMENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)--.*$").unwrap());

/// `local M = {}` — the near-universal declaration of a module table that
/// public members get attached to. Top-level only (no leading indentation),
/// mirroring how the other extractors distinguish top-level declarations
/// from ones nested inside a function/block.
static MODULE_TABLE_DECL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^local\s+(\w+)\s*=\s*\{\s*\}").unwrap());

/// `return M` as the file's final top-level statement — confirms which
/// locally-declared table is the module's actual public export.
static RETURN_MODULE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^return\s+(\w+)\s*;?\s*$").unwrap());

/// `function M.foo(` or `function M:foo(` — the keyword-first form of
/// attaching a member to a table. The colon form is sugar for an implicit
/// `self` first parameter (Lua's method-definition syntax) but is exactly
/// as public as the dot form.
static FUNCTION_MEMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^function\s+(\w+)[.:](\w+)\s*\(").unwrap());

/// `M.foo = ...` — the assignment-first form of attaching a member to a
/// table, whether the right-hand side is `function(...) ... end`, a
/// constant, or another value. Deliberately not split into a
/// function-vs-value variant (which would need a negative lookahead the
/// `regex` crate doesn't support): any assignment onto the module table's
/// namespace makes that member part of its public surface.
static ASSIGN_MEMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^(\w+)\.(\w+)\s*=").unwrap());

/// `return { foo = foo, bar = bar, ... }` — the anonymous-table-literal
/// export style used by modules that never bother naming their module
/// table at all. Captures the whole brace body so its keys can be pulled
/// out separately.
static RETURN_TABLE_BLOCK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?ms)^return\s*\{(.*)\}\s*;?\s*$").unwrap());

/// A `key = ` entry inside a `return { ... }` table body, whether it's the
/// first entry right after `{` or a later one following a comma. Matches at
/// any brace-nesting depth within the block; callers must filter to depth
/// zero (see `extract_top_level_table_keys`) so a *nested* table value's
/// fields (e.g. `config = { timeout = 30 }`) aren't mistaken for top-level
/// exports of the outer table.
static TABLE_KEY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)(?:^\s*|,\s*)(\w+)\s*=").unwrap());

/// Top-level `function foo(` with no table prefix — the plain
/// global-function style used by scripts with no module table at all.
/// Anchored at column zero so it never matches `local function helper(`
/// (which starts with `local`) or a nested/indented function.
static TOP_LEVEL_FUNCTION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^function\s+(\w+)\s*\(").unwrap());

/// Collect every `<module_name>.<member>` / `<module_name>:<member>`
/// attachment in source order, deduped, for the given module table name.
fn collect_module_members(content: &str, module_name: &str) -> Vec<String> {
    let mut hits: Vec<(usize, String)> = Vec::new();

    for caps in FUNCTION_MEMBER.captures_iter(content) {
        let table = caps.get(1).unwrap().as_str();
        if table == module_name {
            let member = caps.get(2).unwrap();
            hits.push((member.start(), member.as_str().to_string()));
        }
    }

    for caps in ASSIGN_MEMBER.captures_iter(content) {
        let table = caps.get(1).unwrap().as_str();
        if table == module_name {
            let member = caps.get(2).unwrap();
            hits.push((member.start(), member.as_str().to_string()));
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

/// Collect `key = ` entries from a `return { ... }` table body, but only
/// ones sitting directly in the outer table (brace depth zero within
/// `block`, which already excludes the outer `{`/`}` themselves). A nested
/// table value, e.g. `config = { timeout = 30, retries = 3 }`, has its own
/// `key = ` entries at depth one; those describe `config`'s shape, not the
/// module's public surface, and must not be reported as top-level exports.
fn extract_top_level_table_keys(block: &str) -> Vec<String> {
    // depth_at[i] = brace nesting depth of the content *before* byte i.
    let bytes = block.as_bytes();
    let mut depth_at = vec![0i32; bytes.len() + 1];
    let mut depth = 0i32;
    for (i, b) in bytes.iter().enumerate() {
        depth_at[i] = depth;
        match b {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            _ => {}
        }
    }
    depth_at[bytes.len()] = depth;

    let mut symbols = Vec::new();
    for caps in TABLE_KEY.captures_iter(block) {
        let whole = caps.get(0).unwrap();
        if depth_at[whole.start()] != 0 {
            continue;
        }
        if let Some(name) = caps.get(1) {
            let n = name.as_str().to_string();
            if !symbols.contains(&n) {
                symbols.push(n);
            }
        }
    }
    symbols
}

/// Extract exported symbols from Lua source code.
///
/// Lua has no built-in visibility keywords, so this follows the real-world
/// conventions used by actual Lua libraries, in priority order:
///
/// 1. Named module table: `local M = {}` with members attached via
///    `function M.foo(...)`, `function M:foo(...)`, or `M.foo = ...`,
///    confirmed by a top-level `return M`.
/// 2. Anonymous table literal returned directly: `return { foo = foo, ... }`.
///    Checked *before* the unconfirmed named-module-table fallback below:
///    both (1) and (2) are confirmed by an actual `return` statement, just in
///    named-variable vs. anonymous-literal style, so either one is strictly
///    more authoritative than a table that merely exists with no return at
///    all. Checking this after that fallback let an unrelated local table
///    that happens to have members attached (e.g. an internal `cache`) win
///    outright and hijack the real, returned anonymous table's export list.
/// 3. Same named-module-table pattern without a confirming `return`
///    (partial files/fragments still following the convention).
/// 4. Plain global-function-style scripts: every non-`local` top-level
///    `function foo() ... end` is public; `local function helper()` is not.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = LONG_COMMENT.replace_all(content, "");
    // Long-bracket string literals (`[[ ... ]]`) can only remain in the text
    // at this point if they weren't part of a `--[[ ... ]]` comment (those
    // were just stripped above), so anything this matches now is a genuine
    // string, not code — strip it so example/doc text embedded in a string
    // can't be mistaken for a real top-level declaration.
    let stripped = LONG_STRING.replace_all(&stripped, "");
    let stripped = LINE_COMMENT.replace_all(&stripped, "");

    // 1. Explicit `return M` naming a locally-declared module table.
    if let Some(caps) = RETURN_MODULE.captures(&stripped) {
        let module_name = caps.get(1).unwrap().as_str();
        let members = collect_module_members(&stripped, module_name);
        if !members.is_empty() {
            return members;
        }
    }

    // 2. Anonymous table literal returned directly. An explicit `return`,
    //    named or anonymous, always outranks an unconfirmed table below.
    if let Some(caps) = RETURN_TABLE_BLOCK.captures(&stripped) {
        let block = caps.get(1).unwrap().as_str();
        let symbols = extract_top_level_table_keys(block);
        if !symbols.is_empty() {
            return symbols;
        }
    }

    // 3. No confirmed `return M`, but a `local M = {}` table with members
    //    attached anyway (e.g. a file fragment, or an unconventional file
    //    that never returns its module table).
    for caps in MODULE_TABLE_DECL.captures_iter(&stripped) {
        let module_name = caps.get(1).unwrap().as_str();
        let members = collect_module_members(&stripped, module_name);
        if !members.is_empty() {
            return members;
        }
    }

    // 4. Fallback: plain global-function-style script, no module table.
    let mut symbols = Vec::new();
    for caps in TOP_LEVEL_FUNCTION.captures_iter(&stripped) {
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
    fn test_lua_module_table_dot_and_assign_forms() {
        let src = r#"
local M = {}

function M.connect(host, port)
    return true
end

M.disconnect = function()
    return true
end

local function internal_retry()
    return false
end

return M
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"connect".to_string()));
        assert!(symbols.contains(&"disconnect".to_string()));
        assert!(!symbols.contains(&"internal_retry".to_string()));
        assert!(!symbols.contains(&"M".to_string()));
    }

    #[test]
    fn test_lua_colon_method_syntax() {
        let src = r#"
local Client = {}
Client.__index = Client

function Client.new(name)
    return setmetatable({ name = name }, Client)
end

function Client:send(msg)
    return self.name .. msg
end

local function validate(msg)
    return #msg > 0
end

return Client
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"new".to_string()));
        assert!(symbols.contains(&"send".to_string()));
        assert!(!symbols.contains(&"validate".to_string()));
    }

    #[test]
    fn test_lua_module_constants_captured() {
        let src = r#"
local M = {}

M.VERSION = "1.2.3"
M.DEFAULT_TIMEOUT = 30

function M.configure(opts)
    return opts
end

return M
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["VERSION", "DEFAULT_TIMEOUT", "configure"]);
    }

    #[test]
    fn test_lua_comment_stripping_safety() {
        let src = r#"
local M = {}

--[[
function M.fake_from_block_comment()
    return true
end
]]

-- function M.fake_from_line_comment() end

function M.real()
    return true
end

return M
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["real"]);
        assert!(!symbols.contains(&"fake_from_block_comment".to_string()));
        assert!(!symbols.contains(&"fake_from_line_comment".to_string()));
    }

    #[test]
    fn test_lua_anonymous_return_table() {
        // A common minimalist module style: build everything as locals,
        // then export via a single anonymous table literal at the end
        // rather than naming a module table up front.
        let src = r#"
local function connect()
    return true
end

local function disconnect()
    return true
end

local function internal_helper()
    return false
end

return {
    connect = connect,
    disconnect = disconnect,
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["connect", "disconnect"]);
        assert!(!symbols.contains(&"internal_helper".to_string()));
    }

    #[test]
    fn test_lua_global_function_style_script_no_module_table() {
        // Older/simpler Lua scripts with no module table at all: every
        // non-local top-level function is the public API.
        let src = r#"
function setup()
    return true
end

local function helper()
    return false
end

function teardown()
    return true
end
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["setup", "teardown"]);
        assert!(!symbols.contains(&"helper".to_string()));
    }

    #[test]
    fn test_lua_nested_function_not_top_level() {
        // A function defined inside another function's body (indented) is
        // not a top-level declaration and must not leak into the global
        // fallback's results.
        let src = r#"
function outer()
    function inner()
        return 1
    end
    return inner()
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"outer".to_string()));
        assert!(!symbols.contains(&"inner".to_string()));
    }

    #[test]
    fn test_lua_module_table_without_explicit_return() {
        // A file fragment that still follows the module-table convention
        // (attach to a locally-declared table) but never actually returns
        // it should still surface the attached members.
        let src = r#"
local Utils = {}

function Utils.trim(s)
    return s
end

function Utils.split(s, sep)
    return {}
end
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["trim", "split"]);
    }

    #[test]
    fn test_lua_multiline_block_comment_with_dashes_inside() {
        // Long comments can contain lines that themselves start with `--`;
        // the whole block must be stripped before line-comment stripping
        // runs, or a truncated leftover could be mistaken for code.
        let src = r#"
local M = {}

--[[
  Changelog:
  -- v1: initial release
  -- v2: added retry logic
]]

function M.run()
    return true
end

return M
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["run"]);
    }

    #[test]
    fn test_lua_unrelated_local_table_not_leaked() {
        // A second, unrelated local table (e.g. internal config/state) that
        // is never returned and has no attached members must not surface
        // anything, and its field assignments must not be attributed to
        // the real, returned module table.
        let src = r#"
local M = {}
local cache = {}

cache.hits = 0
cache.misses = 0

function M.get(key)
    return cache[key]
end

return M
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["get"]);
        assert!(!symbols.contains(&"hits".to_string()));
        assert!(!symbols.contains(&"misses".to_string()));
    }

    #[test]
    fn test_lua_unrelated_local_table_does_not_hijack_anonymous_return() {
        // Regression test: an unrelated local table with members attached (tier 3, the
        // "unconfirmed named module table" fallback) previously ran *before* the
        // anonymous `return { ... }` check (tier 2), so a `cache` table that merely
        // exists with attached fields incorrectly won outright and hijacked the real,
        // returned anonymous table's export list.
        let src = r#"
local cache = {}
cache.hits = 0
cache.misses = 0

local function connect()
    return true
end

return {
    connect = connect,
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["connect"]);
        assert!(!symbols.contains(&"hits".to_string()));
        assert!(!symbols.contains(&"misses".to_string()));
    }

    #[test]
    fn test_lua_anonymous_return_table_nested_table_value_not_leaked() {
        // A nested table *value* (e.g. a `config` sub-table) has its own
        // `key = value` entries. Those describe the nested table's shape,
        // not the outer module's public surface, and must not be reported
        // as top-level exports alongside `connect`/`config`.
        let src = r#"
local function connect()
    return true
end

return {
    connect = connect,
    config = {
        timeout = 30,
        retries = 3,
    },
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["connect", "config"]);
        assert!(!symbols.contains(&"timeout".to_string()));
        assert!(!symbols.contains(&"retries".to_string()));
    }

    #[test]
    fn test_lua_long_string_literal_fake_module_member_not_leaked() {
        // A long-bracket string literal (`[[ ... ]]`) used to hold example
        // or usage text is not a comment, but it must be treated like one:
        // code-shaped text inside it (here, referencing the module table by
        // name) must never be mistaken for a real attached member.
        let src = r#"
local M = {}

local usage = [[
function M.fake_from_doc()
    print("example usage")
end
]]

function M.real()
    return true
end

return M
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["real"]);
        assert!(!symbols.contains(&"fake_from_doc".to_string()));
    }

    #[test]
    fn test_lua_long_string_literal_fake_top_level_function_not_leaked() {
        // Same hazard for the no-module-table fallback style: a `--help`
        // usage string that happens to contain a line starting with
        // `function help(` at column zero must not be reported as one of
        // the script's real top-level functions.
        let src = r#"
local usage = [[
function help()
    print("usage")
end
]]

function setup()
    return true
end
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["setup"]);
        assert!(!symbols.contains(&"help".to_string()));
    }
}
