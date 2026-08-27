---
change: retire-the-spec-text-describing-the-deleted-change-sequence-allocation-including-specsync-sequence-base
artifact: testing
---

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
