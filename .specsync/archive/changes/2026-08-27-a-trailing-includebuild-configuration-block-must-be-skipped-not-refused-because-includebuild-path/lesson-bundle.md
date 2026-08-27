# Lesson bundle — a-trailing-includebuild-configuration-block-must-be-skipped-not-refused-because-includebuild-path

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: A trailing includeBuild configuration block must be skipped, not refused, because includeBuild(path) { dependencySubstitution { ... } } is the common spelling and contributes no module
- **Kind**: BugFix
- **Specs**: manifest
- **Paths**: src/manifest.rs, tests/integration/commands.rs, specs/manifest/manifest.spec.md, specs/manifest/requirements.md, specs/manifest/context.md, specs/manifest/tasks.md, specs/manifest/testing.md
- **Acceptance**: parse_gradle_settings accepts includeBuild("vendor/shared") { dependencySubstitution { ... } } — one line and multi-line — and the composite build contributes no module, so a sibling include(":app") is still discovered. A real Gradle fixture using that spelling runs coverage successfully and reports its modules instead of degrading to a manifest notice or an inconclusive gate. Every refusal that guarded the path survives with an argument-naming diagnostic: includeBuild("../outside") { ... } and every rooted or traversing path still fails beneath-the-root, interpolated and dynamic arguments still fail dynamic, more than one path argument still fails one-literal-path, and an unbalanced trailing block fails with an unbalanced-braces diagnostic. Braces inside string literals and inside comments do not move the balance scan in either direction. Everything inside the block remains subject to every other guard: a block-scoped include, a projectDir mutation, or an unrecognized project mutation written inside the block still fails closed.

## Evidence

- Verification commit: `883e33c58f0903c9e82c6173bc6ed4a75b1f86c2`
- Base commit: `fe55a2179ca298ad1ca4e8fc7b7465890b85cc75`
- Verified by: `ruby --version`, `cargo test`, `python3 -S .github/scripts/validate-release-version.py`, `python3 -S .github/scripts/validate-workflow-runtime-pins.py`

## From the change's context.md

# Context

## What led here

Issue #725, a follow-up from the same adopter as #723, reproduced against the published rc.8
binary. Their settings file says:

```kotlin
includeBuild("vendor/podo-shared") {
    dependencySubstitution {
        substitute(module("com.example:podo")).using(project(":"))
    }
}
```

and discovery answers `Unsupported trailing Gradle includeBuild declaration expression`.

#723 taught the parser to judge `includeBuild` by its path argument instead of its token, and
deliberately kept refusing a trailing configuration block. That left the parser accepting the bare
`includeBuild(path)` minority form and refusing the form almost every composite build actually
uses — substituting a local project for a published coordinate is the reason to declare one.

## Why this is not an outage

#723 also fixed the precedence rule that let a discovery failure veto a stated `source_dirs`. That
class-level fix caught this instance nobody anticipated: the adopter now gets a notice naming the
file, the reason, the fallback, and the consequence, and coverage completes on the configured
`source_dirs`. So this is correctness work, not an incident — and nothing here may be traded for
it. Every refusal that protected the path stays exactly as strict.

## Constraints

- An `includeBuild` contributes no module and no source directory, block or no block. The block
  carries substitution rules, not project declarations. That is the entire argument for skipping
  it, and it is why skipping loses no information.
- The path argument is parsed from inside the parentheses, in front of the block. Skipping the
  block must not become a way to carry `../outside`, an interpolation, or a second argument past
  that check.
- The block's text stays in the parsed content. `reject_non_leading_gradle_includes` resumes its
  scan from the declaration token and `reject_unsupported_gradle_project_dir_mutations` runs over
  the whole file afterwards, so a governed directive written inside the block still fails closed.
  This was asserted, not assumed.
- Comments are already stripped before any of this runs, so a brace in a comment is gone. A brace
  in a *string literal* is not, so the brace scan has to be quote- and escape-aware.
- An unbalanced block must fail: its extent is exactly what is unknown.

## Already ruled out

- Accepting a `{` that opens on a later line. The block must open on the declaration's own line, so
  an unrelated following block cannot be mistaken for one. `if (x) { includeBuild("p") }` opens no
  block of its own and keeps its existing treatment.
- Touching `includeFlat` or `includeWorkspace`. `includeFlat` resolves against the parent of the
  root, so its argument is outside the project by construction; `includeWorkspace` is not a form
  this parser models. Neither becomes supportable by reading an argument or skipping a block.
- Supporting the bare Groovy `includeBuild 'path' { … }` spelling, which is not valid Groovy call
  syntax. It keeps failing closed.

## From the change's design.md

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

## From the change's testing.md

# Testing

## New and changed tests

| Test | File | Claim |
|------|------|-------|
| `manifest::tests::gradle_settings_accept_an_include_build_with_a_configuration_block` | `src/manifest.rs` | The reported multi-line form, the one-line form, Groovy quotes, `settings.` qualification, a trailing `;`, a normalizing path, a `{` inside a string literal, a `}` inside a line comment, a `}` inside a block comment, and both conditional spellings all parse — and each yields exactly the sibling `:app` module, so the composite build contributed nothing |
| `manifest::tests::gradle_settings_still_judge_directives_inside_an_include_build_block` | `src/manifest.rs` | A block-scoped `include`, an `includeFlat`, a `projectDir` mutation, and a `setProperty` mutation written inside the block still fail closed — the skip locates the declaration's end and hides nothing |
| `manifest::tests::gradle_settings_still_refuse_an_escaping_or_dynamic_include_build` | `src/manifest.rs` | Extended: `includeBuild("../outside") { … }` (one line and multi-line), an interpolated path with a block, two path arguments with a block, an unbalanced block, a `}` inside a string literal that must not balance one, and a non-block trailing expression |
| `commands::gradle_include_build_with_a_configuration_block_discovers_modules` | `tests/integration/commands.rs` | A real Gradle fixture using the reported spelling: `coverage` succeeds, `manifest_notices` is empty, `app` is reported, and `vendor/shared` is not measured |

## Failing against `main`

The helper was disabled in place (an early `return Ok(remainder)`) and the suite re-run. Three unit
tests and the integration test failed; every message was the refusal this change removes:

- `gradle_settings_accept_an_include_build_with_a_configuration_block` —
  `includeBuild with a configuration block refused: Unsupported trailing Gradle includeBuild
  declaration expression` on the reported multi-line fixture.
- `gradle_settings_still_judge_directives_inside_an_include_build_block` —
  `unexpected error … : Unsupported trailing Gradle includeBuild declaration expression` (the block
  was refused before its contents were ever judged).
- `gradle_settings_still_refuse_an_escaping_or_dynamic_include_build` —
  `unexpected includeBuild error for includeBuild("../outside") { dependencySubstitution {} }:
  Unsupported trailing Gradle includeBuild declaration expression` (refused, but for the block
  rather than for the escape it actually contains).
- `commands::gradle_include_build_with_a_configuration_block_discovers_modules` — `coverage` exited
  1 instead of reporting modules.

## Honest labels

The refusal fixtures added here are **controls**: every one is refused on the unfixed parser too.
What changes is the reason. On `main` they were refused because the block was refused, which said
nothing about the argument and would have said the same about a safe path. Now the block is skipped,
so the refusal has to come from the argument or not at all — that is what these assert.

## Commands

- `cargo test --bin specsync manifest::tests::gradle`
- `cargo test --test integration gradle_include_build`
- `cargo clippy -- -D warnings` (bare — `--all-targets` has pre-existing failures)
- `cargo fmt --check`

## Where these lessons go

- `specs/manifest/context.md`
