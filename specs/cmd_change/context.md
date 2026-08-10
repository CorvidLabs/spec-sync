---
spec: cmd_change.spec.md
---

# Context

The command layer intentionally contains no lifecycle policy, keeping agent and terminal behavior identical and the domain module independently testable. Its full text/JSON lifecycle passed end-to-end integration tests and project-level dogfooding for 5.0.

`change reopen` prints the domain `ReopenResult` directly in JSON so persisted and emitted audit metadata cannot drift. Human output states that fresh verification and closing approval are required.

`change correct` follows the same thin-dispatch rule. JSON emits the persisted correction, complete ordered history, effective definition, and summary; human correct/show/status output distinguishes original and effective values and names the next gate. All correction policy remains in `src/change.rs`.

`change correct-owner` also remains a thin adapter. It resolves repeated paths, a manifest, or `--all-missing` into domain entries, JSON emits the corrected persisted record, human output names the exact owner repair (or batch count) and next gate, and all path, ownership, state, and transactionality policy remains in `src/change.rs`.

`change review` parses the optional verdict through the typed change-domain enum. Text names the
stored verdict and JSON emits the exact review record; reviewer independence, evidence freshness,
and finalization eligibility remain domain concerns. Reviewer text is a stable ASCII claim, every
attempt is append-only, and hosted required-check provenance supplies authenticated merge trust.

`answer`, `depend`, and `supersede` validate an existing change's correction-ledger health before
calling their mutation-capable domain operations. This preserves the command adapter's thin role
while ensuring a subsequent text/JSON projection cannot fail after persistence already occurred.
