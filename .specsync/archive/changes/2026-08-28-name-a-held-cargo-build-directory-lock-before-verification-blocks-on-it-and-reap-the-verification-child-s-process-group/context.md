---
change: name-a-held-cargo-build-directory-lock-before-verification-blocks-on-it-and-reap-the-verification-child-s-process-group
artifact: context
---

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
