---
change: record-a-change-that-holds-a-relayed-decision-at-workflow-version-3-so-released-specsync-6-2-0-refuses-it-instead-of
artifact: design
---

# Design

- **Version 3 is workflow v2 plus the gate.** `RELAY_GATED_WORKFLOW_VERSION = 3`. Code that branches on `workflow_version >= 2` or `< 2` already treats 3 as v2. The one literal `== 2` comparison (the archived same-PR finalization check) becomes `>= 2`. Loading accepts versions 1, 2 and 3. The anchor accepts (3, Some(2)) on active and historical reads, and refuses (3, None) and (3, Some(other)).
- **History.** `validate_workflow_version_transition` and the HEAD-versus-working-tree check accept one move: 2 to 3, with an origin of 2 (or an omitted origin on an old v2 record) becoming 2. Everything else is still refused, 3 back to 2 included.
- **Digests.** `definition_digest_for_correction_count` serializes the record with version 3 written as 2. Every recorded execution digest, and so verification, review and closing currency, is the same before and after the raise.
- **Raise points.**
  - `approve_definition_with_relay`: when a relay is given on a v2 change, set the version to 3 and save `state.json` before the approval is appended.
  - `record_scoped_review_with_relay`: on a v2 change, `state.json` goes first in the same prepared write as `review.json` and `review-attempts.json`.
  - A workflow-v1 change keeps version 1. A `--portable-5-0-1` pair needs v1, and exists so old binaries can read it. Approve and review print a note there: SpecSync 6.2.0 and older can still drop the relay. REQ-change-relayed-decision-001 is unchanged.
  - Every other `state.json` write (`save_change`, materialize, accept/finalize) goes through one helper. It raises a v2 record to 3 when the change directory's `approvals.json`, `review.json`, `review-attempts.json` or `corrections.json` holds a `relayed_by`. That covers corrections and records written by unreleased builds.
- **Archive.** Finalize copies the loaded record, so the archived `state.json` and `accepted-state.json` keep version 3.
- **Rejected:** refusing relays on workflow-v1 changes. It would remove the portable pair's relay (REQ-change-relayed-decision-001), and the pair exists for old binaries. Also rejected: creating every new change at version 3. It would lock 6.2.0 out of every repository 6.3.0 touches, relay or not. Also rejected: editing archived v2 changes that hold relays, because archives are not edited.
