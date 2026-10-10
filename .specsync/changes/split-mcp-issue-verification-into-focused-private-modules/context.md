---
change: split-mcp-issue-verification-into-focused-private-modules
artifact: context
---

# Context

The user asked to break up SpecSync files above 1000, 5000 and 10000 lines where sensible. At base 684aed366163c29ce3b5df22b3c4716353d93fcf, src/mcp.rs has 9227 lines. The two largest files belong to active PRs #804 and #800, so this first bounded refactor avoids those overlapping lifecycle changes.

This change extracts only MCP issue verification and its 17 existing unit regressions. It preserves public behavior and all security decisions. It is the first cohesive split, not a claim that the entire file-size backlog is resolved. No push, PR, merge or deployment is authorized by this package. Human scope approval and later human implementation review remain distinct.
