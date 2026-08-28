---
change: name-a-held-cargo-build-directory-lock-before-verification-blocks-on-it-and-reap-the-verification-child-s-process-group
artifact: testing
---

# Testing

Three tests in `tests/integration/change.rs` and seven in `src/change_tests.rs`. The three
integration tests were run against a binary built from a **separate checkout of unfixed `main`** at
`d6f266a4` — cloned, the new test file copied in, `src/change.rs` untouched — rather than by
reverting the fix in place.

| Test | Unfixed `main` | Fixed |
|------|----------------|-------|
| `a_held_cargo_build_lock_is_named_before_verification_blocks_on_it` | FAIL: `verification must name the lock it is blocked on:` with empty stderr | pass |
| `a_verification_child_runs_in_its_own_process_group` | FAIL: `assertion left == right failed: … recorded "33068 99024\n"`, `left: 99024`, `right: 33068` | pass |
| `an_unheld_cargo_build_lock_is_not_reported_during_verification` | **pass** | pass |

The seven unit tests in `src/change_tests.rs` cannot be run against the control at all: they call
`cargo_build_lock_path`, `cargo_build_lock_wait_notice`, and `proc_locks_flock_holders`, none of
which exist on `main`, so the control build fails with `E0425: cannot find function
cargo_build_lock_path in this scope` (and `E0422` for `CargoBuildEnvironment`) rather than with a
failing assertion. They are honest coverage of the derivation and of the notice text, but the
load-bearing discrimination is the two integration tests above.

## Why neither discriminator depends on a clock

The blocked state is **created**, not waited for. The test process holds an exclusive `flock` on
the lock file for the whole run; `flock` treats a second open file description in the same process
as a foreign holder, so the child sees genuine contention with no timing involved. The stand-in
`cargo` on the other side returns immediately, so the assertion is about what verification *says*,
not how long it takes.

The process-group test asserts a **structural** property — the child's PGID equals its own PID —
which the child records and then exits. Nothing polls, nothing has a deadline. #707 in this same
family is a wall-clock flake, and #702's commit message argues that a flaky gate teaches everyone
to ignore red; neither of these can become one.

`a_held_cargo_build_lock_is_named_before_verification_blocks_on_it` holds **both** profile locks and
requires the notice to name `target/debug/.cargo-lock` and not `target/release/.cargo-lock`, so the
build-directory derivation is proven end to end and not only in the unit tests.

## Honest labels

- CONTROL: `an_unheld_cargo_build_lock_is_not_reported_during_verification` passes on the unfixed
  binary too, and that is its purpose. It is what makes the notice's silence readable: a healthy
  four-to-five-minute compile with nothing held must print nothing, or the line would appear on
  every run and mean nothing. Also CONTROL:
  `a_non_cargo_verification_command_resolves_no_build_lock`, the first assertion of
  `a_cargo_config_that_moves_the_build_directory_resolves_no_lock` (a Cargo config setting no
  layout key must NOT silence the notice), and the second half of
  `proc_locks_names_only_the_flock_write_holder_of_the_lock_file`.
- DISCRIMINATOR: the two integration tests above, plus
  `a_cargo_verification_command_resolves_the_lock_it_will_contend_on`,
  `an_underivable_cargo_build_layout_resolves_no_lock`,
  `cargo_target_dir_environment_moves_the_resolved_lock`,
  `a_held_cargo_build_lock_produces_a_notice_naming_the_lock`,
  `a_cargo_config_that_moves_the_build_directory_resolves_no_lock`, and
  `proc_locks_names_only_the_flock_write_holder_of_the_lock_file`.

## One assertion removed after it flaked, rather than retried

`a_held_cargo_build_lock_produces_a_notice_naming_the_lock` originally ended by releasing the lock
and requiring the notice to stop. That assertion failed on the first `change check`, on a host
running seven concurrent agent worktrees. A `flock` lives on the open file description, and this
test binary is multithreaded and spawns processes constantly, so a descriptor duplicated into a
concurrent `fork` keeps the lock alive until that child `exec`s — "released" is not observable at a
deterministic instant from inside the process. It is deleted, not given a longer window. What
remains is deterministic in both directions, and the held case cannot pass vacuously because the
test takes the lock with a **blocking** acquisition: a leaked descriptor would hang it, not satisfy
it.

## Stated as untested rather than covered

- **That a `SIGINT`/`SIGTERM` delivered to a live parent asynchronously ends the child's group.**
  Asserting it means racing a real signal against a real process death: wait for the child to
  start, signal the parent, then wait for the child to die. Every step is a wall-clock deadline on
  a machine this change is specifically about being loaded. The structural precondition (own
  process group) is asserted instead; the delivery is stated, not claimed.
- **The Linux `/proc/locks` wiring.** The parser is tested on every platform against fixture text;
  reading `/proc/locks` and stat-ing the lock file for `major:minor:inode` runs only on Linux and
  is not exercised by any test on this host.
- **Windows.** `#[cfg(not(unix))]` keeps `command.status()`; compilation is the only claim made.

## What the independent review found, and what it cost

The scoped review returned BLOCK. Three findings were real, and one of them was in a canonical
spec: the change CLAIMED that a `.cargo/config.toml` `build.target-dir` produced silence while the
derivation never looked, so a project that moved its build directory with a stale `target/` left
behind would have had the notice name a lock nothing waits on. That is now implemented and covered
by `a_cargo_config_that_moves_the_build_directory_resolves_no_lock`, which carries its own CONTROL:
a config setting no layout key must NOT silence the notice, or the feature turns itself off for
most real projects.

The review also established that the issue's own premise is false — Cargo prints `Blocking waiting
for file lock on artifact directory` — which was measured afterwards and corrected everywhere it
had been repeated, rather than left standing because it was in the issue.

## The re-review's two blockers, and what is still not covered

The re-review returned BLOCK a second time. Both blockers were real:

1. The corrected `REQ-change-091` never reached `specs/change/requirements.md`, even though
   `change check` reported success. `materialize_change_deltas` returns early on
   `canonical_applied`, so a delta corrected and re-approved after the first materialisation is
   never written. The corrected block was applied to the canonical file by hand, byte-for-byte as
   `apply_markdown_block` would have, and the mechanism is recorded in `specs/change/context.md`.
   Nothing in the tool compares "what the approval now says" against "what the tree already got",
   so no test here can cover that; it is named as an observation for the module, not fixed inside
   this change's frozen scope.
2. The replacement premise was itself unchecked: the claim that Cargo's `Blocking …` line "is
   discarded entirely by `check_project_quiet`" was read from a stale spec paragraph, and that
   function was deleted by #543. Corrected in five places; the notice is ADDITIVE to Cargo's line.

Stated plainly rather than fixed, in the hermeticity class:

- `cargo_config_moves_build_directory` walks `root.ancestors()`, so on a host with a real
  `/.cargo/config.toml` (or one at any ancestor of the temp directory) the unit tests would see it.
  No test controls that, and bounding the walk would break the behaviour being asserted.
- The `CARGO_HOME` leg IS now covered — the earlier version of the config test pointed it at a
  directory that never existed, so that whole branch was dead in test. It now asserts both that an
  empty `CARGO_HOME` does not silence the notice and that a config there does.

## Suites run

- `cargo fmt --check`
- `cargo clippy -- -D warnings` (bare — the form CI runs)
- `cargo test` — full unit and integration suites
- `specsync change check` and `specsync change audit --strict`

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-091 | Three integration tests in `tests/integration/change.rs` and seven unit tests in `src/change_tests.rs`, the integration three run against a binary built from a separate checkout of unfixed `main` at `d6f266a4`. `a_held_cargo_build_lock_is_named_before_verification_blocks_on_it` — unfixed: `verification must name the lock it is blocked on:` with empty stderr; it holds BOTH profile locks and requires the notice to name `target/debug/.cargo-lock` and not `target/release/.cargo-lock`. `a_verification_child_runs_in_its_own_process_group` — unfixed: `assertion left == right failed … recorded "33068 99024\n"`, because the child inherits the parent's group. CONTROL `an_unheld_cargo_build_lock_is_not_reported_during_verification` passes in BOTH states, which is its purpose: a healthy long compile with nothing held must print nothing. Unit coverage: `a_cargo_verification_command_resolves_the_lock_it_will_contend_on` (eleven argv shapes), `an_underivable_cargo_build_layout_resolves_no_lock` (two triples, custom target JSON, traversal-shaped profile, `--config`, `--manifest-path`, `cargo nextest` whose `--profile` means a nextest profile), `a_cargo_config_that_moves_the_build_directory_resolves_no_lock` (a merged Cargo config setting `build.target-dir`, `build.target` or `build.build-dir`, an `[env]` table setting a layout variable, an unparsable one, an ancestor's, one in `CARGO_HOME`, plus CONTROLs proving neither a config setting no layout key nor an empty `CARGO_HOME` silences the notice), `cargo_target_dir_environment_moves_the_resolved_lock`, `a_held_cargo_build_lock_produces_a_notice_naming_the_lock` (a real `flock` held by a second file description, with the unheld CONTROL asserted first), `proc_locks_names_only_the_flock_write_holder_of_the_lock_file`, and CONTROL `a_non_cargo_verification_command_resolves_no_build_lock`. A third assertion in that unit test — release the lock, require the notice to stop — flaked on the first `change check` under a seven-worktree host and was deleted rather than widened, because a `flock` duplicated into a concurrent `fork` outlives this thread's `drop` until the child execs. Those seven cannot run against the control at all — the functions do not exist on `main`, so it fails with `E0425: cannot find function cargo_build_lock_path in this scope`. Neither discriminator depends on a clock: the lock is held by the test process and the process group is recorded by the child before it exits. NOT tested and stated as such: asynchronous group teardown on a signal delivered to a live parent, the Linux `/proc/locks` wiring, and Windows beyond compilation. `cargo fmt --check`, bare `cargo clippy -- -D warnings`, and the full `cargo test` suite pass |
