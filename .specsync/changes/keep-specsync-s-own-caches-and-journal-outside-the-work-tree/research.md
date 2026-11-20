---
change: keep-specsync-s-own-caches-and-journal-outside-the-work-tree
artifact: research
---

# Research

- **Where Git keeps per-work-tree data.** `git rev-parse --absolute-git-dir` names the work tree's own Git directory. A linked worktree has its own, under `.git/worktrees/<name>`.
  - `git clone` never copies anything there from the remote, and Git refuses to check out a path with a `.git` component.
  - An unpacked tree that ships its own `.git` already controls Git itself (hooks, `core.fsmonitor`), so it is outside this threat.
- **Git's pathspecs are case-sensitive.** `core.ignorecase` does not change `ls-files` or `HEAD:` lookups, so a tracked check cannot be made complete. Moving the file is the class fix.
- **`git ls-files --error-unmatch -- ':(top,literal)<dir>'`** exits 0 when the index holds any file under `<dir>`, and 1 otherwise. Checked in a scratch repository: a tracked `packages/app` gives 0, an untracked `vendor/x` gives 1, and asking from a subfolder gives the same answers.
- **Cargo's dep-info.** `target/debug/specsync.d`, beside `CARGO_BIN_EXE_specsync`, lists every source file compiled into the binary, `include!`d files included and `#[cfg(test)]` module files left out. At `a2a50824` it lists 137 files under `src/`, among them `src/validator/tests.rs` and `src/manifest/tests.rs`.
- **`git rev-parse --git-path hooks`** honors `core.hooksPath`. A relative value is printed relative to the current directory.
