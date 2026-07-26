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
| Empty source list | Emits `files: []`, not YAML null |
| Detected exports | Built-in and custom templates contain Public API rows |
| Replacement-sensitive export | `$value` remains byte-for-byte in the generated table |
| Configured module directory | Expands to sorted, deduplicated source files |
