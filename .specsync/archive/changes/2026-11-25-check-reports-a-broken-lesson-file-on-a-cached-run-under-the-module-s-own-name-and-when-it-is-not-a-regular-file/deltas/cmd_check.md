## MODIFIED

### REQUIREMENT REQ-cmd-check-023

`check` SHALL report a module's `<module>.3md` lesson file that exists beside its spec but cannot be read or parsed as 3md as a warning on that spec, naming the file, the parser's error, and that the module's `context.md` is used for its lessons instead, so `check --strict` fails on it. The file is the one the lifecycle reads: it is named for the spec's module, whatever the spec file is called, and an entry of that name that is not a regular file SHALL be reported as one that cannot be read. A `.3md` that parses, and a module without one, SHALL add no finding. Reading a module's lessons still falls back to `context.md` and never fails a lifecycle command (REQ-change-106); a strict lifecycle check, `change check --strict`, fails on this warning as on any other.

Acceptance Criteria
- A spec whose `<module>.3md` has two planes at the same `z` gets one warning naming the file, the parser's error and `context.md`, and `check --strict` exits non-zero.
- A spec whose `<module>.3md` parses, or that has none, gets no such warning, and `check --strict` passes for it.
- A spec file not named for its module, such as `main.spec.md` for module `billing`, is checked against `billing.3md`; a `main.3md` beside it adds no finding.
- A directory named `<module>.3md` gets the warning, and `check --strict` exits non-zero.
- After a clean `check`, a `check` with no flags reports a lesson file that was added broken or broken since (REQ-hash-cache-011).
