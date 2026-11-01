---
change: let-an-opt-in-bounded-verification-ignored-paths-policy-list-keep-another-tool-s-record-directory-out-of-the
artifact: plan
---

# Plan

## Approach

Add an opt-in, validated `verification_ignored_paths` list to `.specsync/sdd.json`. The list
removes named directory prefixes from the verification project-input digest and from nothing
else; the scoped-review descendant walk is unchanged. It is validated by every lifecycle verb
that loads SDD policy, so an invalid list fails closed everywhere. It is refused per change when
it overlaps the change's declared paths. It is disclosed in status JSON and in text output, and
no persisted record gains a field. With the list absent or empty, every byte stays as on
origin/main a80748ff.

## Out of scope

- `change check --commit` staging of tracked edits under a listed prefix (issue step G). This is
  proposed as a follow-up.
- Any change to `ignored_paths`, `meaningful_paths`, coverage, `config.toml` `exclude_dirs` or
  `exclude_patterns`, or Git ignore handling.
- Glob patterns, per-change lists, and binding the digest to `HEAD`'s tree.
- Any change to the scoped-review descendant walk (`review_commit_walk`). A commit after the
  review still stales it, including one under a listed prefix.
- Any new field in `verification.json`, `review.json` or the finalization record.
- Acceptance-input digests, acceptance manifests, `lifecycle_commit_scope`, archive
  authentication, and the digest domain version.
- Adding the entry to this repository's own `.specsync/sdd.json`.
- Changes in Chough itself, release, tag or merge.

## Steps

1. Write the discriminator and control tests in `testing.md`, and confirm against a build of
   a80748ff which ones fail there.
2. Add `verification_ignored_paths` to `SddPolicy`, with serde default and omitted when empty.
   Assert that `write_default_policy` output is unchanged.
3. Add `src/change/verification_ignored_paths.rs` (validator, per-change overlap check, prefix
   predicate), include it from `src/change.rs`, and list it in the `change` spec's `files:`.
4. Thread the validated list into `project_input_digest_uncached` and its Git and filesystem
   discovery. Keep the empty-list path identical, and add the `verification-ignored` frame (as
   specified in `design.md`) only when the list is non-empty.
5. Call the full validator from `create_change`, `answer_question_with_snapshot` and approval, and
   the per-change overlap check from those and from `verify_change_locked`, review recording and
   acceptance.
6. Make `change status` name an invalid entry as its one next action, and add the ship-status
   blocker in `ship_status_report`, because both read currency through boolean helpers that hide
   the digest error.
7. Add `verification_ignored_paths` to `ChangeSummary` only (omitted when empty). Render the text
   line and the standing note in `src/commands/change.rs` for status and check.
8. Document the key, its bounds, the limit and the CI-pin advice in
   `site/src/content/docs/configuration.md` and `site/src/content/docs/workflow.md`. The canonical
   requirement and Contract text arrive through the approved deltas.
9. Replay the issue #825 repro with the built binary and the list set, and record the output in
   `testing.md`.
10. Run the regression lanes, then `fledge lanes run pre-push` before any push.

## Risks

- **Hiding a product input.** This is the central risk. It is mitigated by prefix-only entries,
  the refusal of overlap with `.specsync/`, specs, `source_dirs`, spec `files:`, policy files and
  meaningful paths, the per-change overlap refusal, a small cap, and validation on every digest.
  The residual risk is the project's own tests reading excluded files. It is documented and
  repeated in status.
- **Digest compatibility.** An accidental change to the empty-list path would stale every recorded
  verification in every project. A byte-identity control test pins the empty-list digest against
  a fixture value computed on a80748ff.
- **Mixed binary versions.** Old binaries ignore the key and see new evidence as stale. This is
  fail-closed, and the docs say to raise the CI pin first.
- **Review currency (HI SHIP-2.b).** An uncommitted edit under a listed prefix after the review
  leaves the review current. The walk is not relaxed, so committing still stales it. Whether this
  reading of SHIP-2.b is acceptable is open question 2 in `design.md`.
- **Performance.** The validator reads `config.toml`, the policy and spec frontmatter on every
  digest. It is cached in the same read scope as `project_input_digest`, so status and audit do
  not repeat it per change.
- **Case-insensitive file systems.** Overlap is refused ignoring ASCII case. Exclusion matches the
  exact case only, so any mismatch errs toward including more paths.
- **File size (REQ-change-109).** The new code lives in a new part. Its tests go in a new test
  part, `src/change_tests/verification_ignored_paths.rs`. `src/commands/change.rs` is at 4,627
  lines and gains only rendering.

## Constraints consulted

- `specs/change/change.3md` and REQ-change-103: v4 digest, object-id frames, and fail-closed
  filters. The empty list stays byte-identical.
- REQ-change-023 and REQ-change-091: `change check` stays in-process and spawns nothing. The
  in-process verdict cannot read an excluded file, because every validated input is refused as an
  entry.
- Issue #676: the policy is refused at load, in every verb, not at audit.
- `ignored_paths` keeps its single meaning (coverage) and is never read by the new validator, so
  it cannot unlock digest exclusion. SDD policy files and the specs tree stay always meaningful.
- The comment on tolerant evidence (`src/change/sdd_version.rs`, above `AcceptanceOwnerCorrection`):
  `ScopedReviewRecord`, `ApprovedScopeV1` and `CorrectionRecord` are digest preimages, and
  `VerificationRecord` bytes are anchored byte for byte (`consider_accepted_evidence_anchor.rs`),
  so none of them gains a field.
- REQ-change-102: lifecycle commit staging is not changed here.
- REQ-change-109: no change source file exceeds 5,000 lines.
- HI: SHIP-2.b, SHIP-3.a, SHIP-1.d and CHANGE-7 are linked. A new criterion is proposed in
  `context.md` for the human, and none is captured.
