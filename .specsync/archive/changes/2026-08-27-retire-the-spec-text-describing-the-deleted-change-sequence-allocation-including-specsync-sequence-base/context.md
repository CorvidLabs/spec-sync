---
change: retire-the-spec-text-describing-the-deleted-change-sequence-allocation-including-specsync-sequence-base
artifact: context
---

# Context

Found while backfilling the 6.0 changelog (#728), by reading the diff of the ordinal
retirement (#665, `7cbe820e`) rather than its title.

`REQ-change-055` was live and unsuperseded. It required change-sequence allocation to floor
on the highest locally observed sequence and on the remote default-branch
`.specsync/change-sequence.json` high-water, and offered `SPECSYNC_SEQUENCE_BASE` to
multi-clone fleets so they would not mint the same numeric `CHG` prefix. The ordinal
retirement deleted `maximum_observed_sequence` and `remote_sequence_high_water`, and change
identity became a slug minted from the description. Re-derived rather than taken on trust:

- `SPECSYNC_SEQUENCE_BASE`, `SEQUENCE_BASE` and `sequence_base` appear in zero `.rs` files,
  so nothing reads it and nothing constructs it dynamically either.
- `create_change` calls `mint_change_slug` and `allocate_change_workspace`. There is no
  sequence allocation, no ledger write, and no force-add of the ledger to `affected_paths`.
- `src/change.rs` says it outright in a diagnostic: "nothing writes this file any more, so
  it cannot be repaired by allocating".

The read side is deliberate and must survive. `.specsync/change-sequence.json` is still
loaded by `validate_change_sequences` and is still the subject of two live gates:
`floor_sequence_ledger_to_committed` raises a stale working-tree copy before staging so a
lifecycle commit cannot record it downwards (#533), and the validation gate refuses a ledger
below the mark the branch's own history recorded. Both are stated by requirements that
remain accurate — REQ-change-070 and REQ-change-072 — which is why REQ-change-055 could be
removed rather than rewritten: rewriting it would have restated REQ-change-070 and
REQ-change-072 and added nothing but a third place to go stale.

## The sibling sweep

The issue named only REQ-change-055. A sweep of every spec mention of sequences, ordinals,
`CHG` prefixes and high-water marks found six more pieces of text describing the same
deleted mechanism, all of which the retirement left behind:

- `change.spec.md` Invariants 1 — "Change IDs are monotonically assigned as `CHG-NNNN-slug`".
- `context.md` — "Numeric change allocation is additionally claimed in the committed ledger",
  and "every newly allocated change includes its generated claim in the affected path scope".
- REQ-change-026 — the same generated-claim criterion.
- REQ-change-022 — "Repository-backed sequence claims make independent next-ID claims
  conflict during Git integration".
- REQ-change-072 — "allocation on it continues to floor against the remote mark".
- REQ-change-070 — a rationale clause resting on "a newer claim is the ordinary result of
  allocating a change".

REQ-change-071 is a second, independent instance of the same defect shape. Its normative
SHALL requires refusing a ledger below the mark the DEFAULT BRANCH published; REQ-change-072
requires the opposite — that a branch not be refused for trailing the default branch, and
that the gate consult no remote — and the shipped gate (`branch_sequence_high_water`) reads
`HEAD` only. Its last acceptance criterion also named the deleted allocation floor as the
source for the published mark. It is retired here rather than left as a live SHALL the code
deliberately violates.

REQ-change-054 and REQ-change-056, which the issue asked to be checked, are about placeholder
TODO artifact content and correction-ledger health. Neither mentions sequences. Verified and
left alone.

## Ruled out

- Rewriting REQ-change-055 into a read-only ledger requirement. The invariant it would carry
  is already carried, twice, by REQ-change-070 and REQ-change-072.
- Touching the ledger file, the gates, or any code. Nothing here is a behavior change, so
  no test is added: there is no assertion that could be made to fail against unfixed `main`
  other than one that reads the spec file, which would pin prose rather than behavior.
- Part 2 of #728 — a distinct `unattributed requirement` outcome in the drift model. Left
  open deliberately; it is a change to the tool's model, not to this spec's text.
