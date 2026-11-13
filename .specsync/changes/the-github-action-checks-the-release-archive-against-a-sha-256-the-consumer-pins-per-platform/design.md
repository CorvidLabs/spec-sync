---
change: the-github-action-checks-the-release-archive-against-a-sha-256-the-consumer-pins-per-platform
artifact: design
---

# Design

- New input `expected-sha256` (default empty) passed as `SPECSYNC_EXPECTED_SHA256`.
- After the sidecar check, `action.yml` runs `.github/scripts/verify-release-digest.sh <archive>
  <os>-<arch> "$SPECSYNC_EXPECTED_SHA256"` from `github.action_path`.
- The script exits 0 for an empty input. It parses trimmed non-empty lines as `<platform>=<64 hex>`.
  It refuses a malformed line, no line or two lines for this platform, and a different
  `shasum -a 256`, each with an `::error::` that names the cause. Comparison ignores hex case.
