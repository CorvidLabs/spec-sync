---
change: name-a-held-cargo-build-directory-lock-before-verification-blocks-on-it-and-reap-the-verification-child-s-process-group
artifact: docs
---

# Docs

No user-facing site page changes. The behaviour is documented where an operator will actually meet
it — in the output itself and in the module's own contract:

- `specs/change/change.spec.md` contract item 5 gains the process-group and build-lock obligations.
- `specs/change/requirements.md` gains `REQ-change-091` with its acceptance criteria.
- `specs/change/context.md` records the three deliberate refusals (no time-based heuristic, silence
  over a wrongly-named lock, a PID only where the platform reports ownership), the rejected
  alternatives, and the accepted `SIGTTIN` cost of moving the child out of the terminal's
  foreground group.
- `specs/change/testing.md` records which assertion is a discriminator, which is a control, and
  what is stated as untested rather than covered.

The notice is self-documenting on purpose. Where the platform cannot name a holder, the second line
is the remediation an operator would otherwise have had to know to look for:

```
specsync: name the holder with `lsof target/debug/.cargo-lock` and end it if it outlived an interrupted check
```
