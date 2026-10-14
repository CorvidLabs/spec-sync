---
change: fix-custom-cargo-library-discovery-and-scoped-npm-generation-for-specsync-6-1-1
artifact: plan
---

# Plan

## Approach

Extend structural Cargo discovery with the declared library target and its scan directory. Build nested-name spec filenames from the module leaf while preserving the full identity and confinement. Expand companion references consistently. Prepare package, Action, CI consumer, Trust and documented pins at 6.1.1; published-release claims continue to identify 6.1.0 until promotion.

## Steps

1. Prove both findings with regressions.
2. Apply discovery and generation fixes; synchronize contracts.
3. Verify local lanes, trust and provenance; prepare a reviewable PR.
4. Obtain human implementation review and archive before merge.
5. Qualify one immutable 6.1.1 candidate via release.yml on Ubuntu and macOS.

## Out of scope

Unrelated PRs and Cargo/Homebrew publication before qualification.

## Risks

Exact target ownership must preserve configured/existing owners. Path-bearing names must remain confined; companions must reference the actual spec filename.

## Constraints consulted

REQ-manifest-020, REQ-generator-002/005/006 and immutable release gates.

## Accepted root-target correction

Leif accepted the prepared correction on October 14. Include the validator source/companions, preserve root lib.rs/main.rs as checked exact-file selections, and preserve literal workspace member ownership for member- and root-declared targets. Existing explicit directory authority, retained-root confinement and input budgets remain unchanged. No glob parser redesign, table-placement work or Git LFS change is included.
