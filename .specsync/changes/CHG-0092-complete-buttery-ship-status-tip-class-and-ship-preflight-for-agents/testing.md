---
change: CHG-0092-complete-buttery-ship-status-tip-class-and-ship-preflight-for-agents
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-cli-args-011` | `change --help` lists ship-status and ship; help text mentions tip/trust/archive. |
| `REQ-cli-args-012` | `change finalize --help` mentions archive tip / PR merge order. |
| `REQ-cmd-change-007` | JSON ship-status includes tip_class and stages; sandbox drill 027. |
| `REQ-cmd-change-008` | ship without readiness exits non-zero; ship when ready finalizes (unit/manual + drill). |

## Sandbox

`SPECSYNC=/path/to/specsync bash drills/027-ship-sequence.sh` in CorvidLabs/spec-sync-sandbox.
