## ADDED

### SPEC SECTION Integration Test Organization

The registered `tests/integration/commands.rs` module is a small entry point for private topical command-regression modules beneath `tests/integration/commands/`. Related tests share fixtures at their nearest common owner. Every existing test name, assertion, fixture behavior, subprocess filter and platform/debug guard is preserved; only module-qualified test paths change. The root integration harness registration remains intact, and compiled inventory comparisons ensure no relocated regression is omitted. This internal organization changes no CLI behavior or production API.
