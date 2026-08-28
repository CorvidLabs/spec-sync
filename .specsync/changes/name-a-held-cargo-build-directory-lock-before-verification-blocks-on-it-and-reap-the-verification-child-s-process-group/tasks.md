---
change: name-a-held-cargo-build-directory-lock-before-verification-blocks-on-it-and-reap-the-verification-child-s-process-group
artifact: tasks
---

# Tasks

- [x] Derive the `.cargo-lock` a configured Cargo command will contend on, and return nothing
      rather than a guess when the build directory is not derivable from argv and environment
- [x] Probe that lock with a non-blocking exclusive acquisition, so the claim is read from the lock
      and never from elapsed time
- [x] Name the holding PID from `/proc/locks` on Linux; name the identifying command everywhere
      else rather than guessing at a holder
- [x] Print the notice on stderr before the command that will wait on it starts
- [x] Give every Unix verification child its own process group
- [x] Forward `SIGINT`/`SIGTERM`/`SIGHUP`/`SIGQUIT` to the registered groups, then restore the
      default disposition and re-raise so the exit status is unchanged
- [x] End the group on drop when a non-signal exit leaves the child unreaped
- [x] Keep the Windows build compiling with `#[cfg(not(unix))]`
- [x] Prove both discriminators fail against a binary built from a separate checkout of unfixed
      `main`, and record the failure output
- [x] Update the `change` contract, requirements, tasks, context, and testing companions
- [x] Act on the independent review's BLOCK: actually read the Cargo config files the spec claimed
      were a silence case, bail on `--config`/`--manifest-path`/`cargo nextest`, say what `lsof`
      answers instead of implying it names the holder, name the forwarded signals in the
      requirement, unpublish the group at reap, and install the handler before the spawn
- [x] Measure rather than repeat the issue's premise: Cargo does print `Blocking waiting for file
      lock on artifact directory`, so correct that claim everywhere it was written
- [x] Then correct the correction: `check_project_quiet` was deleted by #543, so the claim that
      Cargo's line is discarded by it is false too; the notice is ADDITIVE to Cargo's line
- [x] Bail on `[build] build-dir` and on an `[env]` table setting a layout variable, and cover the
      `CARGO_HOME` leg that the first version of the config test never reached
- [x] Rebase onto `main` after #731/#732/#733/#736/#737/#738, keeping both sets of folded lessons
      in `specs/change/context.md`
- [x] Write the corrected `REQ-change-091` into `specs/change/requirements.md` by hand, because
      `materialize_change_deltas` returns early on `canonical_applied` and a re-approved delta
      never reaches the canonical tree; record that mechanism in `specs/change/context.md`
