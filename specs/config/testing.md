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
| Retained JSON/TOML snapshot | Parse exact caller bytes, preserve BOM and omitted-source compatibility, and reject malformed syntax or wrong-shaped known TOML fields |
| Serialization | No retired inference keys |
