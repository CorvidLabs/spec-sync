---
spec: cmd_deps.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/deps.rs` | cargo test commands::deps | No inline tests found; add focused coverage for `cmd_deps`, `validate_deps`, `load_config`, `OutputFormat` before risky changes |

## Coverage Gaps

- Integration gap: add a fixture for "Mermaid output" before changing user-visible CLI output, generated files, or error handling in cmd_deps.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Mermaid output | `--mermaid` flag set, clean dep graph | `cmd_deps` runs | outputs valid Mermaid flowchart syntax |
| Cycle detected | A depends on B, B depends on A | `cmd_deps` runs | prints cycle error and exits 1 |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Circular dependency | Error printed, exits 1 | Keep or add a focused assertion before changing this behavior |
| Missing dependency spec | Error printed, exits 1 | Keep or add a focused assertion before changing this behavior |
| Empty dep graph | Prints hint about `depends_on` | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- deps --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/deps.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
