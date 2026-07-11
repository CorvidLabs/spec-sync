## MODIFIED

### SPEC SECTION Public API

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_comment` | `root: &Path, pr: Option<u64>, _base: &str, strict: bool, enforcement: Option<types::EnforcementMode>, require_coverage: Option<usize>` | `()` | Generate check summary; post as PR comment if `--pr N` is set, otherwise print to stdout |

### SPEC SECTION Dependencies

**Consumes**

| Module | What is used |
|--------|-------------|
| commands | `load_and_discover`, `build_schema_columns` |
| comment | `render_check_comment` |
| github | `resolve_repo` |
| validator | `validate_spec`, `compute_coverage` |

**Consumed By**

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync comment` |

**Frontmatter Synchronization**

Implementation SHALL add these canonical dependency specs to `depends_on`: `specs/change/change.spec.md`, `specs/ignore/ignore.spec.md`, `specs/types/types.spec.md`. This YAML frontmatter update is an explicit implementation edit because semantic section deltas do not apply frontmatter.
