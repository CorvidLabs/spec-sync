## MODIFIED

### SPEC SECTION Dependencies

**Consumes**

| Module | What is used |
|--------|-------------|
| agents | `cmd_install`, `cmd_uninstall`, `cmd_status`, `AgentTool` |
| cli_args | `AgentsAction` enum |

**Consumed By**

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync agents` |

**Frontmatter Synchronization**

Implementation SHALL add these canonical dependency specs to `depends_on`: `specs/cli/cli.spec.md`. This YAML frontmatter update is an explicit implementation edit because semantic section deltas do not apply frontmatter.
