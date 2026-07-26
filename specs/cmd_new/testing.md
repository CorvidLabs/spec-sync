---
spec: cmd_new.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `tests/integration/config.rs` | `cargo test --test integration config::new_` | `new_auto_detects_single_source_file`, `new_warns_when_no_source_files_match` |
| Required section parity | `cargo test --test integration new_emits_every_required_section` | Every standard required section is emitted by the shared renderer |

## Coverage Gaps

- Integration gaps remain for `specsync new --full` companion generation and refusal to overwrite an existing spec.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Quick spec | `src/auth.rs` exists | `cmd_new(root, "auth", false)` runs | creates `specs/auth/auth.spec.md` with detected source and exports |
| Full with companions | `--full` flag | `cmd_new` runs | creates spec.md, tasks.md, context.md, requirements.md, testing.md (and design.md if `companions.design` is enabled) |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Spec already exists | Exits 1 | Keep or add a focused assertion before changing this behavior |
| No source files found | Creates a draft with explicit `files: []` | Protected by the add-spec empty-draft parser/validator matrix and generator unit test |
| Dir creation fails | Exits 1 | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- new --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/new.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
