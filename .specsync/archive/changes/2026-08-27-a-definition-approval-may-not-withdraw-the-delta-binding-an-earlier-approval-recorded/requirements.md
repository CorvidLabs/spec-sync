---
change: a-definition-approval-may-not-withdraw-the-delta-binding-an-earlier-approval-recorded
artifact: requirements
---

# Requirements

## Normative

1. A portable SpecSync 5.0.1 definition approval SHALL record, on both members of its marked
   pair, the per-module digest of the semantic delta bodies it approves — the same claim every
   other `definition` gate records.
2. A portable SpecSync 5.0.1 definition approval SHALL leave its own projection unchanged: the
   pair's current digest, legacy digest, pair metadata and resolution SHALL be exactly what they
   were before the delta binding was recorded on them.
3. Materialization and acceptance SHALL refuse a change whose effective definition approval
   records no semantic delta wording while some definition approval in the same ledger records
   it, and the refusal SHALL name `specsync change approve <id>` as the remedy.
4. Materialization and acceptance SHALL continue to proceed when no definition approval in the
   ledger records semantic delta wording, including when the ledger holds several such
   approvals, because that ledger predates the binding and withdrew nothing.
5. A portable SpecSync 5.0.1 definition approval SHALL remain available on a workflow-v1 change
   that has no prior definition approval.

## Acceptance criteria

- `change approve` followed by `change approve --portable-5-0-1` leaves the change's effective
  definition approval carrying the same per-module delta digests the ordinary approve recorded.
- A ledger whose latest definition approval records no delta wording while an earlier one does
  is refused at materialization, the swapped wording does not reach the canonical spec, and the
  change is not recorded as applied.
- A ledger in which no definition approval ever recorded delta wording still materializes, even
  with a swapped body and even with more than one silent definition approval.
- `--portable-5-0-1` on a workflow-v1 change with an empty approval ledger succeeds and records
  the delta bodies it approved.
- No existing approval ledger in this repository is invalidated: all 197 were scanned for the
  refused shape and none matches.

## Out of scope

- Changing what absence of `approved_delta_digests` means for a single-approval historical
  ledger. It stays UNKNOWN, never VIOLATED.
- The other findings filed alongside #719 (mixed-operation duplicate keys, first-match apply on
  `### {REQ id}`, `registry.rs` as a non-canonical frontmatter reader). Separate defects,
  separate changes.
