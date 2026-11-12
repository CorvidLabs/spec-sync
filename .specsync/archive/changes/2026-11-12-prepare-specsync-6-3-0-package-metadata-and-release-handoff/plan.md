---
change: prepare-specsync-6-3-0-package-metadata-and-release-handoff
artifact: plan
---

# Plan

## Approach

Bump every surface the release validator checks, write the 6.3.0 CHANGELOG section and `docs/releases/6.3.0.md`, and leave the published-default surfaces at 6.2.0.

## Out of scope

- Tags, the RC, promotion, crates.io and Homebrew; all wait for Leif's go.
- The post-publication Action default change.
- Any runtime or spec change.

## Steps

1. Cargo version and lock.
2. CI and Trust candidate inputs.
3. README and site Action examples.
4. RC test anchors.
5. CHANGELOG `[6.3.0]`.
6. `docs/releases/6.3.0.md`.
7. Once #1067 merges, merge main into this branch, so its entry and the upgrade note are in the section.

## Risks

A blanket version replace hits prose about the immutable `@v6.2.0` tag. I reviewed the diff; two such lines were put back to 6.2.0.

## Constraints consulted

none
