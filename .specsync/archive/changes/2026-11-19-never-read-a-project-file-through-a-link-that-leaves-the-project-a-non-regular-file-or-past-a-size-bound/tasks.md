---
change: never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound
artifact: tasks
---

# Tasks

- [x] Failing tests on `3a02a5d5`: `util` unit tests, the integration matrix, the source scan
- [x] `util`: the bound on `read_project_file`, plus `read_project_bytes` and `read_project_string`
- [x] `change` and `cmd_change`: `sdd.json`, the journal, the change records, the spec and companion reads
- [x] `config`, `ignore`, `registry`, `mcp`, `agents`, `hooks`: state and hook files
- [x] `hash_cache`, `exports`, `deps`, `schema`, `validator`: hashing, source, schema and requirements reads
- [x] The spec readers: `archive`, `cmd_check`, `cmd_diff`, `cmd_init`, `cmd_lifecycle`, `cmd_merge`, `cmd_migrate`, `commands`, `cmd_report`, `cmd_resolve`, `cmd_stale`, `compact`, `generator`, `merge`, `output`, `scoring`, `view`
- [x] Specs, lessons, `CHANGELOG.md`, `cli.md`

Human gates, not tasks: the intent wording, the implementation review before the archive, and the decisions to publish the advisory and to release.
