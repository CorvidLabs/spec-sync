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

Structured mutation rendering also selects the normal or explicit-strict summary captured by the
domain operation before it released the project lock. It never recomputes correction health from a
live ledger, so one response cannot contain a validated effective definition beside a contradictory
invalid-correction summary.

Each lifecycle verb builds its own next-action string. That is why `finalize` named the lesson
fold-back and `ship` did not: the same guidance had two authors, and only one was updated. The
archived bundle was written and nothing told the operator it existed — on the verb the tool
itself recommends. When guidance about the same step exists on more than one exit, extract it to
one pure function and pin it with a test; the regression is always a future edit to one verb that
forgets the other. `merge_before_finalize_warning` and `ship_next_action` are both that shape.

Surfacing affordances fail open and never gate: an unreadable context file yields no pointer
rather than an error. This is deliberately the opposite posture from evidence validation, which
fails closed throughout the domain. The distinction is whether the artifact is load-bearing for
trust or an aid to the author.
