---
spec: ai.spec.md
---

## Security Regression Matrix

| Case | Required Result |
|------|-----------------|
| Dependency graph | No `corvid-ai` package |
| Legacy CLI flags | Rejected by Clap |
| Legacy config keys | Ignored by name with value-safe guidance |
| `SPECSYNC_AI_COMMAND` | Never executed by `generate` or `check --fix` |
| Legacy MCP arguments | Explicit tool error without value disclosure |
| Agent integrations | Claude, Cursor, Codex, and Gemini installers remain functional |
