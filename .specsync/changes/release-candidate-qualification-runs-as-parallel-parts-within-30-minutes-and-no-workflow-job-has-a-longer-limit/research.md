---
change: release-candidate-qualification-runs-as-parallel-parts-within-30-minutes-and-no-workflow-job-has-a-longer-limit
artifact: research
---

# Research

## Where the time is

| Lane | Measured | Source |
|---|---|---|
| macOS release-candidate qualification | 78m32s | `v6.4.2-rc.2`, run 45472594205 |
| Ubuntu release-candidate qualification | 22m31s | the same run |
| CI `coverage` | 22m24s | #1163's head, run 45532824335 |
| CI `test` | 10m27s | the same run |

The macOS lane step in rc.2 was 65m48s of `test-nextest`, 8m13s of `test-release-guards` and 3m59s of `build`. Job setup before the lane was under 30 seconds.

## Job limits above 30 minutes, or absent

`ci.yml`: `test` 45, `closed-late-compat` 40, `linux-baseline` 45, `attest` none. `rc-assets.yml`: `build` 45. `release.yml`: `qualify` 120, `build` 45. `trust.yml`: `trust` none. Every one of these jobs ran in under 11 minutes in the runs above, apart from the macOS `qualify`.

## cargo-nextest partitioning

The pinned version is 0.9.151. Its documentation gives two kinds, sliced and hashed, and calls counted partitioning deprecated.

- `--partition slice:m/n`, since 0.9.127: lists all tests across all binaries and assigns them round-robin.
- `--partition hash:m/n`: by a hash of binary and test name; stable when tests are added, less even.
- `cargo nextest list` takes the same `--partition` and marks each test outside the slice as a mismatch with the reason `partition`.

Run on main `778a111a` with the pinned binary: the whole list is 4,187 tests that run and 2 that are ignored. The four slices hold 1,047, 1,047, 1,047 and 1,046 tests, share none, and together are the whole list.

## Expected time per part

From the hosted macOS per-test times of `v6.4.2-rc.1` (4,176 of the 4,187 tests have one), at the three tests at once that runner ran:

| Slices | Test time per slice, minutes |
|---|---|
| 3 | 20.6 to 23.3 |
| 4 | 15.6 to 17.0 on the real slices |
| 5 | 11.6 to 15.9 |
| 6 | 9.9 to 12.0 |

A hashed split into four came to 11.9 to 19.6 minutes, so slices are the even choice. Each macOS job also compiles the test binaries, about 5 minutes, so a slice of four is about 22 to 23 minutes. The release-build part is about 12 minutes on macOS. On Ubuntu a slice is about 2 minutes of tests.

## Why four

GitHub runs at most five macOS jobs at once on the Team plan (its Actions limits reference). Four slices and one release-build part are five jobs, so all start together. More slices would queue behind them and lengthen the wall time.

## Ruled out

- Building the test binaries once and sharing them with `cargo nextest archive`: it removes about 5 minutes of compile from each slice and adds a build job in front of them, so the wall time is the same and the workflow is more complex.
- Raising the limit again: SPEED-2.b.4.
- Dropping, ignoring or shortening tests: SPEED-2.b.2.
