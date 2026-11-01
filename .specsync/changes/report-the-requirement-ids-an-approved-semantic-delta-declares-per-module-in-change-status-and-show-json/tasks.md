---
change: report-the-requirement-ids-an-approved-semantic-delta-declares-per-module-in-change-status-and-show-json
artifact: tasks
---

# Tasks

- [ ] Write the unit tests in `testing.md` in `src/change_tests/approved_delta_requirements.rs` and the integration test in `tests/integration/change.rs`. Record which ones fail on origin/main a80748f.
- [ ] Add `DeltaDeclarations` and `ApprovedDeltaSummary` to `src/change/sdd_version.rs`.
- [ ] Factor `delta_body_digest` and a private `approved_delta_binding` out of `changed_approved_delta_modules`, keeping every existing output and refusal string byte-identical.
- [ ] Add `approved_delta_summaries` with the state and scope gate, the `delta_intact` mapping, the same-bytes item split and `materialized`.
- [ ] Render `approved_deltas` in `print_record` and `print_mutation_record`.
- [ ] Include the new test file from `src/change_tests.rs` and list it in the `change` spec `files:`.
- [ ] Document `approved_deltas` in `site/src/content/docs/cli.md`.
- [ ] Re-run the issue's `repro2.sh` against the candidate build and record the output in `context.md`.
- [ ] Update the change and cmd_change `testing.md`, `tasks.md` and lessons (`change.3md`, `cmd_change.3md`) after verification.
- [ ] Run the regression lanes in `testing.md` and record the observed results.

## Human gates

A human must approve the scope of this change before implementation starts. This draft records no
approval.

Decisions for the human to make before approval:

- Placement. The draft puts `approved_deltas` at the top level of single-record JSON, beside
  `acceptance_entries`, not in `summary`. So `change list --json` and the roster do not get it.
- Unknown integrity. For a ledger that never recorded delta digests, the draft reports
  `delta_intact: null` and withholds the requirement lists. It does not report the current body's
  IDs beside a null flag.
- Accepted changes. The draft returns `[]` for accepted changes and does not report their
  (materialized) declarations.
- Requirement IDs. REQ-change-113 and REQ-cmd-change-023 are the next free IDs on a80748f. A
  sibling draft opened in the same session may claim the same ones, and whichever merges second
  renumbers.
- HI. No `hi/` criterion is captured or changed. The draft links CHANGE-5, CHANGE-6.a, CHANGE-7
  and AGENT-2. If the human wants a dedicated criterion, for example "A tool can see which
  requirements an approved change will add before they reach the contract", that wording is a
  proposal for the human only.

A human reviewer runs the implementation review
(`specsync change review <id> --reviewer <human>`) after a current `change check`. Finalization,
merge and any release are separate human decisions.

## After verification

Run `fledge lanes run pre-push` before every push and `fledge trust verify` before calling the work
complete. Record hosted CI and Trust results for the exact head as evidence. Do not assume them.
