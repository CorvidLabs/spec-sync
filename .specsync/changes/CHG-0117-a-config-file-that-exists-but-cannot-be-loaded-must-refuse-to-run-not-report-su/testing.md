---
change: CHG-0117-a-config-file-that-exists-but-cannot-be-loaded-must-refuse-to-run-not-report-su
artifact: testing
---

# Testing

## Strategy

This change makes a command refuse to run, so the risk is over-reach: a project with no
config file must keep working, or `check` becomes unusable as the first command a new
project runs — the defect CHG-0107 fixed. Two controls bracket it, and the second is the
load-bearing one.

## Verified by hand

| fixture | before | after |
|---|---|---|
| config exists, one `]` deleted | exit 0, `✓ All required sections present` | **exit 1**, refuses and names the file |
| **control** — valid config demanding a Threat Model section | exit 1, reports the missing section | unchanged |
| **control** — no config file at all | exit 0 | **exit 0**, unchanged, built-in defaults |

The third row confines the change. The refusal fires only when a file **exists and could not
be loaded** — never on its absence, which is a legitimate configuration.

## Where the guard lives, and why

At `load_and_discover`, the single entry point every spec-reading command passes through.

The alternative considered and rejected was threading the condition into `compute_exit_code`
and `exit_with_status` — **36 call sites across 8 files**, and a command that forgot to pass
it would silently keep the old behaviour. One choke point cannot be forgotten.

## Found while implementing

The four fallback sites are not one shape. Two handle an unreadable file, and the parse
failure — the actual #570 trigger — is a **separate** site in `validator.rs` reached during
retained discovery. A fix applied only to the sites matching the first pattern would have
left the reported defect intact.

## Regression surface

2210 unit and 331 integration tests pass unchanged, including this repository's own config,
which loads cleanly and therefore takes the unchanged path.

## Not covered

No unit test asserts the refusal. Behavioural pinning belongs in the sandbox: a drill that
breaks a config file and asserts every command refuses would cover all of them at once, and
no drill currently touches configuration at all.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-types-007 | `cargo test`; the condition is carried on the configuration and set at every fallback, so it survives to the point of use rather than being lost at the boundary where it was detected |
| REQ-config-009 | Both unreadable-file sites record it; the absent-file path does not, confirmed by the no-config control still exiting 0 |
| REQ-validator-013 | The parse-failure site records it, which is what makes the hand-verified #570 repro exit 1 — the other sites alone would not have |
| REQ-commands-009 | The refusal is applied once at `load_and_discover`; the broken-config fixture exits 1 naming the file, while the valid-config and no-config controls are byte-for-byte unchanged |
