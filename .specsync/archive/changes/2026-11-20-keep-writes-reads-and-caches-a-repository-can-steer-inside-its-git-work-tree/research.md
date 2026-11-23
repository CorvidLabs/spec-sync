---
change: keep-writes-reads-and-caches-a-repository-can-steer-inside-its-git-work-tree
artifact: research
---

# Research

- **The review's report** (`security/codex-review/report.md`, private) names each path with file and line at `0d9cbde5`. The builder checked each one against the branch before writing a test.
- **Links into `.git`.** On the branch before this change, two cases replaced Git's own files, compared by bytes only:
  - `change answer` with `change.md -> ../../../.git/hooks/pre-commit` replaced an executable hook;
  - `hooks install --claude` with `CLAUDE.md -> .git/hooks/post-checkout` appended into a hook.
- **Monorepo (R1).** In a subproject fixture, `hooks install` worked on 6.3.0 and was refused on the branch, and `deps` listed 1 module on 6.3.0 and 0 on the branch. `check` refused a linked specs directory in both.
- **`git rev-parse`.**
  - `--show-toplevel` fails outside a work tree, inside `.git`, and in a bare repository; each falls back to the root.
  - `--absolute-git-dir` names the Git directory even when it is not called `.git`, as with `--separate-git-dir`. For a linked worktree or a submodule it lies outside the top folder, where the handle cannot reach anyway.
- **Versions.**
  - Tracked hash cache replay: from 6.0.0, where validation snapshots arrived.
  - Changelog `git show`: from 3.4.0.
  - `import --from-dir`: from 3.8.0.
  - `change.md` writes: from 5.0.0.
  - `--backup`: from 4.3.1.
  - `init-registry`: from 2.0.0.
  - The unbounded spec walk: in every version.
- **The generator.** The plain `generate_specs_for_unspecced_modules` is now used only by tests. `specsync generate` and the MCP tool already wrote through confined writers.
