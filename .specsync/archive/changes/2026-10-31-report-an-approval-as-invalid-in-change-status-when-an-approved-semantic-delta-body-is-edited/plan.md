---
change: report-an-approval-as-invalid-in-change-status-when-an-approved-semantic-delta-body-is-edited
artifact: plan
---

# Plan

## Approach

Make every active-state reader of approval currency ask the same two questions that
materialization and acceptance already ask:

- Does the approval still match the scope?
- Do the delta bodies it recorded still match?

The readers are JSON status, the handoff verdict, `change audit` and text `Next:`. The domain
gains one composed predicate, a module list that carries the reason, and one boolean export for
the command layer. `ensure_definition_approval_valid` and all historical paths stay as they are.

## Out of scope

- Adding `plan.md` or any other execution artifact to the approval. They stay
  execution-digest-bound by design.
- Changing `ensure_definition_approval_valid`, archived or accepted authentication, terminal
  evidence, digests, persisted ledgers or schemas.
- Changing the materialization or acceptance refusal text.
- `ship-status` stage tables beyond the lifecycle next action they already take from
  `text_mode_next_action`.
- The strict-verification defect (Chough BUG-0003), which is a separate package.
- Releasing, tagging or merging.

## Steps

1. Write the discriminator tests listed in `testing.md` and confirm they fail on f240d37.
2. Refactor `ensure_approved_delta_bodies_unchanged` into a private helper that returns the
   drifted module list. Keep the public refusal text byte-identical.
3. Add the composed predicate and `changed_approved_deltas` to `ChangeSummary`. Use the predicate
   in `summarize_change_with_effective`, `handoff_summary` and `check_project_with_command_output`
   for approved, implementing and verifying records only.
4. Export `approval_current_for_guidance` and use it in `text_mode_next_action`.
5. Run the controls. Confirm that archives, ledgers without digests, CRLF-only bodies and
   `plan.md` edits are unchanged.
6. Update `site/src/content/docs/cli.md`, the change and cmd_change companions, and lessons.
   Approval materializes the deltas into `specs/`.
7. Run the regression lanes, then `fledge lanes run pre-push` before any push.

## Risks

- `change audit` now exits 1 for an active change whose delta was edited after approval. Any
  pipeline that ran audit on such a branch was green while `change check` refused, and it will now
  fail. That is the intended correction, and it is called out in the PR.
- A consumer that read `approval_valid` as "scope digest matches" sees `false` with an empty
  `scope_expansion`. `changed_approved_deltas` names the reason.
- Status reads each delta body once more per call. The read is bounded by
  `read_bounded_change_text` and is small next to `project_input_digest`.
- Two files are near the 5,000-line limit. `historical_definition_digest_matches.rs` is at 4,398
  lines and the chosen test file at 3,997. Additions must stay under the limit (REQ-change-109).

## Constraints consulted

- `specs/change/change.3md`: absent delta binding is unknown, and absence is judged per ledger
  (REQ-change-089, REQ-change-090). The binding folds CRLF only. Bool predicates discard reasons.
  Sweep every sibling reader.
- `specs/cmd_change/cmd_change.3md`: no lifecycle policy in the command layer, no digest in text,
  and the Handoff line renders the domain verdict verbatim.
- REQ-change-109: the line limit.
- HI: CHANGE-6.a, CHANGE-6.c, CHANGE-7, CHANGE-7.a and SHIP-7. No new HI criterion is proposed.
