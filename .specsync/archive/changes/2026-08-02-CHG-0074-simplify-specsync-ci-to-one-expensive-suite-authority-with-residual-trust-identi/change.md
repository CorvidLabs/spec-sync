---
id: CHG-0074-simplify-specsync-ci-to-one-expensive-suite-authority-with-residual-trust-identi
state: archived
type: operations
base_commit: 0ea95ce5a409ee7685bb31a182ef2c633bf9baee
---

# Simplify SpecSync CI to one expensive-suite authority with residual Trust identity gates, preserving full local verification and documenting the 95% confidence model

## Intent

Simplify SpecSync CI to one expensive-suite authority with residual Trust identity gates, preserving full local verification and documenting the 95% confidence model

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- Trust lifecycle no longer invokes the full verify lane or cargo test; fledge keeps a full local verify lane and adds a fast trust-lifecycle lane; docs assign each confidence signal to one authority, describe the current protected-workflow boundary and Tier B multi-OS plan, and quantify expected Trust/product-tip wall-clock improvement; fledge lanes run trust-lifecycle and strict spec validation pass; this PR contains no protected workflow files or unrelated ship-status/#486 history.

## No-spec Rationale

Not applicable
