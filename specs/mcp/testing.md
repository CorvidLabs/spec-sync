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
| Cargo dependency path `../sibling` or `..\sibling` normalizes inside the server root | Accepted and snapshotted; drive, UNC, rooted, and traversal escapes are rejected |
| Cargo package metadata contains an unrelated `path` key | Ignored for input discovery; semantic target/dependency paths are still snapshotted |
| No-config source scan symlink escape | Rejected by a four-level, ignore-aware, bounded preflight |
| Excluded subtree contains an outside symlink | Exclusion is honored; unrelated subtree does not reject the request |
| Wrong argument type, unknown key, or malformed `tools/call` params | JSON-RPC -32602 before execution |
| Invalid JSON-RPC version, method, ID, params, or top-level container | JSON-RPC -32600 before dispatch; mutator does not run |
| Malformed or extended `resources/read` params | JSON-RPC -32602 before resource access |
| Generate | Deterministic local scaffold |
| Generate destination collision, public-parent replacement, or incomplete write | Tool error; retained capabilities preserve public replacements; empty parents created by the failed batch may remain |
| Successful init/generate and collision rollback on Windows | Quarantine cleanup consumes its final handle; success remains success and collision errors retain their intended publication diagnostic |
| Generate count over 1,000, cumulative content over 64 MiB, or oversized result | Rejected before any destination is published |
| Legacy inference argument | JSON-RPC -32602 migration error, value not echoed |
| Notification, including mutating and unknown methods | No response and no dispatch |
| Parse error | JSON-RPC -32700 with null ID |
| Input line over 1 MiB followed by a valid request | Oversized line is rejected and drained; following request succeeds |
| Response over 1 MiB | Replaced by a bounded -32603 response; request IDs over 4 KiB are rejected with -32600 before dispatch |
| Project file over 8 MiB or configuration plus inputs over 64 MiB | Rejected before downstream parsing using actual copied bytes |
| Selected config is malformed, invalid UTF-8, or has wrong-typed specs/source selectors | Every allow-empty tool/resource returns an error before compatibility fallback; valid BOM-prefixed JSON/TOML remains supported |
| Explicitly configured normally ignored source root | Included in the bounded snapshot and validated |
| Ignored or configured-exclusion basename is a symlink | Skipped before following target metadata unless an explicit configured input names it or a descendant; broad ancestor inputs do not override `excludeDirs` |
| `sourceDirs: ["."]` or Cargo/Gradle manifest member beneath an ignored directory | Included and copied from exact bytes charged to the same bounded snapshot budget |
| Commented `[workspace]` text or multiline Cargo workspace members beneath an ignored directory | Parsed as TOML, included only when real, and bounded; cannot produce 0/0 false-green coverage |
| Duplicate Cargo members or Node workspace patterns | Every declaration is charged, normalized completed nodes are reused, and limit-plus-one fails without repeated subtree parsing |
| Commented, escaped, malformed, or partial Gradle settings | Parsed through the shared checked parser; malformed discovery is inconclusive for every coverage consumer |
| Manifest discovery crosses 64 MiB before snapshot copying | Rejected by the shared cumulative operation budget |
| Manifest grows after discovery | Snapshot retains only the charged preflight buffer and cannot exceed the cumulative budget |
| Python package name beneath an ignored directory | Manifest-derived package is included and charged to the snapshot budget |
| Root path replaced after the initial handle but before canonicalization/reopen | Startup fails; no outside capability is retained |
| Duplicate/overlapping configured tree scans | One cumulative 100,000-entry confinement budget is enforced |
| Redirected `.git` metadata without explicit `github.repo` | Rejected without Git auto-detection or outside metadata disclosure |
| FIFO/device replaces a recognized manifest or selected config after discovery | Explicit no-follow, non-blocking retained-handle acquisition rejects it without waiting |
| Selected config or manifest path changes after the retained handle is opened | Path-to-handle identity comparison rejects replacement bytes on Windows and Unix |
| MCP score in a Git-backed project snapshot | Reports Git freshness unavailable and withholds five freshness points |
| Transport reader or writer failure | Returned as an error; never reported as a successful server exit |
| Windows junction/reparse-point read, generate, or init destination | Native-join fixture proves the reparse target, then accepts rejection during either capability snapshot traversal or destination publication confinement; outside victim bytes remain exact and no staging debris remains |
| Windows absolute child read root | A valid child project reaches coverage and reports 1/1 files when the child uses either identity-bound startup spelling; sibling-prefix, rooted, and drive-relative lookalikes fail for the intended root-validation reason |
| GitHub inaccessible repository, post-404 access loss, timeout, or malformed API output | In-process REST access returns an inconclusive error, not successful zero/not-found counts; no provider subprocess exists |
| Duplicate issue IDs across specs or more than 100 unique IDs | IDs are globally deduplicated; over-limit batches fail before provider access |
| MCP issue scan encounters unreadable bytes or malformed/missing frontmatter | Entire issue result is inconclusive with an attributed path; no zero-reference success |
| MCP issue scan encounters a wrong `implements`/`tracks` shape or traversal error | Entire issue result is inconclusive; invalid IDs or undiscovered entries cannot be silently dropped |
| MCP issue scan encounters duplicate keys or malformed YAML anywhere | Entire issue result is inconclusive with a stable content-free reason |
| MCP issue YAML contains comments/trailing commas plus nested extension or block-scalar lookalikes | Valid top-level positive unsigned lists are accepted; nested/text lookalikes are ignored |
| MCP spec read fails beneath a host-absolute root | Diagnostic contains only a sanitized relative path and content-free reason; no root, OS detail, or spec bytes |
| MCP issue finding contains a literal Unix backslash | Unix preserves the backslash as filename data; Windows alone normalizes separators to `/`, so no nested-path identity collision is introduced |
| Public-entry replacement before quarantine during staging, publication, or file rollback | Atomic quarantine preserves the replacement and rejects the batch |
| Replacement reuses the same Unix inode or rewrites the same filesystem entry | Exact-byte identity rejects and preserves the replacement instead of trusting inode identity alone; hashing fails closed above the 64 MiB output bound |
| Same-user process races a private staging or quarantine name | Outside the MCP caller/path-confinement threat boundary; deployments must isolate server-root mutation |
| Drive, extended-drive, and UNC Windows roots with case differences | Normalize to one identity and derive only confined relative suffixes |
| Non-ASCII Windows root component with a case difference | Compare through native ordinal ignore-case semantics without lossy UTF-8 conversion |
| Missing/unresolvable server root | Exit 2 with an actionable stderr diagnostic |
| EOF | Graceful exit |

Unix FIFO assertions always run. Socket assertions run when the host permits `UnixListener`
fixture creation; a host-level `PermissionDenied` skips only that unavailable socket fixture, while
all other acquisition and replacement checks remain mandatory.

The adversarial integration matrix lives in `tests/integration/mcp.rs`; its local write-enabled
process helper leaves the shared `mcp_request` helper and all existing read-only callers unchanged.
Focused MCP source and integration coverage includes exact envelope and resource validation,
explicit Git repository configuration, generation-failure reporting,
Windows read/write junctions, identity-bound capability acquisition, cycle/bound and actual-byte budgets, bounded
input/output, configured-ignore exceptions, conservative Git scoring, rollback, and transport
failure branches. Exact-head remediation covers Cargo/Node duplicate-chain expansion,
declaration charging, injectable limit/limit-plus-one behavior, and retained zero-config detection.
The reported targeted runs passed 111 MCP unit tests and 62 MCP integration tests, including the
end-to-end duplicate Cargo/Node workspace case. The full post-remediation suite passed 1,930 unit
plus 307 integration tests. Fresh independent rereview and hosted-Windows junction/reparse runtime
remain pending.
`snapshot_ignores_nonsemantic_cargo_metadata_paths`,
`snapshot_normalizes_confined_windows_native_cargo_paths`,
`issue_tool_fails_inconclusive_for_malformed_frontmatter`, and
`issue_tool_fails_inconclusive_for_unreadable_spec_text` cover the independent-review follow-up.
`issue_tool_fails_inconclusive_for_malformed_known_issue_fields`,
`issue_reference_field_validation_accepts_supported_list_forms`, and
`issue_reference_field_validation_ignores_nested_extensions_and_block_scalars`,
`issue_spec_file_name_rejects_non_utf8_spec_suffix`,
`issue_tool_rejects_non_utf8_spec_filename_after_snapshot_copy`, and
`issue_read_diagnostics_are_bounded_relative_and_content_free` cover strict top-level shapes,
checked real-YAML behavior, lossy-name discovery, and diagnostic redaction. Final source-worker
counts are intentionally not recorded here while the tree remains active; fresh Windows runtime
and final repository/trust/provenance/CI evidence remain pending.
`generation_rejects_cumulative_output_bytes_before_publication` proves the exact
64 MiB cumulative boundary before publication, and
`generation_rejects_an_oversized_result_during_response_preflight` proves the final response-size
preflight. Windows-only generate/init junction cases compile in the cross-target lane and run
on Windows CI.
`read_root_suffix_accepts_the_identity_bound_startup_alias_only_as_a_prefix` and
`mcp_windows_read_roots_accept_absolute_children_and_reject_ambiguous_prefixes` cover canonical
8.3 expansion, original-spelling absolute children, and sibling-prefix rejection.
The `mcp_allow_empty_tool_and_resource_*selected_config*` integration group covers malformed,
invalid-UTF-8, wrong-typed, and valid BOM-prefixed selected config behavior across tools/resources.
