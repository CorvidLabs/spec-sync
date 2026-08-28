---
change: name-a-held-cargo-build-directory-lock-before-verification-blocks-on-it-and-reap-the-verification-child-s-process-group
artifact: design
---

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
