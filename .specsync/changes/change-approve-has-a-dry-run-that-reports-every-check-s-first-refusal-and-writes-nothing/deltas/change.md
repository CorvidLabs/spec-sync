---
change: change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing
artifact: delta
module: change
---

## ADDED

### REQUIREMENT REQ-change-163

The system SHALL offer a dry run of the definition approval that says whether `change approve` would accept a change as it stands, and writes nothing. It SHALL run the checks `change approve` runs, in the same order and through the same functions, so the two cannot differ: the relayed-decision check, loading the change, the sequence-collision check, the state guard, the roster check, the legacy-baseline binding, the definition check, the delta check, the declared-path ownership check, the `verification_ignored_paths` check, building the approval record, and resolving the actor. It SHALL run every check that can run on its own and report the first problem from each, with the message `change approve` gives for it. It SHALL name each check it skipped and the refused check that check waits on: every later check when the change does not load or its state does not admit approval, and building the approval record when the delta check refuses. The first problem it reports SHALL be the one refusal `change approve` gives for the same change and flags, except when an interrupted lifecycle write is waiting to be recovered. It SHALL create, change or remove no file: not the change's `state.json`, `approvals.json` or `change.md`, no canonical spec file, not `.specsync/hashes.json`, not the lifecycle lock, and not the read cache. The legacy-baseline binding and a relayed decision's workflow-version raise SHALL stay in memory. When an interrupted lifecycle write is waiting to be recovered, it SHALL refuse with that reason and run no other check, because a real approval recovers it first, which is a write. It SHALL accept every state `change approve` accepts, including a draft and a change that was sent back. `change approve` itself SHALL refuse exactly what it refused before, with the same messages.

Acceptance Criteria
- For a complete draft with one of these faults, the dry run reports the message `change approve` then gives, and `change approve` still refuses with it: an unknown delta operation heading; an invalid, bare or non-ASCII delta item heading; a delta with no recognized heading or no item; an empty item; an empty delta file; an ADDED requirement whose id already exists; one item declared twice; a requirement with no SHALL; an ADDED requirement with no Acceptance Criteria; another module's requirement id; a malformed requirement id; an affected path no declared module owns, with and without a declared spec; a declared spec whose front matter does not parse; an unfinished interview.
- A clean draft reports no refusal, and `change approve` then succeeds.
- A draft with an open interview question and an invalid delta reports both in one run, each under its own check.
- A change id that does not exist reports one refusal and names every later check as skipped.
- Every file of the project, with ignored and untracked files, the Git index and SpecSync's caches, is byte for byte the same before and after the dry run, for a refused draft and for a clean one, with and without an existing lifecycle lock and hash cache.
- A change that was sent back reports what its re-approval would.
