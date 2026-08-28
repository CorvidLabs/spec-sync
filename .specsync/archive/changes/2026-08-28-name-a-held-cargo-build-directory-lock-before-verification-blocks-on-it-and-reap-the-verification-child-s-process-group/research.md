---
change: name-a-held-cargo-build-directory-lock-before-verification-blocks-on-it-and-reap-the-verification-child-s-process-group
artifact: research
---

# Research

Four things were checked against the machine rather than assumed, because each of them decided a
piece of the design.

**Where Cargo's build lock lives, and what is in it.** `target/debug/.cargo-lock` and
`target/release/.cargo-lock` both exist in this checkout and both are **zero bytes**. Cargo writes
nothing into the file, so there is no PID to read out of it and holder identification has to come
from the operating system or not at all.

**Whether `lsof` reports the lock on macOS.** It does not. Holding a `flock` from a Python process
and running `lsof -Fpcl` on the file returns `p86212`, `cPython`, `f3`, and an **empty** `l` field:
macOS `lsof` reports the file as *open*, not as *locked*. So even paying the cost of spawning it —
which contract item 5 forbids — would buy an opener, not a proven holder. On Linux `/proc/locks`
reports ownership directly and needs no subprocess at all.

**Whether `libc` gives a Darwin route.** `libc-0.2.185` defines `proc_listpids`, `proc_pidinfo`,
`PROC_PIDLISTFDS`, and `proc_fdinfo` for Apple targets, but **not** `vnode_fdinfowithpath` or
`PROC_PIDFDVNODEPATHINFO`. Mapping an fd to a path would mean hand-writing the struct layout in
unsafe code. Rejected for a diagnostic.

**Whether Cargo is actually silent when it blocks.** It is not, and this was checked only after the
independent review said so — the issue's premise had been repeated in four artifacts before anyone
measured it. A minimal crate, its `target/debug/.cargo-lock` held by the Python holder above, and
`cargo build` prints exactly one line: `Blocking waiting for file lock on artifact directory`. What
it omits is the file, the holder and the remedy. That is the honest size of the gap: this notice is
additive to Cargo's line, not a replacement for one that goes missing.

**Whether `check_project_quiet` still exists.** It does not — `grep -rn check_project_quiet src/`
returns nothing, because #543 severed `comment` from the trust layer. The first correction of the
premise above had claimed Cargo's line "is discarded entirely by `check_project_quiet`", read from
a `specs/change/context.md` paragraph that #543 had made false and #738 corrected the same night.
Checked here only because the coordinator caught it. The lesson is narrower than "verify claims":
a *correction* is exactly as likely to be assumed as the thing it corrects, and the source of this
one was a spec paragraph rather than the code.

**Whether `flock` can be held and probed inside one process.** Yes, and this is what makes the
tests deterministic rather than timed. `flock` locks are attached to the open file description, so
two separate `open` calls in the same process conflict — POSIX says so explicitly, and it was
confirmed here by `a_held_cargo_build_lock_produces_a_notice_naming_the_lock` passing. The test
process holds the lock; the child under test sees genuine contention; no clock is involved.

One incidental confirmation from the same session: while the suite was running, `ps` showed seven
concurrent agent worktrees on this host, each with its own `cargo test`. That is the load that made
"the check seems slow" ambiguous in the first place, and it is the reason no assertion in this
change is allowed to depend on wall-clock timing.
