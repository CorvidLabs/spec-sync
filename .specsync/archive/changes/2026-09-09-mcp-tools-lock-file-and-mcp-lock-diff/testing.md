---
change: mcp-tools-lock-file-and-mcp-lock-diff
artifact: testing
---

# Testing

- Unit: `cargo test mcp_tools_lock` — match, description drift, schema drift,
  missing lock, removed tool, catalog identity, title omission.
- Smoke: `specsync mcp lock --write` then `specsync mcp diff` → exit 0; missing
  root → exit 1; corrupted sha → exit 1.
- Spec: `specsync check --strict mcp cli_args` → pass.

## Requirement evidence

| ID | Evidence |
|----|----------|
| REQ-mcp-009 | `match_returns_ok`, `missing_lock_returns_err_with_remedy`, `description_drift_returns_nonzero`, `schema_drift_via_sha_mismatch`, `removed_tool_detected`, `lock_uses_same_catalog_as_server`, `sha256_omits_null_title`, `canonical_json_sorts_keys_and_is_compact` |
| REQ-cli-args-018 | `lock_uses_same_catalog_as_server`, `match_returns_ok`, `missing_lock_returns_err_with_remedy` |
