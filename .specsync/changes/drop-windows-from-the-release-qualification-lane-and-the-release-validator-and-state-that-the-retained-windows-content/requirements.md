---
change: drop-windows-from-the-release-qualification-lane-and-the-release-validator-and-state-that-the-retained-windows-content
artifact: requirements
---

# Requirements

Recorded in `context.md`, `docs.md` and `testing.md` for this change: the decision (drop Windows
from the qualification lane and the release validator), the argument it overrules and why that
argument was correct, the one piece of #734 that is kept because it stands on its own merits, the
`REQUIRED_PLATFORMS`/matrix coupling that must move together, and the disclosure that the retained
`#[cfg(windows)]` guarantees are now best-effort and unverified.

No canonical spec text changes: this is CI configuration, one validator constant, a test-only `cfg`
attribute, and prose. No CLI behaviour, API surface, or output format changes — the published binary
set is unchanged; what changes is which platforms we verify, and the documentation now says so.
