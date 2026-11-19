---
spec: cmd_merge.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/merge.rs` | cargo test commands::merge | No inline tests found; add focused coverage for `cmd_merge`, `load_config`, `OutputFormat` before risky changes |

## Coverage Gaps

- Integration gap: add a fixture for "Auto-resolved" before changing user-visible CLI output, generated files, or error handling in cmd_merge.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Auto-resolved | 3 specs with simple conflicts | `cmd_merge` runs | all auto-resolved |
| Manual needed | complex conflict | `cmd_merge` runs | flags file, exits 1 |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| No conflicts | Prints "No spec files with merge conflicts found." | Keep or add a focused assertion before changing this behavior |
| Complex conflict | Prints "needs manual merge: <path>" and exits 1 | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- merge --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/merge.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.

## Bounded reads (REQ-cmd-merge-005)

FIFOs run under a timeout and over-bound files are sparse; nothing reads `/dev/zero`. Unix-only.

| Case | Required behavior | Test |
|------|-------------------|------|
| A project file this module reads is a FIFO, a link out of the project, or over its bound | Refused without waiting or reading past the bound | `tests/integration/project_reads.rs`: `no_project_file_is_read_outside_the_bounded_read`, the source scan that fails on any direct file read outside the bounded project read |
