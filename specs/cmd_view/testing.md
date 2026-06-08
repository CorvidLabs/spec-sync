---
spec: cmd_view.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/view.rs` | cargo test commands::view | No inline `#[cfg(test)]` module in the command wrapper; add a CLI fixture before risky changes |
| `src/view.rs` | cargo test view | Role/section logic is unit-tested here (`test_sections_for_role` covers dev/qa/product/agent and rejects unknown roles) |

## Coverage Gaps

- Integration gap: no end-to-end CLI fixture asserts that `specsync view --role <r>` prints only the role's sections or that `--spec <module>` filters correctly. The section mapping itself is covered by `test_sections_for_role` in `src/view.rs`.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Dev view | `specsync view --role dev --spec auth` | `cmd_view` runs | renders auth spec with dev-relevant sections only |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| No specs found | Exits 1 | Keep or add a focused assertion before changing this behavior |
| Spec read error | Error printed, continues | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- view --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/view.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
