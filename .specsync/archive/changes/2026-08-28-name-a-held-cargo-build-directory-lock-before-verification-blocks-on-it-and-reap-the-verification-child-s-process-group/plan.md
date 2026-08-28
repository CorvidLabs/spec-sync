---
change: name-a-held-cargo-build-directory-lock-before-verification-blocks-on-it-and-reap-the-verification-child-s-process-group
artifact: plan
---

# Plan

1. Add the build-lock derivation, probe, holder lookup, and notice to `src/change.rs`, all private
   and all reachable only from `run_configured_command`.
2. Print the notice on stderr immediately before the child is spawned, so it lands before the wait
   rather than after it.
3. Replace `command.status()` with a Unix path that sets `process_group(0)`, registers the group,
   waits, and disarms; keep `command.status()` behind `#[cfg(not(unix))]`.
4. Install the forwarding signal handler lazily on first use, preserving inherited `SIG_IGN`.
5. Cover the derivation and the notice with unit tests in `src/change_tests.rs`, holding a real
   `flock` from the test process for the held case.
6. Cover the wiring and the process group end to end in `tests/integration/change.rs` with a
   stand-in `cargo` and a stand-in group reporter.
7. Clone unfixed `main` into a separate checkout, copy only the new integration tests in, and
   record the failure output of each discriminator against it.
8. Update the `change` contract (item 5), add `REQ-change-091`, and update the module's tasks,
   context, and testing companions.

Ordering note specific to this change: step 7 has to run before the lifecycle is committed to,
because the delivery scope freezes at `change new` (#542) and the recorded control output is part
of the requirement evidence.
