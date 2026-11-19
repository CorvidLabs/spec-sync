---
id: never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound
state: implementing
type: bug_fix
base_commit: 3a02a5d5903b08612c0dad8c44ecfdf28fe2b0ef
---

# Never read a project file through a link that leaves the project, a non-regular file, or past a size bound

## Intent

Never read a project file through a link that leaves the project, a non-regular file, or past a size bound

## Affected Canonical Specs

- `util`
- `agents`
- `archive`
- `change`
- `cmd_change`
- `cmd_check`
- `cmd_diff`
- `cmd_init`
- `cmd_lifecycle`
- `cmd_merge`
- `cmd_migrate`
- `commands`
- `cmd_report`
- `cmd_resolve`
- `cmd_stale`
- `compact`
- `config`
- `deps`
- `exports`
- `generator`
- `hash_cache`
- `hooks`
- `ignore`
- `mcp`
- `merge`
- `output`
- `registry`
- `schema`
- `scoring`
- `validator`
- `view`

## Acceptance Criteria

- With .specsync/sdd.json, .specsync/change-transaction.json, a config file, a change record, a spec, a companion, a source or schema file, .specsync/hashes.json or another project file SpecSync reads: a symbolic link to a file outside the project, a dangling link, a FIFO, or a regular file larger than its bound is not read, the command does not wait on it, and the read fails the way that read's other read errors do today with a message naming the path and the reason; a link to a regular file inside the project is read as before; the bounds are 4 MiB for policy, config, state and change records, 8 MiB for specs, companions, templates, hook files, source and schema files, and 64 MiB for the transaction journal and the hash cache; each new test fails on 3a02a5d5 and passes with the fix

## No-spec Rationale

Not applicable
