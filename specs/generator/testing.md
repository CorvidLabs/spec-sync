---
spec: generator.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/generator.rs` | cargo test generator:: | `detect_language_rust`, `detect_language_typescript`, `detect_language_python`, `detect_language_go`, `detect_language_mixed_majority_wins`, `detect_language_empty` |
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
| Generate spec for unspecced module | a module "auth" with source files in `src/auth/` and no existing spec | `generate_specs_for_unspecced_modules` is called | creates `specs/auth/auth.spec.md`, `specs/auth/tasks.md`, `specs/auth/context.md`, `specs/auth/requirements.md`, `specs/auth/testing.md`, and `specs/auth/design.md` if `companions.design` is enabled in config |
| Skip existing spec | a module "auth" that already has `specs/auth/auth.spec.md` | `generate_specs_for_unspecced_modules` is called | skips the module, returns 0 |
| Design companion opt-in | `companions.design` is enabled in config | `generate_companion_files_for_spec` is called for module "dashboard" | creates design.md with YAML frontmatter (`spec: dashboard.spec.md`, `sources: []`) and sections for Layout, Components, Tokens, Assets |
| Design companion disabled by default | no `companions.design` config (default: false) | `generate_companion_files_for_spec` is called | creates tasks.md, context.md, requirements.md, testing.md but NOT design.md |
| AI generation fallback | an AI provider that fails with an error | generating a spec for module "auth" | falls back to template-based generation and prints a warning |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Cannot create spec directory | Prints error to stderr, skips module | Keep or add a focused assertion before changing this behavior |
| Cannot write spec file | Prints error to stderr, skips module | Keep or add a focused assertion before changing this behavior |
| AI generation fails | Falls back to template, prints warning | Keep or add a focused assertion before changing this behavior |
| No source files found for module | Skips module entirely | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/generator.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `./target/release/specsync score --all`.
