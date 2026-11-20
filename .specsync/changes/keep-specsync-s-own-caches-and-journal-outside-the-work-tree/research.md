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
- **Cargo's dep-info.** Cargo writes dep-info for every compiled unit as `target/debug/deps/<name>-<hash>.d`, with paths relative to the crate root. It lists every source file compiled into that unit: `include!`d files are there, and `#[cfg(test)]` module files are not. The copy beside the binary, `target/debug/specsync.d`, is written only by a plain build. A clean `cargo test` writes none, which the first Linux run showed. So the scans use the `deps/` dep-info whose `specsync-<hash>` holds exactly the bytes of `CARGO_BIN_EXE_specsync`. On macOS that file is a copy of the uplifted binary, not a hard link, so the bytes are compared. At `a2a50824` the binary's list has 137 files under `src/`, among them `src/validator/tests.rs` and `src/manifest/tests.rs`. A walk of `mod`, `#[path]` and `include!` from `src/main.rs` reaches the same 137.
- **`O_PATH` on Linux.** cap-std opens a directory it only needs to traverse with `O_PATH`, and `fsync` refuses such a descriptor. A durable write therefore syncs the folder through a readable open of `.` taken through the handle.
- **`git rev-parse --git-path hooks`** honors `core.hooksPath`. A relative value is printed relative to the current directory.
