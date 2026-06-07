---
spec: cmd_compact.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/compact.rs` | cargo test commands::compact | No inline tests found; add focused coverage for `cmd_compact`, `compact_changelogs`, `load_config` before risky changes |

## Coverage Gaps

- Integration gap: add a fixture for "Compact changelogs" before changing user-visible CLI output, generated files, or error handling in cmd_compact.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Compact changelogs | a spec has 25 changelog entries, `--keep 10` | `cmd_compact` runs | 15 oldest entries removed, 10 newest kept |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| No specs with changelogs | Prints "nothing to compact" | Keep or add a focused assertion before changing this behavior |
| Fewer entries than keep | File unchanged | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- compact --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/compact.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
