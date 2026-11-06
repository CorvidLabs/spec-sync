# Lesson bundle: describe-the-cli-recursion-guard-as-refusing-an-inherited-verification-context-not-a-specsync-made-child

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Describe the CLI recursion guard as refusing an inherited verification context, not a SpecSync-made child
- **Kind**: Documentation
- **Specs**: cli
- **Paths**: specs/cli/requirements.md, specs/cli/cli.spec.md, specs/cli/cli.3md, docs/HLD.md, tests/integration/change.rs, tests/integration/helpers.rs, CHANGELOG.md, hi/ship.md, INTENT.md
- **Acceptance**: SHIP-1.c.1: REQ-cli-003 says the dispatcher refuses a check, change or lifecycle command that runs under an inherited SPECSYNC_VERIFICATION_CONTEXT and cites REQ-change-023 for SpecSync setting that marker on no child of its own; the cli spec Purpose, the cli.3md context plane and the docs/HLD.md failure-mode row no longer describe a SpecSync-made verification child.
- **Acceptance**: SHIP-1.c.2: cli_contracts_describe_an_inherited_verification_context_not_a_child fails on main and passes after the change.
- **Acceptance**: Runtime behavior is unchanged: no file under src/ changes, and indirect_recursive_lifecycle_check_fails_once_with_context and indirect_recursive_lifecycle_subcommands_fail_once_with_context still pass unmodified.

## Evidence

- Verification commit: `bf59d973be208c2cb54fdefcb1578df86596d88e`
- Base commit: `5ff95d10b0fb9d9bbb3d3fbd15f35d1d261f173c`
- Verified by: `specsync check --spec cli`

## From the change's context.md

# Context

Issue #911, split out of the #899 review (Codex P2). #899 (#848, audit D4) reconciles the change and
cmd_comment contracts with the runtime: `change check` and `comment` spawn no configured verification
command (REQ-change-023, REQ-change-091). Its review found the same promise in the cli module, which was
outside its approved specs:

- REQ-cli-003: "The root CLI dispatcher SHALL fail closed when a configured verification child re-enters
  lifecycle checking or mutation."
- The cli spec Purpose and the `cli.3md` context plane: "blocks configured verification children from
  recursively dispatching ...".
- docs/HLD.md, the failure-mode table: "Recursive calls (a verification child calling `check` or
  `change`) are refused".

The guard (`reject_recursive_lifecycle_dispatch` in src/main.rs) is live and correct. Only the wording
implies that SpecSync makes such a child.

Intent: SHIP-1.c.1 and SHIP-1.c.2, which Leif approved on 2026-11-05 at 20:24 MST (relayed by orc t18).
They are captured here with byte-identical text to #899, so the two branches merge cleanly. No runtime
change. The orc asked for this to go up to "ready for approval" only.

## From the change's design.md

# Design

| Clause | Was | Becomes |
|---|---|---|
| REQ-cli-003 body | a configured verification child re-enters the lifecycle | a check/change/lifecycle command that runs under an inherited `SPECSYNC_VERIFICATION_CONTEXT` |
| REQ-cli-003, new criterion | (none) | SpecSync sets the marker on no child of its own (REQ-change-023) |
| cli spec Purpose, `cli.3md` context | blocks configured verification children | refuses commands under an inherited marker; SpecSync starts no verification child |
| docs/HLD.md failure-mode row | a verification child calling check or change | a check or change that inherits the marker from a caller |

The requirement and Purpose change through the semantic delta. The 3md context plane, the HLD, the lesson
plane and the CHANGELOG are direct edits inside the declared paths.

## From the change's testing.md

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

## Where these lessons go

- `specs/cli/cli.3md`
