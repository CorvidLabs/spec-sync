---
change: a-trailing-includebuild-configuration-block-must-be-skipped-not-refused-because-includebuild-path
artifact: design
---

# Design

## Shape

One new private helper in `src/manifest.rs`, called from one place.

```rust
fn skip_gradle_include_build_configuration_block(remainder: &str) -> Result<&str, String>
```

`gradle_include_build_target` already splits the declaration into `(inside)` — the parenthesized
argument list — and `remainder`, the text after the closing parenthesis. The helper is inserted on
`remainder` only:

```rust
let (inside, remainder) = gradle_parenthesized(arguments)?;
let remainder = skip_gradle_include_build_configuration_block(remainder)?;
let statement_remainder = remainder.split_once('\n').map_or(remainder, |(line, _)| line);
require_gradle_include_build_statement_end(statement_remainder)?;
gradle_string_arguments(inside)?
```

`inside` is untouched, so the literal-only parse, the arity check, and
`normalize_gradle_project_relative_path` all still run on exactly the bytes they ran on before.

## Behavior of the helper

1. Trim horizontal whitespace only (` `, `\t`, `\r`). A `{` must open on the declaration's own
   line; a newline ends the search.
2. If the next character is not `{`, return the input unchanged. This is what keeps the bare form
   and the enclosing-`}` case (`if (x) { includeBuild("p") }`) exactly as they were.
3. Otherwise scan forward with a signed depth counter, quote state, and escape state — the same
   idiom `gradle_brace_depth` and `gradle_parenthesized` already use in this file. Return the slice
   after the matching `}`.
4. Reaching the end without returning to depth zero is
   `Gradle includeBuild configuration block has unbalanced braces`.

Comments need no handling: `strip_gradle_comments` runs before `reject_non_leading_gradle_includes`,
so a brace inside `//` or `/* */` is already a space by the time the scan sees it.

## Why the skip cannot weaken anything

- **Path confinement.** The path comes from `inside`, which precedes the block and is never
  rewritten. `includeBuild("../outside") { … }` fails on the path exactly as
  `includeBuild("../outside")` does — the diagnostic even improves, because it now names the path
  rather than the trailing expression.
- **Other directives.** Nothing is removed from `content`. The caller's loop advances from the
  declaration token, so an `include` or `includeFlat` inside the block is still found;
  `reject_unsupported_gradle_project_dir_mutations` then walks the whole file, so a `projectDir` or
  `project(...)` mutation inside the block is still found. Both are asserted directly.
- **Worst case.** Even a mis-scanned extent can only drop trailing text from the statement-end
  verdict. It cannot add, remove, or alter an argument, and it cannot hide a directive from the
  guards that scan the file independently.

## Rejected alternatives

- **Search for the next `}`.** Fails on nested blocks, which `dependencySubstitution { … }` always
  has, and on braces inside string literals.
- **Allow the block to open on a later line.** Kotlin rarely writes it that way, and it would let
  an unrelated following block be mistaken for the declaration's own.
- **Parse the block's contents.** There is nothing in it this parser needs: substitution rules
  declare no project and no source directory.
