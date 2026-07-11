---
spec: cmd_generate.spec.md
---

## Regression Matrix

| Case | Required Result |
|------|-----------------|
| Default generate | Deterministic local spec paths |
| Batch selection | Requested, covered, and unknown modules reported |
| JSON output | No AI-specific fields |
| Legacy provider/model flags | Rejected by Clap |
| Inference environment variables | Do not affect output or execute commands |
