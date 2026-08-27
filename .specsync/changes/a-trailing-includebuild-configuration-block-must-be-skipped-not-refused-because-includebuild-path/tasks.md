---
change: a-trailing-includebuild-configuration-block-must-be-skipped-not-refused-because-includebuild-path
artifact: tasks
---

# Tasks

- [x] Add `skip_gradle_include_build_configuration_block` and call it from
      `gradle_include_build_target` before the statement-end check.
- [x] Correct the two doc comments that stated a configuration block is refused.
- [x] Add `gradle_settings_accept_an_include_build_with_a_configuration_block` (11 fixtures).
- [x] Add `gradle_settings_still_judge_directives_inside_an_include_build_block` (4 fixtures).
- [x] Extend `gradle_settings_still_refuse_an_escaping_or_dynamic_include_build` with the escaping,
      interpolated, multi-argument, unbalanced-block, brace-in-string, and non-block
      trailing-expression cases.
- [x] Add `gradle_include_build_with_a_configuration_block_discovers_modules` over a real fixture.
- [x] Prove the new tests fail on `main` by disabling the helper in place, then restore it.
- [x] Update `specs/manifest/manifest.spec.md` (invariant 14, Error Cases, version 22, change log).
- [x] Update `specs/manifest/requirements.md`, `context.md`, `tasks.md`, `testing.md`.
- [x] Write `deltas/manifest.md`.
- [x] `cargo fmt`, bare `cargo clippy -- -D warnings`, targeted tests.

## Lifecycle (tracked by the workflow, not by this list)

`change check --commit`, `change audit --strict`, and the branch push are gates recorded by the
lifecycle itself; they are not delivery work items.

