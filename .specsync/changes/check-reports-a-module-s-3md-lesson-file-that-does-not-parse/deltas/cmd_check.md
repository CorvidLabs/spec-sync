## ADDED

### REQUIREMENT REQ-cmd-check-018

`check` SHALL report a module's `<module>.3md` lesson file that exists beside its spec but cannot be read or parsed as 3md as a warning on that spec, naming the file, the parser's error, and that the module's `context.md` is used for its lessons instead, so `check --strict` fails on it. A `.3md` that parses, and a module without one, SHALL add no finding. Lifecycle commands still read `context.md` for that module's lessons and never fail on it (REQ-change-106).

Acceptance Criteria
- A spec whose `<module>.3md` has two planes at the same `z` gets one warning naming the file, the parser's error and `context.md`, and `check --strict` exits non-zero.
- A spec whose `<module>.3md` parses, or that has none, gets no such warning, and `check --strict` passes for it.
