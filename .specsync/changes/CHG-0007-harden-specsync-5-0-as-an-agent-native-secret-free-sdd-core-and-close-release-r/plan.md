---
change: CHG-0007-harden-specsync-5-0-as-an-agent-native-secret-free-sdd-core-and-close-release-r
artifact: plan
---

# Plan

1. Add issue #334 parity fixtures before changing Rust visibility behavior.
2. Remove embedded inference from dependencies, types, config, CLI, generation, and MCP while retaining agent-native integrations.
3. Upgrade Astro and bound/sanitize PR comment output.
4. Update README, canonical specs, companions, and repository documentation; record CorvidLabs/site as a separate follow-up.
5. Run focused regressions, full local gates, executable lifecycle examples, clean builds, independent security review, and the GitHub matrix.
