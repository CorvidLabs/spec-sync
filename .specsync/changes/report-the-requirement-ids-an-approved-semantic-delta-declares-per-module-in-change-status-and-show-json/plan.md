---
change: report-the-requirement-ids-an-approved-semantic-delta-declares-per-module-in-change-status-and-show-json
artifact: plan
---

# Plan

## Approach

Expose a domain projection, `approved_delta_summaries`, that reports for each module what the
current approval covers. It is computed from one read per delta: the binding comparison
`change check` enforces, the parsed item headings of that same body, and the derived
"materialized" answer #741 introduced. The command layer renders it under a new top-level
`approved_deltas` key in single-record JSON. Nothing else changes.

## Out of scope

- Any change to `summary.approval_valid`, `changed_approved_deltas`, `next_action` or the
  handoff. #823 already made those report an edited delta.
- Adding the field to `ChangeSummary`, `change list --json`, the roster, `change approve --json`,
  text output or the MCP tools.
- Publishing or changing the delta digest construction, the delta grammar, materialization,
  persisted ledgers or `ChangeRecord`.
- Reporting delta declarations for draft, accepted or archived changes. Accepted changes have
  materialized into the canonical spec, which consumers can read directly.
- Chough-side changes (CorvidLabs/chough#11). That is a separate repository and package.
- Releasing, tagging or merging.

## Steps

1. Write the tests in `testing.md` in a new `src/change_tests/approved_delta_requirements.rs` and
   in `tests/integration/change.rs`. Confirm the discriminators fail on a80748f, built from a
   separate checkout with only the tests applied.
2. Add `DeltaDeclarations` and `ApprovedDeltaSummary` to `src/change/sdd_version.rs`.
3. In `src/change/collect_requirement_ids.rs`, factor the per-module digest framing into
   `delta_body_digest`, and factor `changed_approved_delta_modules` onto a private
   `approved_delta_binding` that keeps each body it read. Keep every existing output and refusal
   string byte-identical.
4. Add `approved_delta_summaries` with the state and scope gate, the `delta_intact` mapping, the
   same-bytes `parse_delta` split, and `materialized` from `prepare_pending_delta_application`.
5. Render `approved_deltas` in `print_record` and `print_mutation_record`
   (`src/commands/change.rs`).
6. Include the new test file from `src/change_tests.rs`. Add it to `specs/change/change.spec.md`
   `files:`. Approval materializes the deltas (REQ-change-113, REQ-cmd-change-023 and the Public
   API rows).
7. Document the key in `site/src/content/docs/cli.md`. After verification, record lessons on
   later planes of `specs/change/change.3md` and `specs/cmd_change/cmd_change.3md`, and update the
   module `testing.md` and `tasks.md`.
8. Run the regression lanes in `testing.md`, then `fledge lanes run pre-push` before any push.

## Risks

- **Drift between projections.** `delta_intact` and `changed_approved_deltas` could disagree.
  Both are derived from one private helper, and tests assert the invariants in `design.md` on
  every fixture.
- **Status cost.** `materialized` runs an in-memory `prepare_pending_delta_application` once per
  single-record JSON call, and only after `canonical_applied`. It reads each affected spec and
  `requirements.md`. That is bounded and small next to `project_input_digest`. Roster and list
  output are not affected.
- **Misreading null.** A consumer might read `delta_intact: null` as intact. The docs say null
  means unknown and that no IDs are reported, and the entry carries no `requirements` to misuse.
- **Schema-strict consumers** see one new top-level key. This is called out in the PR body for
  the release notes.
- **Line budget.** `src/commands/change.rs` is at 4,627 lines and `collect_requirement_ids.rs` at
  4,411. Additions must stay under 5,000 (REQ-change-109). Unit tests go in a new file.
- **Requirement ID collision.** Sibling drafts may also claim REQ-change-113 or
  REQ-cmd-change-023 (see `tasks.md`).

## Constraints consulted

- `specs/change/change.3md`:
  - An absent binding is unknown, and absence is judged per ledger (REQ-change-089,
    REQ-change-090).
  - The binding folds CRLF only (#730).
  - Every predictor of a gate composes the same halves (REQ-change-111, "Status and the delta
    binding").
  - `canonical_applied` means materialization ran, and "applied on disk" is derived (#741).
  - No new live `ChangeRecord` field.
- `specs/cmd_change/cmd_change.3md`: the command layer holds no lifecycle policy and renders
  domain values verbatim. No digest reaches a text sink.
- REQ-change-109: the 5,000-line limit.
- HI: CHANGE-5, CHANGE-6.a, CHANGE-7 and AGENT-2 are linked as they are. No new criterion is
  proposed.
