---
change: report-states-the-require-coverage-verdict-when-another-finding-decides-the-exit
artifact: context
---

# Context

Issue #605. In a tree that is not a git repository, staleness cannot be measured and
`report` exits 1 (REQ-cmd-report-003, deliberate). Under gating enforcement that
finding exits before `--require-coverage` is consulted, so the gate never ran and
never printed a verdict. `--require-coverage 0` exited 1, and a CI reader could not
tell a coverage failure from the staleness refusal.

When another finding will decide the exit, text and markdown output now print the
coverage gate's own verdict (met or not met, with the measured figure) and name the
findings that set the exit status. Exit codes are unchanged. JSON already carries
`staleness_inconclusive` and the coverage figure, and CSV gates silently by design.

Kimi left an unapproved stub draft that planned new JSON fields
(`require_coverage`, `coverage_gate_passed`). That would grow the public contract
beyond the bug, so it was discarded. Its constraint ("the staleness refusal stays;
both gates report") is kept.

Authorization: Leif's 2026-11-10 merge rule lets bug fixes that make behavior match
existing intent (COVERAGE-2) run the full lifecycle with 0xLeif as approve and review
actor.
