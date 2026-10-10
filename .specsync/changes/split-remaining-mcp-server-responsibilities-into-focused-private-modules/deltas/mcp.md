## MODIFIED

### SPEC SECTION Internal Organization

The MCP server facade preserves its existing public entry points and delegates to private responsibility modules for protocol/catalog, resources, tool operations, retained capabilities, read-root selection, atomic publication, shared manifest-path interpretation, confinement, and snapshot construction. Snapshot collection, retained input reading and traversal are private children sharing one budget owner. The complete publication transaction retains a single ownership boundary. Debug integration barriers remain available under `cfg(debug_assertions)`.

Issue verification remains private, with bounded content-free diagnostics and fail-closed checked discovery. Owner-specific unit tests reside alongside their implementation modules and share reusable fixtures. Files target fewer than 1000 lines where cohesive boundaries allow; no include-based textual shards or duplicate implementations are introduced.

This organization does not expand the public API or change CLI behavior, MCP JSON/catalog, diagnostics, defaults, limits, authentication, serialization, call ordering, or retained filesystem authority. Platform and test cfg guards remain unchanged.
