---
change: CHG-0138-release-profile-test-barriers-must-not-fail-the-suite-they-cannot-synchronise
artifact: context
---

# Context

`cargo test --release` exited 101 on a clean `main`, with five integration failures:

    generate exited before the post-coverage barrier
    tool exited before the directory-enumeration barrier

They read as TOCTOU regressions in the root-identity guards. They were not. `cargo test` in debug
passed, so anyone running the release profile before shipping saw five failures nobody else could
reproduce.

## Mechanism

Each affected test spawns the real binary, waits for it to publish a marker file at a
synchronisation point, swaps a path underneath it, then asserts the command refuses to report a
result. Those synchronisation points are `#[cfg(debug_assertions)]` and compile to `Ok(())` in
release, so the marker never appears, the child runs to completion, and the test polls for a file
that will never exist.

Seven tests carry the defect, not five: two more are `#[cfg(windows)]` and share a failing test's
helper, so they cannot surface on Linux or macOS.

## The shipped binary was never weaker

Every `#[cfg(debug_assertions)]` item in `src/` is a test **rendezvous**, not a guard. The guards
they synchronise are compiled unconditionally:

    RetainedGenerateRoot::verify_public_path        src/commands/generate.rs:491
    verify_coverage_project_root                    src/validator.rs:1692
    open_server_root_capability                     src/mcp.rs:233
    ConfinedReadRoot::revalidate_before_success     src/mcp.rs:5088

None carries a `cfg` attribute; every call site is unconditional. A release binary put under a
live symlink race still refuses, with `Coverage project root … changed during retained traversal`
and exit 1.

## Why the rendezvous cannot simply ship

Compiling it into release would add an env-var-triggered wait loop of up to 30 seconds plus a file
write at a caller-named path (`PathBuf::from(<env var>).join(<fixed name>)` with `create_new`).
On `revalidate_before_success` that sits on **every** read-success path, so it is a repeated
per-operation stall rather than a one-shot. All four MCP rendezvous lack the
`SPECSYNC_TEST_CONTEXT` interlock that the generate and coverage ones have.

That is a net regression in the shipped binary — the opposite of the goal.

## Verified adversarially before adoption

This change was flagged by the harness as a potential security-test removal and put through four
independent verification lenses — exhaustive static audit, live exploitation of the release
binary, counterfactual analysis of the rejected alternative, and coverage-gap analysis. **Zero
refuted the claim.** Two landed real symlink races against the release artifact and watched the
guards fire, including a 120-run randomised fuzz with zero suspicious outcomes.

The verification also corrected two false claims in the original write-up: a cited substitute test
that asserts a *different* guard's message, and a CI claim contradicted by `release.yml:633`. Both
are fixed here. See #614 for the residual coverage gap, filed separately.
