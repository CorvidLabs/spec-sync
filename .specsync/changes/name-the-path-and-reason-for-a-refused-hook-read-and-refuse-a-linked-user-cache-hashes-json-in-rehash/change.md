---
id: name-the-path-and-reason-for-a-refused-hook-read-and-refuse-a-linked-user-cache-hashes-json-in-rehash
state: approved
type: bug_fix
base_commit: 7527da3a7aea52436404766df845bbc8fb2f19b6
---

# Name the path and reason for a refused hook read, and refuse a linked user-cache hashes.json in rehash

## Intent

Name the path and reason for a refused hook read, and refuse a linked user-cache hashes.json in rehash

## Affected Canonical Specs

- `hooks`
- `hash_cache`
- `rehash`

## Acceptance Criteria

- With an installed CLAUDE.md grown past 8 MiB, or not valid UTF-8, hooks uninstall --claude exits 1 naming CLAUDE.md and the reason and leaves the file unchanged; hooks install --claude exits 1 naming it; hooks status prints the reason for that target instead of not installed (REQ-hooks-007).
With the Git hooks folder reached through a symbolic link, hooks uninstall --precommit exits 1 naming the path; in a folder that is not a Git repository, or whose hooks folder does not exist, it still reports nothing to remove and exits 0 (REQ-hooks-009).
With the user-cache hashes.json a symbolic link or a directory, rehash exits 1 with the reason check's warning gives, writing nothing: the link is still a link and its target unchanged; with a regular or absent cache, rehash still rebuilds it (REQ-hash-cache-007).

## No-spec Rationale

The code is brought to requirements already approved: REQ-hooks-007 (a refused bounded read of a hook target is handled as that read's other errors, naming the path and the reason, for install, uninstall and status), REQ-hooks-009 (a link on the way to the hooks folder is refused with the path named) and REQ-hash-cache-007 (save refuses, writing nothing, when hashes.json is a symbolic link or anything other than a regular file). No requirement text changes.
