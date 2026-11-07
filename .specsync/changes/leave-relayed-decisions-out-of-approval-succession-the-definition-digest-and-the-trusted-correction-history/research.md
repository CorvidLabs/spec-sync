---
change: leave-relayed-decisions-out-of-approval-succession-the-definition-digest-and-the-trusted-correction-history
artifact: research
---

# Research

- `ledger_succession` (`src/change/consider_accepted_evidence_anchor.rs`) parses both ledgers as JSON values and compares the earlier approvals and reopen events with the current ones, then the first new reopen event's `superseded_approval` with the earlier terminal approval, all raw.
- `definition_digest_from_snapshot_with_task_mode` (`src/change/collect_requirement_ids.rs`) builds `CorrectionLedger { corrections }` and hashes `json_content` of it.
- `validate_trusted_correction_history` (`src/change/historical_definition_digest_matches.rs`) serializes each trusted correction and the current one and refuses on any byte difference.
- `correction_digest_projection` (`src/change/relayed_decision.rs`) already clears `relayed_by` on both embedded approvals for the two correction digests, and `reviews_match_ignoring_relays` does the same for review history.
- #900 is unreleased (no 6.x release carries `relayed_by`), so the definition digest of a change whose correction embeds a relay moves only for builds of main from 2026-11-06; no published binary computed it with the relay.
