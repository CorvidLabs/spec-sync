---
change: split-mcp-issue-verification-into-focused-private-modules
artifact: design
---

# Design

Issue verification is a cohesive leaf of the MCP adapter. Its handler discovers checked references, invokes the existing in-process GitHub verifier, rejects inconclusive results, and shapes the same JSON. Its private diagnostic and parsing helpers do not belong in the server's orchestration file.

The new private issues module depends on the existing parent load_confined_config, read_file_bounded and validate_spec_file_mappings helpers. These remain authoritative and unchanged. The caller imports only issues::tool_issues. No crate-wide visibility or public API is added. The module-local tests retain access to the parent ProjectSnapshot for the existing non-UTF-8 filename regression and share the existing fixtures through test-only pub(super) access.

No include!-based textual shards, duplicate implementations, new runtime dependencies or data migrations are introduced. Future independent seams include protocol/catalog, resources, manifest discovery, snapshots, confinement and atomic publication; they are outside this first patch.
