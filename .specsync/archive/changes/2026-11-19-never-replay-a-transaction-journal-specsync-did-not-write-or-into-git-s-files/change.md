---
id: never-replay-a-transaction-journal-specsync-did-not-write-or-into-git-s-files
state: archived
type: bug_fix
base_commit: cf5284076db2b692d8e8df8953eb6525d5952690
---

# Never replay a transaction journal SpecSync did not write, or into Git's files

## Intent

Never replay a transaction journal SpecSync did not write, or into Git's files

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- A journal at .specsync/change-transaction.json that Git tracks, in the index or in HEAD, is never replayed: the writing change command refuses and names the git rm fix. A journal entry with a .git component in any letter case, or with trailing dots or spaces, or naming a file outside .specsync/, the configured specs_dir and the registry-mapped spec files with their requirements.md and history fragments, refuses the whole journal and replays nothing. A bare-array journal is refused with the invalid-journal recovery. The person's own interrupted write is still recovered. The new tests fail on cf528407 and pass with the fix

## No-spec Rationale

Not applicable
