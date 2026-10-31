---
change: report-an-approval-as-invalid-in-change-status-when-an-approved-semantic-delta-body-is-edited
artifact: context
---

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
