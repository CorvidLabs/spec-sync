---
change: lifecycle-commits-stage-only-what-the-change-owns-never-every-untracked-file
artifact: context
---

# Context

`change check --commit` and `change ship --push` both committed through `git_commit_all` in
`src/commands/change.rs`, which ran `git add -A`. That stages every untracked, non-ignored file
in the project, and `--push` publishes it. In one week it put a private debug zip into a pushed
arcsite commit, and an agent's `.agents/` directory and an `exp2.sh` experiment script into
corvid-bot commits. Nothing warned, because nothing in the lifecycle knew which files were its own.

The same function carried a doc comment on `run_checked_commit` saying nothing is committed unless
verification passes. That holds for the first pass only: when the second pass fails, the
materialize commit is already on the branch.

Constraints a session picking this up needs:

- Verification digests the working tree, tracked and untracked (`project_input_digest` walks
  `git ls-files --cached --others --exclude-standard`). A tracked edit left unstaged would be
  verified and never committed, so tracked edits stay in the commit.
- An untracked file that is left out stays inside that digest. Committing it later keeps the
  evidence current. Removing it stales the evidence, so the warning has to say so.
- `change new` writes `.specsync/workflow-v2-baseline.json` in a freshly adopted project. The
  first test run of the fix left it out, which would have committed a change whose origin anchor
  never reached history. It is a lifecycle ledger and is owned.
- `.specsync/change.lock` and the transaction journal are runtime files. `init` ignores them,
  but a project without that ignore file used to commit the lock through `git add -A`. They are
  neither committed nor reported now.
- The command layer holds no lifecycle policy (`specs/cmd_change/context.md`), so the decision
  about which untracked paths the change owns lives in `src/change.rs`.

Ruled out: owning the change's `affected_paths`. They are prefixes such as `src/` or `.`, and
owning every untracked file under them brings the sweep back. Also ruled out: owning a whole spec
directory, because a stray file dropped beside a spec is not a spec.
