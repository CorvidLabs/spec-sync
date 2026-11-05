---
change: cut-git-process-spawns-per-specsync-command-and-run-the-release-candidate-suite-with-cargo-nextest-so-macos
artifact: requirements
---

# Requirements

- REQ-git-utils-008 (added): repository facts a command does not change are asked of git at most once per command, and are re-read after the command's own commit, index write or ref move.
- REQ-git-utils-009 (added): committed blob and tree reads at fixed commits within one command go through one `git cat-file --batch` process per repository and give the same answers as per-file reads, including for missing objects.
- REQ-change-125 (added): lifecycle commands stay within a tested git spawn ceiling, and the slowest lifecycle integration test spawns at most 40% of its 15,563-process baseline.
- Traces to HI SPEED-2.b.1 and SPEED-2.b.2.
