---
change: report-the-requirement-ids-an-approved-semantic-delta-declares-per-module-in-change-status-and-show-json
artifact: research
---

# Research

## What #823 already settled (origin/main a80748f)

| #826 suggested acceptance | Status on a80748f |
|---------------------------|-------------------|
| Edited delta: `approval_valid` no longer true, `next_action` names re-approval | Done by REQ-change-111. Measured in `context.md` B. |
| `changed_approved_deltas` names the module | Done. Measured. |
| Approved delta requirement IDs listed per module | Missing. |
| `delta_intact` per module, CRLF-agnostic | Missing as a per-module field. The comparison exists (`changed_approved_delta_modules`). |
| `materialized` distinct from `canonical_applied` (#741 window) | Missing. The derivation exists (`prepare_pending_delta_application`). |
| Prose mention not reported | Not applicable yet. `parse_delta` keys items only by `### REQUIREMENT` and `### SPEC SECTION` headings. |

The issue's optional `delta_integrity` summary field is not needed. `approval_valid` already carries
that verdict, and `changed_approved_deltas` carries the reason.

## Readers of delta items

| Reader | Location | Read | Splits operations |
|--------|----------|------|-------------------|
| `collect_requirement_ids` (verification coverage) | `src/change/collect_requirement_ids.rs:1557` | `fs::read_to_string`, unbounded | no; drops REMOVED |
| `delta_body_digests` (binding) | `:1298` | `read_bounded_change_text` | n/a; digests the body |
| `prepare_pending_delta_application` (materialization) | `:1834` | `read_bounded_change_text` | yes, through `parse_delta` |
| `validate_delta_files` (approval) | `:1150` | per file | yes |

The new projection reads each delta once with `read_bounded_change_text`, through the binding
read that `changed_approved_delta_modules` (and so `summary`) also uses, memoized per invocation.
It digests, parses and runs the materialization check on those same bytes, so the IDs it reports
are the IDs of the body it judged intact. A separate `collect_requirement_ids` call would read the
file a second time, and an edit between the two reads could report IDs from a body that was never
compared.

`collect_requirement_ids` itself stays as it is. It is a `change check` coverage input that must
read live, and moving it to the bounded read would change a gate's behavior and refusal text,
which is outside this status-only scope. It shares `parse_delta` with the new projection, so the
grammar is one. See `design.md`, "Not reused".

## Where the field goes

- In `ChangeSummary` it would also reach `change list --json`, the roster, audit and every
  mutation summary built under the project lock. `materialized` reads canonical spec files.
  Computing it for every change in a roster is new cost, and it adds to a type whose fields
  `classify_handoff` and next-action guidance read.
- As a top-level key beside `acceptance_entries` it follows the precedent that key set: a domain
  projection computed by one exported function and rendered verbatim by `print_record` and
  `print_mutation_record`. This one is chosen.

## Line budget (REQ-change-109)

| File | Lines on a80748f | Expected addition |
|------|------------------|-------------------|
| `src/change/collect_requirement_ids.rs` | 4,411 | about 130 |
| `src/change/sdd_version.rs` | 4,383 | about 40 |
| `src/commands/change.rs` | 4,627 | about 10 |
| `src/change_tests/semantic_application_resolves_registry_backed_canonical_paths.rs` | 4,641 | none; too close to the limit |
| `tests/integration/change.rs` | 3,244 | about 120 |

New unit tests go in a new `src/change_tests/approved_delta_requirements.rs`, included from
`src/change_tests.rs` and listed in the `change` spec `files:`. It can reach the existing private
fixtures (`completed_section_only_current_record`, `change_with_an_approved_delta`,
`strip_recorded_delta_digests`, `approved_delta_body_as_a_crlf_checkout_writes_it`,
`change_materialized_then_corrected_and_re_approved`) through the shared `include!` scope.

## Collision check

No branch on origin at fetch time (`git grep` over `codex/*`, `grok/*`, `0xleif/*`, `fix/*`, `refactor/*`)
uses REQ-change-113 or REQ-cmd-change-023. Local sibling drafts from the same session, checked in
their worktrees:

| Draft | Branch | `change` delta | `cmd_change` delta | Source overlap |
|-------|--------|----------------|--------------------|----------------|
| #825 verification ignored paths | `0xleif/feat/verification-ignored-paths` (5797cb9f) | ADDED REQ-change-114, 115; MODIFIED `Contract` | ADDED REQ-cmd-change-024; MODIFIED `Contract` | `collect_requirement_ids.rs`, `sdd_version.rs`, `commands/change.rs`, `change_tests.rs`; adds the exported `SddPolicy.verification_ignored_paths` |
| #827 generated markdown dashes | `0xleif/fix/generated-markdown-no-em-dash` (b286c396) | ADDED REQ-change-113 | none | not checked in detail |

So REQ-change-113 collides with #827. Neither sibling modifies `### SPEC SECTION Public API` today,
but this delta restates that whole section. If #825 (or any other change) adds a Public API row
for its new exported item and merges first, this delta's restated section drops that row at
materialization unless it is rebased and re-approved. See the open question in `tasks.md`.
