## MODIFIED

### SPEC SECTION Dependencies

| Module | What is used |
|--------|-------------|
| change | Lifecycle operations and projections |
| types | Output format |

**Frontmatter Synchronization**

Implementation SHALL add `specs/cli_args/cli_args.spec.md` to `depends_on`. Rust source-module ownership maps
`crate::cli::ChangeAction` to the `cli_args` contract rather than the top-level `cli` executable contract.
