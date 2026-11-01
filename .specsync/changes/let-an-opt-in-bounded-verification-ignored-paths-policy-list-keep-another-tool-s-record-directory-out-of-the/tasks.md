---
change: let-an-opt-in-bounded-verification-ignored-paths-policy-list-keep-another-tool-s-record-directory-out-of-the
artifact: tasks
---

# Tasks

- [x] Add the discriminator and control tests in `testing.md`, and record which fail on origin/main a80748ff.
- [x] Add `verification_ignored_paths` to `SddPolicy`, defaulting to empty and omitted when empty.
- [x] Add `src/change/verification_ignored_paths.rs` (validator, per-change overlap check, prefix predicate), include it from `src/change.rs`, and list it and the new test part in the `change` spec's `files:`.
- [x] Thread the validated list into `project_input_digest_uncached` and its Git and filesystem discovery, keeping the empty-list path byte-identical.
- [x] Run the full validator in `change new`, `change answer` and `change approve`, and refuse entries that overlap a change's `affected_paths` at create, at the paths answer, and at approve, check, review and acceptance.
- [x] Make `change status` name an invalid entry as its one next action, and add the `change ship-status` blocker.
- [x] Add the list to `ChangeSummary` only (no persisted record gains a field), and render the text line and the standing note in `change status` and `change check`.
- [x] Document the key, its bounds, its limit and the CI-pin advice in `site/src/content/docs/configuration.md` and `site/src/content/docs/workflow.md`.
- [x] Replay the issue #825 repro with the built binary and the list set, and record the output in `testing.md`.
- [x] Add the lesson to a later plane of `specs/change/change.3md` after verification.
- [x] Run the regression lanes in `testing.md` and record the observed results.
- [x] Review round 1 on #830: refuse entries covering `schema_dir`, path-form `depends_on` entries and `Consumed By` references, and add the missing anchor, archive, walk-message and Git byte-identity tests (see `testing.md`, "Review round 1").

## Human gates

Scope approval of this change by a human is required before implementation starts. This draft
records no approval.

Decided in this draft, for the human to confirm or overturn at approval:

- The scoped-review descendant walk is out of scope. A commit after the review, including one
  under a listed prefix, still stales the review. The stored criteria, REQ-change-114 and Contract
  item 24 say so.
- `ignored_paths` never unlocks an entry: an entry overlapping a `meaningful_paths` scope is
  refused.
- No persisted record gains a field.

Open questions in `design.md`, for the human to settle before approval:

- the refusal for projects whose `source_dirs` or `meaningful_paths` is `.`;
- whether an uncommitted edit under a listed prefix after the review leaving the review current is
  an acceptable reading of HI SHIP-2.b ("editing the code after a review makes that review
  stale");
- step G as a follow-up;
- the bounds of 8 entries and 128 bytes.

The REQ IDs are provisional and are decided at approval, in landing order (see
`requirements.md`).

The criterion wording proposed in `context.md` is the human's to accept, reword or reject. Nothing
is captured in `hi/`.

Implementation review (`specsync change review <id> --reviewer <human>`) by a human follows a
current `change check`. Finalization, merge and any release are separate human decisions.

## After verification

Run `fledge lanes run pre-push` before every push and `fledge trust verify` before calling the work
complete. Hosted CI and Trust results for the exact head are recorded as evidence, not assumed.

Follow-ups, each filed only if the human agrees:

- step G: `change check --commit` should not stage tracked edits under a listed prefix;
- relaxing the scoped-review descendant walk for listed prefixes, which would have to read the
  list from the policy blob at the review's implementation commit and convert entries with
  `git_repo_relative_path`.
