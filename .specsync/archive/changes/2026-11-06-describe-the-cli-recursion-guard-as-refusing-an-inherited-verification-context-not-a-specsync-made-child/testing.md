---
change: describe-the-cli-recursion-guard-as-refusing-an-inherited-verification-context-not-a-specsync-made-child
artifact: testing
---

# Testing

The guard's behavior does not change. `indirect_recursive_lifecycle_check_fails_once_with_context` and
`indirect_recursive_lifecycle_subcommands_fail_once_with_context` pin it and must still pass unmodified.

Contract pin (SHIP-1.c.2): `cli_contracts_describe_an_inherited_verification_context_not_a_child`.
REQ-cli-003 must name the inherited `SPECSYNC_VERIFICATION_CONTEXT` and cite REQ-change-023. Three retired
phrases must be absent from the six cli contract files and docs/HLD.md. It failed on main cd8cab5e with
the contracts unchanged (2 passed, 1 failed). It reads the tracked contracts, not the binary.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-cli-003 | `indirect_recursive_lifecycle_check_fails_once_with_context`, `indirect_recursive_lifecycle_subcommands_fail_once_with_context` (inherited marker, one diagnostic naming the parent, exit 1, no validation output), `cli_contracts_describe_an_inherited_verification_context_not_a_child` (contract pin) |

## Acceptance criteria evidence

| Criterion | How it is proven |
| --- | --- |
| SHIP-1.c.1 | `cli_contracts_describe_an_inherited_verification_context_not_a_child` |
| SHIP-1.c.2 | the pin failed on main and passes after (recorded) |
| Runtime unchanged | `git diff origin/main --stat -- src` is empty; the guard tests pass unmodified |

## Verification commands

- `fledge lanes run verify` on the pinned rust 1.89 (rustup proxy first on PATH)
- `specsync change check <id> --commit`
- `fledge lanes run pre-push`
