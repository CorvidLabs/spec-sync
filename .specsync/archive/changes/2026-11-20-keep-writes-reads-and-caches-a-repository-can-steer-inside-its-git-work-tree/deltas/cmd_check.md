## ADDED

### REQUIREMENT REQ-cmd-check-020

`check --fix --backup` SHALL read each spec through the bounded project read and write its backup to `.specsync/backup-fix/<spec path>` through the confined project write (REQ-util-002). A `backup-fix` directory, or a file in it, that leads out of the repository SHALL be refused with the path, and `--fix` SHALL stop before it changes a spec. `--fix` SHALL rewrite specs through the confined project write.

Acceptance Criteria
- A `backup-fix` linked to an outside directory receives nothing, and the spec is unchanged; a backup file linked to an outside file keeps that file's bytes.
- With no link, the backup is written where it was and the fix goes on.

### REQUIREMENT REQ-cmd-check-021

When Git tracks `.specsync/hashes.json`, `check` SHALL print once `warning: hash cache not used:` with the reason and the `git rm --cached .specsync/hashes.json` fix, validate every spec, and leave the file unchanged (REQ-hash-cache-009).

Acceptance Criteria
- After the cache is committed, `check` warns, replays nothing, and the committed cache keeps its bytes.
