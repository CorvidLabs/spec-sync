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
| Cargo dependency path `../sibling` or `..\sibling` normalizes inside the server root | Accepted and included; drive, UNC, rooted, traversal, symlink, and junction escapes remain rejected |
| Cargo metadata contains an unrelated `path` key | Ignored for snapshot authority; semantic target/workspace/dependency paths remain included |
| Windows quarantine cleanup after init, generation, or collision rollback | Final directory capability is consumed before removal; no sharing violation changes the outcome |
| Manifest discovery exceeds the 64 MiB cumulative budget | Rejected before parsing further manifests or snapshot copying |
| Root replaced after the initial handle but before canonicalization | Identity/capability error before serving requests |
| Response >1 MiB or request ID >4 KiB | Compact bounded error; oversized ID is rejected before dispatch |
| Public staged/destination entry swapped before quarantine during publication or rollback | `isError`; current entry is atomically quarantined and verified; replacements are preserved |
| Existing destination, replaced nested parent, or later multi-file write failure | `isError`; publication/rollback stay on retained parent capabilities and preserve public replacements; empty created parents may remain |
| Same-user process mutates private transaction names | Outside the MCP caller/path boundary; deployment must isolate server-root mutation |
| Generation over 1,000 specs, 64 MiB, or response budget | Rejected before destination publication |
| Windows generate/init junction destination | Native-join fixture proves the outside reparse target, then accepts rejection at either capability snapshot traversal or generation-destination confinement; outside bytes remain exact and no stage file escapes |
| Windows absolute child read root | A valid one-file covered project reaches downstream coverage at 1/1 beneath either identity-bound startup spelling; sibling-prefix, rooted, and drive-relative lookalikes fail for root validation |
| Python package path beneath a normally ignored name | Manifest-derived source remains present and budgeted |
| GitHub inaccessible repository, auth, transport, timeout, or malformed provider | Inconclusive tool error; never successful zero/not-found counts |
| GitHub read/list/verify path is invoked | Uses explicit-token in-process REST; no provider subprocess or descendant exists |
| GitHub batch import spans pages or encounters malformed/duplicate/cap-truncated pagination | Every valid page is returned in order; ambiguity or truncation is an error |
| GitHub issue-list page contains exactly 100 or more than 100 raw entries | Exactly 100 is accepted before pull-request filtering; 101 fails before parsing any item |
| Single GitHub import is invoked | Uses explicit-token shared typed REST details; no `gh issue view` subprocess exists |
| Direct issue detail contains a pull-request marker | Rejected before importer conversion |
| GitHub issue title normalizes to an empty module slug | Import fails before constructing an item or output path |
| Issue lookup returns 404 after repository access changes | Access is rechecked inside the operation/batch deadline; inaccessible is inconclusive, still-accessible is not_found |
| Duplicate IDs across specs or more than 100 unique IDs | Globally deduplicated; over-limit batch fails before provider access |
| All references fail verification | Text summary reports the error count and does not print no-reference guidance |
| No issue references, with or without valid `github.repo` | No-reference guidance; configured syntax is checked, but Git auto-detection/provider access are skipped |
| Missing/empty specs with malformed configured `github.repo` | Repository syntax error before no-spec/no-reference success; no Git auto-detection or provider access |
| Selected project config is malformed, invalid UTF-8, or unreadable | Structured `<project-config>` finding, exit 1, and no default-path no-spec/no-reference success |
| Selected CLI config is linked, non-regular, replaced during read, over 4 MiB, or has wrong-shaped known TOML fields | Same content-free configuration finding; only exact same-handle retained bytes can configure the scan |
| Selected CLI config omits source directories and the ambient root is replaced | Source discovery remains bound to the original retained project capability and selects only the original tree |
| MCP selected config is linked/reparse-backed, non-regular, blocking, replaced, non-object JSON, malformed, invalid UTF-8, or has wrong-typed known fields | Every allow-empty tool/resource is inconclusive before compatibility fallback; FIFO acquisition returns within the regression timeout and valid BOM-prefixed config remains supported |
| Selected config or manifest path is replaced after retained open | No-follow opened-handle identity remains authoritative; later path observation mismatch fails on Windows and Unix |
| Recognized snapshot manifest/config becomes a FIFO or special file | Non-blocking no-follow acquisition rejects it without waiting or parsing bytes |
| Exact checked JSON has non-object `github` or non-string/non-null `github.repo` | Rejected before compatibility sentinel/default substitution |
| Direct issue detail contains object, null, or scalar `pull_request` marker | Every marker shape is rejected as a pull request |
| Imported title is reserved/non-portable/overlong | No output path is created; batch processing continues later valid items and exits 1 after its summary |
| Omitted-source scan sees an ignored-name symlink or recognized special manifest | Ignored entry is skipped before metadata; recognized special manifest is a structured inconclusive finding |
| Missing/empty specs or repository failure under JSON/Markdown/GitHub | One valid selected-format report is rendered; JSON parses and failure exits remain non-zero |
| Legacy JSON `github.repo` is a number, boolean, object, or list | Surrounding valid config remains, but repository resolution fails closed without Git auto-detection |
| CLI/MCP scan encounters unreadable or malformed/missing-frontmatter spec | Inconclusive with safe path attribution; no spec bytes and no successful zero-reference result |
| Valid checked issue frontmatter uses CRLF delimiters | Parsed identically to LF by parser, CLI issue inspection, and MCP issue inspection |
| `implements`/`tracks` has a wrong shape or invalid list member | Checked inspection is inconclusive; no invalid value is silently filtered into an empty reference set |
| Checked issue YAML has duplicate keys or malformed extension YAML | Complete parse is inconclusive with a stable content-free error |
| Checked issue YAML has comments/trailing commas and nested extension/block-scalar lookalikes | Valid top-level positive unsigned lists are accepted; nested/text lookalikes are ignored |
| Recursive spec discovery encounters a walker error | Checked discovery reports an inconclusive finding; no partial/empty success |
| Finding path contains controls, pipes, backticks, or line breaks | Every format remains parseable and content-free; text has no raw controls and Markdown/GitHub emits one escaped row with a valid code span |
| MCP issue finding contains a literal Unix backslash | Unix preserves the backslash as filename data; Windows alone normalizes separators to `/`, preventing diagnostic identity collisions |
| Finding path begins or ends with a backtick | Markdown/GitHub pads code-span content inside a longer delimiter |
| Finding/repository text contains bidi formatting or Unicode Zl/Zp separators | Every renderer escapes the characters; visual order and line/table structure cannot be injected |
| CLI spec path is replaced by a symlink, regular file, or hardlink after discovery | Identity remains binding through read; replacement bytes are rejected and never parsed |
| CLI spec discovery exceeds 4 MiB per spec, 64 MiB retained bytes, or 10,000 specs | Bounded findings make inspection inconclusive; no unbounded snapshots are retained |
| CLI recursive inventory exceeds 100,000 total entries, including non-spec files | Bounded finding before accumulation; no false empty/no-reference result |
| Ambient project root changes between spec and source collection | Both remain authorized by the original retained project capability |
| `issues --create` mapped-source retention exceeds 4 MiB per source or 64 MiB cumulative | Supplied source observations fail closed without ambient fallback |
| `issues --create` validates a stable snapshot with drift | Normal drift issue creation remains available through `validate_spec_content_with_sources` without reopening spec/source paths |
| Retained TypeScript source snapshot contains `export *` | Supplied-content extraction skips ambient wildcard resolution while preserving local exports |
| Drift creation receives hostile path, provider, URL, or validation text | Terminal diagnostics and GitHub title/body arguments contain sanitized text |
| MCP spec read fails under a host-absolute root | Error contains only a sanitized relative path and stable content-free reason; no root, raw OS detail, or spec bytes |
| GitHub issue-list item contains `pull_request: null` | Entire page fails as malformed provider data before filtering |
| Raw GitHub issue or PR is closed or has mismatched repo/resource/number URL | Entire page fails before PR filtering |
| Raw GitHub URL number has leading zeros | Entire item/page fails even when the parsed numeric value matches |
| Raw GitHub duplicate involves a filtered pull request within/across pages | Entire page/traversal fails; filtering cannot hide the duplicate identity |
| Windows ordinary, extended-drive, UNC, ASCII/non-ASCII case-varied confined root | Same normalized relative suffix through native ordinal comparison; confined root accepted |
| Valid or malformed settings-only Gradle workspace | Modules discovered or checked discovery fails closed without a root build script |
| Raw drive-qualified Gradle include or `project(...)` selector | Rejected before colon mapping; valid nested colon identities remain supported |
| Literal official `setProjectDir` forms | `file(<literal>)` and `new File(rootDir, <literal>)` are parsed/confined; dynamic or unsupported forms fail closed |
| Interpolated or encoded-dynamic Gradle string | `$name`, `${expression}`, and Unicode/octal-encoded dollars reject before partial discovery; escaped and Groovy single-quoted dollars remain literal |
| Linked, special, oversized, or invalid-UTF-8 Gradle build/settings manifest | Bounded retained-capability acquisition fails inconclusively without referent disclosure or partial discovery |
| Gradle-derived Unix symlink or Windows reparse component | Checked discovery is inconclusive before CLI/MCP source probing or traversal; outside sentinel bytes remain unchanged |
| Unsafe lower-precedence Gradle filename variant | Every present build/settings name is preflighted before precedence; a shadowed special/link/replacement/oversized input fails closed |
| Gradle manifest changes before/opened/after retained read | Native path and opened-handle identities must agree at every checkpoint on Unix and Windows |
| Unrelated Gradle control flow plus supported include | Unrelated logic remains valid; only governed indirect/conditional directives and invoked unsupported inclusion APIs fail closed |
| Coverage exceeds 8 MiB/file, 64 MiB total, 100,000 entries, or 256 components | Iterative retained traversal returns inconclusive before a partial percentage |
| Coverage source has invalid UTF-8 name/content or special/replaced entry | One retained project authority rejects it before partial file, LOC, or module totals |
| Generic MCP project file becomes FIFO/socket/link/regular replacement | Both tool and resource requests fail promptly without replacement bytes or partial results |
| Ignored/configured-exclusion name is a symlink | Skipped before target metadata is followed unless explicitly configured |
| Public rendered drift errors include overlapping paths and `": "` in a legal spec path | Longest exact discovered path receives the errors; public `Vec<String>` signatures remain unchanged |
| Valid notification | No output and no dispatch |

Focused evidence lives in `src/parser.rs`, `src/validator.rs`, `src/mcp.rs`, `src/manifest.rs`,
`src/github.rs`, `src/importer.rs`, `src/commands/issues.rs`, `tests/integration/commands.rs`, and
`tests/integration/mcp.rs`. Historical evidence before the final adversarial patch includes 1,854
unit tests, 275 integration tests, release compilation, RustSec audit, and focused Windows
cross-target checks. The current patch has focused green formatting, linting, 22 issue integration
tests, the MCP diagnostic regression, and its unit helper. Those results do not substitute for a
fresh exact-revision repository lane. The prior private `CorvidLabs/spec-sync-sandbox` replay,
coverage accounting, score results, independent PASS verdicts, trust verification, Attest
provenance, and hosted CI all predate the final patch and must be refreshed. Native Windows runtime
evidence remains pending.
The newest real-YAML, exact-snapshot validation, same-handle discovery, configured-repository,
bidi/Zl/Zp, raw GitHub item, checked-discovery/non-UTF-8, confined-`specs_dir`,
relative-diagnostic, and null-marker regressions are present in the active source tree. Fresh
definition reapproval, both independent rereviews, sandbox replay, Windows runtime, final repository
lane, trust verification, Attest provenance, and GitHub CI remain pending. The latest local focused
run passes formatting, lint, 27 `issues_` integrations, four MCP selected-config integrations, 87
config unit tests, and 14 config-filtered integrations; these are not substitutes for final-tree
full-lane evidence.
The subsequent adversarial pass found non-object JSON fallback and linked/special-file MCP config
acquisition. Both are now characterized and fixed. The latest focused run passes lint, installed
Windows GNU cross-target compilation, 100 MCP unit tests, and 52 MCP integration tests, including
non-object JSON, symlink, and bounded FIFO regressions. The earlier full lane passed 1,857 unit and
287 integration tests plus release build and strict 100% coverage, but it predates this final patch
and must be rerun.
The renewed independent reviews then found a selected-config pre-open identity interval, blocking
or silently skipped special manifests, checked-JSON GitHub shape substitution, retained-source
ignore divergence, direct-detail pull-request acceptance, and punctuation-only import titles.
Focused regressions for the original six facets passed on the prior tree. The newest reviewer
amendment adds retained-handle replacement/FIFO races, non-object `github`, all direct PR-marker
shapes, portable provider/directory slugs, and nonzero partial-batch outcomes. Focused local tests
pass. The implementation tree also passed 1,877 unit tests and 290 integration tests, formatting,
linting, and the installed Windows GNU cross-target test build before it was committed as
`b3e4696633f54ff57e42bdee7a8f20ef2bf32391`; final lifecycle/trust lanes remain pending. The prior
sandbox PASS used an unversioned workspace binary and unhashed untracked inputs, so it is
superseded by the hash-bound exact-commit replay below.

## Superseded private sandbox receipt

- Time: `2026-07-23T11:31:42Z`
- SpecSync implementation: `f6bb7a3b1aaf570b20a3a669ee2ecf46202d1f7b`
- Private testbed: `CorvidLabs/spec-sync-sandbox` at
  `758c144808d80169a44a740660b0d73c5b2f6ddd`
- Command:
  `SS=/Users/leif/Development/_CorvidLabs/spec-sync/target/debug/specsync bash drills/024-mcp-confined-sibling.sh`
- Result: `PASS: MCP accepts a manifest-relative sibling dependency confined beneath its server root`
- Testbed status disclosed two preserved untracked audit inputs: `drills/024-mcp-confined-sibling.sh`
  and `drills/fixtures/`; neither was modified or deleted by the replay.
- Reproducibility verdict: superseded because the executable and untracked inputs were not hashed.

## Reproducible private sandbox receipt

- Time: `2026-07-23T14:44:19Z`
- Exact implementation commit:
  `b3e4696633f54ff57e42bdee7a8f20ef2bf32391`
- Build: isolated local clone at the exact commit; `cargo build --locked --offline` exited 0; the
  clone remained Git-clean because build output is ignored.
- Executable SHA-256:
  `2b89fc91ffb8830a37468d60da2e55e73c98c4226101bd82a627308feca70e83`
- Private testbed: disposable `--no-local` clone of `CorvidLabs/spec-sync-sandbox` at
  `758c144808d80169a44a740660b0d73c5b2f6ddd`. The real private checkout was clean before and after
  the replay.
- The original mutable untracked drill inputs were absent, so the audit recreated their documented
  confined-sibling behavior in the disposable clone from the checked-in
  `snapshot_normalizes_confined_cargo_sibling_dependency` characterization. No real private
  checkout or remote branch was modified.
- Drill: `drills/024-mcp-confined-sibling.sh`, mode `0755`, SHA-256
  `1d53799cd33db3eed54edb19f7faee42205b449cd11582d3bb2c797b53d157c7`.
- Fixture SHA-256 values:
  - `Cargo.toml`:
    `2e0e11dd88ff3a367882ba2d048aeddd911171086c5cc9b8e3a434dc0c9d52fa`
  - `crates/a/Cargo.toml`:
    `23fc2e0fa8807a7e1572731ad1b317779e9ae835c05c356cf4fd4ef3bcc80705`
  - `crates/a/src/lib.rs`:
    `003ee424e49547851521809f34b42fd502a98cf810678857b75d37a960173cfd`
  - `crates/b/Cargo.toml`:
    `b9bf56495fb12ec2260d81f356f98c1407b62b7106f445b2cb2f051857763c20`
  - `crates/b/src/lib.rs`:
    `3857e64ee4dec2eae618add217f7235b3c05db581129dae6b62e79a33d4bd999`
  - `specs/workspace/workspace.spec.md`:
    `c1020283aec8841dceeaa44ef5a1b379ea90c41114bc40314276d06cafdd2bf5`
  - `specsync.json`:
    `a41caeccbe40569fddbe49fc2d94bbc41d02d7bc36951edde4b522408b999b4a`
- Command:
  `SS=<isolated exact-commit executable> bash drills/024-mcp-confined-sibling.sh`
- Exit: 0
- Stdout:
  `PASS: MCP accepts a manifest-relative sibling dependency confined beneath its server root`
- Disposable testbed status after replay contained only the intentionally created untracked audit
  inputs: `?? drills/024-mcp-confined-sibling.sh` and `?? drills/fixtures/`.

- `REQ-mcp-002`: `mcp::tests::test_repeated_tree_scans_share_one_confinement_budget`,
  `mcp::tests::snapshot_copies_the_exact_manifest_bytes_charged_during_discovery`,
  `mcp::tests::snapshot_includes_all_standard_gradle_module_forms_under_ignored_directories`,
  `mcp::tests::snapshot_ignores_nonsemantic_cargo_metadata_paths`,
  `mcp::tests::snapshot_normalizes_confined_windows_native_cargo_paths`,
  `mcp::mcp_absolute_outside_roots_do_not_disclose_existence`,
  `mcp::mcp_issues_requires_explicit_repo_before_redirected_git_metadata`,
  `mcp::mcp_issues_rejects_a_git_symlink_to_outside_metadata`,
  `mcp::mcp_issues_without_references_skips_repository_resolution`,
  `mcp::mcp_read_root_rejects_symlink_escape_and_preserves_referent`,
  `mcp::mcp_read_root_rejects_windows_junction_escape_and_preserves_referent`,
  `mcp::mcp_windows_read_roots_accept_absolute_children_and_reject_ambiguous_prefixes`, and
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
  `mcp::tests::issue_tool_fails_inconclusive_for_malformed_frontmatter`,
  `mcp::tests::issue_tool_fails_inconclusive_for_unreadable_spec_text`,
  `mcp::tests::issue_tool_fails_inconclusive_for_malformed_known_issue_fields`,
  `mcp::tests::issue_reference_field_validation_accepts_supported_list_forms`,
  `mcp::tests::issue_discovery_accepts_crlf_frontmatter_and_retains_references`,
  `mcp::tests::issue_read_diagnostics_are_bounded_relative_and_content_free`,
  `mcp::mcp_invalid_json_rpc_envelopes_return_invalid_request_without_mutating`,
  `mcp::mcp_resources_read_requires_exact_params_and_confines_direct_uris`,
  `mcp::mcp_bounds_outbound_resource_responses`, and
  `mcp::mcp_generate_reports_destination_collisions_and_io_failures_as_tool_errors`, and
  `mcp::mcp_write_tools_reject_windows_junction_destinations_without_touching_outside_bytes`
  prove
  fail-closed protocol validation, bounded output/IDs, and transactional generation integrity.
  Checked MCP discovery now retains walker/non-file and non-UTF-8 failures as inconclusive rather
  than returning a partial or empty success.
- `REQ-github-001`: `github::tests::malformed_issue_provider_output_fails_closed`,
  `github::tests::batch_prepares_once_deduplicates_fetches_and_attributes_per_spec`,
  `github::tests::batch_prepare_auth_and_repository_errors_are_inconclusive_for_every_spec`,
  `github::tests::batch_attributes_malformed_and_transport_provider_errors`,
  `github::tests::batch_full_deadline_includes_prepare_and_stops_before_fetch`,
  `github::tests::batch_cap_is_enforced_before_provider_prepare`,
  `github::tests::issue_list_pagination_collects_every_page_in_order`,
  `github::tests::issue_list_accepts_one_hundred_provider_entries_including_pull_requests`,
  `github::tests::issue_list_rejects_one_hundred_one_entries_before_parsing_malformed_pull_request`,
  `github::tests::issue_list_requires_present_pull_request_marker_to_be_an_object` (including
  explicit `pull_request: null`),
  `github::tests::issue_list_rejects_semantically_malformed_pull_requests_before_filtering`,
  `github::tests::issue_list_requires_exact_raw_open_state_for_issues_and_pull_requests`,
  `github::tests::issue_list_rejects_wrong_issue_and_pull_request_url_identity`,
  `github::tests::issue_list_rejects_duplicate_identities_involving_pull_requests_before_filtering`,
  `github::tests::issue_list_filters_fully_valid_pull_requests_after_validation`,
  `github::tests::issue_list_pagination_rejects_duplicates_hidden_by_pull_request_filtering`,
  `github::tests::issue_list_pagination_fails_instead_of_truncating_or_deduplicating`,
  `github::tests::link_header_parsing_detects_next_and_rejects_malformed_values`,
  `github::tests::link_header_rejects_wrong_or_malformed_repository_identity_and_resource`,
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
  `commands::issues::tests::inspection_findings_suppress_no_reference_guidance`,
  `commands::issues::tests::malformed_snapshots_are_retained_as_findings_without_parser_details`,
  `commands::issues::tests::crlf_snapshot_issue_references_are_retained`,
  and
  `issues_retains_unreadable_and_malformed_specs_as_safe_findings` prove text, JSON, Markdown, and
  GitHub output retain safe findings, suppress false no-reference guidance, omit fixture bytes,
  and exit nonzero.
  `issues_rejects_malformed_known_issue_fields_in_every_format`,
  `issues_accepts_yaml_trailing_commas_and_comments_through_shared_parser`, and
  `issues_sanitizes_control_characters_and_table_delimiters_in_unix_paths` cover the checked-shape
  and hostile-path facets. `issues_ignores_nested_extension_and_block_scalar_issue_keys`,
  `issues_retains_spec_shaped_discovery_failures`,
  `issues_retains_non_utf8_spec_filenames_as_redacted_discovery_findings`,
  `issues_rejects_absolute_and_parent_escaping_specs_directories_without_reading_them`, and
  `issues_rejects_symlinked_specs_directory_without_reading_its_target` cover the remaining
  top-level-only, discovery, and confinement facets.
  `commands::issues::tests::snapshot_validation_ignores_post_read_symlink_replacement` and
  `commands::issues::tests::snapshot_validation_never_reopens_replaced_mapped_source` plus
  `issues_create_runs_normal_drift_creation_for_stable_snapshots` prove exact spec/source
  `--create` validation through `validate_spec_content_with_sources` preserves normal drift
  behavior without reopening replaced paths.
  `commands::issues::tests::discovery_to_read_replacement_hook_rejects_regular_file_replacement`,
  `commands::issues::tests::discovery_to_read_replacement_hook_rejects_hardlink_replacement`, and
  `commands::issues::tests::snapshot_discovery_enforces_per_file_cumulative_and_file_count_limits`
  prove identity-through-read and the 4 MiB / 64 MiB / 10,000-spec ceilings.
  `commands::issues::tests::untrusted_diagnostics_are_safe_in_text_json_and_markdown` and
  `commands::issues::tests::markdown_code_spans_pad_leading_and_trailing_backticks` plus
  `issues_rejects_malicious_configured_repo_without_rendering_unsafe_bytes` cover bidi/Zl/Zp
  sanitization, edge-backtick padding, and configured-repository syntax validation before
  missing/empty-spec success.
- `REQ-parser-001`:
  `parser::tests::checked_issue_references_accept_normal_lists_and_inline_comments`,
  `parser::tests::checked_issue_references_accept_yaml_trailing_commas`,
  `parser::tests::checked_issue_references_accept_crlf_frontmatter`,
  `parser::tests::checked_issue_references_keep_crlf_validation_strict`,
  `parser::tests::checked_issue_references_reject_duplicate_keys`,
  `parser::tests::checked_issue_references_reject_invalid_known_values`,
  `parser::tests::checked_issue_references_ignore_nested_extensions_and_block_scalars`,
  `parser::tests::checked_issue_references_reject_malformed_unknown_extensions`, and
  `parser::tests::checked_issue_references_reject_reviewer_reproducer_without_leaking_content`
  prove the maintained real-YAML contract.
- `REQ-validator-001`:
  `commands::issues::tests::snapshot_validation_ignores_post_read_symlink_replacement` and
  `commands::issues::tests::snapshot_validation_never_reopens_replaced_mapped_source` prove
  `validate_spec_content_with_sources` consumes exact pre-read spec/source snapshots after ambient
  path replacement; the command integration create fixture proves normal drift issue creation
  remains available.
- `REQ-exports-005`:
  `exports::tests::supplied_content_extraction_never_resolves_ambient_typescript_imports` proves
  the module-internal supplied-content entry point preserves local exports without reopening the
  logical source path or resolving ambient wildcard targets.
- `REQ-commands-003`:
  `github::tests::drift_issue_capture_sanitizes_untrusted_title_and_markdown_arguments` proves the
  GitHub title/body boundary; `create_drift_issues` routes repository errors, spec paths, returned
  URLs, and provider errors through `safe_diagnostic` before terminal rendering.
  `commands::tests::rendered_drift_errors_prefer_longest_discovered_spec_path` exercises the
  rendered-vector compatibility route with overlapping paths and a legal `": "` path; compile-time
  function-pointer assertions in the same test module bind the public `run_validation` return
  channels and `create_drift_issues(&[String])` parameter.
- `REQ-commands-004`:
  `commands::tests::validate_module_name_rejects_windows_reserved_basenames_portably`,
  `commands::tests::validate_module_name_rejects_windows_invalid_characters_portably`,
  `commands::tests::validate_module_name_rejects_trailing_spaces_and_dots`, and
  `commands::tests::validate_module_name_enforces_portable_spec_filename_byte_limit` prove the
  shared validator rejects portable device/character/suffix hazards and preserves valid ASCII and
  multibyte names at the exact 247-byte boundary on every host.
- `REQ-importer-001`:
  `importer::tests::test_import_github_issue_entry_path_converts_shared_typed_details`,
  `importer::tests::test_import_github_issue_entry_path_returns_no_item_on_provider_failure`,
  `importer::tests::test_import_github_issue_details_full`, and
  `importer::tests::test_import_github_issue_details_empty_body` prove the importer entry seam,
  failure non-production, and complete typed detail conversion while
  `importer::tests::test_external_imports_reject_nonportable_slugs` proves GitHub, Jira, and
  Confluence all reject reserved or overlong output names before producing an item; the shared
  GitHub regressions prove explicit-token, no-subprocess, timeout, and 404 revalidation behavior.
- `REQ-cmd-import-001`: `github::tests::issue_list_pagination_collects_every_page_in_order`,
  `github::tests::issue_list_accepts_one_hundred_provider_entries_including_pull_requests`,
  `github::tests::issue_list_rejects_one_hundred_one_entries_before_parsing_malformed_pull_request`,
  `github::tests::issue_list_pagination_fails_instead_of_truncating_or_deduplicating`, and
  `github::tests::link_header_parsing_detects_next_and_rejects_malformed_values` prove complete
  bounded batch traversal and fail-closed partial-list handling;
  `single_github_import_fails_closed_without_a_rest_token_or_output` and
  `batch_github_import_fails_closed_without_a_rest_token_or_output` prove both real CLI entry paths
  fail before creating output when explicit REST authorization is absent;
  `commands::import::tests::github_batch_continues_after_unsafe_item_and_requires_nonzero_exit`
  and
  `commands::import::tests::directory_batch_imports_valid_item_but_command_exits_nonzero_for_unsafe_item`
  prove unsafe entries create no output, later valid entries still import, and every partial-error
  batch exits 1 after its truthful summary.
- `REQ-manifest-001`:
  `manifest::tests::gradle_settings_support_groovy_kotlin_multiline_and_project_dir_overrides`,
  `manifest::tests::gradle_settings_ignore_comments_and_decode_escaped_values`,
  `manifest::tests::gradle_manifest_discovery_fails_closed_for_malformed_settings`,
  `manifest::tests::gradle_manifest_discovery_rejects_dynamic_include_without_partial_modules`,
  `manifest::tests::gradle_settings_reject_unsupported_project_dir_bases_and_suffixes`, and
  `manifest::tests::gradle_manifest_discovery_accepts_comments_and_escaped_paths` prove shared,
  normalized, checked Gradle discovery.
  `manifest::tests::gradle_settings_reject_project_root_escapes` and
  `manifest::tests::gradle_manifest_discovery_rejects_project_root_escape_without_partial_modules`
  prove rooted, drive-qualified, UNC, and parent-underflow paths are rejected before discovery.
  The amended implementation tree includes
  `manifest::tests::gradle_settings_support_literal_set_project_dir_forms`,
  `manifest::tests::gradle_settings_reject_dynamic_or_ambiguous_set_project_dir_forms`, expanded
  `manifest::tests::gradle_settings_reject_project_root_escapes`, and
  `manifest::tests::gradle_manifest_discovery_rejects_symlinked_module_directories`,
  `manifest::tests::gradle_settings_reject_interpolated_includes_without_partial_modules`,
  `manifest::tests::gradle_settings_preserve_literal_dollars`, and
  `manifest::tests::gradle_manifest_discovery_rejects_non_regular_oversized_and_non_utf8_manifests`.
  The amended tree passes all 21 selected Gradle unit tests; hosted-Windows reparse-point runtime
  is a separate required gate.
- `REQ-cmd-check-001`, `REQ-cmd-comment-003`, `REQ-cmd-coverage-001`,
  `REQ-cmd-generate-001`, `REQ-cmd-report-001`, and `REQ-cmd-score-001`:
  `malformed_gradle_is_inconclusive_for_coverage_gating_commands` proves every command rejects
  malformed discovery, preserves parseable JSON where supported, and prevents mutation/reporting.
  `commands::gradle_root_escape_is_inconclusive_for_coverage_gating_commands` proves
  project-root escapes return nonzero structured inconclusive outcomes across check, coverage,
  generate, report, and score without reading outside bytes or mutating either tree.
  The amended implementation tree includes
  `gradle_set_project_dir_escapes_are_inconclusive_for_coverage_gating_commands`,
  `gradle_symlink_module_escape_is_inconclusive_for_coverage_gating_commands`,
  `gradle_junction_module_escape_is_inconclusive_for_coverage_gating_commands`,
  `gradle_interpolated_project_dirs_are_inconclusive_for_coverage_gating_commands`,
  `gradle_symlinked_manifests_are_inconclusive_without_reading_outside_bytes`,
  `mcp_gradle_set_project_dir_escapes_fail_closed_without_outside_access`, and
  `mcp_gradle_symlink_module_escape_fails_closed_without_outside_access` plus
  `mcp_gradle_interpolated_project_dirs_fail_closed_without_outside_access`. The amended tree
  passes all 12 selected Gradle integration tests.
- `REQ-config-005`: `config::tests::checked_source_detection_surfaces_malformed_gradle_settings`
  proves checked errors and compatibility fallback separation.
- `REQ-config-006`:
  `config::tests::json_github_repo_wrong_types_fail_closed_without_discarding_valid_config`
  proves wrong-shaped legacy JSON repository values remain invalid without discarding valid
  surrounding configuration or falling back to Git auto-detection.
- `REQ-config-007`: retained-config hook tests plus
  `issues_rejects_wrong_shaped_toml_path_fields_from_retained_snapshot` and
  `commands::issues::tests::omitted_source_dirs_are_detected_through_the_retained_project_capability`
  prove exact-byte parsing, same-capability authority, post-snapshot pathname replacement
  resistance, capability-bound omitted-source discovery, and known-field type rejection.
- `REQ-validator-008`:
  `validator::tests::malformed_gradle_settings_make_coverage_inconclusive` proves checked coverage
  returns no partial report and the compatibility wrapper carries an inconclusive diagnostic.
- Final review regressions:
  `mcp::tests::server_root_capability_rejects_a_root_replaced_before_canonicalization` and
  `mcp_real_cli_rejects_requested_root_replacement_after_identity_binding` prove both the internal
  server and spawned CLI bind the requested root before canonicalization;
  `commands::issues::tests::mapped_sources_use_original_root_capability_after_root_symlink_replacement`
  and
  `commands::issues::tests::snapshot_discovery_bounds_huge_non_spec_inventories_before_accumulation`
  prove single-capability source/spec authority and the 100,000-entry total inventory bound;
  `github::tests::provider_item_urls_require_canonical_decimal_numbers_in_list_and_detail`
  rejects leading-zero provider URL identities; and
  `commands::tests::rendered_drift_errors_prefer_longest_discovered_spec_path` proves exact private
  attribution while public rendered-vector compatibility remains intact.
  `issues_missing_or_empty_specs_use_selected_structured_renderer` and
  `issues_repository_resolution_failures_use_selected_structured_renderer` prove early selected
  output, while the `mcp_allow_empty_tool_and_resource_*selected_config*` group proves MCP
  malformed/UTF-8/type rejection and valid BOM compatibility.

## Current final-tree evidence for the Gradle amendment

- No historical test, reviewer, Windows cross-target, sandbox, trust, Attest, or CI result is
  evidence for the reviewer-driven Gradle fixes documented in this amendment.
- Exact commit `d05896b` passed the full local lane: 1,882 unit tests and 296 integration tests,
  plus `fledge run fmt`, `fledge run lint`, and the release `fledge run build`. That evidence
  predates the interpolation/encoded-escape amendment and is not final-tree evidence.
- Characterization reproduced the interpolation, indirect-directive, multiline-literal/comment,
  constructor-token, MCP build-manifest, drive-relative, and post-discovery source-swap false
  greens. Focused parser, CLI, MCP tool/resource, FIFO/socket/link/size/race, and retained coverage
  snapshot tests pass without outside disclosure or partial generation.
- The complete amended code tree passed `fledge run test`: 1,892 unit tests and 302 integration
  tests in 276.8 seconds. `fledge run fmt`, `fledge run lint`, focused coverage tests, and all
  focused Gradle tests pass.
- The post-review amended tree passes 1,907 unit tests and 306 integration tests with zero
  failures. Focused evidence includes 32 Gradle unit and 16 Gradle integration tests, retained
  coverage byte/entry/depth/UTF-8 checks, two generic MCP snapshot unit races, and both spawned
  tool/resource FIFO/socket/replacement integrations. Formatting and the repository Clippy lane
  pass; the broader non-gating all-target Clippy invocation reports pre-existing test-only warnings
  in lifecycle/init/scaffold/types code outside this change.
- The same post-review tree passes the locked release build, 23 docs tests, Astro diagnostics with
  zero errors/warnings/hints, a 43-page site build, and score 100/100 for all 62 specs.
- `cargo check --target x86_64-pc-windows-gnu --tests` passed; it emitted the existing cfg-specific
  unused-variable warning in `src/change.rs`, outside this amendment. Cross-target compilation is
  not a substitute for hosted-Windows junction runtime.
- The hosted-Windows post-discovery junction and build-manifest reparse fixtures compile under the
  Windows GNU test target but remain pending hosted-Windows runtime CI.
- Pending: release build, docs, strict coverage and score gates, two independent clean reviews,
  private-sandbox refresh, fresh exact-digest definition approval, GitHub CI, `fledge trust verify`,
  Attest provenance, lifecycle verification, and closing approval.
  Pending characterization and post-fix evidence must cover top-level compatibility; inert quoted
  and nested-comment content; aliased/qualified/compound directives; same-line and multiline
  conditional blocks; unsupported triple-quoted include/project-directory arguments; all CLI/MCP
  checked gates; and unchanged outside bytes with no partial output.
