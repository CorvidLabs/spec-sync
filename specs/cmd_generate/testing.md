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
| Malformed Gradle settings | No files generated; exits 1 with parseable `valid: false` / `inconclusive: true` JSON |

| Scoped batch rerun | `generate_scoped_npm_names_create_valid_specs_and_companions` repeats the named batch in text and JSON, verifies classification and all existing spec/companion bytes; `batch_rerun_retains_flat_module_classification` retains flat recognition and unknown-module classification. |
