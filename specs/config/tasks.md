---
spec: config.spec.md
---

## Tasks

(none open for 5.0)

## Done

- [x] Preserve current and legacy layout loading and deterministic round trips
- [x] Remove embedded inference fields
- [x] Ignore legacy AI key names with value-safe migration guidance
- [x] Auto-detect zero-config HTML, HTM, and CSS source directories
- [x] Add fallible checked source-directory and manifest discovery while retaining compatibility wrappers
- [x] Fail closed when legacy JSON supplies `github.repo` with a non-string shape, without
  falling back to Git auto-detection or discarding otherwise valid configuration
- [x] Parse retained exact-byte JSON/TOML snapshots with known TOML field-type validation for
  capability-rooted callers
- [x] Accept capability-derived source-directory discovery when retained config omits its source
  list, without scanning an ambient root pathname
- [x] Reject wrong-shaped `github` and `github.repo` values in exact-byte checked JSON parsing
- [x] Share config precedence and lexical source classification with retained CLI discovery
- [x] Preserve explicit-source validation and malformed legacy config warning compatibility
