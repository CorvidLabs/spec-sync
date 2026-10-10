---
change: split-manifest-discovery-and-parser-responsibilities-into-focused-private-modules
artifact: context
---

# Context

The owner requested splitting large SpecSync files without behavior changes, then explicitly directed continued work and local branch commits. This independent branch starts at latest main 815690d. A fresh public GitHub API check found only PR 800 open, with no manifest overlap. MCP and validator checkpoints remain separate and are not prerequisites. Manifest currently has 5465 lines, about 3322 production/support and 2143 tests. Every cohesive planned module fits below 1000 lines without splitting function bodies.
