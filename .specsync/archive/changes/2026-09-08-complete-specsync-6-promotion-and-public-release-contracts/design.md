---
change: complete-specsync-6-promotion-and-public-release-contracts
artifact: design
---

# Design

Keep the implementation narrow. Correct both shell evidence-count guards to match the existing required platform set. Add executable tests extracting each actual guard and running it with zero/one/two/three receipt names, using the validator's REQUIRED_PLATFORMS length as the acceptance oracle. Existing validator tests retain rejection of duplicate platforms, failed outcomes, mixed identities, missing fields, and extra records. Restore the obsolete count separately in each guard as a negative control and require failure of its regression.

Only help wording changes in src/cli.rs; command parsing, reviewer validation, persisted schema, state transitions, timeout behavior, and provenance policy are unchanged. Update canonical descriptions and the matching requirement to state the actual authentication boundary explicitly. The stored provider declaration remains format-validated metadata, not a claimed identity check.

Release version remains6.0.0. Immutable tags, release token permissions, branch protections, supported-platform set, and required workflows do not change. Actual stable publication follows qualification of a fresh candidate at the final merged tree.
