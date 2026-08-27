---
change: a-configured-source-dirs-must-survive-a-manifest-discovery-failure-and-an-in-repo-includebuild-must-be-judged-by-its
artifact: plan
---

# Plan

Delivery scope was fixed at proposal. Every file below was decided before implementation began.

## 1 — Reproduce first (done before any edit)

Build `main`, construct the reported shape (`includeBuild("vendor/podo-shared")`, `include(":app")`,
`source_dirs = ["app/src/main/java"]`), and confirm `coverage` and `check --strict` both exit 1 with
`Unsupported Gradle workspace mutator includeBuild`. Without this the fix is aimed at a description.

## 2 — R1, the precedence (primary)

1. `src/types.rs` — `SpecSyncConfig.source_dirs_set` (`#[serde(skip)]`, following `enforcement_set`);
   `CoverageReport.manifest_notices`.
2. `src/config.rs` — both loaders record what the file stated, before any fallback overwrites the
   list.
3. `src/validator.rs` — `retained_config` sets the flag from the SAME predicate that decides its
   fallback; `retained_coverage_manifest` replaces the unconditional `?`; both `CoverageReport`
   constructions carry the notices.
4. `src/output.rs` — render the notices in text and markdown, and in `coverage_json`.
5. `src/commands/check.rs` — the `check --format json` payload carries them too.
6. `src/comment.rs`, `src/main.rs`, `src/generator.rs` — exhaustive `CoverageReport` test literals
   gain the field. Mechanical; the compiler enumerates them.

## 3 — R2, the parser

7. `src/manifest.rs` — split `includeBuild` out of the token arm into
   `gradle_include_build_target`, reusing the existing literal-only helpers and the same path
   confinement `include(...)` already uses.

## 4 — Tests that can fail for the right reason

8. `gradle_settings_accept_an_in_repo_include_build` — the case with no coverage at all.
9. `gradle_settings_still_refuse_an_escaping_or_dynamic_include_build` — the control, honestly
   labelled: the refusals pass on the unfixed parser too; the diagnostics are what is new.
10. `configured_source_dirs_survive_a_manifest_that_cannot_be_parsed` — both halves of the
    precedence in one test, so it cannot pass against a change that merely stopped failing.
11. Verify all three FAIL against the unfixed code before relying on them.
12. Re-run the reproduction against the fixed binary.

Plus `tests/integration/commands.rs`, which the six existing
`gradle_*_is_inconclusive_for_coverage_gating_commands` tests live in. Their fixtures state
`sourceDirs`, so they assert the configured case — exactly the one this change stops treating as
fatal. Each is re-pointed at an inferred source list, and one new test covers the degraded half.

## 5 — Specs

13. `specs/manifest/` — invariant 14, an Error Cases pair, requirements (two lines that state the
    OLD behaviour and are now wrong), change log, version 18 → 19.
14. `specs/validator/` — invariant 13, an Error Cases pair, REQ-validator-008 acceptance criteria,
    change log, version 36 → 37.
15. `specs/types/` 12 → 13, `specs/config/` 21 → 22, `specs/output/` 8 → 9,
    `specs/cmd_check/` 24 → 25.
16. Fold the lesson into `specs/manifest/context.md` and `specs/validator/context.md`.

## 6 — Verification

17. `cargo fmt`, `cargo test`, and `cargo clippy -- -D warnings` **bare** — clippy is in the
    `verify`/`ci` lanes, not in `change check`, so `change check` goes green while CI blocks the PR.
18. `change check --commit`, then `change audit --strict`.
