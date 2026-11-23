---
change: keep-writes-reads-and-caches-a-repository-can-steer-inside-its-git-work-tree
artifact: context
---

# Context

An independent review of the private advisory branch at `0d9cbde5` found nine more paths a cloned repository can steer (F2 to F9), two regressions of the earlier fixes (R1, R2), and a gap in the guard (G1):

- **F2:** `change answer`, `depend`, `supersede`, `approve` and `start` wrote `change.md` and artifacts through committed links, and `change new` built its workspace in a linked `.specsync/changes`.
- **F3:** `check --fix --backup` copied specs through a linked `.specsync/backup-fix`.
- **F4:** a committed `.specsync/hashes.json` replayed a forged pass in `check`.
- **F5:** `specs_dir` was not confined, so the spec walk could run over `/`.
- **F6:** `changelog` kept a `git show` blob of any size.
- **F7:** `import --from-dir` read through links with no bound.
- **F8:** `init-registry` created its file in a linked `.specsync`.
- **F9:** `write_project_file` opened a file to write it without the non-blocking open and type check the read has.
- **R1:** links that stay inside the repository but leave `--root` were refused, which broke monorepo subprojects.
- **R2:** absolute links were refused with a misleading message.
- **G1:** nothing caught a new direct write, and the read scan missed `fs::copy`, `OpenOptions` reads and `BufReader`.

While building this, the builder also found that a link that stays inside the clone can lead into `.git`. With `change answer`, a committed `change.md -> ../../../.git/hooks/pre-commit` replaced the hook with text the repository chose.

Human decisions, relayed by the planning session (hc t4c):
- "yes" to the five points of the review (change 5).
- "yes" (2026-11-19, about 16:00Z) to CHECK-1.f as written and to choices 1 to 8 as recommended in `security/change5-scope.md`:
  1. a tracked `hashes.json` is neither read nor saved, with one warning and the `git rm --cached` fix;
  2. the confinement root is the Git work tree's top folder, else `--root`; the caches stay strict; absolute links are refused with an "absolute link" message;
  3. `specs_dir`, `source_dirs` and `schema_dir` are confined, with a budgeted confined walk;
  4. a changelog blob over 8 MiB is skipped with a warning;
  5. `import --from-dir` is confined to the named directory, with the 8 MiB bound;
  6. `--fix --backup` keeps its layout and writes confined;
  7. the wider write scan;
  8. the two low items (`prune`, and the empty-directory cleanup in `agents`) are included.
- "All as recommended" (about 19:43Z) to point 6: the project helpers refuse a path that resolves into the Git directory, directly or through links (a `.git` component in any case with trailing dots and spaces, or a canonical target under `git rev-parse --absolute-git-dir`), with `<path> leads into the Git directory`. SpecSync's own writes into `.git` keep their own paths.

Change 4 (F1) shipped first, on the same private PR. The fix ships in 6.4.0 with the dry run (#1100), after a second independent review.
