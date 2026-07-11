---
spec: mcp.spec.md
---

## Regression Matrix

| Case | Required Result |
|------|-----------------|
| Initialize/tools/resources | Valid JSON-RPC metadata |
| Generate | Deterministic local scaffold |
| Legacy inference argument | Tool error, value not echoed |
| Unknown generate argument | Tool error names only the unsupported key |
| Notification | No response |
| EOF | Graceful exit |
