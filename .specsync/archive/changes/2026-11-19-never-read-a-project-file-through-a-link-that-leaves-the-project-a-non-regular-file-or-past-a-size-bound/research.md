---
change: never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound
artifact: research
---

# Research

The 144 direct file reads in non-test source at `3a02a5d5`, by what they read:

| Reads | Modules | Bound |
|---|---|---|
| `.specsync/sdd.json` | `change`, `cmd_change`, `cmd_init` | 4 MiB |
| Change records: `state.json`, `approvals.json`, `verification*.json`, `review*.json`, `finalization.json`, `history.json`, `retirement.json`, ledgers | `change`, `cmd_change` | 4 MiB |
| `.specsync/change-transaction.json` | `change` | 64 MiB |
| Config files, `.specsyncignore`, registry, `mcp-tools.lock.json`, `agent-artifacts.json`, `version`, `lifecycle/*.json`, `.gitignore`, `.gitattributes` | `config`, `ignore`, `registry`, `mcp`, `agents`, `cmd_migrate`, `cmd_lifecycle`, `cmd_init`, `cmd_merge` | 4 MiB |
| Spec files, companions, history fragments, templates | `archive`, `change`, `cmd_check`, `cmd_diff`, `cmd_lifecycle`, `cmd_merge`, `cmd_migrate`, `commands`, `cmd_report`, `cmd_resolve`, `cmd_stale`, `compact`, `deps`, `generator`, `hash_cache`, `merge`, `output`, `scoring`, `validator`, `view` | 8 MiB |
| Hook files | `hooks` | 8 MiB |
| Generated agent artifacts | `agents` | 8 MiB |
| Source and schema files | `exports`, `deps`, `hash_cache`, `schema` | 8 MiB |
| `.specsync/hashes.json` | `hash_cache` | 64 MiB |
| A delivery file whose digest is compared | `change` | 64 MiB |

Out of scope and left as they are:

- `commands/issues.rs` reads through a confined handle with its own bounds already.
- `git_utils.rs` reads Git's output.
- `merge-file` reads the files Git hands it.
- `.git/shallow` and the pre-commit hook are in the repository's own `.git`.
- `change correct-owner --manifest` and `import --from-dir` read paths the person names.
- `watch`, coverage discovery, the Gradle manifest reader and MCP already read through bounded, confined handles.

Of the in-scope sites, 112 already have the project root in scope. The others gain it as an argument: `hash_file`, `read_config_file`, `validate_config_file`, `scan_exported_symbols_full`, the TypeScript re-export resolver, `read_frontmatter_section`, `build_schema_snapshot`, `view_spec`, `count_completed_tasks`, `read_bounded_change_text`, the relayed-decision workspace checks, and the unknown-members merge.

On the branch build before this change, in a scratch fixture:
- a link from `sdd.json` to a file outside was read ("invalid SDD policy … expected value at line 1 column 1" from the outside bytes);
- a FIFO at `sdd.json` blocked `change status --json` until it was killed after 8 seconds;
- a 5 MiB sparse `sdd.json` was read whole.
