---
change: cut-git-process-spawns-per-specsync-command-and-run-the-release-candidate-suite-with-cargo-nextest-so-macos
artifact: research
---

# Research

- Raw data: per-test nextest durations and the full git argument log for the slowest test, gathered through a PATH shim around `/usr/bin/git` (the shim adds its own spawn, so absolute times under the shim are inflated; counts are exact).
- macOS spawn cost measured directly: `/usr/bin/git --version` 200 times took 7.98s (about 40ms each, 2% CPU); the Xcode git binary directly took 2.13s (about 10ms each). Spawn cost on macOS is several times Linux's, which is why the same suite is about 9.5x slower there.
- cargo-nextest 0.9.151 ran the full suite with no failures and no code changes, so process-per-test isolation does not break any test.
- Existing guards to reuse: `*_git_processes_do_not_grow_with_other_archives` tests already count spawns per command; `git_utils::git_cmd` is the single cleared-environment spawn point (REQ-git-utils-007), and `src/change/read_cache.rs` already keys a persistent cache by content.
