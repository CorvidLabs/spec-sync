---
change: bounded-failure-output-never-splits-a-multi-byte-character-and-gate-refusal-evidence-is-tested
artifact: context
---

# Context

PR #1027 (Fixes #640), round 2. The corvid-agent review at `85056618` reproduced a panic:
`bounded_failure_output` sliced `joined[len-4096..]` by byte offset, so findings over
4 KiB whose cut lands inside `↔`, `—` or `…` (common in this repo's specs) panicked
instead of recording the failure #640 is about. The fix walks the cut forward to a
character boundary before looking for the next line break.

The review also found that REQ-change-151 AC3 (gate-refusal evidence carries the refusal
texts) had no test. The existing integration test that produces the
`(effective contract: auth)` entry now asserts its `output`. A separate fixture that only
drifted an export hit the spec↔code check, not the gate, so it was dropped.

The spec nits (invariant 52 below 51, the z=45 plane formatting) landed as a separate
commit before this change. Main's #1021 took z=44.

Authorization: Leif's 2026-11-10 merge rule lets bug fixes that make behavior match
existing specs run the full lifecycle with 0xLeif as approve and review actor. The added
acceptance criterion tightens this PR's own unmerged REQ-change-151 to the behavior its
first criterion already promised (a bounded record of findings, not a crash).
