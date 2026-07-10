---
module: exports
version: 1
status: stable
files:
  - src/exports/mod.rs
  - src/exports/typescript.rs
  - src/exports/python.rs
  - src/exports/rust_lang.rs
  - src/exports/go.rs
  - src/exports/java.rs
  - src/exports/kotlin.rs
  - src/exports/swift.rs
  - src/exports/dart.rs
  - src/exports/csharp.rs
  - src/exports/php.rs
  - src/exports/ruby.rs
  - src/exports/yaml.rs
  - src/exports/c.rs
  - src/exports/cpp.rs
  - src/exports/scala.rs
  - src/exports/crystal.rs
  - src/exports/nim.rs
  - src/exports/erlang.rs
  - src/exports/elixir.rs
  - src/exports/perl.rs
  - src/exports/lisp.rs
  - src/exports/ast/mod.rs
  - src/exports/ast/typescript.rs
  - src/exports/ast/python.rs
  - src/exports/ast/rust_lang.rs
  - src/exports/ast/c.rs
  - src/exports/ast/cpp.rs
  - src/exports/ast/scala.rs
  - src/exports/ast/erlang.rs
  - src/exports/ast/elixir.rs
  - src/exports/ast/perl.rs
  - src/exports/ast/lisp.rs
  - src/exports/haskell.rs
  - src/exports/lua.rs
  - src/exports/r.rs
  - src/exports/ocaml.rs
  - src/exports/groovy.rs
  - src/exports/fsharp.rs
  - src/exports/clojure.rs
  - src/exports/d.rs
  - src/exports/objective_c.rs
  - src/exports/bash.rs
  - src/exports/powershell.rs
  - src/exports/vala.rs
db_tables: []
tracks: [60]
depends_on:
  - specs/types/types.spec.md
---

# Exports

## Purpose

Language-aware export extraction from source files. Auto-detects the programming language from file extension and extracts public/exported symbol names using regex-based parsing or tree-sitter AST analysis. Supports 33 languages: TypeScript/JS, Rust, Go, Python, Swift, Kotlin, Java, C#, Dart, PHP, Ruby, YAML, C, C++, Scala, Crystal, Nim, Erlang, Elixir, Perl, Lisp (Common Lisp/Scheme/Emacs Lisp), Haskell, Lua, R, OCaml, Groovy, F#, Clojure, D, Objective-C, Bash, PowerShell, and Vala.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `get_exported_symbols` | `file_path: &Path` | `Vec<String>` | Extract exported symbol names from a source file, auto-detecting language from extension |
| `get_exported_symbols_with_level` | `file_path: &Path, level: ExportLevel` | `Vec<String>` | Extract exports with configurable granularity — Type (declarations only) or Member (all symbols) |
| `is_test_file` | `file_path: &Path, root: &Path` | `bool` | Check if a file is a test file by filename convention or a test directory *within `root`*; the directory check is bounded to project-relative components so ancestors above `root` (e.g. a parent dir named `spec`/`test`) do not misclassify ordinary sources |
| `is_source_file` | `file_path: &Path` | `bool` | Check if a file extension belongs to a supported source language |
| `has_extension` | `file_path: &Path, extensions: &[String]` | `bool` | Check if file matches specific extensions, or any supported language if extensions is empty |
| `extract_exports` | `content: &str` | `Vec<String>` | Per-language backend function that parses source text and returns exported symbol names (one per backend file) |
| `extract_exports_with_resolver` | `content: &str, resolver: Option<&ImportResolver>` | `Vec<String>` | TypeScript-specific: extract exports with optional wildcard re-export resolution via file resolver callback |
| `get_exported_symbols_full` | `file_path: &Path, level: ExportLevel, parse_mode: ParseMode` | `Vec<String>` | Extract exports with full control over granularity and parse mode (Regex or Ast); a read/parse failure or unsupported language yields an empty vector |
| `scan_exported_symbols` | `file_path: &Path` | `ExportScan` | Like `get_exported_symbols` but returns an `ExportScan`, distinguishing a genuine empty result from an unreadable/unsupported file |
| `scan_exported_symbols_full` | `file_path: &Path, level: ExportLevel, parse_mode: ParseMode` | `ExportScan` | Like `get_exported_symbols_full` but returns an `ExportScan` so gating callers (`diff`, `score`) can tell a failure from genuine emptiness |

### Exported Types

| Type | Description |
|------|-------------|
| `ExportScan` | Outcome of an extraction attempt: `Parsed(Vec<String>)` (recognized language, read + parsed — the vec may be empty, meaning genuinely no exports), `UnknownLanguage` (extension not a source language, e.g. a `.md`/`.sql` file — not a failure), or `Unreadable` (missing / permission-denied / non-UTF-8 — exports unknown). Lets gating callers avoid treating an unreadable file as export-free. |

### Exported Modules

| Module | Source | Description |
|--------|--------|-------------|
| `ast` | `src/exports/mod.rs` | Tree-sitter based AST export extraction backends |

### Exported AST Sub-modules

Tree-sitter based export extraction backends for TypeScript, Python, Rust, C, C++, Scala, Erlang, Elixir, Perl, and Lisp (Common Lisp/Scheme/Emacs Lisp). Used when `ParseMode::Ast` is selected. Falls back to regex extraction for unsupported languages (Nim, Crystal — no published tree-sitter grammar) or when AST parsing returns nothing.

| Sub-module | File | Description |
|------------|------|-------------|
| `typescript` | `ast/typescript.rs` | Tree-sitter based TypeScript/JS export extraction with wildcard resolver support |
| `python` | `ast/python.rs` | Tree-sitter based Python export extraction using `__all__` and top-level definitions |
| `rust_lang` | `ast/rust_lang.rs` | Tree-sitter based Rust `pub` item extraction |
| `c` | `ast/c.rs` | Tree-sitter based C export extraction: top-level struct/union/enum names and non-static function definitions/declarations |
| `cpp` | `ast/cpp.rs` | Tree-sitter based C++ export extraction: class/struct/union/enum/namespace names (including nested types) and non-static, non-private/protected functions and methods |
| `scala` | `ast/scala.rs` | Tree-sitter based Scala export extraction: class/object/trait/def/val/var declarations excluding `private`/`protected` |
| `erlang` | `ast/erlang.rs` | Tree-sitter based Erlang export extraction: function names listed in `-export([...])` attributes |
| `elixir` | `ast/elixir.rs` | Tree-sitter based Elixir export extraction: `defmodule`/`def`/`defmacro` excluding `defp`/`defmacrop` |
| `perl` | `ast/perl.rs` | Tree-sitter based Perl export extraction: all `sub` declarations |
| `lisp` | `ast/lisp.rs` | Tree-sitter based Lisp export extraction across three dialects (Common Lisp, Scheme, Emacs Lisp, dispatched by file extension): `defun`/`defmacro`/`defvar`/`defparameter` forms |

### Language Backend Functions

Each language backend exposes a single `extract_exports(content: &str) -> Vec<String>` function that parses source code and returns exported symbol names. These are internal to the exports module (not re-exported) and called by `get_exported_symbols`.

| Backend | File | Extraction Strategy |
|---------|------|-------------------|
| TypeScript/JS | `typescript.rs` | `export function/class/interface/type/const/enum`, re-exports (`export { }`, `export type { }`), wildcard re-exports (`export * from`, `export * as Ns from`), default exports (`export default class/function`), and CommonJS-style `export = Name`; with `as` alias support; strips `//` and `/* */` comments |
| Python | `python.rs` | Uses `__all__` list if present; otherwise top-level `def`/`class`/`async def` not prefixed with `_` |
| Rust | `rust_lang.rs` | `pub fn/struct/enum/trait/type/const/static/mod` including `pub async/unsafe`; excludes restricted visibility `pub(crate)`, `pub(super)`, `pub(self)`, and `pub(in path::to::mod)`; strips comments |
| Go | `go.rs` | Top-level `func/type/var/const` starting with uppercase letter; also exported methods `func (receiver) Name()`; strips comments |
| Java | `java.rs` | `public class/interface/enum/record/@interface` types and `public` methods/fields; handles `static`, `final`, `abstract`, `sealed` modifiers |
| Kotlin | `kotlin.rs` | All top-level `fun/class/object/interface/typealias/val/var/enum class/data class/sealed class` unless marked `private`/`internal`/`protected`; handles `suspend`/`inline` modifiers |
| Swift | `swift.rs` | `public`/`open` declarations: `func/class/struct/enum/protocol/typealias/var/let/actor`; detects `public init` and `public subscript` separately; recognizes that protocol requirements and `public extension` members don't repeat the container's access keyword — scans protocol/extension/enum bodies (associatedtype, subscript, func, var/let, `case` lines) at brace-depth 0, descending past nested blocks without scanning inside them; handles `static class func` |
| Dart | `dart.rs` | `class/mixin/enum/extension/typedef` types, `final`/`const` declarations, top-level functions; excludes `_`-prefixed private identifiers |
| C# | `csharp.rs` | `public class/struct/interface/enum/record/delegate` types and `public` members; handles `static`, `partial`, `sealed`, `abstract`, `virtual`, `override`, `async` modifiers |
| PHP | `php.rs` | `class/interface/trait/enum` types (always public); `public`/unqualified `function` and `const` declarations; skips `private`/`protected` members and `__` magic methods; handles `abstract`, `final`, `readonly`, `static` modifiers; strips `//`, `/* */`, and `#` comments |
| Ruby | `ruby.rs` | `class`/`module` declarations; top-level `def` (always public); class methods with visibility tracking (`public`→`private`→`protected`→`public` toggles); `CONSTANT` assignments; `attr_accessor`/`attr_reader`/`attr_writer` symbols; skips `_`-prefixed names and `initialize`; strips `#` and `=begin/=end` comments |
| YAML | `yaml.rs` | Top-level mapping keys from `.yaml`/`.yml` files; named entries under well-known parent keys (e.g., `jobs.test`, `services.web`); YAML anchors (`&name`) |
| C | `c.rs` | Top-level `struct/union/enum` names and non-`static` function definitions/declarations; strips comments |
| C++ | `cpp.rs` | `class/struct/union/enum/namespace` names plus methods declared under a `public:` section (class defaults private, struct defaults public), excluding `static`/`private:`/`protected:`; strips comments |
| Scala | `scala.rs` | `class/object/trait/def/val/var` declarations at any nesting depth (line-based scan), excluding lines starting with `private`/`protected`; strips comments |
| Crystal | `crystal.rs` | `class/module/struct/enum/def/alias` declarations, public by default (no access keyword required) |
| Nim | `nim.rs` | Exported symbols use Nim's explicit `*` suffix convention, e.g. `proc foo*(...)`, `type Bar* = ...` |
| Erlang | `erlang.rs` | Function names listed in one or more `-export([name/arity, ...])` attributes only — not every defined function, just the declared export list |
| Elixir | `elixir.rs` | `defmodule`/`def`/`defmacro` declarations, excluding `defp`/`defmacrop` |
| Perl | `perl.rs` | Every `sub` declaration (Perl has no built-in per-sub privacy convention) |
| Lisp | `lisp.rs` | `defun`/`defmacro`/`defvar`/`defparameter` forms, shared across three dialects dispatched by extension: Common Lisp (`.lisp`/`.lsp`), Scheme (`.scm`), Emacs Lisp (`.el`) |
| Haskell | `haskell.rs` | Explicit module export list `module Foo (bar, Baz(..)) where` takes precedence when present; with no export list, every top-level `data/newtype/type/class/instance` and function binding is exported (Haskell's actual "no list = everything public" rule); character-literal-aware comment/string stripping so a quote inside `'"'` doesn't desync the scanner |
| Lua | `lua.rs` | Detects the idiomatic `local M = {}` ... `function M.foo()` / `M.foo = function()` ... `return M` module-table pattern; also supports bare top-level `function foo()` scripts and the anonymous `return { key = val }` table idiom (brace-depth aware so nested table values aren't leaked as top-level exports); strips `--`/`--[[ ]]` comments and `[[ ]]` long-bracket strings |
| R | `r.rs` | roxygen2 `#' @export` tags (column-0 anchored) take precedence when present; falls back to top-level `name <- function(...)` / `name = function(...)` / `name <- \(...)` not prefixed with `.` (R's internal-symbol convention) |
| OCaml | `ocaml.rs` | Top-level `let`/`let rec`/`and`/`type`/`module`/`exception` bindings, public by default (mirrors OCaml's real rule: with no `.mli` signature file, everything in the `.ml` is public); comment stripping preserves column-0 position so a same-line leading comment doesn't hide the declaration that follows it |
| Groovy | `groovy.rs` | `class/interface/trait/enum` types and `def`/typed methods, public by default (Groovy's actual default, unlike Java's package-private default) unless marked `private`/`protected`; brace-depth scope tracking (annotation-aware) so method-local variables aren't leaked as class members |
| F# | `fsharp.rs` | `let`/`let rec`/`and`/`type`/`module` bindings, public by default, excluding those marked `private` |
| Clojure | `clojure.rs` | `defn`/`def`/`defrecord`/`deftype`/`defprotocol`/`defmacro` forms, excluding `defn-`/`def-` (Clojure's trailing-dash private convention) and `^:private`-tagged forms |
| D | `d.rs` | `class/struct/interface/enum` types and functions, public by default (D's actual module-scope default) unless marked `private`/`protected`/`package` |
| Objective-C | `objective_c.rs` | `@interface`/`@implementation`/`@protocol` class and protocol names plus `-`/`+` method signatures within `@implementation`/`@protocol` blocks; best-effort approximation since Objective-C has no private-method keyword (real visibility is header-vs-implementation architecture, not scanned here) |
| Bash | `bash.rs` | `export -f name` statements (if any exist) are the authoritative export list; otherwise every `function name`/`name()` declaration not prefixed with `_` |
| PowerShell | `powershell.rs` | `Export-ModuleMember -Function ...` statements (if any exist) are the authoritative export list; otherwise every `function Verb-Noun { ... }` declaration |
| Vala | `vala.rs` | `public class/struct/interface/enum/delegate` types and `public` methods/auto-properties, excluding `private`/`protected`/`internal` |

## Invariants

1. Language detection is purely extension-based — no content inspection needed
2. Symbols are deduplicated while preserving order
3. Unreadable files or unknown extensions return an empty vector (never panic)
4. `has_extension` with an empty extensions list delegates to `is_source_file` (matches all supported languages)
5. Test file detection uses language-specific patterns (e.g. `.test.ts`, `_test.go`, `Test.java`)
6. Each language backend uses `LazyLock<Regex>` for compiled patterns — compiled once, reused across calls
7. TypeScript backend handles `export function/class/type/const/enum/interface`, re-exports, wildcard re-exports (`export * from`), namespace re-exports (`export * as Ns from`), default exports, and CommonJS-style `export = Name`
7a. Wildcard `export * from './module'` is resolved via `resolve_ts_import` which tries .ts/.tsx/.js/.jsx/.mts/.cts extensions and /index.ts etc.
7b. Wildcard resolution is one level deep — resolved modules are parsed without a resolver to avoid infinite loops
7c. `export * as Ns from './module'` emits the namespace name (Ns) as the export, not the individual symbols
7d. Without a resolver (e.g. in unit tests), wildcard `export *` lines are silently skipped
8. Rust backend extracts only plain `pub fn/struct/enum/trait/type/const/static/mod` items; `pub(crate)`, `pub(super)`, `pub(self)`, and `pub(in path::to::mod)` are not treated as exported
9. Go backend extracts uppercase (exported) identifiers and methods
10. Python backend uses `__all__` if present, otherwise top-level non-underscore `def/class`
11. Swift backend distinguishes `public` and `open` visibility (both are exported)
12. Kotlin treats everything as public by default unless marked `private`/`internal`/`protected`
13. Dart treats everything as public by default unless prefixed with `_`
14. Java and C# backends require explicit `public` keyword for exports
15. All backends strip single-line (`//`) and multi-line (`/* */`) comments before extraction (except Python which doesn't use this pattern)
16. Go backend deduplicates methods that might also match top-level declarations
17. PHP backend treats types (class/interface/trait/enum) as always public; methods and constants require `public` or unqualified visibility; `private`/`protected` are excluded; magic methods (`__construct`, `__toString`, etc.) are excluded
18. Ruby backend tracks visibility state via `public`/`private`/`protected` toggle statements; defaults to public; `initialize` is excluded; `_`-prefixed names are excluded; `attr_accessor`/`attr_reader`/`attr_writer` emit attribute names as symbols
19. YAML backend extracts top-level mapping keys, named entries under well-known parent keys (any indentation level), and anchors; no test file patterns (YAML files are not test files)

## Behavioral Examples

### Scenario: Extract TypeScript exports

- **Given** a `.ts` file containing `export function authenticate(token: string): User`
- **When** `get_exported_symbols(path)` is called
- **Then** includes "authenticate" in the returned vector

### Scenario: Extract Rust pub items

- **Given** a `.rs` file containing `pub fn validate_spec(...)`
- **When** `get_exported_symbols(path)` is called
- **Then** includes "validate_spec" in the returned vector

### Scenario: Unsupported file type

- **Given** an unsupported file (e.g., `.lua`)
- **When** `get_exported_symbols(path)` is called
- **Then** returns an empty vector

### Scenario: Extract PHP exports with visibility

- **Given** a `.php` file with a `class AuthService` containing `public function validate()`, `private function internalCheck()`, and `public const DEFAULT_TTL`
- **When** `get_exported_symbols(path)` is called
- **Then** includes "AuthService", "validate", "DEFAULT_TTL" but not "internalCheck"

### Scenario: Ruby visibility toggles

- **Given** a `.rb` file with `class Foo` containing `def public_method` then `private` then `def secret_method`
- **When** `get_exported_symbols(path)` is called
- **Then** includes "Foo" and "public_method" but not "secret_method"

### Scenario: Python __all__ takes precedence

- **Given** a `.py` file with `__all__ = ["create_auth", "AuthService"]` and additional top-level functions
- **When** `get_exported_symbols(path)` is called
- **Then** returns only the symbols listed in `__all__`, not all top-level definitions

### Scenario: Go uppercase convention

- **Given** a `.go` file with `func CreateAuth()` and `func privateHelper()`
- **When** `get_exported_symbols(path)` is called
- **Then** includes "CreateAuth" but not "privateHelper"

### Scenario: Kotlin default visibility

- **Given** a `.kt` file with `fun publicFun()` and `private fun privateFun()`
- **When** `get_exported_symbols(path)` is called
- **Then** includes "publicFun" (public by default) but not "privateFun"

### Scenario: TypeScript re-exports with aliases

- **Given** a `.ts` file with `export { Foo as Bar }`
- **When** `get_exported_symbols(path)` is called
- **Then** includes "Bar" (the alias), not "Foo"

### Scenario: Wildcard re-export from barrel file

- **Given** a `.ts` barrel file containing `export * from './helpers'` and `helpers.ts` exports `helperA` and `helperB`
- **When** `get_exported_symbols(barrel_path)` is called
- **Then** includes "helperA" and "helperB" (resolved via `resolve_ts_import`)

### Scenario: Namespace re-export

- **Given** a `.ts` file containing `export * as Utils from './utils'`
- **When** `get_exported_symbols(path)` is called
- **Then** includes "Utils" (the namespace name), not the individual exports from `./utils`

### Scenario: Default export

- **Given** a `.ts` file containing `export default class MyApp {}`
- **When** `get_exported_symbols(path)` is called
- **Then** includes "MyApp"

### Scenario: CommonJS-style export

- **Given** a `.ts` file containing `class AuthService {}` and `export = AuthService`
- **When** `get_exported_symbols(path)` is called
- **Then** includes "AuthService"

### Scenario: Wildcard resolution is one level deep

- **Given** `top.ts` has `export * from './middle'` and `middle.ts` has `export * from './bottom'`
- **When** `get_exported_symbols(top_path)` is called
- **Then** includes symbols directly exported by `middle.ts` but NOT symbols from `bottom.ts` (no recursive resolution)

### Scenario: Comments are stripped before extraction

- **Given** a `.ts` file with `// export function notExported()` inside a comment
- **When** `get_exported_symbols(path)` is called
- **Then** does not include "notExported"

### Scenario: Test file detection

- **Given** a file named `auth.test.ts`
- **When** `is_test_file(path, root)` is called
- **Then** returns `true`

### Scenario: Test-named ancestor above the project root

- **Given** a project at `/home/user/spec/proj` with source `/home/user/spec/proj/src/app.ts`
- **When** `is_test_file(path, root)` is called with `root = /home/user/spec/proj`
- **Then** returns `false` — the `spec` component is above `root` and is not a project test directory

## Error Cases

| Condition | Behavior |
|-----------|----------|
| File cannot be read | Returns empty vector |
| Unknown file extension | Returns empty vector |
| File has no exports | Returns empty vector |
| Binary or non-text file | Returns empty vector (read_to_string fails gracefully) |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| types | `Language` enum for extension-to-language mapping |

### Consumed By

| Module | What is used |
|--------|-------------|
| validator | `get_exported_symbols`, `has_extension`, `is_test_file` |
| scoring | `get_exported_symbols` |
| generator | `has_extension`, `is_test_file` |
| config | `has_extension` |

## Change Log

| Date | Change |
|------|--------|
| 2026-03-25 | Initial spec |
| 2026-03-28 | Document get_exported_symbols_with_level |
| 2026-03-29 | Add PHP and Ruby language support |
| 2026-04-12 | Add YAML language support (yaml.rs) |
| 2026-07-09 | Add AST parse mode support for C, C++, Scala, Erlang, Elixir, Perl, and Lisp (Common Lisp/Scheme/Emacs Lisp) |
| 2026-07-09 | Filter Rust `pub(crate)`/`pub(super)`/`pub(self)`/`pub(in path)` from exports; add TypeScript `export = Name` support |
