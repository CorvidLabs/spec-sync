---
change: CHG-0048-prepare-the-specsync-5-1-1-stabilization-release-from-merged-pr-387-bump-accur
artifact: design
---

# Design

The release is a monotonic promotion pipeline:

1. **Candidate** — archive integrated changes, bump current metadata to 5.1.1, update Action and
   documentation defaults, pin the hosted JavaScript runtime exactly, and verify locally without
   publishing.
2. **Review** — push a release PR and require exact-head hosted CI, Trust, CodeQL, security,
   documentation, Action-consumer, and review-thread clearance.
3. **Acceptance** — record explicit closing approval only after the exact candidate evidence is
   current; acceptance itself does not publish.
4. **Integration** — squash-merge the accepted candidate, then re-run strict and Trust verification
   plus the required hosted main gate on the integrated commit.
5. **Immutable publication** — create `v5.1.1` on that main commit and let the release workflow
   build and publish platform artifacts; verify every checksum and asset.
6. **Registry publication** — publish the already-tagged source as crates.io 5.1.1 and verify a
   clean exact-version installation.
7. **Compatibility promotion** — smoke-test `@v5.1.1`, then create or advance `v5` to the identical
   commit and smoke-test the floating ref.
8. **Downstream promotion** — update and test Homebrew, then repair the dependent Trust rollout.

The immutable `v5.1.1` tag is never force-moved. The floating `v5` ref is intentionally mutable but
is updated last, after exact-version verification. If any stage fails, later stages remain unchanged
and the failed stage is retried against the same content or superseded by a new patch version.
