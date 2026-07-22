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
| Existing child read root | Allowed only through the retained root capability, including after ambient root replacement |
| Outside, nonexistent, traversing, or symlink read root | Rejected; outside victim bytes remain identical |
| Traversing/absolute configured path or unsafe module name | Rejected before project discovery or generation; outside bytes remain identical |
| Configured or nested symlink escape | Rejected for reads and writes, including a dangling init destination |
| Escaping spec frontmatter file mapping | Rejected before list/check/score consumers can read it |
| Manifest workspace/member, Gradle module, Python package, cache, dependency, or metadata escape | Rejected before autodetection or downstream consumers access it; outside bytes remain identical |
| No-config source scan symlink escape | Rejected by a four-level, ignore-aware, bounded preflight |
| Excluded subtree contains an outside symlink | Exclusion is honored; unrelated subtree does not reject the request |
| Wrong argument type, unknown key, or malformed `tools/call` params | JSON-RPC -32602 before execution |
| Invalid JSON-RPC version, method, ID, params, or top-level container | JSON-RPC -32600 before dispatch; mutator does not run |
| Malformed or extended `resources/read` params | JSON-RPC -32602 before resource access |
| Generate | Deterministic local scaffold |
| Generate destination collision, public-parent replacement, or incomplete write | Tool error; retained capabilities preserve public replacements; empty parents created by the failed batch may remain |
| Generate count over 1,000, cumulative content over 64 MiB, or oversized result | Rejected before any destination is published |
| Legacy inference argument | JSON-RPC -32602 migration error, value not echoed |
| Notification, including mutating and unknown methods | No response and no dispatch |
| Parse error | JSON-RPC -32700 with null ID |
| Input line over 1 MiB followed by a valid request | Oversized line is rejected and drained; following request succeeds |
| Response over 1 MiB | Replaced by a bounded -32603 response; request IDs over 4 KiB are rejected with -32600 before dispatch |
| Project file over 8 MiB or configuration plus inputs over 64 MiB | Rejected before downstream parsing using actual copied bytes |
| Explicitly configured normally ignored source root | Included in the bounded snapshot and validated |
| Ignored or configured-exclusion basename is a symlink | Skipped before following target metadata unless an explicit configured input names it or a descendant; broad ancestor inputs do not override `excludeDirs` |
| `sourceDirs: ["."]` or Cargo/Gradle manifest member beneath an ignored directory | Included and copied from exact bytes charged to the same bounded snapshot budget |
| Commented `[workspace]` text or multiline Cargo workspace members beneath an ignored directory | Parsed as TOML, included only when real, and bounded; cannot produce 0/0 false-green coverage |
| Commented, escaped, malformed, or partial Gradle settings | Parsed through the shared checked parser; malformed discovery is inconclusive for every coverage consumer |
| Manifest discovery crosses 64 MiB before snapshot copying | Rejected by the shared cumulative operation budget |
| Manifest grows after discovery | Snapshot retains only the charged preflight buffer and cannot exceed the cumulative budget |
| Python package name beneath an ignored directory | Manifest-derived package is included and charged to the snapshot budget |
| Root path replaced after the initial handle but before canonicalization/reopen | Startup fails; no outside capability is retained |
| Duplicate/overlapping configured tree scans | One cumulative 100,000-entry confinement budget is enforced |
| Redirected `.git` metadata without explicit `github.repo` | Rejected without Git auto-detection or outside metadata disclosure |
| MCP score in a Git-backed project snapshot | Reports Git freshness unavailable and withholds five freshness points |
| Transport reader or writer failure | Returned as an error; never reported as a successful server exit |
| Windows junction/reparse-point read, generate, or init destination | Rejected on Windows CI; outside victim bytes remain exact and no staging debris remains |
| GitHub inaccessible repository, post-404 access loss, timeout, or malformed API output | In-process REST access returns an inconclusive error, not successful zero/not-found counts; no provider subprocess exists |
| Duplicate issue IDs across specs or more than 100 unique IDs | IDs are globally deduplicated; over-limit batches fail before provider access |
| Public-entry replacement before quarantine during staging, publication, or file rollback | Atomic quarantine preserves the replacement and rejects the batch |
| Same-user process races a private staging or quarantine name | Outside the MCP caller/path-confinement threat boundary; deployments must isolate server-root mutation |
| Drive, extended-drive, and UNC Windows roots with case differences | Normalize to one identity and derive only confined relative suffixes |
| Non-ASCII Windows root component with a case difference | Compare through native ordinal ignore-case semantics without lossy UTF-8 conversion |
| Missing/unresolvable server root | Exit 2 with an actionable stderr diagnostic |
| EOF | Graceful exit |

The adversarial integration matrix lives in `tests/integration/mcp.rs`; its local write-enabled
process helper leaves the shared `mcp_request` helper and all existing read-only callers unchanged.
The focused MCP suite contains 44 non-Windows integration tests and 78 MCP unit tests, including
exact envelope and resource validation, explicit Git repository configuration, generation-failure reporting,
Windows read/write junctions, identity-bound capability acquisition, cycle/bound and actual-byte budgets, bounded
input/output, configured-ignore exceptions, conservative Git scoring, rollback, and transport
failure branches. `generation_rejects_cumulative_output_bytes_before_publication` proves the exact
64 MiB cumulative boundary before publication, and
`generation_rejects_an_oversized_result_during_response_preflight` proves the final response-size
preflight. Windows-only generate/init junction cases compile in the cross-target lane and run
on Windows CI.
