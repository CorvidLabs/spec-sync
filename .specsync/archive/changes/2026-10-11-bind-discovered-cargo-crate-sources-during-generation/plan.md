---
change: bind-discovered-cargo-crate-sources-during-generation
artifact: plan
---

# Plan

## Approach

Preserve Cargo mapping metadata in the coverage snapshot and share source binding across generation callers.

## Out of scope

Release publication, manifest parser feature expansion, and unrelated generation defects.

## Steps

1. Add the exact init/generate regression and observe failure before implementation.
2. Preserve discovered mappings and bind uncovered source files with existing configuration precedence.
3. Cover binaries, workspace isolation, named submodules, existing ownership, and missing-module failure.
4. Synchronize specs and companions, run focused tests, full verify, provenance, and Trust.
5. Push a draft PR for actual human implementation review before finalization.

## Risks

A broad crate root can overlap named modules. Preserve their more specific bindings and never claim files already referenced by a spec. Manifest failures still follow configured-source degradation rules.

## Constraints consulted

Generator and validator .3md lessons require retained capabilities, deterministic scaffolds, bounded coverage, and no-overwrite publication.
