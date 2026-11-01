---
change: report-an-approval-as-invalid-in-change-status-when-an-approved-semantic-delta-body-is-edited
artifact: design
---

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
