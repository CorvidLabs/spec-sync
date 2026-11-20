---
change: prepare-specsync-6-4-1-package-metadata-and-release-handoff
artifact: plan
---

# Plan

## Approach

Bump every surface the release validator checks to 6.4.1, move `[Unreleased]` into a dated 6.4.1 section, write `docs/releases/6.4.1.md`, and leave the published-default surfaces at 6.4.0.

## Out of scope

- Tags, the RC, promotion, crates.io and Homebrew; each waits for Leif's go.
- The post-publication Action default change.
- Any runtime or spec change, and the 6.4.1 fix itself.

## Steps

1. Cargo version and lock.
2. CI and Trust candidate inputs.
3. README and site Action examples, and the two sentences that say they pin a candidate.
4. RC test anchors.
5. CHANGELOG `[6.4.1]`.
6. `docs/releases/6.4.1.md`.

## Risks

A blanket version replace hits prose about the immutable `@v6.4.0` tag and the published defaults. Only YAML example lines and the named sentences move, and the validator's list is checked against every changed line.

## Constraints consulted

6.4.0's prep (#1109).
