---
change: report-the-requirement-ids-an-approved-semantic-delta-declares-per-module-in-change-status-and-show-json
artifact: context
---

# Context

Requested in CorvidLabs/spec-sync#826, reported downstream as CorvidLabs/chough#11. Chough's
`cases check` refuses a test case that links `REQ-shipping-002` while that ID exists only in an
approved semantic delta. Nothing in native SpecSync JSON names that ID until `change check`
materializes it into `specs/<module>/requirements.md`. The only way to find it is to parse
`.specsync/changes/<id>/deltas/*.md`. That duplicates SpecSync's delta grammar. The parser also
cannot tell whether the body on disk is the approved one, because `approved_delta_digests` is a
domain-separated, framed SHA-256 over the module name and the CRLF-folded body
(`delta_body_digests` and `canonical_delta_body` in `src/change/collect_requirement_ids.rs`).
That construction is internal and undocumented, and it changed in #730.

## Reproduction (origin/main a80748f, built from this worktree)

The issue's `base.sh` and `repro2.sh` were run unchanged against a release build of a80748f. The
fixture was Chough's `examples/shipping` at CorvidLabs/chough@2756dc2, and the approver was a
simulated actor in a disposable repository under the session scratchpad. No real approval was
recorded.

A. After approval, REQ-shipping-002 appears in no native output:

```text
{ "state": "approved", "approval_valid": true }
status --json bytes: 2589, mentions: 0
show --json bytes:   2589, mentions: 0
specs/shipping/requirements.md mentions: 0
delta file mentions: 1
top-level keys: ["acceptance_entries","briefing","change","corrections","effective_definition","questions","summary"]
approvals.json approved_delta_digests: [{"shipping":"de755d1e24cbb98b22b938f6efea918cc9a21dd4b08a7a7746173d55673cbe79"}]
sha256 of delta file: 9b610ff6932c9d13de80a2442f8ce7f0d0800762a12eb5fa65dfa5013d3d6fc4
```

B. After the approved body is edited, status is already correct on main. #823 merged
REQ-change-111, which made `approval_valid` false and `next_action` name re-approval:

```text
{"state":"approved","approval_valid":false,"next_action":"run `specsync change approve free-shipping-at-the-threshold --actor <name>`"}
summary.changed_approved_deltas: ["shipping"], handoff: not-yet
```

C. `change check` exits 1 with "semantic delta for `shipping` changed after approval; ...".

With `base.sh <dir> verify`, which approves, implements and runs `change check --commit`, the change
is `verifying` with `canonical_applied: true`. REQ-shipping-002 is now in
`specs/shipping/requirements.md`, and the status JSON still has no field that names it.

So the second half of #826 (an integrity flag in `approval_valid` and `next_action`) shipped in
#823. What is still missing is the first half: a per-module report of the declared requirement IDs
and sections, a per-module integrity flag, and a materialization flag.

## Cause

- `print_record` and `print_mutation_record` (`src/commands/change.rs`) emit `change`,
  `effective_definition`, `acceptance_entries`, `corrections`, `summary`, `questions` and
  `briefing`. None of them carries delta items.
- The only delta-item reader that reports IDs is the private `collect_requirement_ids`
  (`src/change/collect_requirement_ids.rs:1557`). It feeds verification's requirement coverage, it
  does not split added, modified and removed, and it reads with unbounded `fs::read_to_string`.
- The per-module drift answer exists as the private `changed_approved_delta_modules` (`:1365`).
  Status uses it only through `approval_currency` (`:1416`), which reduces it to the
  `changed_approved_deltas` list.
- Whether the canonical tree carries the delta on disk is derived only inside
  `prepare_pending_delta_application` (`:1834`, #741). `canonical_applied` records only that
  materialization ran.

## Constraints already settled

- An absent delta binding is unknown, not a violation. That is judged per ledger: no definition
  approval recorded one (REQ-change-089, REQ-change-090).
- `approval_currency` decides which states the binding is judged in (REQ-change-111). This change
  must read the same predicate and add no second comparison.
- No new live field on `ChangeRecord`. It is hashed into `definition_digest` in four places
  (`change.3md`, #741 plane).
- HI: links CHANGE-5 (the delta is readable: added, changed, retired), CHANGE-6.a (the approval is
  bound to its content, so later edits cannot ride on it), CHANGE-7 (ask the state of a change) and
  AGENT-2 (structured data, so an agent never scrapes). No new HI criterion is proposed or captured.

## Re-run of the issue reproduction (candidate build)

The issue's `repro2.sh` and `base.sh` were run unchanged against a debug build of this branch, with
the same fixture (Chough `examples/shipping` at CorvidLabs/chough@2756dc2) and a simulated
approver in a disposable repository under the session scratchpad.

A. After approval, `status.json` and `show.json` each mention REQ-shipping-002 once, under the new
key:

```text
{ "state": "approved", "approval_valid": true }
status --json bytes: 2979, mentions: 1
show --json bytes:   2979, mentions: 1
top-level keys: ["acceptance_entries","approved_deltas","briefing","change","corrections","effective_definition","questions","summary"]
approved_deltas: [{"delta_intact":true,"materialized":false,"module":"shipping","requirements":{"added":["REQ-shipping-002"],"modified":[],"removed":[]},"sections":{"added":[],"modified":["Public API"],"removed":[]}}]
```

B. After the approved body is edited: `{"approval_valid":false,"next_action":"run `specsync change approve free-shipping-at-the-threshold --actor <name>`"}`
and `approved_deltas: [{"delta_intact":false,"module":"shipping"}]`.

C. `change check` still exits 1 with the unchanged "semantic delta for `shipping` changed after
approval" refusal.

With `base.sh <dir> verify` the change is `verifying` and the entry reports `materialized: true`
with the same requirement and section lists.

## Deviation found while testing

The design expected `change depend` on an approved change to carry real entries. Dependencies
are part of the approved scope, so a NEW dependency stales the scope and the in-lock
`approved_deltas` is `[]`, exactly what `status` then reports. Re-declaring a dependency the
approval already covers keeps the scope current and carries the intact entries. The unit and
integration tests drive both paths.

## Observed results of the regression lanes

Recorded after the review pass on PR #828. Figures marked "first run" come from the first
implementation pass, whose logs were not committed; this pass did not re-run the a80748f stub
labelling. Every other figure was observed in this pass on the working tree that the review fix
commit records.

- Unit, first run: against the a80748f stub described in `testing.md`, 12 failed and 14 passed.
  With the change, 26 passed.
- Unit, this pass: the 17 tests in `src/change_tests/approved_delta_requirements.rs` pass.
  `cargo test --bin specsync change::` gave 557 passed, 0 failed. `cargo test --bin specsync
  commands::change::` gave 34 passed, 0 failed.
- Integration, before: `status_and_show_json_report_approved_delta_requirements`, run with
  `CARGO_BIN_EXE_specsync` pointing at a debug build of unfixed a80748f, fails at its first
  assertion. The draft `change status --json` has no `approved_deltas` key (`left: Null`,
  `right: Array []`).
- Integration, after: the same test passes against this branch.
- `summary` key set: the integration fixture was replayed through both binaries. In the draft,
  approved and `change new --json` outputs, a80748f and this branch print the same 13 `summary`
  keys. The test now asserts that set (testing.md step 2), and asserts that `change new --json`
  carries `approved_deltas == []`.
- Full suite inside `fledge lanes run verify`: 2640 unit tests passed and 465 integration tests
  passed, 0 failed. The lane passed (exit 0), including fmt, clippy, the release build and strict
  specs.
- `specsync check --strict --require-coverage 100`: 62 specs checked, 62 passed, 0 warnings. File
  coverage 130/130 and LOC coverage 147486/147486.
- `specsync change audit`, run before this commit's `change check --commit`, reported only
  "verification evidence is stale for the current commit or contract". That is expected after
  an edit, and the re-run after `change check --commit` is recorded in the PR.
- `fledge lanes run pre-push` and `fledge trust verify` run before each push. Their results for
  each head are recorded in the PR, not here, because a new commit outdates them.

## Review finding: `change new` is not built from a mutation snapshot

REQ-cmd-change-023 AC2 and the first `cli.md` text list `change new` among the outputs built
under the project lock. In the code, `change new` renders through `print_record` after
`create_change` releases the lock. Its `approved_deltas` is always `[]` because a new change is
a draft, so nothing observable differs. `cli.md` and `docs.md` now claim the in-transaction build
only for `answer`, `depend` and `supersede`. The requirement wording then still listed
`change new`. Correcting it was a delta edit that needed re-approval. That could be folded into the REQ-change-114
renumber re-approval that the #829-first merge order already requires. Routing `change new`
through a snapshot would instead add a crate-private function to the approved Public API section,
which also needs re-approval. The human chose between the two (below).

Decision (0xLeif, 2026-11-01): amend, then accept. REQ-cmd-change-023 AC2 in
`deltas/cmd_change.md` no longer lists `change new` among the outputs built in the same
transaction as their `summary`, and states that `change new --json` always carries an empty
`approved_deltas`. Requirement IDs are unchanged. 0xLeif re-approved the amended definition on
2026-11-01, and `change check` re-materialized `specs/cmd_change/requirements.md` to match.
