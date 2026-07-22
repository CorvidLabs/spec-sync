---
spec: mcp.spec.md
---

## Regression Matrix

| Case | Required Result |
|------|-----------------|
| Initialize/tools/resources | Valid JSON-RPC metadata |
| Default and write-enabled tool lists | Five read tools by default; two mutators added only with `--allow-write`; all schemas are exact |
| Direct mutator call in default mode | Tool error before execution; filesystem unchanged |
| Authorized mutator | Operates only at the canonical server root and rejects `root` with -32602 |
| Existing child read root | Allowed after canonicalization |
| Outside, nonexistent, traversing, or symlink read root | Rejected; outside victim bytes remain identical |
| Traversing/absolute configured path or unsafe module name | Rejected before project discovery or generation; outside bytes remain identical |
| Configured or nested symlink escape | Rejected for reads and writes, including a dangling init destination |
| Escaping spec frontmatter file mapping | Rejected before list/check/score consumers can read it |
| Manifest workspace/member, Gradle module, Python package, cache, dependency, or metadata escape | Rejected before autodetection or downstream consumers access it; outside bytes remain identical |
| No-config source scan symlink escape | Rejected by a four-level, ignore-aware, bounded preflight |
| Excluded subtree contains an outside symlink | Exclusion is honored; unrelated subtree does not reject the request |
| Wrong argument type, unknown key, or malformed `tools/call` params | JSON-RPC -32602 before execution |
| Generate | Deterministic local scaffold |
| Legacy inference argument | JSON-RPC -32602 migration error, value not echoed |
| Notification, including mutating and unknown methods | No response and no dispatch |
| Parse error | JSON-RPC -32700 with null ID |
| Input line over 1 MiB followed by a valid request | Oversized line is rejected and drained; following request succeeds |
| Duplicate/overlapping configured tree scans | One cumulative 100,000-entry confinement budget is enforced |
| Missing/unresolvable server root | Exit 2 with an actionable stderr diagnostic |
| EOF | Graceful exit |

The adversarial integration matrix lives in `tests/integration/mcp.rs`; its local write-enabled
process helper leaves the shared `mcp_request` helper and all existing read-only callers unchanged.
The focused MCP suite contains 30 integration tests and 44 MCP unit tests, including explicit
configuration, cycle/bound, cumulative-budget, bounded-input, and built-in-ignore branches.
