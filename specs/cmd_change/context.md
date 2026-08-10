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

`answer`, `depend`, and `supersede` delegate correction-ledger health enforcement to their
mutation-capable domain operations, which reload and validate the ledger while holding the same
project lock used for persistence. Successful mutations return the validated effective definition
and correction history used for output; the command adapter retains live ledger validation only on
read-only rendering paths. This preserves the thin-dispatch boundary while preventing both the
lock-wait race and a false post-persistence failure. Human mutation output obtains its optional
correction counts from a separate best-effort `state.json` reload keyed by the original command ID;
the validated correction snapshot remains confined to JSON output so correction data cannot flow
into cleartext sinks.
