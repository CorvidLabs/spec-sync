## ADDED

### SPEC SECTION Internal Organization

Dependency analysis retains its public API paths and report types behind a small facade. Private owners separate graph construction and validation, the complete import scanner and source-ownership subsystem, and disclosure/report rendering. Existing tests are grouped beside their owners with shared fixture builders. The split preserves source-root guards, fail-loud source/spec errors, ImportDialect authority, Rust masking and attribution, JVM declaration-before-layout ownership including ambiguous and unresolved outcomes, cycle traversal, topological order and rendered output without adding public behavior.
