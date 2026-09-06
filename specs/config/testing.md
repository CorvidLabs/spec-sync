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
| Malformed Gradle settings through checked discovery | Return an error naming the manifest; do not expose partial manifest results |
| Malformed Gradle settings through compatibility discovery | Preserve infallible signatures and use the documented fallback behavior |
| Legacy JSON `github.repo` is a number, boolean, object, or list | Preserve otherwise valid configuration but make repository resolution fail closed without Git auto-detection |
| Exact-byte checked JSON has a non-object `github` or wrong-typed `github.repo` | Return `Err`; retained callers cannot expose sentinel/default success |
| Retained JSON/TOML snapshot | Parse exact caller bytes, preserve BOM and omitted-source compatibility, and reject malformed syntax or wrong-shaped known TOML fields |
| Retained CLI config with an explicit linked source root | Do not pre-scan the root; normal validation reports the unsafe mapping |
| Retained CLI config with invalid UTF-8 | Emit the established fail-loud warning and use safe defaults; MCP selected-config reads still reject it |
| Serialization | No retired inference keys |
| `[modules."<name>"] owns` | Round-trips through `config_to_toml` and `load_config`, alone or beside `files`; a non-array value is rejected by checked TOML validation; coverage ignores it |
