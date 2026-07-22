---
change: CHG-0062-harden-mcp-root-confinement-write-authorization-argument-validation-and-notif
artifact: testing
---

# Testing

## Required Regression Matrix

| Case | Expected result |
|------|-----------------|
| Default `tools/list` | Mutating tools are absent |
| Direct mutator call in default mode | Rejected before execution; filesystem unchanged |
| `mcp --allow-write` | Mutating tools are listed and operate at the server root |
| Existing child root on read tool | Allowed |
| Existing/nonexistent outside root | JSON-RPC/tool failure; victim remains unchanged |
| Symlink from root to outside | Rejected; referent remains unchanged |
| Configured path traversal or unsafe module name | Rejected before discovery/generation |
| Configured source/specs symlink escape | Rejected before read/write; referent remains unchanged |
| Dangling symlink at init destination | Rejected; outside target is not created |
| Escaping spec file mapping | Rejected before project data is returned |
| Manifest workspace parent/symlink escape | Rejected before recursive read or directory enumeration |
| Manifest escape with explicit config, with or without source dirs | Rejected before config loading and unconditional coverage rediscovery |
| Gradle/Python manifest-derived escape | Rejected before source-path existence probes |
| Metadata/cache/dependency escape | Rejected before downstream check/list/score consumers |
| No-config source autodetection symlink | Rejected by bounded four-level preflight |
| Configured exclusion contains outside symlink | Exclusion is honored; request remains compatible |
| Write tool with `root` override | Rejected before mutation |
| Unknown key, wrong type, non-object arguments | JSON-RPC `-32602` |
| Known or unknown request without `id` | No output line |
| Mutating notification | No output and no filesystem change |
| Parse error | JSON-RPC `-32700` with `id: null` |
| Input line over 1 MiB followed by a valid request | Rejected and drained; following request succeeds |
| Repeated/overlapping configured scans | One cumulative 100,000-entry budget is enforced |

Targeted coverage belongs in `tests/integration/mcp.rs`; existing initialize, resource, EOF, and
deterministic-generation tests remain compatibility controls. Final verification runs the targeted
integration binary, all Rust tests, strict spec validation, score, full repository lane, trust gate,
and independent agent reviews.

## Implementation Evidence

- `REQ-cli-005`: `mcp_help_documents_explicit_write_authorization` proves the dispatcher-facing
  capability surface, while `mcp_rejects_a_nonexistent_server_root` proves startup fails with status
  2 before reading a request.
- `REQ-cli-args-008`: `mcp_allow_write_lists_mutating_tools_with_exact_schemas` and
  `mcp_help_documents_explicit_write_authorization` prove the opt-in CLI capability and help.
- `REQ-mcp-002`: `mcp_tools_list_defaults_to_read_only_tools`,
  `mcp_read_only_rejects_direct_mutators_and_preserves_outside_victim`,
  `mcp_allow_write_uses_server_root_and_rejects_root_overrides`,
  `mcp_read_roots_allow_existing_children_and_reject_escapes`, and
  `mcp_read_root_rejects_symlink_escape_and_preserves_referent`,
  `mcp_rejects_configured_read_and_write_path_escapes`,
  `mcp_rejects_configured_symlink_trees_and_dangling_write_destinations`,
  `mcp_rejects_unsafe_module_names_and_spec_file_mappings`,
  `mcp_rejects_metadata_symlinks_and_traversing_dependency_references`,
  `mcp_manifest_autodetection_rejects_workspace_escapes`,
  `mcp_manifest_autodetection_rejects_gradle_and_python_path_escapes`, and
  `mcp_confinement_scan_honors_configured_excluded_directories` plus the cumulative-budget unit test
  prove capability separation and request-, configuration-, content-, manifest-, metadata-, and
  symlink-level confinement with bounded aggregate scans and exact outside-victim byte preservation.
- `REQ-mcp-003`: `mcp_tools_call_rejects_shape_type_and_unknown_key_errors` and
  `mcp_notifications_never_respond_or_mutate`, and
  `mcp_rejects_an_oversized_line_and_processes_the_next_request` prove exact -32602 validation,
  no-dispatch notification semantics, and bounded/draining input behavior.
- `cargo test mcp::tests::`: 44 passed, 0 failed after adding cumulative scan-budget and bounded
  line-reader tests.
- `cargo test --test integration mcp::`: 30 passed, 0 failed after adding explicit-config,
  cycle/bound, ignore, root-type, and oversized-line coverage.
- The post-explicit-config `fledge run test` result (1,956 passing tests) is green but was superseded
  by the final availability hardening; the definitive full suite must be rerun.
- `git diff --check`: passed after the final implementation and artifact refresh.
- Outside-root, write-root override, traversal, configured-path, unsafe-module, spec-mapping,
  nonexistent-root, symlink-escape, dangling-symlink, and mutating-notification tests assert exact
  victim byte preservation or non-creation.
- `fledge lanes run pre-commit`: passed after the final cumulative-budget/input-limit hardening.
- `cargo run -- score mcp cli_args`: both affected specs scored 100/100.
- Independent correctness and adversarial security rereviews report no remaining high, medium, or
  low findings after the final availability hardening.
- The earlier full repository and trust runs were superseded by review-driven changes and must be
  rerun after the updated definition receives human reapproval.
- Lifecycle closing approval, acceptance, archival, provenance, and GitHub CI remain pending.
