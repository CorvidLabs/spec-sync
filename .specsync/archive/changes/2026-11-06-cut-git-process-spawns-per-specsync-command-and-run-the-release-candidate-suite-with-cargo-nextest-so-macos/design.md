---
change: cut-git-process-spawns-per-specsync-command-and-run-the-release-candidate-suite-with-cargo-nextest-so-macos
artifact: design
---

# Design

1. Repository-fact memo in `git_utils`. A per-process memo keyed by the canonical repository root caches the answers to: HEAD commit (`rev-parse --verify HEAD`), `--show-prefix`, `--is-inside-work-tree`, `--is-shallow-repository`, the origin default branch and its commit, `--git-path index`, `--shared-index-path`, `core.fsmonitor` and the core settings regexp. Callers in `change/` and `commands/change.rs` ask the memo instead of spawning. Every SpecSync code path that commits, writes the index, runs `update-ref` or switches branches calls one invalidation function for that root, so the next read asks git again. Failures are not memoized (git_utils keeps its safe-default contract).
2. Batched object reads. One lazily started `git cat-file --batch` child per repository root serves `<commit>:<path>` blob reads and tree listings at fixed commits. Reads at `HEAD:` resolve HEAD through the memo first, so a commit inside the command invalidates correctly. The child is closed when the command ends. Missing objects return the same "absent" answer the per-file `git show` path gives today.
3. Spawn bound. A test-only spawn counter around `git_cmd` asserts a ceiling per lifecycle command, extending the existing `*_git_processes_do_not_grow_*` tests, so the saving cannot silently regress.
4. Release-candidate lane. `fledge.toml` gains a `test-nextest` task (`cargo nextest run --no-fail-fast` with the default profile, no filters) and `lanes.release-candidate` uses it in place of `test`. `release.yml` installs a pinned cargo-nextest release asset verified by sha256, the same way it installs Fledge. `lanes.verify`, `lanes.ci` and Ubuntu CI keep `cargo test`, so both runners stay exercised.

No public CLI, output or file format changes. The memo and reader live only for one process.
