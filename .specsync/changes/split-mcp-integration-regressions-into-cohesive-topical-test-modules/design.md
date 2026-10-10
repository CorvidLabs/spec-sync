---
change: split-mcp-integration-regressions-into-cohesive-topical-test-modules
artifact: design
---

# Design

The existing integration root still owns mcp through tests/integration/mcp.rs. That facade owns private configuration, gradle, issues, limits, protocol, read_roots, snapshots, tools, and workspaces modules. Each child imports its parent's private fixture scope with use super::*; no helper becomes public. The facade keeps setup_minimal_mcp_project_at, coverage_request, and mcp_request_with_write because they serve multiple topics. Timeout, descriptor-limit, Gradle assertion, selected-config assertion, and generation-failure assertion helpers stay with their sole topical callers.

This is an internal test topology change. The canonical MCP spec continues to own the unchanged production paths; its test-organization section and testing/context companions describe the registered integration facade and children separately.
