---
spec: config.spec.md
---

## Regression Matrix

| Case | Required Result |
|------|-----------------|
| JSON/TOML deterministic fields | Load and round-trip |
| Legacy AI keys | Ignored; warning names keys but never values |
| Unreadable config | Fail-loud warning then safe defaults |
| Missing config | Auto-detect source directories |
| Static-only root or nested project | Detect `.` or the containing top-level directory from HTML, HTM, or CSS |
| Serialization | No retired inference keys |
| Detection confidence | Empty projects return `(["src"], false)`; real manifest/scan sources return `true` |
| Mutation preflight | `validate_config_file_rejects_syntax_and_required_field_shapes` |
| Control-safe TOML | `config_to_toml_escapes_every_control_character_as_valid_toml` |
