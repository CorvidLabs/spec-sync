---
change: CHG-0151-a-test-that-pinned-deleted-release-lane-behaviour-must-go-with-it
artifact: requirements
---

# Requirements

`--no-spec-change`: `.github/scripts/` is CI tooling with no owning spec module, so this adds
no `REQ-` block to the living tree (precedent CHG-0014).

The obligation is narrow: no test in `test-validate-release-candidate.py` may anchor on
`release.yml` text that no longer exists, and removing the orphan must not weaken any assertion
whose subject survives. Evidence is the anchor enumeration (1 orphan of 20) and the full suite
passing at 49 tests.
