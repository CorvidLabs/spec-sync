---
spec: generator.spec.md
---

## Regression Matrix

| Case | Required Result |
|------|-----------------|
| Uncovered module | Spec and companions created |
| Existing spec | No overwrite |
| Custom template | Used with per-file fallback |
| Inference env vars set | Output remains deterministic; no command/network execution |
| Generation outcome | Count and relative paths only |
