---
change: never-replay-a-transaction-journal-specsync-did-not-write-or-into-git-s-files
artifact: design
---

# Design

All of it is in `src/change/sdd_version.rs`, with the path rule beside `transaction_relative_path` in `collect_requirement_ids.rs`.

- **`journal_is_tracked(root)`:**
  - `git ls-files --error-unmatch` or `git cat-file -e HEAD:./…` succeeding means tracked.
  - Checked before the journal is parsed. When it is tracked, the replay refuses with its own message: `the transaction journal at … is tracked by Git, so it came with the repository and is never replayed; remove it with `git rm --cached .specsync/change-transaction.json` and delete the file`.
- **`replayable_transaction_path(root, relative)`:**
  - Refuses a component that equals `.git` after trailing dots and spaces are removed and ASCII is lowercased.
  - Judges the path after backslashes become separators, because the write (`normalize_project_path`) does the same.
  - Accepts a path under `.specsync/`, and a spec file, `requirements.md` or `history/*.md` under the configured `specs_dir`. A `specs_dir` of `.` therefore opens nothing else.
  - Accepts a path that equals a registry-mapped spec, is the `requirements.md` beside it, or is in its `history/` directory.
  - Refuses anything else with `entry \`<path>\` is outside what the change workflow writes`. A `.git` refusal names Git's own files.
- **Replay.**
  - The journal is parsed as JSON first. An array is the 5.x form and is refused with `a bare-array journal from SpecSync 5.x is no longer replayed`.
  - The durable form is checked as before.
  - Every entry is then checked, by `safe_project_path`, the rule above and the existing symlink-component check, before any entry is applied.
  - The first refusal returns the invalid-journal error, so nothing is replayed and the journal stays for the person to inspect.
- **Writing.** `validate_prepared_transaction_target` applies the same rule, so this build never journals a write it would refuse to replay.
- **The legacy enum arm is removed.**
