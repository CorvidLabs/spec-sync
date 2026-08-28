# Lesson bundle — name-a-held-cargo-build-directory-lock-before-verification-blocks-on-it-and-reap-the-verification-child-s-process-group

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Name a held Cargo build-directory lock before verification blocks on it, and reap the verification child's process group
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change.rs, src/change_tests.rs, tests/integration/change.rs, specs/change/change.spec.md, specs/change/requirements.md, specs/change/context.md, specs/change/tasks.md, specs/change/testing.md
- **Acceptance**: A verification command that is about to wait on a held Cargo build-directory lock prints one line naming that lock and saying the run is blocked rather than compiling, derived from a non-blocking exclusive acquisition and never from elapsed time
- **Acceptance**: A Cargo build directory that cannot be derived exactly from the argv and process environment produces no notice at all, and a command that takes no build lock is never probed
- **Acceptance**: On Unix every verification child leads its own process group, so an interrupted parent can end the whole group instead of orphaning cargo on the lock

## Evidence

- Verification commit: `581fa8061fdac0728c5bb762b8af1db534f6fb1d`
- Base commit: `d6f266a4fd683246469eb15a8f632061dd5cfbb4`
- Verified by: `cargo test change::`, `cargo test`, `python3 -S .github/scripts/validate-release-version.py`, `python3 -S .github/scripts/validate-workflow-runtime-pins.py`

## From the change's context.md

# Context

Issue #721. An interrupted `specsync change check` leaves an orphaned `cargo test` alive holding
`target/debug/.cargo-lock`. The next `change check` blocks on that lock **at 0.0% CPU**. Observed
concretely in this repository: an interrupted run left `cargo test` (PID 75733) and its parent
`change check` (PID 2318) alive; the re-run appeared to compile for minutes and was waiting.
Killing both made the next check start immediately.

The cost is not the wait. `change check` here legitimately takes four to five minutes — a release
build plus 400+ tests — so **"nothing is happening yet" is exactly what a healthy run looks like**.
That produced at least two wrong diagnoses in this repository (a verification failure attributed to
host load; several checks re-run "because they seemed slow"), each costing a full verification
cycle, and neither distinguishable after the fact.

**One premise in the issue is wrong, and this change repeated it before anyone measured — then
replaced it with a second unchecked claim.** #721 says "There is no output distinguishing
*compiling* from *blocked on a lock*." Cargo 1.89, run against a held lock, prints `Blocking
waiting for file lock on artifact directory`. That was measured only after the independent review
asked.

The first correction then asserted that the line "is discarded entirely by `check_project_quiet`",
which would have made the gap larger again. **`check_project_quiet` does not exist** — #543 deleted
it. The claim came from a stale paragraph in `specs/change/context.md`, which #738 corrected the
same night for exactly this reason. So the same substitution happened twice inside one change:
Cargo was measured, and then where its output goes was assumed from prose. Both are recorded here
rather than quietly dropped, because a wrong lesson is read at `change new` before anything is
scoped (#714).

What is left is true and smaller than the issue claims: Cargo's line reaches the operator on
inherited stderr and names neither the file, the holder, nor a remedy. This notice is **additive**
to it, and is printed before the child starts rather than after it has already blocked.

Same family as #707 — a gate whose message cannot separate a violated invariant from a busy
machine — and the repository's own guidance makes it more likely, not less: long checks are
routinely backgrounded, and a backgrounded check that is later killed is exactly the shape that
orphans a child.

## Constraints carried into the design

- SpecSync targets Linux and macOS; the Windows binary was dropped for 6.0 but the qualification
  lane still builds and runs there, so nothing may stop compiling on Windows.
- Contract item 5 says verification executes **only project-configured commands**. That rules out
  spawning `lsof` from the verification path to identify a lock holder, however convenient.
- This must not become its own false-success. Eight separate instances in this release of a check
  reporting success it had not verified; a "probably wedged" heuristic firing on a slow-but-healthy
  compile would be a ninth.

## Ruled out

- **Elapsed-time heuristics.** Any "this is taking suspiciously long" signal fires on the healthy
  four-minute compile that is the normal case here. Everything printed is read from the lock.
- **`lsof` for the holder PID.** Correct answer, wrong mechanism: contract item 5 forbids it, and a
  diagnostic that can itself hang on an unresponsive mount is a poor trade for a PID.
- **Darwin `libproc` FFI for the holder PID.** `libc` exposes `proc_listpids`/`proc_pidinfo` but
  not `vnode_fdinfowithpath` or `PROC_PIDFDVNODEPATHINFO`, so it would mean hand-writing a struct
  layout in unsafe code for a diagnostic. Rejected on risk, not on effort.
- **A sidecar file recording the child PID.** Cross-platform and cheap, but it names a PID from
  recorded state rather than from the lock, so a stale sidecar plus PID reuse produces a confident
  wrong answer — the exact failure this change exists to remove.
- **Item 3 of the issue (stale-lock remediation).** It needs the holder, so it would offer
  different advice about identical state on the two supported platforms. Not built.

## What the first `change check` taught

The first verification run failed, and it failed on **this change's own test**, which is the best
possible place for it to fail. `a_held_cargo_build_lock_produces_a_notice_naming_the_lock` ended
with a third assertion — release the lock, probe again, require `None` — and under a host running
seven concurrent agent worktrees it reported the lock as still held after `drop`.

The mechanism is the same property the fix relies on. A `flock` lives on the **open file
description**, not on the file and not on the process. This test binary is multithreaded and spawns
child processes constantly, so a descriptor duplicated into a concurrent `fork` keeps its lock
alive until that child `exec`s and `O_CLOEXEC` closes it. "The lock is released" is therefore not
observable at a deterministic instant from inside this process, and the assertion depended on how
busy the machine was — which is precisely the gate shape #707 documents and #702 argues teaches
everyone to ignore red.

The assertion is gone rather than retried or given a longer window. The two that remain are
deterministic in both directions: nothing has ever locked a file in a fresh `TempDir`, and the
held case is held by this process for the whole assertion. The held case cannot pass vacuously
either, because the test takes the lock with a **blocking** acquisition first: a leaked descriptor
would hang the test, not silently satisfy it.

Carried into the production code as a bounded, documented cost rather than a bug: the probe itself
acquires the lock for the duration of one syscall pair, so a process that forks concurrently can
extend that acquisition until the child `exec`s. The descriptor is `O_CLOEXEC`, so the window
cannot outlive the `exec`, and the CLI runs verification sequentially on one thread with no
concurrent spawn — worst case a real Cargo waits microseconds longer.

## The second failed attempt in the ledger was environmental, and the evidence says so

`verification-attempts.json` also records a run where `cargo test` exited 101 because the
integration test binary died of `signal: 15, SIGTERM` with every test passing up to that point.
That was **not** this change's reaper, and the evidence excludes it rather than an argument doing
so: the only `SIGTERM` sender in the codebase is the new reaper, every call site signals a process
*group*, the victim's group leader is `cargo`, and `cargo` survived and exited 101 normally — a
group kill would have taken both. The cause was found directly afterwards: the agent harness stops
long-running background tasks, and it stopped a later run of the same command with an explicit
"was stopped". Re-running the check detached from that supervisor passed. It is named here because
an unexplained failed attempt in an append-only ledger is worse than a failure with a cause.

## What the independent review changed

The scoped review returned BLOCK, and three of its findings were real defects rather than taste:

- The first version of this section CLAIMED that a `.cargo/config.toml` `build.target-dir` produced
  silence. It did not; the derivation never looked, fell through to `<root>/target`, and a stale
  `target/` from an earlier layout would have made the notice name a lock nothing was waiting on.
  Now the files Cargo merges are actually read, and `--config` and `--manifest-path` bail too.
- The notice recommended `lsof` as though it named the holder, in the branch that only ever runs on
  the platform this change's own research says `lsof` does not report lock state on. It now says
  what `lsof` answers.
- `REQ-change-091` said "a catchable terminating signal" while four are forwarded. The requirement
  now names what verification actually forwards.

Two narrower ones were also fixed: `disarm` unpublishes the group at the moment the child is
reaped, so a signal arriving before the guard drops cannot signal a recycled PID; and the handler
is installed before `spawn` rather than after, so the window in which the child is out of the
terminal's foreground group with nothing forwarding to it is one instruction rather than a
function call.

## From the change's design.md

# Design

Both halves land in `run_configured_command` in `src/change.rs`, the single place verification
spawns a child.

## Half one — say what is happening

```
cargo_build_lock_path(root, program, args, environment) -> Option<PathBuf>
cargo_build_lock_is_held(path)                          -> bool
cargo_build_lock_holders(path)                          -> Vec<u32>
cargo_build_lock_wait_notice(..)                        -> Option<String>
```

`cargo_build_lock_path` derives the `.cargo-lock` the command will contend on from the argv and a
`CargoBuildEnvironment` read once at the call site (`CARGO_TARGET_DIR`, `CARGO_BUILD_TARGET_DIR`,
`CARGO_BUILD_TARGET`). Passing the environment in rather than reading it inside keeps the
derivation a pure function of its inputs — `std::env::set_var` is `unsafe` under edition 2024 and a
parallel test suite must not share process environment anyway.

It returns `None` — deliberate silence — for a non-`cargo` program, for a subcommand outside an
explicit allowlist of the ones that take the build lock, for a `--config` or `--manifest-path`
argument, for two `--target` triples, for a custom target JSON, and for a profile or triple that is
not a single path component. `cargo_config_moves_build_directory` additionally READS the
`.cargo/config.toml` and `.cargo/config` files Cargo merges — every ancestor of the project root
plus `CARGO_HOME` — and bails when one sets `build.target-dir` or `build.target`, or cannot be
parsed. An earlier version of this design claimed that silence without implementing it: the
derivation fell through to `<root>/target`, so a project that moved its build directory while a
stale `target/` remained would have had the notice name a lock nothing was waiting on.

`cargo_build_lock_is_held` is the authoritative half and the only claim always made: open the file
(never create it — Cargo creates it when it takes the lock, so absent means unheld) and attempt a
non-blocking exclusive `fs2` acquisition. Contention is the answer; success means nothing held it,
and dropping the handle releases it again. `fs2` is already a dependency and already how
`acquire_project_lock` works, and it matches Cargo's own primitive on both platforms (`flock` on
Unix, `LockFileEx` on Windows), so the probe is a true "would I block" test rather than a proxy.

The probe's own acquisition is a bounded, documented cost. An `flock` lives on the open file
description, so a process that forks while the probe holds it extends that acquisition until the
child `exec`s; the descriptor is `O_CLOEXEC`, so the window cannot outlive the `exec`, and the CLI
runs verification sequentially on one thread with no concurrent spawn. Worst case a real Cargo
waits microseconds longer.

`cargo_build_lock_holders` is best-effort and platform-split. Linux publishes ownership in
`/proc/locks`; `proc_locks_flock_holders` parses it, matching `FLOCK` + `WRITE` on the file's
`major:minor:inode` and skipping the `->` waiter lines, which are queued behind the holder rather
than holding it. Everywhere else it returns empty, which the notice reads as *not determinable*,
never as *nobody holds it*.

Cargo is not silent here either, and the design does not pretend otherwise: measured against a held
lock it prints `Blocking waiting for file lock on artifact directory`. What it does not print is
the file, the holder, or a remedy. This notice is ADDITIVE to that line, not a replacement for a
suppressed one — an earlier draft claimed `check_project_quiet` discarded it, and that function was
deleted by #543. The message has two shapes and both are true statements:

```
specsync: waiting on target/debug/.cargo-lock held by PID 75733: the Cargo build-directory lock is taken, so this command is blocked rather than compiling
```

```
specsync: waiting on target/debug/.cargo-lock: the Cargo build-directory lock is held by another process, so this command is blocked rather than compiling
specsync: `lsof target/debug/.cargo-lock` lists the processes holding it open — a lock holder is always among them — and one of those is an orphan if a check was interrupted
```

The second line says what `lsof` answers rather than implying it names the holder: on macOS it
reports the file as open, not locked (`lsof -Fpcl` returns an empty lock field). Holding a `flock`
requires an open descriptor, so the holder is always in that list, which is a true claim at the
precision the platform allows. The line is `#[cfg(unix)]`-gated so a Unix tool is not recommended
where it does not exist.

It goes to stderr, which keeps `--json` stdout intact.

## Half two — reap the child

On Unix the child gets `process_group(0)`, so it leads its own group and the group can be ended as
a unit — `cargo` plus every `rustc` and test binary under it, not only the process that was waited
on.

A group that nothing forwards to would be worse than none: in a terminal, Ctrl-C reaches the
foreground group, and moving the child out of it would stop the signal reaching `cargo` at all. So
a handler for `SIGINT`, `SIGTERM`, `SIGHUP`, and `SIGQUIT` forwards **the signal it received** to
every registered group, then restores the default disposition and re-raises, preserving the exit
status a wrapping shell would have seen. A signal the parent inherited as ignored stays ignored, so
a backgrounded job keeps its inherited dispositions.

Registration is a fixed table of eight `AtomicI32` slots rather than a lock, because the reaper
runs inside a signal handler where an atomic load and `kill` are async-signal-safe and taking a
mutex is not. A `VerificationChildGroup` guard covers the non-signal exits: it ends the group on
drop unless `wait` already reaped it, and `disarm` unpublishes the slot at the same instant it
records the reap — once a child is reaped its PID is free, so a signal arriving between `wait`
returning and the guard dropping must not find a group id the handler would signal. The handler is
installed BEFORE `spawn` for the mirror-image reason: the child leaves the terminal's foreground
group the moment it exists, so anything between `spawn` and registration is a window in which this
change orphans a child the unmodified code would not have.

Windows keeps `command.status()` unchanged behind `#[cfg(not(unix))]`.

## What this does not fix

A `SIGKILL`ed parent still orphans its child, because no process runs code after `SIGKILL`. That is
not a gap to close later; it is why half one is the core deliverable and half two the durable one.
A genuinely concurrent Cargo invocation also holds the lock legitimately, and the operator still
deserves to be told which file they are waiting on.

## From the change's testing.md

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

## Where these lessons go

- `specs/change/context.md`
