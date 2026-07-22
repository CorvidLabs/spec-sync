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
| Serialization | No retired inference keys |
