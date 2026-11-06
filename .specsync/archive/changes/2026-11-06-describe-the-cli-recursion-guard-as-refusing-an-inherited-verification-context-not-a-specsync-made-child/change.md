---
id: describe-the-cli-recursion-guard-as-refusing-an-inherited-verification-context-not-a-specsync-made-child
state: archived
type: documentation
base_commit: 5ff95d10b0fb9d9bbb3d3fbd15f35d1d261f173c
---

# Describe the CLI recursion guard as refusing an inherited verification context, not a SpecSync-made child

## Intent

Describe the CLI recursion guard as refusing an inherited verification context, not a SpecSync-made child

## Affected Canonical Specs

- `cli`

## Acceptance Criteria

- SHIP-1.c.1: REQ-cli-003 says the dispatcher refuses a check, change or lifecycle command that runs under an inherited SPECSYNC_VERIFICATION_CONTEXT and cites REQ-change-023 for SpecSync setting that marker on no child of its own; the cli spec Purpose, the cli.3md context plane and the docs/HLD.md failure-mode row no longer describe a SpecSync-made verification child.
- SHIP-1.c.2: cli_contracts_describe_an_inherited_verification_context_not_a_child fails on main and passes after the change.
- Runtime behavior is unchanged: no file under src/ changes, and indirect_recursive_lifecycle_check_fails_once_with_context and indirect_recursive_lifecycle_subcommands_fail_once_with_context still pass unmodified.

## No-spec Rationale

Not applicable
