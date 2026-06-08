---
spec: cmd_issues.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/issues.rs` | (none) | No inline `#[cfg(test)]` module and no integration fixtures — the command depends on the live GitHub API. |
| `src/github.rs` | cargo test github | `verify_spec_issues`/`resolve_repo` classification and repo resolution are covered in the `github` module's tests. |

## Coverage Gaps

- No fixture exercises the verification flow. The most testable, network-free case is "no spec references": a project whose specs have neither `implements` nor `tracks` should print "No issue references found in spec frontmatter." and exit 0 — add that first.
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
| GitHub repo unresolvable | Prints error, exits 1 | Add a focused assertion before changing repo resolution. |
| Issue returns 404 (not found) | Counted as not-found; triggers exit 1 | Add a mocked fixture before changing exit logic. |
| Verification error (e.g. API/auth failure) | Counted as error; **triggers exit 1** (`total_not_found > 0 || total_errors > 0`) | Add a mocked fixture before changing exit logic. |
| Closed issue only | Warned but does **not** by itself force a non-zero exit | Add a mocked fixture before changing the exit condition. |
| Spec without `implements`/`tracks` | Skipped (not verified) | Cover with the network-free "no references" fixture. |

## Reviewer Checklist

- Run `cargo run -- issues --help` and confirm `--create` and the format flags are present.
- For changes to verification/classification, run the `github` module's tests — that is where the API logic lives.
- Confirm the exit-code condition (`not_found > 0 || errors > 0`) still matches the documented Regression Matrix before changing it.
- If an output or error message changes, update the matching Regression Matrix row and assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
