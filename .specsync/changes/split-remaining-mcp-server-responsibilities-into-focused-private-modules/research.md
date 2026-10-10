---
change: split-remaining-mcp-server-responsibilities-into-focused-private-modules
artifact: research
---

# Research

The MCP server has natural responsibility boundaries; all proposed production clusters fit below 800 lines before imports. The current remaining inline test block is roughly 2936 lines and requires topical files as well. A read-only architectural review mapped concrete function groups before implementation.

The directory opener named open_snapshot_configuration_directory is shared by snapshot configuration, read-root authorization and publication parent revalidation; it therefore belongs in common retained capability primitives. Pure manifest-path interpretation is also shared by snapshot and confinement and must not create a circular dependency. The complete publication state machine fits below 1000 lines once the interleaved read-root type is separated, so splitting the transaction further would harm ownership clarity.

Debug race barriers must remain cfg(debug_assertions), not cfg(test), because integration tests exercise the CLI binary. Rust platform cfg branches and test-only imports require explicit review. The first independent review caught exactly this kind of Linux-only import mismatch and it was fixed before publication.

Active PRs #804 and #800 overlap change.rs/change_tests.rs and remain outside this scope. The unrelated oversized validator, manifest and integration-command files remain later candidates.
