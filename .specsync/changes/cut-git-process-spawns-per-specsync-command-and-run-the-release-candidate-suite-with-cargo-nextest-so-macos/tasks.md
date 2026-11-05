---
change: cut-git-process-spawns-per-specsync-command-and-run-the-release-candidate-suite-with-cargo-nextest-so-macos
artifact: tasks
---

# Tasks

- [ ] Repository-fact memo and invalidation in `src/git_utils.rs`, with tests
- [ ] Route repeated fact reads in `src/change/*` and `src/commands/change.rs` through the memo
- [ ] Batched `cat-file --batch` reader, routing blob and tree reads through it
- [ ] Spawn-ceiling tests per lifecycle command; slowest test at most 40% of 15,563 spawns
- [ ] `test-nextest` fledge task and `lanes.release-candidate` switch
- [ ] Pinned, checksum-verified cargo-nextest install in `.github/workflows/release.yml`
- [ ] Before and after measurements in `testing.md`
- [ ] Specs (git_utils, change, cmd_change), RELEASING.md and CHANGELOG
