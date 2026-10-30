# mcp delta

## MODIFIED

### Spec Section Purpose

Model Context Protocol (MCP) server for AI agent integration. Implements JSON-RPC 2.0 over stdio, exposing spec-sync functionality as tools callable from Claude Code, Cursor, Windsurf, and other MCP-compatible agents.

The implementation uses private protocol, tools, confinement, preflight, snapshot, snapshot_manifest and publication modules, with shared fixtures compiled only for tests. The facade retains the existing server and tool-catalog entry points.
