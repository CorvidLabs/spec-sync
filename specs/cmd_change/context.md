---
spec: cmd_change.spec.md
---

# Context

The command layer intentionally contains no lifecycle policy, keeping agent and terminal behavior identical and the domain module independently testable. Its full text/JSON lifecycle passed end-to-end integration tests and project-level dogfooding for 5.0.

`change reopen` prints the domain `ReopenResult` directly in JSON so persisted and emitted audit metadata cannot drift. Human output states that fresh verification and closing approval are required.

`change correct` follows the same thin-dispatch rule. JSON emits the persisted correction, complete ordered history, effective definition, and summary; human correct/show/status output distinguishes original and effective values and names the next gate. All correction policy remains in `src/change.rs`.

`change correct-owner` also remains a thin adapter. It resolves repeated paths, a manifest, or `--all-missing` into domain entries, JSON emits the corrected persisted record, human output names the exact owner repair (or batch count) and next gate, and all path, ownership, state, and transactionality policy remains in `src/change.rs`.

`change review` parses the optional verdict through the typed change-domain enum. Text names the
stored verdict and JSON emits the exact review record; reviewer-claim validation, evidence
freshness, and finalization eligibility remain domain concerns. The reviewer MAY be the same
actor as definition approval. Next-action copy uses `--reviewer <human>` so the command layer
does not tell solo projects to invent a second identity. Reviewer text is a stable ASCII claim,
every attempt is append-only, and hosted required-check provenance supplies authenticated merge
trust. GitHub required reviews remain the two-person gate when a repository wants one.

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

`ready_to_finalize` and `finalize` are two answers to one question, and the report is where they were
allowed to disagree. `review_present` asked `review.json.is_file()` while `finalize` required the
review to still be current, so ship-status recommended `specsync change ship` in the same second the
verb refused. #689 had already moved the verification half of that conjunction onto content and left
the review half asking whether a file existed — the same fix, one term short. When a status command
and the command it recommends both compute readiness, the status command must consult the *predicate*
the other one gates on, not a proxy for it; and the regression test must assert that the two AGREE
rather than that either returns a particular value, or it goes red the day the underlying policy is
decided the other way.

The existence-only question had three call sites inside one function, not one. Fixing only
`ready_to_finalize` would have left the `review_tip` stage reporting `done` for a review finalize
rejects and the text line printing `recorded` — and, because `archive_tip` is gated on
`ready_to_finalize`, it would have left NO stage marked `current`, which silently drops `ship_next`
back to the generic lifecycle sentence instead of naming the recovery. A report that has stopped
lying in its summary field and still lies in its stage list has not been fixed. `ChangeSummary` asked
the currency question correctly the whole time, which is the tell: when one projection of a value is
wrong, look for the sibling projections before concluding the defect is where it was reported.

Where a guarantee cannot be evaluated, the honest report is a third answer, not a rounded second one.
`unavailable` renders as `ready_to_finalize: false` plus a warning naming the re-review that
re-anchors it — not a blocker, because asserting that an unobtainable guarantee ought to block would
decide an open design question by accident. Readiness can decline to answer without also deciding
what should happen next.
