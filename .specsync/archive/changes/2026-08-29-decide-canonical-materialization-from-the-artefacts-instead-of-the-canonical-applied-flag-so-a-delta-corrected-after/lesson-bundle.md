# Lesson bundle — decide-canonical-materialization-from-the-artefacts-instead-of-the-canonical-applied-flag-so-a-delta-corrected-after

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Decide canonical materialization from the artefacts instead of the canonical_applied flag so a delta corrected after review and re-approved reaches the canonical spec with its version bump and Change Log row
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change.rs, src/change_tests.rs
- **Acceptance**: A delta corrected after review and re-approved is materialized by the next change check, and the superseded wording does not survive; a canonical spec carrying the change's contract text without the version bump or a Change Log row naming the change receives both; a byte-identical re-approval writes nothing at all, leaving the spec byte for byte, one version bump and one Change Log row; re-materialization does not refuse a removal its own earlier run performed, while every first-run application refusal still fires; the refusal for a drifted delta names change check after change approve.

## Evidence

- Verification commit: `93a7a8e7f5e493da1e0782e967a963b1ce68322c`
- Base commit: `7df407728de3ac6458ef8807e79bbadb51da3324`
- Verified by: `cargo test change::`, `cargo test`, `python3 -S .github/scripts/validate-release-version.py`, `python3 -S .github/scripts/validate-workflow-runtime-pins.py`

## From the change's context.md

# Context

## What led here

Issue #741, found by an independent re-review of the #721 change and confirmed against that
change's own ledger. The ordinary review loop discarded its own result:

1. `change approve` records `approved_delta_digests` for the delta.
2. `change check` materializes the delta into the canonical spec and sets `canonical_applied`.
3. Review finds the wording wrong. The author corrects the delta and re-approves.
4. `change approve` records a NEW digest for the corrected body.
5. `change check` runs again and exits 0 without materializing anything.

`ensure_approved_delta_bodies_unchanged` sits deliberately ABOVE the `canonical_applied`
short-circuit, and its comment states the intent exactly right. But it asks *does the delta match
the approval that signed it?* — and re-approval makes that pass by construction, because a new
approval signs the new body. Nothing asked the other question: *does the canonical spec still
match the delta?* Then `if record.canonical_applied { return Ok(record); }` returned before
`prepare_delta_application` was reached.

## Why it was wider than filed

`bump_spec_version` and `append_changelog` had exactly one caller each, inside
`prepare_delta_application`, which is invoked BELOW the short-circuit. So the flag skipped all
three of materialization's outputs, not just the applied delta:

| output | skipped after `canonical_applied` |
|---|---|
| the delta applied to the canonical spec | yes |
| the spec's `version:` frontmatter bump | yes |
| the spec's Change Log row | yes |

A canonical spec could therefore carry changed contract text with no version bump and no Change
Log row while `change check`, `change audit --strict` and `specsync check --strict` all passed —
drift in the tool's own drift-detection artefact, produced by the tool, invisible to the tool.

This ruled out the narrowest repair. "Re-apply the delta when the digest differs" leaves the
version and the Change Log still skipped; neither is derivable from a delta digest.

## Fix direction chosen, and why not the other two

The issue offered three directions.

1. **Clear `canonical_applied` when a new approval records a differing digest.** Needs a recorded
   `materialized_delta_digests`, which `state.json` does not have. Rejected on evidence: the
   change record is SERIALIZED AND HASHED for `definition_digest` in four places
   (`definition_digest_for_correction_count`, its legacy-task sibling,
   `historical_definition_digest_matches`, `portable_definition_digest_pair_v501`), each of which
   normalizes `canonical_applied`, `correction_count` and `updated_at` out by hand before hashing.
   A new field present on a live record would move every one of those digests unless normalized
   out in all four, and the `explicit_false` byte-splicing those functions already carry shows
   what that costs. The issue's own comment also notes the digest is not enough: the version bump
   and the Change Log row are not derivable from it.
2. **Ask the second question above the short-circuit and refuse.** This is the shape that was
   taken, minus the refusal. Refusing would have been the #542 trap again — the author's only
   remedy would be to reopen, for the entirely ordinary act of correcting a delta after review.
   Answering the question and then DOING the outstanding work is strictly better than answering it
   and reporting an error the author cannot act on cheaply.
3. **Refuse re-approval once `canonical_applied` is set.** Makes the review-correct-reapprove loop
   a dead end. #542 already gives this lifecycle one of those.

**What landed:** the short-circuit is now conditional on evidence that all three outputs are
current, and that evidence is DERIVED FROM THE ARTEFACTS rather than recorded beside them. Per
module:

* the delta is applied when every item it declares is already reflected — a `## REMOVED` block
  absent, an `## ADDED`/`## MODIFIED` block present with matching content (`delta_item_is_applied`);
* the bump and the row are written together by the same two lines, exactly once per
  (change, module), so a Change Log row naming the change is the evidence that both happened
  (`changelog_records_change`). A bumped `version:` leaves nothing behind naming the change that
  bumped it, so the integer alone cannot say whose bump it was.

A module with both is not rewritten at all; a module missing either is materialized again and
given only the halves it is missing. Nothing new is persisted, so no archived ledger changes shape
and no digest moves.

## Constraints discovered along the way

- **`prepare_delta_application` was not idempotent.** `apply_markdown_block` refuses to remove a
  block that is not there, which is correct on a first run — it means the delta names something
  that never existed. Re-running it over an already-materialized `## REMOVED` delta would have
  turned a silent skip into a hard error, a different defect rather than none. Convergence is
  therefore scoped to `record.canonical_applied`: on a first materialization every refusal the
  applier makes still fires unchanged, and only afterwards does "already reflected" read as done.
- **The short-circuit now reads the canonical spec on every check of an applied change.** A change
  whose module spec has since been deleted will error where it used to pass. That is the honest
  answer — the spec is the artefact the change produced — and the message names the file.
- **`markdown_block_range` was extracted** from `apply_markdown_block` so that applying an item
  and asking whether an item is already applied read the same block out of the same scan.

## Blast radius, measured

- All **208** `state.json` under `.specsync/` (archived and active), **193** of them with
  `canonical_applied: true`, deserialize into `ChangeRecord` and round-trip; all **208**
  `approvals.json` deserialize into `ApprovalLedger`. No field was added, so this is unchanged
  ground, and it was measured rather than assumed.
- The Change Log row is a reliable per-(change, module) marker in real history: **446 of 454**
  archived (change, module) pairs carry a row naming the change in the module's current spec. The
  eight that do not are all pre-6.0 (`CHG-0003`, `CHG-0068`, `CHG-0069`, `CHG-0088`, `CHG-0103`),
  archived, and never re-materialized — `check` only runs over approved/implementing/verifying
  changes.

## The diagnostic

`ensure_approved_delta_bodies_unchanged` told a blocked author to re-run `specsync change approve
<id>` and stopped there. That was the action that recorded a new digest, satisfied the guard, and
triggered the silent skip: the message steered into the defect it was reporting. Both refusals in
that function now name `specsync change check <id>` as the second step, because approval binds the
wording and only `check` puts it in the canonical spec.

## From the change's design.md

# Design

## The shape

`materialize_change_deltas` used to read:

```rust
ensure_approved_delta_bodies_unchanged(root, &record)?;   // does the delta match its approval?
...
if record.canonical_applied { return Ok(record); }        // <- everything below is skipped
if is_ci_project(root) { return Err(...); }
let mut prepared = prepare_delta_application(root, &record)?;
```

It now reads:

```rust
ensure_approved_delta_bodies_unchanged(root, &record)?;   // does the delta match its approval?
...
let prepared = prepare_pending_delta_application(root, &record)?;   // does the tree match the delta?
if record.canonical_applied && prepared.pending.is_empty() { return Ok(record); }
if is_ci_project(root) { return Err(... names prepared.pending ...); }
let mut prepared = prepared.files;
```

Preparation writes nothing; it returns `(files, pending)`. `pending` is empty exactly when every
affected module's canonical outputs are already current, so the short-circuit survives intact for
the case it was built for and only that case.

`accept_change_with_gate` gets the same replacement, because acceptance is the second place deltas
reach the canonical spec and had the identical `if canonical_applied { Vec::new() }` hole. An
already-current tree yields an empty `files`, so the acceptance manifest it feeds is byte-identical
to what the old branch produced.

## The evidence, per output

Materialization produces three outputs per module. The short-circuit skipped all three, because
`bump_spec_version` and `append_changelog` have exactly one caller and it sits below the flag.

| output | evidence that it is current |
|---|---|
| delta applied to the canonical files | every item the delta declares is already reflected (`delta_item_is_applied`) |
| the spec's `version:` bump | the Change Log names this change (`changelog_records_change`) |
| the spec's Change Log row | the Change Log names this change (`changelog_records_change`) |

The bump and the row share one piece of evidence deliberately. They are written by the same two
adjacent lines, exactly once per (change, module); a bumped `version:` leaves nothing behind naming
the change that bumped it, so the integer alone cannot distinguish "this change bumped it" from
"a later change did". Keying both on the row also makes re-materialization of a CORRECTED delta
refresh the text without a second bump and a second row — one change bumps one module's version
exactly once.

## Two edges that shaped it

**`prepare_delta_application` was not idempotent.** `apply_markdown_block` refuses to remove a
block that is not there. That refusal is right on a first run: it means the delta names something
that never existed. Re-running the applier over an already-materialized `## REMOVED` delta would
therefore have converted the silent skip into a hard error. The fix is not to weaken the applier —
`delta_item_is_applied` asks the question the applier cannot answer, because the applier applies —
and the resulting convergence is scoped to `record.canonical_applied`. A first materialization
still fires every refusal it ever did; only afterwards may "already reflected" read as done,
because only then is "absent" a state this change itself produced.

**The record is hashed.** `ChangeRecord` is serialized into `definition_digest` in four places
(`definition_digest_for_correction_count`, `legacy_task_definition_digest_for_correction_count`,
`historical_definition_digest_matches`, `portable_definition_digest_pair_v501`), each normalizing
`canonical_applied`, `correction_count` and `updated_at` out by hand — one of them by splicing
`,"canonical_applied":false` into the serialized bytes to reproduce an older writer. Adding a
`materialized_delta_digests` field would have moved every one of those digests on any live record
unless normalized out in all four. Deriving the answer from the artefacts persists nothing, so no
archived ledger changes shape and no digest moves.

## Refactor carried along

`markdown_block_range` is extracted from `apply_markdown_block`, so that applying an item and
asking whether an item is already applied read the same block out of the same scan. Two answers
derived from two scans is how a tree and the record of it drift apart, which is the subject of this
change.

## From the change's testing.md

# Testing

Every assertion below was failed against a binary built from a **separate clean checkout of
unfixed `main`** at `7df4077`, with only these tests added to it. Nothing was reverted in place.

## Evidence for REQ-change-092

| Test | Label | Covers |
|------|-------|--------|
| `a_delta_corrected_after_materialization_reaches_the_canonical_spec_on_the_next_check` | DISCRIMINATOR | The filed defect: the corrected wording reaches the spec and the superseded wording does not survive |
| `a_materialized_spec_missing_its_version_bump_and_change_log_row_gets_both_back` | DISCRIMINATOR | The widening: the two outputs no delta digest can derive |
| `re_approving_a_byte_identical_delta_leaves_the_canonical_spec_byte_for_byte_alone` | **CONTROL** | "Always re-materialize" is not the fix; the spec, the version and the row are all left alone |
| `a_corrected_delta_re_materializes_over_a_block_its_own_earlier_run_removed` | DISCRIMINATOR | Re-materialization does not refuse the removal it performed itself |
| `the_refusal_for_a_changed_delta_names_the_second_step_that_finishes_the_job` | DISCRIMINATOR | The diagnostic names `check` as well as `approve` |

## Verbatim control failures (unfixed `main`)

```
thread '...a_delta_corrected_after_materialization_reaches_the_canonical_spec_on_the_next_check'
panicked at src/change_tests.rs:14034:5:
the corrected wording must reach the canonical spec: ---
module: auth
version: 1.0.1
...
## Purpose

Auth tracks credentials. Reviewed and approved wording.
```

```
thread '...a_materialized_spec_missing_its_version_bump_and_change_log_row_gets_both_back'
panicked at src/change_tests.rs:14090:5:
a spec carrying this change's contract text must carry its version bump: ---
module: auth
version: 1.0.0
```

```
thread '...a_corrected_delta_re_materializes_over_a_block_its_own_earlier_run_removed'
panicked at src/change_tests.rs:14197:5:
the corrected section must reach the canonical spec: ---
module: auth
version: 1.0.1
...
## Purpose

Auth.
```

```
thread '...the_refusal_for_a_changed_delta_names_the_second_step_that_finishes_the_job'
panicked at src/change_tests.rs:14230:5:
the remedy must name the step that puts the approved wording in the canonical spec; naming only
`approve` walked the author into the silent skip: semantic delta for `auth` changed after
approval; the approved wording is what rewrites the canonical spec, so re-run `specsync change
approve add-passkeys` to approve the current delta bodies (or restore them)
```

The CONTROL passes on that same unfixed binary, as it must:

```
test change::tests::re_approving_a_byte_identical_delta_leaves_the_canonical_spec_by
te_for_byte_alone ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2423 filtered out
```

## Measurements, not arguments

- 208 `state.json` (193 with `canonical_applied: true`) and 208 `approvals.json` under `.specsync/`
  all deserialize and round-trip under the changed binary. No field was added to `ChangeRecord`,
  so no persisted shape moved and no definition digest moved.
- 446 of 454 archived (change, module) pairs carry a Change Log row naming the change in the
  module's current spec, which is what makes the row usable as per-module materialization
  evidence. The eight exceptions are all pre-6.0 archived changes that `check` never revisits.

## Gates

- `cargo fmt --check` clean.
- `cargo clippy -- -D warnings` clean.
- Full `cargo test` green (`change::tests::` 419 passed, 0 failed).

## Where these lessons go

- `specs/change/context.md`
