---
change: CHG-0063-close-independent-mcp-security-review-gaps-for-issue-414
artifact: testing
---

# Testing

## Required regression matrix

| Case | Expected result |
|------|-----------------|
| Existing and nonexistent absolute outside roots | Same pre-canonicalization error; no path disclosure |
| Redirected `.git` file or symlink | Explicit-repository/confinement error; outside metadata absent |
| Mixed-case `.GIT` configuration | Rejected on every platform; metadata is never snapshotted |
| Unix symlink or Windows junction escape | Rejected; referent bytes remain exact |
| Missing/wrong JSON-RPC envelope members | `-32600`; mutator never executes |
| Invalid/extended `resources/read` params | `-32602`; no resource access |
| Project file >8 MiB or actual inputs/config >64 MiB | Tool/resource error before parsing |
| Explicit normally ignored root | Included when configured and still bounded/confined |
| Root-wide source or manifest-derived workspace beneath ignored name | Included and budgeted; commented Cargo headers and multiline/commented/escaped Gradle members cannot be omitted |
| Manifest grows after discovery | Snapshot copies only the exact bytes charged during discovery and remains within 64 MiB |
| Cargo dependency path `../sibling` normalizes inside the server root | Accepted and included; a path whose normalized target escapes the root remains rejected |
| Windows quarantine cleanup after init, generation, or collision rollback | Final directory capability is consumed before removal; no sharing violation changes the outcome |
| Manifest discovery exceeds the 64 MiB cumulative budget | Rejected before parsing further manifests or snapshot copying |
| Root replaced after the initial handle but before canonicalization | Identity/capability error before serving requests |
| Response >1 MiB or request ID >4 KiB | Compact bounded error; oversized ID is rejected before dispatch |
| Public staged/destination entry swapped before quarantine during publication or rollback | `isError`; current entry is atomically quarantined and verified; replacements are preserved |
| Existing destination, replaced nested parent, or later multi-file write failure | `isError`; publication/rollback stay on retained parent capabilities and preserve public replacements; empty created parents may remain |
| Same-user process mutates private transaction names | Outside the MCP caller/path boundary; deployment must isolate server-root mutation |
| Generation over 1,000 specs, 64 MiB, or response budget | Rejected before destination publication |
| Windows generate/init junction destination | Rejected; outside bytes remain exact and no stage file escapes |
| Python package path beneath a normally ignored name | Manifest-derived source remains present and budgeted |
| GitHub inaccessible repository, auth, transport, timeout, or malformed provider | Inconclusive tool error; never successful zero/not-found counts |
| GitHub read/list/verify path is invoked | Uses explicit-token in-process REST; no provider subprocess or descendant exists |
| GitHub batch import spans pages or encounters malformed/duplicate/cap-truncated pagination | Every valid page is returned in order; ambiguity or truncation is an error |
| Single GitHub import is invoked | Uses explicit-token shared typed REST details; no `gh issue view` subprocess exists |
| Issue lookup returns 404 after repository access changes | Access is rechecked inside the operation/batch deadline; inaccessible is inconclusive, still-accessible is not_found |
| Duplicate IDs across specs or more than 100 unique IDs | Globally deduplicated; over-limit batch fails before provider access |
| All references fail verification | Text summary reports the error count and does not print no-reference guidance |
| No issue references, with or without `github.repo` | No-reference guidance without repository/provider resolution |
| Windows ordinary, extended-drive, UNC, ASCII/non-ASCII case-varied confined root | Same normalized relative suffix through native ordinal comparison; confined root accepted |
| Valid or malformed settings-only Gradle workspace | Modules discovered or checked discovery fails closed without a root build script |
| Ignored/configured-exclusion name is a symlink | Skipped before target metadata is followed unless explicitly configured |
| Valid notification | No output and no dispatch |

Focused evidence lives in `src/mcp.rs`, `src/manifest.rs`, `src/github.rs`, `src/importer.rs`,
`src/commands/issues.rs`, `tests/integration/commands.rs`, and `tests/integration/mcp.rs`. The
current inventory includes 83 MCP unit tests, 23 GitHub unit tests, 14 manifest unit tests, 26
importer unit tests, focused command unit/integration regressions, and 44 non-Windows MCP
integration tests. The final post-defensive-review repository evidence passes 1,792 unit tests and
260 integration tests, including both no-subprocess amendments and the refined configured-input
precedence. The amended tree also passes
formatting, linting, type checking, and Windows GNU test cross-target checking; repository evidence
includes a release build, Astro diagnostics, 23 documentation tests, the 43-page site build,
RustSec audit, and VS Code extension
compile/package. A local replay against a clean
clone of the private `CorvidLabs/spec-sync-sandbox` repository confirms read-only tool listing,
write opt-in, write-root immutability, traversal/absolute/symlink rejection, exact argument typing,
notification silence, and an unchanged clone. Strict coverage passes at 100% across 105 source
files and 93,197 LOC, and all 62 canonical specs score 100. Both independent acceptance and
defensive compatibility rereviews are clean. Remaining evidence is the complete repository lane,
strict trust verification, Attest provenance, and GitHub CI including Windows runtime coverage.

- `REQ-mcp-002`: `mcp::tests::test_repeated_tree_scans_share_one_confinement_budget`,
  `mcp::tests::snapshot_copies_the_exact_manifest_bytes_charged_during_discovery`,
  `mcp::tests::snapshot_includes_all_standard_gradle_module_forms_under_ignored_directories`,
  `mcp::mcp_absolute_outside_roots_do_not_disclose_existence`,
  `mcp::mcp_issues_requires_explicit_repo_before_redirected_git_metadata`,
  `mcp::mcp_issues_rejects_a_git_symlink_to_outside_metadata`,
  `mcp::mcp_issues_without_references_skips_repository_resolution`,
  `mcp::mcp_read_root_rejects_symlink_escape_and_preserves_referent`,
  `mcp::mcp_read_root_rejects_windows_junction_escape_and_preserves_referent`, and
  `mcp::mcp_bounds_per_file_and_cumulative_project_inputs` prove lexical authorization,
  metadata/link confinement, case-insensitive Git exclusion, and actual-byte budgets.
- `REQ-mcp-003`: `mcp::tests::test_invalid_json_rpc_envelopes_fail_before_dispatch`,
  `mcp::tests::generation_publication_and_rollback_remain_bound_to_a_replaced_parent`,
  `mcp::tests::staging_detects_a_same_entry_replacement_after_identity_capture`,
  `mcp::tests::publication_quarantines_a_replacement_swapped_after_verification`,
  `mcp::tests::file_rollback_quarantines_a_replacement_swapped_after_verification`,
  `mcp::tests::test_resources_read_rejects_non_exact_arguments`,
  `mcp::tests::generation_rejects_cumulative_output_bytes_before_publication`,
  `mcp::tests::generation_rejects_an_oversized_result_during_response_preflight`,
  `mcp::tests::test_oversized_response_is_replaced_with_a_bounded_error`,
  `mcp::mcp_invalid_json_rpc_envelopes_return_invalid_request_without_mutating`,
  `mcp::mcp_resources_read_requires_exact_params_and_confines_direct_uris`,
  `mcp::mcp_bounds_outbound_resource_responses`, and
  `mcp::mcp_generate_reports_destination_collisions_and_io_failures_as_tool_errors` prove
  fail-closed protocol validation, bounded output/IDs, and transactional generation integrity.
- `REQ-github-001`: `github::tests::malformed_issue_provider_output_fails_closed`,
  `github::tests::batch_prepares_once_deduplicates_fetches_and_attributes_per_spec`,
  `github::tests::batch_prepare_auth_and_repository_errors_are_inconclusive_for_every_spec`,
  `github::tests::batch_attributes_malformed_and_transport_provider_errors`,
  `github::tests::batch_full_deadline_includes_prepare_and_stops_before_fetch`,
  `github::tests::batch_cap_is_enforced_before_provider_prepare`,
  `github::tests::issue_list_pagination_collects_every_page_in_order`,
  `github::tests::issue_list_pagination_fails_instead_of_truncating_or_deduplicating`,
  `github::tests::link_header_parsing_detects_next_and_rejects_malformed_values`,
  `github::tests::link_header_rejects_wrong_repository_or_resource_path`,
  `github::tests::link_header_rejects_query_mismatch`,
  `github::tests::provider_process_construction_is_absent_from_every_read_path`,
  `github::tests::token_present_read_list_verify_and_import_paths_never_spawn_gh`,
  `github::tests::rest_not_found_is_confirmed_when_repository_recheck_succeeds`,
  `github::tests::rest_not_found_is_inconclusive_when_repository_recheck_fails`,
  and `github::tests::gh_issue_reads_fail_closed_without_spawning_a_provider` prove typed
  fail-closed classification, strict parsing, global bounds, and absence of read-provider
  subprocess execution. The source-boundary test forbids `gh` process construction in read,
  list, verify, importer, command, and MCP modules on every platform. The Unix token-present
  sentinel additionally exercises the real entry paths against an isolated unreachable local REST
  endpoint and fails if a PATH-injected `gh` executable runs.
- `REQ-cmd-issues-001`:
  `mcp::tests::issue_tool_enforces_one_deduplicated_invocation_cap_across_specs` proves references
  are gathered across specs and rejected at the project-wide cap before provider selection; the
  GitHub batch unit regressions prove deterministic per-spec attribution after deduplication, and
  `commands::issues::tests::all_error_batches_report_errors_instead_of_no_reference_guidance`
  proves truthful text summaries. `issues_without_references_does_not_require_repository_configuration`
  and `issues_without_references_preserves_configured_repository_outputs` prove no-reference
  ordering for both configured and unconfigured projects;
  `issues_reference_batch_fails_closed_without_a_rest_token` proves a command-level provider
  failure is non-zero and attributed to the referencing spec.
- `REQ-importer-001`:
  `importer::tests::test_import_github_issue_entry_path_converts_shared_typed_details`,
  `importer::tests::test_import_github_issue_entry_path_returns_no_item_on_provider_failure`,
  `importer::tests::test_import_github_issue_details_full`, and
  `importer::tests::test_import_github_issue_details_empty_body` prove the importer entry seam,
  failure non-production, and complete typed detail conversion while the shared GitHub regressions
  prove explicit-token, no-subprocess, timeout, and 404 revalidation behavior.
- `REQ-cmd-import-001`: `github::tests::issue_list_pagination_collects_every_page_in_order`,
  `github::tests::issue_list_pagination_fails_instead_of_truncating_or_deduplicating`, and
  `github::tests::link_header_parsing_detects_next_and_rejects_malformed_values` prove complete
  bounded batch traversal and fail-closed partial-list handling;
  `single_github_import_fails_closed_without_a_rest_token_or_output` and
  `batch_github_import_fails_closed_without_a_rest_token_or_output` prove both real CLI entry paths
  fail before creating output when explicit REST authorization is absent.
- `REQ-manifest-001`:
  `manifest::tests::gradle_settings_support_groovy_kotlin_multiline_and_project_dir_overrides`,
  `manifest::tests::gradle_settings_ignore_comments_and_decode_escaped_values`,
  `manifest::tests::gradle_manifest_discovery_fails_closed_for_malformed_settings`,
  `manifest::tests::gradle_manifest_discovery_rejects_dynamic_include_without_partial_modules`,
  `manifest::tests::gradle_settings_reject_unsupported_project_dir_bases_and_suffixes`, and
  `manifest::tests::gradle_manifest_discovery_accepts_comments_and_escaped_paths` prove shared,
  normalized, checked Gradle discovery.
- `REQ-cmd-check-001`, `REQ-cmd-comment-003`, `REQ-cmd-coverage-001`,
  `REQ-cmd-generate-001`, `REQ-cmd-report-001`, and `REQ-cmd-score-001`:
  `malformed_gradle_is_inconclusive_for_coverage_gating_commands` proves every command rejects
  malformed discovery, preserves parseable JSON where supported, and prevents mutation/reporting.
- `REQ-config-005`: `config::tests::checked_source_detection_surfaces_malformed_gradle_settings`
  proves checked errors and compatibility fallback separation.
- `REQ-validator-008`:
  `validator::tests::malformed_gradle_settings_make_coverage_inconclusive` proves checked coverage
  returns no partial report and the compatibility wrapper carries an inconclusive diagnostic.
