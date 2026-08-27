---
change: a-trailing-includebuild-configuration-block-must-be-skipped-not-refused-because-includebuild-path
artifact: testing
---

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
