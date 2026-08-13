---
change: CHG-0108-stop-reporting-success-for-checks-that-did-not-happen-gate-drafts-that-document
artifact: plan
---

# Plan

Implementation and the full suite ran **before** `change new`, per #542: delivery scope
freezes at the interview and cannot be widened, while blast radius only becomes visible at
compile, test, and verification time. The declared scope is measured.

## Sequence

1. **Quoting.** Add `unquote_yaml_scalar` to `parser`; apply to block list items and to
   scalar values, passing `[`-prefixed values through to `parse_flow_string_list`. Mirror
   the unquoting inline in `hash_cache::extract_frontmatter_files`.
2. **Cold cache.** Add `HashCache::has_baseline`, `ChangeClassification::baseline_known`
   and `::reportable`; switch both reporting sites in `commands/check.rs` from `has` to
   `reportable`.
3. **Remediation.** Add `UNCOVERED_PATH_FLAG_LIMIT` and the remainder summary.
4. **Draft gate.** Add `had_present_source` and `documents_contract` to `ValidationResult`;
   set them in `validator`; warn in `commands/mod.rs` on the conjunction.
5. **Verify.** `cargo fmt --check`, `cargo clippy -- -D warnings`, full `cargo test`.
6. **Author the change** with the measured scope and update the seven affected specs.

## Ordering constraint discovered mid-implementation

Step 4 was first written as "warn whenever a draft has present source". The suite reported
exactly three failures, all in the same family, and their names —
`draft_planned_mapping_passes_strict_and_is_absent_from_coverage` — showed the rule was
breaking spec-first authoring rather than catching a defect. The rule was narrowed to
require a documented Public API, which is why all three now pass unedited.

**The failing test names were the design input.** Had the tests been rewritten to match the
first rule, the change would have shipped having quietly deleted a designed behavior.

## Rollout

Lands before the `v6.0.0` tag, so no migration is owed.

Adopters whose specs are draft-with-present-source-and-documented-API will see
`check --strict` begin to fail. That is the intended effect and it is exactly what the RC
should surface: `3md` and `attest` are in that state and have been reporting 100% coverage
over unvalidated source. The remedy is one word — `status: active` — and it turns the gate
on rather than off.

Bare `specsync check` is unchanged for every draft case, so nothing breaks for anyone not
running `--strict`.

## Stacking

Branched off `0xleif/rc1-first-five-minutes` (PR #544, all checks green) rather than main,
because both are RC1 product-surface work and the CHANGELOG edits are adjacent. The PR
targets main and rebases cleanly once #544 merges.
