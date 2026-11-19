---
spec: view.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/view.rs` | cargo test view:: | `test_sections_for_role`, `test_split_sections`, `test_strip_frontmatter` |

## Coverage Gaps

- Integration gap: add a fixture for "Dev view" before changing user-visible CLI output, generated files, or error handling in view.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Dev view | a spec with all standard sections | `view_spec(path, "dev")` is called | returns Purpose, Public API, Invariants, Dependencies, and Change Log sections only |
| Agent view with policy | a spec with `agent_policy: read-only` in frontmatter | `view_spec(path, "agent")` is called | output header includes `**Status:** stable` and `**Agent Policy:** read-only` lines |
| Product view with requirements | a spec at `specs/auth/auth.spec.md` with a companion `specs/auth/requirements.md` | `view_spec(path, "product")` is called | returns Purpose, Change Log, and appended requirements.md content |
| Invalid role | role string `"manager"` | `view_spec(path, "manager")` is called | returns `Err` with descriptive message listing valid roles |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Unknown role string | Returns `Err` listing valid roles | Keep or add a focused assertion before changing this behavior |
| Spec file unreadable | Returns `Err` with read error description | Keep or add a focused assertion before changing this behavior |
| Frontmatter parse failure | Returns `Err` with parse error | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/view.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.

## Bounded reads (REQ-view-002)

FIFOs run under a timeout and over-bound files are sparse; nothing reads `/dev/zero`. Unix-only.

| Case | Required behavior | Test |
|------|-------------------|------|
| A project file this module reads is a FIFO, a link out of the project, or over its bound | Refused without waiting or reading past the bound | `tests/integration/project_reads.rs`: `no_project_file_is_read_outside_the_bounded_read`, the source scan that fails on any direct file read outside the bounded project read |
