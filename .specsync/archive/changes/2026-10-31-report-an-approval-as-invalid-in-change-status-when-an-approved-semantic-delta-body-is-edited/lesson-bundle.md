# Lesson bundle — report-an-approval-as-invalid-in-change-status-when-an-approved-semantic-delta-body-is-edited

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Report an approval as invalid in change status when an approved semantic delta body is edited
- **Kind**: BugFix
- **Specs**: change, cmd_change
- **Paths**: src/change/historical_definition_digest_matches.rs, src/change/collect_requirement_ids.rs, src/change/sdd_version.rs, src/change_tests/semantic_application_resolves_registry_backed_canonical_paths.rs, src/commands/change.rs, tests/integration/change.rs, site/src/content/docs/cli.md, specs/change, specs/cmd_change
- **Acceptance**: After a workflow-v2 change is approved by a definition approval that recorded semantic delta digests, editing an approved delta body (committed or not) makes `specsync change status <id> --json` report `summary.approval_valid: false`, `summary.next_action` naming `specsync change approve <id> --actor <name>`, and `summary.handoff.readiness` `not-yet` in the approved, implementing and verifying states, the same verdict `specsync change check <id>` reaches when it refuses that delta.
- **Acceptance**: That summary lists the edited modules, sorted, in a new `changed_approved_deltas` array that is omitted when empty, so no other status payload gains a key.
- **Acceptance**: Text `specsync change status <id>` prints the same re-approval `Next:` action and a `not yet` `Handoff:` line, and `specsync change audit` reports the edited delta as an error for that active change. Text `Next:` names re-approval for every approved, implementing or verifying change whose approval is not current, including the existing stale-scope case.
- **Acceptance**: Restoring the approved delta bytes, or re-approving, returns status to `approval_valid: true` and `run specsync change check <id>`. Re-encoding an unedited delta to CRLF does not invalidate the approval. A later definition approval that withdraws a recorded delta binding also reports `approval_valid: false`.
- **Acceptance**: Unchanged: a ledger that never recorded delta digests (every pre-binding approval and every archive) keeps today's status; draft, accepted and archived summaries keep today's `approval_valid` and terminal-evidence verdicts; historical and archived validation, digests and persisted ledgers are not altered; editing `plan.md` or another artifact outside the approved scope keeps `approval_valid: true` and only stales execution evidence.
- **Acceptance**: Regression tests fail on origin/main f240d37 and pass with the fix: a discriminator for each state above, labelled controls for every unchanged case, and an integration test that drives `change status --json` and text status through the built binary.

## Evidence

- Verification commit: `284e5ee5c8dd0556fba476d3273f48a0133234e6`
- Base commit: `f240d379b9f023f5766e5c9b39508eb84e03e2bb`
- Verified by: `specsync check --spec change --spec cmd_change`

## From the change's context.md

# Context

Reported downstream as Chough BUG-0002. After a change is approved, editing its semantic delta
leaves `specsync change status <id> --json` reporting `summary.approval_valid: true` with the next
action `run specsync change check <id>`. `change check` then refuses the change, saying the delta
for the module changed after approval and must be approved again. Status sends the operator to a
command that is certain to fail, and the handoff verdict calls the moment safe.

## Reproduction (released SpecSync 6.1.1)

The reporter reproduced this in a disposable copy of Chough's `examples/shipping` with a
synthetic approver. No real approval was involved.

1. Create a workflow-v2 change for module `shipping` with a semantic delta, complete its
   artifacts, and approve it.
2. Append a `### REQUIREMENT` block to `.specsync/changes/<id>/deltas/shipping.md`. It does not
   matter whether the edit is committed.
3. `specsync change status <id> --json` reports `state: approved`, `summary.approval_valid: true`
   and `next_action: run specsync change check <id>`.
4. `specsync change check <id>` exits 1: "semantic delta for `shipping` changed after approval;
   ... re-run `specsync change approve` ...".

Status recomputes the scope digest on every call. Changing `affected_paths` in `state.json` flips
`approval_valid` to false, so this is not stale caching.

## Measured on origin/main f240d37

A throwaway unit test in a scratch export (fixture approver, not committed) used the existing
`completed_section_only_current_record` fixture. After approval it appended an `## ADDED`
requirement to the `auth` delta. Results:

- `summarize_change`: `approval_valid=true`, next action `run specsync change check add-passkeys`.
- `handoff_summary`: `Safe`.
- `audit_project`: no errors.
- `check_change`: `Err("semantic delta for `auth` changed after approval; ...")`.

When `plan.md` was edited after approval instead, status kept `approval_valid=true` and
`check_change` passed. That is the intended behavior (see below).

## Cause

- `summarize_change_with_effective` sets
  `approval_valid = ensure_definition_approval_valid(root, record).is_ok()`
  (`src/change/historical_definition_digest_matches.rs:3413`).
  `ensure_definition_approval_valid` (`src/change/consider_accepted_evidence_anchor.rs:3242`)
  resolves the effective definition approval against `definition_digest`. For workflow v2 that is
  `scope_digest` over `ApprovedScopeV1` (`src/change/collect_requirement_ids.rs:2499` and `:2510`,
  `src/change/sdd_version.rs:874`). The scope covers intent, criteria, answers and declared scope,
  and contains no delta body.
- Delta bodies are bound separately (#704, REQ-change-089 and REQ-change-090). The definition
  approval records `approved_delta_digests` (`src/change/sdd_version.rs:990`,
  `src/change/validate_loaded_change.rs:595` and `:640`). Only
  `ensure_approved_delta_bodies_unchanged` (`src/change/collect_requirement_ids.rs:1290`) compares
  them, and only materialization (`src/change/sdd_version.rs:2922`) and acceptance
  (`src/change/historical_definition_digest_matches.rs:2222`) call it.
- Three other readers have the same gap:
  - `handoff_summary` (`src/change/historical_definition_digest_matches.rs:3192`) calls the edit
    safe.
  - The active-workspace audit (`check_project_with_command_output`, `:3947`) stays green.
  - Text status `text_mode_next_action` (`src/commands/change.rs:807`) never consults approval
    currency for approved, implementing or verifying changes. It therefore also says
    `change check` after a stale-scope edit, which is a pre-existing gap with the same cause.
- Workflow v1 did not have this bug: its definition digest hashed every delta payload. The defect
  appeared when workflow v2 split the scope digest from the execution digest and #704 added a
  separate delta binding without updating status.

## Plan edits are outside approval by design

Workflow-v2 approval binds only the stable scope projection. Evidence:

- `change.spec.md` Contract 2.
- The workflow-v2 requirement in `specs/change/requirements.md`: "Scope approval binds stable
  intent, acceptance criteria, public-contract/risk declarations, and affected
  spec/path/dependency/supersession scope—not implementation, test/evidence ...".
- The same requirement: "Non-material execution/evidence changes preserve scope approval".
- `hi/change.md` CHANGE-6.c.

`plan.md` is hashed into the execution digest (`execution_digest`,
`src/change/collect_requirement_ids.rs:2506`). Editing it stales verification and review, not
approval. Delta bodies are contract wording, and REQ-change-089 binds them to the approval. This
change keeps that split and does not add `plan.md` to the approval.

## Contract tension found

REQ-cmd-change-005 says status "requests renewed approval only for a material stable-scope change
... execution/evidence-only changes direct the user to `change check` instead". Since
REQ-change-089, an edited delta body is not execution-only. The bullet predates the binding and is
modified here.

## Human intent

This change links to existing criteria and adds none:

- CHANGE-6.a: approval is bound to the content it approved.
- CHANGE-6.c: changing the contract costs a fresh approval.
- CHANGE-7: be told the single next thing to do.
- CHANGE-7.a: be told whether it is safe to walk away.
- SHIP-7: audit live changes for coherence.

## From the change's design.md

# Design

## Decision: correct `approval_valid`, and add a list of the edited modules

Recommendation: `summary.approval_valid` reports the approval as the gates use it. It is true only
when two things hold:

- the effective definition approval matches the current scope digest, and
- the delta bodies that approval recorded still match the delta files.

The new `summary.changed_approved_deltas: Vec<String>` names the modules whose recorded body no
longer matches. It is sorted and omitted when empty.

Reasons:

1. Every command that consumes the approval refuses in this state. `change check` refuses through
   `materialize_change_deltas`, and `finalize` refuses through acceptance. `approval_valid` exists
   to predict those refusals. A second boolean would leave the existing field wrong for every
   consumer that never learns the new one, which is how Chough hit this.
2. Workflow v1 already reports `approval_valid: false` after a delta edit, because its definition
   digest hashes the delta. This restores parity for v2 and does not invent a new meaning.
3. `next_action` and `classify_handoff` already branch on the one boolean. Feeding them the
   composed value keeps JSON next action, text next action and handoff in agreement without new
   branches.
4. The list carries the reason a boolean throws away. Without it, `approval_valid: false` beside
   an empty `scope_expansion` gives no clue why. Status already explains a scope renewal in plain
   language (`scope_expansion`), and the list does the same for deltas.

Rejected alternative: keep `approval_valid` as "scope digest matches" and add
`delta_approval_valid: bool`. That needs no consumer migration only because it leaves consumers
wrong. It would also be a second verdict that must always be AND-ed with the first.

## Where the predicate lives

There is one domain predicate in the change module:

```
approval is current = ensure_definition_approval_valid(root, record).is_ok()
                      && ensure_approved_delta_bodies_unchanged(root, record).is_ok()
```

It is evaluated only for `Approved | Implementing | Verifying`, the states in which
materialization still guards the delta. `ensure_approved_delta_bodies_unchanged` is refactored to
return the drifted module list (`Result<Vec<String>, String>` under a private name). The refusal
text materialization and acceptance print stays byte-identical. The list is computed only when
the scope half holds, because the delta helper itself resolves the effective approval first.

Callers:

- `summarize_change_with_effective`: `approval_valid`, `changed_approved_deltas` and, through
  them, `next_action` and the `HandoffSignals` it builds.
- `handoff_summary`: the text `Handoff:` line. Pinned equal to `summary.handoff` by the existing
  `change_summary_carries_the_same_handoff_the_domain_computes`.
- `check_project_with_command_output`: `change audit` adds the same refusal string `change check`
  prints for that active change.
- A new public `approval_current_for_guidance(root, record) -> bool` for the command layer. Text
  `text_mode_next_action` returns `run specsync change approve <id> --actor <name>` for an
  approved, implementing or verifying change when it is false. That covers both the delta case
  and the pre-existing stale-scope case. The `ship-status` lifecycle next inherits it. The
  function returns a boolean built from comparisons, so no digest reaches a text sink, which
  satisfies the CodeQL cleartext-logging rule text mode follows.

## What does not change

- `ensure_definition_approval_valid` and every historical, accepted or archived authentication
  path.
- Draft, accepted and archived summaries keep today's `approval_valid` computation, so terminal
  evidence verdicts and archive status do not move.
- An absent binding stays unknown. A ledger in which no definition approval recorded digests
  returns `Ok` from the existing helper, so status stays valid exactly where materialization
  proceeds.
- No persisted file, schema, digest, approval ledger or archive changes.
- `plan.md` and other execution artifacts stay outside approval (see `context.md`).

## Observable changes and compatibility

| Surface | Before | After | Who is affected |
|---------|--------|-------|-----------------|
| `change status/show --json` `summary.approval_valid` | `true` after a delta edit | `false` | consumers reading it, which were previously misled |
| `summary.next_action` | `run specsync change check <id>` | `run specsync change approve <id> --actor <name>` | agents following next action |
| `summary.handoff` | `safe`, or `conditional` with scoped edits | `not-yet`, "definition changed after it was approved" | session handoff |
| `summary.changed_approved_deltas` | absent | present only in this state | additive; older readers ignore it |
| text `Next:` | `change check` even when approval is stale (scope or delta) | `change approve` | humans |
| `change audit` | exit 0 | exit 1 with the `change check` refusal for that change | CI or local audits on a branch in this state |

The JSON shape is unchanged. A value changes only where it was wrong, plus one optional key. This
is still an observable public-contract change, and it is classified that way.

## From the change's testing.md

# Testing

Each new test carries an honest label. A DISCRIMINATOR must fail on origin/main f240d37 and pass
with the fix. A CONTROL passes on both. The label is confirmed by running the new tests against a
build of unfixed f240d37 from a separate checkout, not by disabling the fix in place.

## Unit tests

These go in `src/change_tests/semantic_application_resolves_registry_backed_canonical_paths.rs`.
They reuse `change_with_an_approved_delta`, `SWAPPED_DELTA_BODY`, `strip_recorded_delta_digests`
and `approved_delta_body_as_a_crlf_checkout_writes_it`.

DISCRIMINATORS:

- `status_reports_an_edited_approved_delta_as_an_invalid_approval`: approved state, delta body
  edited. Asserts:
  - `summarize_change` returns `approval_valid == false`,
  - `next_action` names `change approve`,
  - `changed_approved_deltas == ["auth"]`,
  - `handoff.readiness == NotYet`,
  - `materialize_change_deltas` refuses the same tree, so status and check agree.
- `status_reports_an_edited_approved_delta_while_implementing_and_verifying`: the same assertions
  after `start_implementation`, and after a passing `check_change` moves the change to verifying.
- `handoff_summary_is_not_yet_for_an_edited_approved_delta`: the text-path `handoff_summary`
  equals `summary.handoff` and is `NotYet`.
- `audit_reports_an_edited_approved_delta`: `audit_project` reports, for this change, the
  "changed after approval" refusal `check_change` returns.
- `status_reports_a_withdrawn_delta_binding_as_an_invalid_approval`: uses the ledger shape of
  `a_later_definition_approval_may_not_withdraw_a_recorded_delta_binding`.

CONTROLS:

- `restoring_or_reapproving_an_edited_delta_restores_approval_validity`.
- `a_crlf_checkout_of_an_approved_delta_keeps_status_approval_valid`.
- `status_keeps_approval_valid_for_a_ledger_without_delta_digests`: strips digests, edits the
  delta, and expects `approval_valid` true, because materialization proceeds there too.
- `editing_the_plan_after_approval_keeps_approval_valid`: `plan.md` edited; `approval_valid` stays
  true and `check_change` passes.
- `changed_approved_deltas_is_omitted_when_empty`: serialized summary of a clean approved change
  has no such key.
- `finalized_change_summary_keeps_its_approval_and_terminal_verdicts`: archived fixture; summary
  `approval_valid` and `terminal_evidence` equal what f240d37 computes.

Existing handoff tests stay green unchanged, including
`handoff_stale_approval_is_not_yet_in_every_approved_state` and
`change_summary_carries_the_same_handoff_the_domain_computes`.

## Command tests

These go in `src/commands/change.rs`, `mod tests`, using `approved_change_fixture`.

- DISCRIMINATOR `text_next_action_names_reapproval_when_an_approved_delta_was_edited`.
- DISCRIMINATOR `text_next_action_names_reapproval_when_the_approved_scope_changed`, for the
  pre-existing stale-scope case.
- CONTROL `text_next_action_names_check_for_a_current_approval`.

## Integration test

This goes in `tests/integration/change.rs`, through the built binary, following
`status_prints_a_handoff_line_and_json_carries_it`.

`status_json_and_text_report_an_edited_approved_delta`:

1. Create, answer and approve a change with a fixture actor in a temp Git repo.
2. Append a requirement block to its delta.
3. Assert:
   - `change status --json` has `summary.approval_valid == false`, a `change approve` next action,
     `summary.changed_approved_deltas == ["auth"]` and `summary.handoff.readiness == "not-yet"`,
   - text status prints a `change approve` `Next:` line and a `not yet` `Handoff:` line,
   - `change check` and `change audit` both exit 1.
4. Restore the delta. Status is valid again with a `change check` next action.

## Regression lanes

- `cargo test change::` (the component route for `change`).
- `cargo test commands::change::`.
- `cargo test --test integration change::`.
- `fledge lanes run verify` (fmt, clippy, types, full test, release build, strict specs).
- `specsync check --strict --require-coverage 100`.
- `specsync change audit`.
- `fledge lanes run pre-push` before every push.
- `fledge trust verify` before calling the work complete.

## Requirement evidence

| Requirement | Required evidence |
|-------------|-------------------|
| REQ-change-111 | Discriminators: `status_reports_an_edited_approved_delta_as_an_invalid_approval`, `status_reports_an_edited_approved_delta_while_implementing_and_verifying`, `handoff_summary_is_not_yet_for_an_edited_approved_delta`, `audit_reports_an_edited_approved_delta`, `status_reports_a_withdrawn_delta_binding_as_an_invalid_approval`. Controls: CRLF-only, restored or re-approved body, ledger without digests, `plan.md` edit, omitted empty list, finalized-change summary. Integration: `status_json_and_text_report_an_edited_approved_delta`. |
| REQ-cmd-change-005 | `text_next_action_names_reapproval_when_an_approved_delta_was_edited`, `text_next_action_names_reapproval_when_the_approved_scope_changed`, `text_next_action_names_check_for_a_current_approval`, and the text half of `status_json_and_text_report_an_edited_approved_delta`. Existing REQ-cmd-change-005 coverage keeps passing. |

No result is claimed here. Observed results are recorded after the tests are written and run.

## Observed results (2026-10-31)

Labels were confirmed against an export of f240d37 with only the new tests applied (no fix):

- Failed on f240d37, as labelled DISCRIMINATOR: `status_reports_an_edited_approved_delta_as_an_invalid_approval`, `status_reports_an_edited_approved_delta_while_implementing_and_verifying`, `handoff_summary_is_not_yet_for_an_edited_approved_delta`, `audit_reports_an_edited_approved_delta`, `status_reports_a_withdrawn_delta_binding_as_an_invalid_approval`, `text_next_action_names_reapproval_when_an_approved_delta_was_edited`, `text_next_action_names_reapproval_when_the_approved_scope_changed`, and the integration test `status_json_and_text_report_an_edited_approved_delta` (status reported `approval_valid: true`).
- Passed on f240d37, as labelled CONTROL: `restoring_or_reapproving_an_edited_delta_restores_approval_validity`, `a_crlf_checkout_of_an_approved_delta_keeps_status_approval_valid`, `status_keeps_approval_valid_for_a_ledger_without_delta_digests`, `editing_the_plan_after_approval_keeps_approval_valid`, `changed_approved_deltas_is_omitted_when_empty`, `finalized_change_summary_keeps_its_approval_and_terminal_verdicts` and `text_next_action_names_check_for_a_current_approval`. The refactored `a_later_definition_approval_may_not_withdraw_a_recorded_delta_binding` also still passes there.
- With the fix, all of them pass. The full `cargo test` run passes: 2,614 unit tests and 463 integration tests.

Deviation: the integration fixture uses a `"version": 2` policy. Under a version-1 policy `change new` writes a workflow-v1 record, and the test then passed on f240d37 for the wrong reason. The appended requirement carries Acceptance Criteria so that `change check` refuses at the delta binding and not in `validate_definition`.

## Where these lessons go

- `specs/change/change.3md`
- `specs/cmd_change/cmd_change.3md`
