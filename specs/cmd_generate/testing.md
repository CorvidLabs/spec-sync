---
spec: cmd_generate.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/generate.rs` | cargo test commands::generate | No inline tests found; add focused coverage for `cmd_generate`, `generate_spec_template`, `IgnoreRules::load`, `compute_coverage` before risky changes |
| `tests/integration.rs` | cargo test --test integration generate_creates_spec_for_unspecced_module | End-to-end fixture: `generate_creates_spec_for_unspecced_module` |
| `tests/integration.rs` | cargo test --test integration generate_no_op_when_fully_covered | End-to-end fixture: `generate_no_op_when_fully_covered` |
| `tests/integration.rs` | cargo test --test integration generate_with_multiple_languages | End-to-end fixture: `generate_with_multiple_languages` |
| `tests/integration.rs` | cargo test --test integration generate_uncovered_flag_accepted | End-to-end fixture: `generate_uncovered_flag_accepted` |
| `tests/integration.rs` | cargo test --test integration generate_batch_empty_list_skips_gracefully | End-to-end fixture: `generate_batch_empty_list_skips_gracefully` |
| `tests/integration.rs` | cargo test --test integration generate_creates_companion_files | End-to-end fixture: `generate_creates_companion_files` |
| `tests/integration.rs` | cargo test --test integration generate_creates_design_md_when_enabled | End-to-end fixture: `generate_creates_design_md_when_enabled` |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| AI-assisted generation | `--provider claude` set, 3 unspecced modules | `cmd_generate` runs | generates 3 AI-populated specs |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| AI provider not found | Exits 1 | Keep or add a focused assertion before changing this behavior |
| AI fails for one module | Error printed, continues | Keep or add a focused assertion before changing this behavior |
| All modules already specced | Prints "all covered" | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- generate --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/generate.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
