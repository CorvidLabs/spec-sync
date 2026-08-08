---
change: CHG-0100-ship-push-wait-archive-tip-orchestration-for-buttery-multi-tip-ship
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-cmd-change-008 | `cargo test change_check_review_and_finalize_are_plain_commands`; manual `change ship --help` shows push/wait flags; dry-run rejects push/wait |
| REQ-cli-args-013 | `cargo test change_check_review_and_finalize_are_plain_commands` covers `--push --wait --wait-timeout-secs` parse |

## Notes

Offline wait path: `SPECSYNC_SHIP_LOCAL_GUIDANCE=1` or no GITHUB_TOKEN reports local_guidance.
