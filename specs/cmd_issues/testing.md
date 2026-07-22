---
spec: cmd_issues.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/issues.rs` | cargo test commands::issues::tests | Pure summary regression distinguishes no references from an all-error batch. |
| `tests/integration/commands.rs` | cargo test --test integration commands::issues_without_references | No-reference projects succeed before repository/provider resolution with and without `github.repo`. |
| `tests/integration/commands.rs` | cargo test --test integration commands::issues_reference_batch_fails_closed_without_a_rest_token | A referenced issue with configured repo but no token exits non-zero with attributed JSON error output. |
| `src/github.rs` | cargo test github | Typed classification, global deduplication/cap, strict provider parsing, transport failure, and timeout are covered in the GitHub module. |
| MCP batch cap | cargo test mcp::tests::issue_tool_enforces_one_deduplicated_invocation_cap_across_specs | Multiple individually safe specs exceed the project-wide cap before provider access. |

## Coverage Gaps

- No end-to-end fixture exercises live successful/closed/not-found classification; no-reference
  behavior and explicit-token provider failure are covered without network access.
- Add recorded/mocked GitHub responses to cover the valid/closed/not-found/error classification and the non-zero exit on 404 or error.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| No references | specs have no `implements`/`tracks` | `specsync issues` | Prints "No issue references found in spec frontmatter." and exits 0. |
| All references valid | issues #10, #15, #20 open | `specsync issues` | Prints "3 valid, 0 closed, 0 not found" and exits 0. |
| Closed reference | a referenced issue is closed | `specsync issues` | Warns "(closed — spec may need updating)"; still exits 0 if nothing is not-found/errored. |
| Stale reference (404) | spec references a deleted issue | `specsync issues` | Reports it as not found and exits 1. |
| `--create` with drift | specs that fail validation | `specsync issues --create` | Runs validation and opens drift issues for the failing specs. |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| GitHub repo unresolvable with references | Prints error, exits 1 | Add a focused assertion before changing repo resolution. |
| No references and no repository | Guidance, exit 0, no provider access | Keep both `issues_without_references_*` command regressions. |
| Issue returns 404 (not found) | Counted as not-found; triggers exit 1 | Add a mocked fixture before changing exit logic. |
| Verification error (e.g. API/auth failure) | Counted as error; **triggers exit 1** (`total_not_found > 0 || total_errors > 0`) | Missing-token command path is covered by `issues_reference_batch_fails_closed_without_a_rest_token`; retain a recorded provider-error fixture before changing classification. |
| All references produce errors | Summary includes the error count and does not print no-reference guidance | Keep `all_error_batches_report_errors_instead_of_no_reference_guidance`. |
| Duplicate references across specs | One provider lookup per unique ID | Keep the GitHub batch deduplication regression. |
| More than 100 unique references | Batch error before provider access and exit 1 | Keep the MCP cross-spec cap regression. |
| Closed issue only | Warned but does **not** by itself force a non-zero exit | Add a mocked fixture before changing the exit condition. |
| Spec without `implements`/`tracks` | Skipped; repository resolution is also skipped if all specs are empty | Keep the network-free no-reference fixtures. |

## Reviewer Checklist

- Run `cargo run -- issues --help` and confirm `--create` and the format flags are present.
- For changes to verification/classification, run the `github` module's tests — that is where the API logic lives.
- Confirm the exit-code condition (`not_found > 0 || errors > 0`) still matches the documented Regression Matrix before changing it.
- If an output or error message changes, update the matching Regression Matrix row and assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
