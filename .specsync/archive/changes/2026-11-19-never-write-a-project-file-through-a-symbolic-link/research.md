---
change: never-write-a-project-file-through-a-symbolic-link
artifact: research
---

# Research

The repository already has a race-free way to stay inside a root: a `cap_std::fs::Dir`, which resolves each path component beneath its handle and refuses a path that leaves it, an absolute link target included. A link that stays inside is followed. `generator`, `watch`, `validator` and `mcp` use it. That is exactly the rule decided here, so the shared functions are thin wrappers over a `Dir` on the project root plus the classification that produces a readable refusal.

Two things `Dir` does not give by itself:

- A dangling link. Opening for write with create would follow an inside-pointing dangling link and create its target. An existing target is therefore opened without the create flag, and a missing one is created with an exclusive create, which never follows a link at the last component.
- A readable reason. `Dir` reports an escape as a bare permission error. The functions look at the path first to say which path it is and why, and the `Dir` operation is what enforces it.

Write sites on base `7b6e4779`, all plain path writes:

| Module | Site |
|---|---|
| `hooks` | `install_markdown_snippet`, `install_claude_code_hook`, `remove_section_from_file` |
| `agents` | `write_generated_agent_artifact`, `write_agent_artifact_manifest`, the inspect reads and the uninstall removals |
| `mcp` | `cmd_lock_write` in `src/mcp_tools_lock.rs` |
| `registry` | `register_module` |
| `cmd_resolve` | `SpecCache::get` and `set` |
| `cmd_migrate` | backup copies and manifest, directories, `config.toml`, registry move, lifecycle files, frontmatter rewrite, `.specsync/.gitignore`, `cross-project-refs.json`, `version`, removal of the old config files |
| `generator` | the ten companion writes behind `generate_companion_files_for_spec` and `generate_companion_files_from_template` |
| `cmd_scaffold`, `cmd_new`, `cmd_wizard`, `cmd_import` | the spec file write after `exists()` |

Already guarded and left alone: the pre-commit hook, the root `.gitignore` and `.gitattributes`, `init`, the change lifecycle, `compact`, `archive-tasks`, generation, MCP publication.
