---
change: CHG-0081-make-a-fresh-project-usable-out-of-the-box-stop-a-leftover-directory-from-block
artifact: context
---

# Context

Three defects found by running the real binary the way a person would, in the
spec-sync-sandbox drills. None is visible to the Rust suite, which uses
single-process single-root fixtures.

`specsync init` detected a test command for Cargo, bun, Swift and fledge and
silently wrote an empty verification_commands list for everything else. Every
lifecycle command then failed closed on it, naming a file the user had never
opened, so a freshly initialised project could not complete its own lifecycle.

Git cannot track empty directories, so checking out a branch that does not contain
a change leaves a husk behind, typically an empty deltas/. Two active-change read
paths treated that husk as a corrupt change and failed closed on the missing
state.json, so `change new` failed outright on any branch not containing an
earlier change. That also caused `audit --strict` to report the sequence ledger
as an uncovered path.

`verify_change_with_strict` acquired the project lock and ran its whole body
inline, so no caller already holding the lock could re-run verification.
`acquire_project_lock` is a non-reentrant flock, so such a call hangs rather than
fails.
