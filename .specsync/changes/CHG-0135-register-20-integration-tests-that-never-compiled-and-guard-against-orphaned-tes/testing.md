---
change: CHG-0135-register-20-integration-tests-that-never-compiled-and-guard-against-orphaned-tes
artifact: testing
---

# Testing

## Counting, not reading

Test totals are verified by counting `#[test]` markers before and after, never by trusting
diff output. This codebase contains `{` inside string literals and a brace-matching edit has
previously eaten unrelated functions.

| | before | after |
|---|---|---|
| `regression_w1.rs` `#[test]` | 20 | 20 |
| `regression_w1.rs` `#[tokio::test]` | 0 | 0 |
| `tests/integration.rs` `#[test]` | 0 | 1 |
| `tests/integration/` files on disk | 13 | 13 |
| `#[path]` registrations | 12 | 13 |

Assertions were rewritten *inside* two existing tests. None added, removed, renamed, or
`#[ignore]`d.

One counting hazard recorded so nobody later "fixes" a phantom discrepancy: after the guard
lands, `grep -c '#\[path' tests/integration.rs` returns 17, not 13, because the guard's doc
comment and assert message contain the literal text `#[path`. The true count needs the line
anchor `grep -c '^#\[path = "integration/'`.

## Suite

    cargo test                    rc=0    2276 unit passed, 395 integration passed, 0 failed
    cargo clippy -- -D warnings   rc=0

395 = 374 baseline + 20 resurrected + 1 guard.

`cargo clippy --all-targets` is red, but it is red on unmodified `main` too, with a
byte-identical finding set — pre-existing debt, filed as #608, not introduced here.

## The guard must be able to fail

A guard that cannot fail is the defect this change exists to close, wearing a nicer costume.
Verified after rebase, against the exact code that merges:

    stray file planted → rc=101
      1 test file(s) in tests/integration/ are not registered in tests/integration.rs
      and therefore never compile and never run: ["zz_orphan_probe.rs"].
    probe removed      → rc=0

It asserts set equality in both directions, so it also catches a `#[path]` pointing at a
deleted file. It lives inline in `tests/integration.rs` rather than in its own module,
because a guard placed inside `tests/integration/` could be orphaned and would then be the
very thing it exists to detect.

## The #607 fixture must discriminate, not merely pass

Acceptance was never "both tests green". Same tree, three flag values:

| fixture | `--require-coverage 0` | `100` | `101` |
|---|---|---|---|
| old (bare `TempDir`, no git) | rc=1 | rc=1 | rc=1 |
| new (`git init` + initial commit) | rc=0 | rc=0 | rc=1, gate message printed |

The old row is constant across three different flag values — zero information, which is
exactly why `report_require_coverage_above_actual_exits_1` passed while proving nothing. The
new row varies with the flag. The vacuity is removed rather than relocated.

## The #606 pin must fire

The pin asserts today's wrong behaviour on purpose and states its own inversion condition.
Verified by temporarily neutering the duplicate emission at `src/deps.rs:261`:

    rc=101
    assertion `left == right` failed: PIN(#606): one missing dependency is currently
    reported twice by two separate code paths. If this now reads 1, #606 is fixed —
    update this assertion to 1 and remove the pin.

The `Edges: 1` assertions still passed under that probe, so the pin is isolated to #606 and
does not double as a dedupe check. Probe reverted; `git diff origin/main..HEAD` confirms
`src/` is untouched.

## Scope control

`git diff --name-only origin/main..HEAD -- src/` returns 0 files, and the same for
`.specsync/` before this change record. The diff is `tests/integration.rs`,
`tests/integration/regression_w1.rs`, and `CHANGELOG.md`.
