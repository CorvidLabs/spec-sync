## MODIFIED

### SPEC SECTION Dependencies

**Consumes**

| Module | What is used |
|--------|-------------|
| config | `detect_source_dirs`, `config_to_toml` |

**Consumed By**

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync init` |

**Frontmatter Synchronization**

Implementation SHALL add these canonical dependency specs to `depends_on`: `specs/types/types.spec.md`. This YAML frontmatter update is an explicit implementation edit because semantic section deltas do not apply frontmatter.
