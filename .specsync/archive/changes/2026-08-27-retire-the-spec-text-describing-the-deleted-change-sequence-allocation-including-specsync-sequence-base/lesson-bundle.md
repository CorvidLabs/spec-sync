# Lesson bundle — retire-the-spec-text-describing-the-deleted-change-sequence-allocation-including-specsync-sequence-base

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Retire the spec text describing the deleted change-sequence allocation, including SPECSYNC_SEQUENCE_BASE
- **Kind**: Documentation
- **Specs**: change
- **Paths**: AGENTS.md
- **Acceptance**: REQ-change-055 is absent from specs/change/requirements.md: the allocation floor, the remote high-water floor and SPECSYNC_SEQUENCE_BASE it describes were all deleted by the ordinal retirement (#665) and none of them is stated anywhere else that survives.
- **Acceptance**: REQ-change-071 is absent: its normative SHALL is directly reversed by REQ-change-072 and its implementation criterion names the deleted allocation floor as its source.
- **Acceptance**: No canonical change spec text asserts that a sequence is allocated, that a next ID is claimed, or that a newly created change generates a ledger claim; the sibling clauses in REQ-change-022, REQ-change-026, REQ-change-070 and REQ-change-072, in the change.spec.md invariants and in context.md all describe the read-only ledger that actually ships.
- **Acceptance**: AGENTS.md no longer instructs agents to set SPECSYNC_SEQUENCE_BASE or to expect change new to floor on a remote ledger, and instead states the slug identity model and the read-only ledger rule.
- **Acceptance**: The ledger invariants that do still hold and are still enforced are left intact and attributed: the commit-side floor in REQ-change-070 and the branch-own-history gate in REQ-change-072.

## Evidence

- Verification commit: `d6f266a4fd683246469eb15a8f632061dd5cfbb4`
- Base commit: `d6f266a4fd683246469eb15a8f632061dd5cfbb4`
- Verified by: `cargo test change::`, `cargo test`, `python3 -S .github/scripts/validate-release-version.py`, `python3 -S .github/scripts/validate-workflow-runtime-pins.py`

## From the change's context.md

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

## From the change's testing.md

# Testing

**No test is added, and that is the honest answer rather than a gap.**

This change alters no code. It deletes two requirements whose entire subject was deleted by
#665, and corrects six clauses of surviving spec text that still described that deleted
subject. There is no behavior on either side of the edit, so there is no assertion that
could be shown to FAIL against a binary built from unfixed `main` — the discrimination
protocol's requirement — other than one that reads `specs/change/requirements.md` and
asserts on its prose. Such a test pins wording, not behavior; it would pass for the wrong
reason the moment anyone rephrased the paragraph, and it would have to be edited by every
future change to the same file. The repository already has the right instrument for this:
`specsync change check` materializes the semantic delta into the canonical spec and refuses
if the delta and the living tree disagree, and `specsync change audit --strict` gates the
result. Both are run below.

The four requirements this change MODIFIES keep their behavior exactly. Their existing
coverage is unchanged and still passes; it is listed here as the standing evidence that the
corrected text describes what ships, not as evidence of anything new.

## Requirement evidence

| Requirement | Standing coverage (unchanged by this change) |
|---|---|
| `REQ-change-022` | `change::tests::sequence_ledger_rejects_unacknowledged_active_and_archived_collisions`, `change::tests::exact_historical_collision_baseline_preserves_immutable_records`, `change::tests::two_archived_packages_sharing_an_ordinal_are_still_refused_until_acknowledged`, `change::tests::ordinal_free_change_ids_do_not_block_numeric_sequence_validation` — CHARACTERIZATION for this change: they already prove the gate finds only historical ordinals and that an ordinal-free identity takes part in no numeric accounting, which is exactly what the corrected criterion now says instead of "independent next-ID claims". |
| `REQ-change-026` | `change::tests::change_sequences_allow_more_than_four_digits`, `change::tests::acknowledged_collision_rejects_mutable_active_records`, `change::tests::a_non_canonical_ordinal_notation_still_fails_closed`, `change::tests::generated_sequence_scope_does_not_suppress_delivery_scope_question` — CHARACTERIZATION: protected-path coverage of the ledger and collision-acknowledgement exactness are untouched; only the criterion asserting that a *newly allocated* change generates a claim is corrected, and `create_change` generates none. |
| `REQ-change-070` | `change::tests::a_stale_sequence_ledger_is_raised_to_the_committed_mark_before_staging`, `change::tests::a_sequence_ledger_ahead_of_the_committed_mark_is_left_alone`, `change::tests::a_sequence_ledger_equal_to_the_committed_mark_is_not_reported` — CHARACTERIZATION: the middle test is the one whose rationale clause is corrected. The behavior it pins (a ledger at or above the committed mark is left byte-identical) is unchanged; only the reason the requirement gives for it stops resting on allocation. |
| `REQ-change-072` | `change::tests::a_branch_that_lowered_the_ledger_after_diverging_is_still_refused`, `change::tests::a_branch_that_raised_then_rewrote_the_ledger_is_refused` — CHARACTERIZATION: they pin `branch_sequence_high_water` reading `HEAD` only. That the gate consults no remote is why the deleted clause about allocation flooring against the remote mark had to go, and why `REQ-change-071` — which required a remote comparison — is retired rather than kept. |

Requirements REMOVED by this change (`REQ-change-055`, `REQ-change-071`) carry no evidence
row by construction: a removed requirement has no implementation to attest to, and both were
removed precisely because theirs no longer exists.

## Acceptance criteria evidence

| Criterion | How it is proven |
|---|---|
| `REQ-change-055` is absent, and nothing it stated survives elsewhere | `grep -rn "REQ-change-055" specs/` returns nothing after materialization; `grep -rn "SPECSYNC_SEQUENCE_BASE\|SEQUENCE_BASE\|sequence_base" src/` returned nothing before it, which is why the requirement had no implementation to lose. |
| `REQ-change-071` is absent | `grep -rn "REQ-change-071" specs/` returns nothing after materialization. `REQ-change-072` remains and is the sole live statement of the ledger gate. |
| No canonical change spec text claims a sequence is allocated | `grep -rniE "allocat\|mint\|next-ID\|monotonic\|high-water" specs/change/` — every surviving hit describes the read-only ledger, the slug identity model (`REQ-change-086`), or scratch-path allocation in an unrelated requirement. |
| `AGENTS.md` no longer instructs agents to set an inert variable | `grep -rn "SPECSYNC_SEQUENCE_BASE" .` matches only the frozen archive packages, which are history and must not be rewritten. |
| The surviving ledger invariants are intact and attributed | `REQ-change-070` and `REQ-change-072` are still present, and their coverage above still passes. |

## Verification commands

- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --check`
- `cargo test`
- `specsync change check <id>` (materializes the delta and refuses on disagreement)
- `specsync change audit --strict`

## Where these lessons go

- `specs/change/context.md`
