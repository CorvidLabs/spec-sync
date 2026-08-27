---
change: a-trailing-includebuild-configuration-block-must-be-skipped-not-refused-because-includebuild-path
artifact: plan
---

# Plan

1. Add `skip_gradle_include_build_configuration_block` to `src/manifest.rs` and call it from
   `gradle_include_build_target`, between the parenthesized-argument split and the statement-end
   check. Update the doc comments on `gradle_include_build_target` and
   `require_gradle_include_build_statement_end`, both of which currently say a configuration block
   is refused.
2. Add `gradle_settings_accept_an_include_build_with_a_configuration_block` covering the reported
   multi-line form, the one-line form, Groovy quotes, `settings.` qualification, a trailing `;`, a
   normalizing path, braces inside a string literal, braces inside line and block comments, and both
   conditional spellings. Each asserts the sibling `:app` module and only that module.
3. Add `gradle_settings_still_judge_directives_inside_an_include_build_block`, proving the skip
   hides nothing from the `include`, `includeFlat`, `projectDir`, and `project(...)` guards.
4. Extend `gradle_settings_still_refuse_an_escaping_or_dynamic_include_build`: replace the
   now-accepted block fixture with a non-block trailing expression, and add the escaping,
   interpolated, multi-argument, unbalanced-block, and brace-in-string cases.
5. Add the integration test `gradle_include_build_with_a_configuration_block_discovers_modules`
   over a real fixture, asserting discovered modules, no manifest notice, and that the included
   build contributes nothing.
6. Prove at least one new test fails on `main` by disabling the helper in place (early return) and
   re-running, then restore it.
7. Update `specs/manifest/` — spec invariant 14, the Error Cases table, version and change log,
   `requirements.md`, `context.md` (decision plus a #725 lesson), `tasks.md`, `testing.md` — and
   write the `manifest` delta.
8. `cargo fmt`, `cargo clippy -- -D warnings` (bare), targeted tests, `change check`,
   `change audit --strict`, then commit and push a branch.
