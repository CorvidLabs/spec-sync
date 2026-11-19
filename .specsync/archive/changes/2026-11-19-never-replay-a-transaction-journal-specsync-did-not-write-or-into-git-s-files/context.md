---
change: never-replay-a-transaction-journal-specsync-did-not-write-or-into-git-s-files
artifact: context
---

# Context

An independent review of the private advisory branch found F1. Every writing `change` command takes the project lock, and the lock replays `.specsync/change-transaction.json` before the command does anything else. The journal is a file in the working tree, so a repository can commit one. Its entries only passed `safe_project_path`, which stops `..` and links out but not `.git/config`, so a committed journal could set `core.fsmonitor` (a command at the next `git status`), replace a hook keeping its mode, or delete `.git/HEAD`. The bare-array form has no envelope, and the newer digest is unkeyed. This affects 5.0.0, where the journal and its replay arrived (#335), through 6.3.0.

Human decisions, relayed by the planning session:
- "yes" (2026-11-19, about 15:49Z) to three points: never replay a journal Git tracks; replay only to paths the lifecycle itself writes and refuse any `.git` component case-insensitively; drop the legacy form.
- "yes" (about 15:55Z) to the scope details: SETUP-8 as written; a tracked journal refuses the command with the `git rm` fix; "the spec tree" means `.specsync/`, `specs_dir` and registry-mapped specs with their `requirements.md` and `history/`; `.git` refused case-insensitively with trailing dots and spaces stripped; one entry outside refuses the whole journal; the legacy form refused with the existing message and recovery.
- Left out by the same decision: moving the journal into the Git directory, and keying the digest. With the two path rules, a forged journal can only write files the repository could commit directly.

The fix ships in 6.4.0 with the dry run (#1100).
