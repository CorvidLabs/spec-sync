---
spec: generator.spec.md
---

## Regression Matrix

| Case | Required Result |
|------|-----------------|
| Uncovered module | Spec and companions created |
| Existing spec | No overwrite |
| Custom template | Used with per-file fallback, including retained-root generation for configured modules |
| Inference env vars set | Output remains deterministic; no command/network execution |
| Generation outcome | Count and relative paths only |
| Root retarget after coverage | Default and batch JSON generation fail inconclusively; original and replacement bytes remain unchanged |
| Retained destination path | Absolute/rooted/prefix/parent components reject before a write |
