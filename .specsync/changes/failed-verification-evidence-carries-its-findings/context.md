---
change: failed-verification-evidence-carries-its-findings
artifact: context
---

# Context






GitHub issue #640: a failed verification.json command recorded only command/success/exit_code,
so a flake and a regression were indistinguishable without a 10-minute re-run. The current
verification flow records the scoped spec↔code check instead of external commands, but the same
gap exists: `passed: false` with no findings text. The findings are computed but never persisted.
