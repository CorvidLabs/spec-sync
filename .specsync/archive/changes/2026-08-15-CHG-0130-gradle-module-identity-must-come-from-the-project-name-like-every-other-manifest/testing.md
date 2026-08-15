---
change: CHG-0130-gradle-module-identity-must-come-from-the-project-name-like-every-other-manifest
artifact: testing
---

# Testing

Sandbox gate 065 is the judge, and the whole-board check is what distinguishes
this from the rejected attempt — which also closed its own gate:

    gate 065     before  FAIL (pending)      after  pass=5 fail=0 pending=0
    full board   before  pass=39 fail=16     after  pass=40 fail=15

Exactly one drill changed state. The rejected attempt would have failed this
check: it collapsed monorepo and workspace layouts into a module named `src` and
made `generate` write one wrong spec for the whole tree.

The single case worth stating on its own, because a plausible-but-wrong name is
how the last attempt passed a naive gate: **a Gradle build with no
`rootProject.name` is named from the project directory** — Gradle's own default.
On the drill tree that directory is `kt`, so coverage reports `kt`, and
`generate` lists it without writing a spec.

Suite: fmt clean, clippy clean, 2258 unit + 367 integration, 0 failures.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-manifest-002 | Gate 065 flips to 0 pending. A build with no `rootProject.name` falls back to the project directory rather than to any package segment — the case that separates a real fix from a differently-wrong name |
| REQ-validator-016 | No child of a JVM source root is a module, so the "which segment" question that produced both the original bug and the rejected fix no longer exists. The whole-board check confirms no other discovery path moved |
