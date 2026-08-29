---
change: decide-canonical-materialization-from-the-artefacts-instead-of-the-canonical-applied-flag-so-a-delta-corrected-after
artifact: context
---

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
