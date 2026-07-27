---
id: CHG-0064-add-capability-safe-filesystem-support-for-mcp-security-hardening
state: archived
type: refactor
base_commit: a0d993b7d10d177f9a4770f54fbe14045750221c
---

# Add capability-safe filesystem support for MCP security hardening

## Intent

Add capability-safe filesystem support for MCP security hardening

## Affected Canonical Specs

- None

## Acceptance Criteria

- cap-std is a direct runtime dependency; tempfile is available at runtime without duplication; the Rust lockfile is reproducible; native and Windows cross-target checks pass; MCP capability-snapshot and confined-write regression tests pass; no public CLI or spec contract changes are introduced by the dependency wiring.

## No-spec Rationale

Dependency wiring only; the MCP behavior and canonical contract are governed by approved CHG-0063.
