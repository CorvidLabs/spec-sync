---
change: CHG-0139-declaring-a-module-must-never-reduce-the-verification-a-change-receives
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-069 | `declaring_an_unrouted_module_never_reduces_verification` asserts a superset relation — the command set for `{routed, unrouted}` contains every command selected for `{unrouted}` and for `{routed}` alone — so any future regression of this shape is caught, not only this instance. It FAILS on an unfixed binary built from a separate checkout of `998df28e` with `src/change.rs` provably unmodified (`declaring both modules dropped 'project-wide'; got ["component-routed"]`) and passes on the fixed one. `a_fully_routed_change_still_runs_only_its_component_commands` is the vacuity control and passes on BOTH binaries, proving targeted verification survived; without it, "always append the project list" passes the superset assertion while deleting the feature. The no-module-declared case is covered by the unchanged `verification_routing_fails_closed_without_any_validator` and by CHG-0138's own live run, which received all four project commands with `affected_specs = []` |

## Suite

    cargo test                    rc=0    2286 unit passed, 400 integration passed, 0 failed
    cargo clippy -- -D warnings   rc=0
    cargo fmt --check             rc=0

`#[test]` markers in `src/change_tests.rs`: 311 before, **313** after — exactly the two added,
counted rather than read off a diff.

`cargo clippy --all-targets` remains rc=101, unchanged from `main`; pre-existing debt filed as
#608.

## Discrimination — a separate checkout, not a revert

The unfixed binary was built from a fresh worktree at `998df28e` with **only the new test block**
injected. Proven, not asserted:

    change.rs modified lines:            0
    'unrouted_modules' in change.rs:     0     (fixed tree: 3)
    UNFIXED TEST BUILD rc=0

The build's exit code is checked explicitly. Earlier in this release a reverted-file build failed
to compile, leaving drills running against the still-fixed binary and passing — a false proof
caught only by checking that code.

    UNFIXED  998df28e
      declaring_an_unrouted_module_never_reduces_verification ... FAILED
        declaring both modules dropped `project-wide`; got ["component-routed"]
      a_fully_routed_change_still_runs_only_its_component_commands ... ok

    FIXED
      both ... ok

The asymmetry is the point. Both failing would mean the fix changed more than intended; both
passing would mean the first test asserts nothing.

## Live evidence that motivated this

Two real changes on this repository, same binary, same config, consecutive ledger entries:

    CHG-0137  --spec validator --spec manifest   ->  1 command,  63 tests, 0 integration tests
    CHG-0138  no --spec at all                   ->  4 commands, full suite

`ruby --version` ran for the first time in this repository's history during CHG-0138, because it
is reachable only through the fallback the routing was suppressing.

## Whole-board

No drill covers verification-command selection, so the board must be **unchanged** at `45/10`. A
board that moves would mean this change altered behaviour it should not have.
