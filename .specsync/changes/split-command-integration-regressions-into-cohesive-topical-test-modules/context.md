---
change: split-command-integration-regressions-into-cohesive-topical-test-modules
artifact: context
---

# Context

The owner requested splitting large SpecSync files without behavior changes and explicitly continued work with local branch commits. The next safe file is tests/integration/commands.rs, 5496 lines. Latest main remains 815690d; the only open PR is 800, which has no overlap with this test path or CLI spec. This independent branch does not depend on the MCP, validator or manifest checkpoints. Pristine-main full integration evidence contains 120 passing command tests. Its whole integration suite has 425 passes and 13 known lifecycle/comment failures; production is unchanged by this test-only split.
