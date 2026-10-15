---
id: refuse-incomplete-configured-root-coverage-after-cargo-discovery-loss-for-specsync-6-1-1
state: archived
type: bug_fix
base_commit: 5f0c9c69a0b40963c7a62c06feb74854e4aa8341
---

# Refuse incomplete configured root coverage after Cargo discovery loss for SpecSync 6.1.1

## Intent

Refuse incomplete configured root coverage after Cargo discovery loss for SpecSync 6.1.1

## Affected Canonical Specs

- `validator`

## Acceptance Criteria

- Coverage and check must never report a complete partial inventory when an eligible selected regular root source file loses checked Cargo exact-target authority, including a previously inferred selection whose manifest disappears between discovery phases. Valid checked Cargo files retain existing coverage; malformed, missing or changed manifests with an unrecognized eligible selected regular root file fail inconclusive instead of omitting it. Explicit directory-only fallback, extension/test/exclusion filtering, no-follow confinement, exact-file boundaries and existing input budgets remain intact. Preserve the reproduced nonempty50-to-false100 before case, prove corrected text/JSON controls and pass independent review and final pinned local verification before genuine acceptance and release gates.

## No-spec Rationale

Not applicable
