## ADDED

### REQUIREMENT REQ-cli-args-013

The change ship CLI SHALL expose optional multi-tip orchestration flags without
changing the default finalize-only behavior.

Acceptance Criteria

- `change ship` accepts `--push`, `--wait`, and `--wait-timeout-secs` (default 900).
- `--dry-run` remains available and is mutually exclusive with `--push` and `--wait`.
- Help text names the flags; default ship without flags still only finalizes when ready.
