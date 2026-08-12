---
change: CHG-0106-make-verification-currency-a-content-question-delete-the-git-ancestry-walk-the
artifact: context
---

# Context

Step 2 of the 6.0 reduction, and the edit the design called load-bearing: this is where
content and identity actually separate.

`verification_is_current_checked_with_project_digest` interleaved two questions. Its first
four checks asked *"is this validation still true?"* — passed, plan digest matches,
project-input digest matches. Its fifth asked *"can this evidence be trusted as un-tampered
history?"* — a git walk over descendants of the verification commit, filtered by the
`REQ-change-016` path allowlist. The function split cleanly on that line.

`workspace_digest` and `project_input_digest` were never two values; they are one computation
with two names. So the separation happens at the consumers, not the computation:
content-freshness consumers keep comparing, history-trust consumers are deleted.

## What dissolves by construction

- **Squash orphaning.** A squash discards the recorded verification commit, so
  `merge-base --is-ancestor` could never pass again and squash-merged changes were permanently
  unfinalizable.
- **The finding-4 deadlock.** The allowlist forbade `approvals.json` and
  `change-sequence.json` moving between verification and review, while the guided path told
  authors to commit exactly those. The lifecycle instructed a commit its own gate refused.

Measured, not assumed: sandbox drill 008 previously reported "squash-merge before finalize
strands the change and fails closed". Against this build it no longer does — it stops at
*scoped review* staleness instead, a different subsystem removed in the next step. Half the
squash problem is gone; the other half is step 3.

## Reduced detection, deliberately

A source change followed by a revert no longer stales evidence. The tree is byte-identical to
the one verified, so the content answer is "still current"; the ancestry walk used to see the
intervening commit. Detecting that work happened in between is a provenance question, and
`attest` records provenance against commit SHAs in git notes. This is a real reduction in what
the tool notices and is recorded in the CHANGELOG as such.
