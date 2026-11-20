---
change: prepare-specsync-6-4-0-package-metadata-and-release-handoff
artifact: plan
---

# Plan

## Approach

Bump every surface the release validator checks, write the 6.4.0 CHANGELOG section and `docs/releases/6.4.0.md`, and leave the published-default surfaces at 6.3.0.

## Out of scope

- Tags, the RC, promotion, crates.io and Homebrew; each waits for Leif's go.
- The post-publication Action default change.
- Any runtime or spec change.

## Steps

1. Cargo version and lock.
2. CI and Trust candidate inputs.
3. README and site Action examples.
4. RC test anchors.
5. Run released 6.3.0 and this version on one fixture, to state what 6.3.0 does where this version has written history fragments.
6. CHANGELOG `[6.4.0]`.
7. `docs/releases/6.4.0.md`.

## Risks

A blanket version replace hits prose about the immutable `@v6.3.0` tag and the published defaults. Every replaced line is reviewed against the validator's list, and any such line is put back to 6.3.0.

## Constraints consulted

none
