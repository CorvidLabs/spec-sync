---
spec: cmd_change.spec.md
---

# Context

The command layer intentionally contains no lifecycle policy, keeping agent and terminal behavior identical and the domain module independently testable. Its full text/JSON lifecycle passed end-to-end integration tests and project-level dogfooding for 5.0.

`change reopen` prints the domain `ReopenResult` directly in JSON so persisted and emitted audit metadata cannot drift. Human output states that fresh verification and closing approval are required.

`change correct` follows the same thin-dispatch rule. JSON emits the persisted correction, complete ordered history, effective definition, and summary; human correct/show/status output distinguishes original and effective values and names the next gate. All correction policy remains in `src/change.rs`.

`change correct-owner` also remains a thin adapter. It resolves repeated paths, a manifest, or `--all-missing` into domain entries, JSON emits the corrected persisted record, human output names the exact owner repair (or batch count) and next gate, and all path, ownership, state, and transactionality policy remains in `src/change.rs`.

The adapter now brackets only list, show, status, and check with the change domain's bounded read
snapshot guard. The guard drops before command return. Mutation arms remain unchanged and uncached.
