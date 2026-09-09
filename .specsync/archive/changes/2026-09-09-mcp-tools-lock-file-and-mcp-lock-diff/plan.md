---
change: mcp-tools-lock-file-and-mcp-lock-diff
artifact: plan
---

# Plan

1. Extract shared catalog + implement lock/diff module and CLI wiring.
2. Unit tests: match → 0; description/schema drift → non-zero; missing lock → non-zero.
3. Generate dogfood `.specsync/mcp-tools.lock.json` from live read-only catalog.
4. Update mcp + cli_args specs, CHANGELOG, and docs (cli / ai-agents / workflow).
5. Open PR (prefer merge-commit); archive-before-merge via SpecSync lifecycle.
