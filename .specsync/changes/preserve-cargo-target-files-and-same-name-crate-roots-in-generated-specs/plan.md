# Plan

## Approach

Keep exact Cargo target identities while the initial change owns generation and coverage corrections.

## Out of scope

New releases, unrelated parser ecosystems, and new product features.

## Risks

A directory can cover multiple targets; distinguish scan roots from target ownership and preserve configured/previous owners.

## Constraints consulted

Manifest, generator and validator .3md lessons require retained discovery, deterministic publication, and faithful source ownership.

## Steps

1. Reproduce both findings and add failing regressions.
2. Correct exact target retention, coverage ownership, and generation binding.
3. Keep module specs and companions synchronized.
4. Reverify both audited change packages, run full local validation, push and wait for product CI and Trust.
5. Record actual human review, finalize sequentially and merge with no active changes.
