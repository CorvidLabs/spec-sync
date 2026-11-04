---
id: refuse-a-stale-modified-delta-instead-of-overwriting-newer-canonical-text
state: implementing
type: bug_fix
base_commit: f82eb09e2d3da852ff58e0cfbb734ab6faba724e
---

# Refuse a stale modified delta instead of overwriting newer canonical text

## Intent

Refuse a stale modified delta instead of overwriting newer canonical text

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- A change approved against older contract text never silently overwrites newer text another change put there.
- Checking that change stops and names the requirement or section whose text moved.
- The project health check (audit) reports the same problem before anything is lost.
- The message tells me the way forward: rewrite the change onto the current text and approve it again.

## No-spec Rationale

Not applicable
