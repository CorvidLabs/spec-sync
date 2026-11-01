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
