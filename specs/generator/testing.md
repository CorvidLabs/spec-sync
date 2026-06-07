---
spec: generator.spec.md
---

## Automated Coverage

### Unit tests (`src/generator.rs`, `cargo test generator::`)

| Group | Tests | What It Covers |
|-------|-------|----------------|
| `detect_primary_language` | `detect_language_rust`, `detect_language_typescript`, `detect_language_python`, `detect_language_go`, `detect_language_mixed_majority_wins`, `detect_language_empty`, `detect_language_unknown_extensions` | Picks the dominant language by extension; `None` when unknown/empty |
| `language_template` | `template_rust_has_structs_enums_section`, `template_swift_has_protocols_section`, `template_go_has_package_terminology`, `template_kotlin_has_classes_interfaces`, `template_python_has_classes`, `template_typescript_uses_default` | Per-language Public API sections; TS falls back to the default template |
| `generate_spec` | `generate_spec_fills_module_name`, `generate_spec_hyphenated_name_title_case`, `generate_spec_uses_custom_template`, `generate_spec_rust_files_use_rust_template` | Frontmatter rewriting, dash-to-title-case, custom-template precedence, language selection |
| Template content | `tasks_template_has_required_sections`, `requirements_template_has_required_sections`, `context_template_has_required_sections`, `testing_template_has_required_sections`, `design_template_has_required_sections`, `default_template_has_all_required_sections` | Built-in companion/spec templates carry their required sections and `{module}` placeholder |
| Companion creation | `companion_files_created_when_absent`, `companion_files_created_with_design_enabled`, `companion_files_not_overwritten`, `companion_files_from_template_uses_custom_testing`, `companion_files_from_template_falls_back_for_testing`, `companion_files_from_template_uses_custom_design`, `companion_files_from_template_falls_back_for_design` | Companions created only when absent, design opt-in, custom-template-dir use with per-file fallback |
| `find_files_for_module` | `find_files_flat_module`, `find_files_subdir_module`, `find_files_excludes_test_files`, `find_files_no_match`, `find_files_user_defined_module` | Discovery via flat files, subdirs, config `modules`; test-file exclusion |

### Integration tests (`tests/integration.rs`, `cargo test --test integration`)

| Fixture | What It Asserts |
|---------|-----------------|
| `generate_creates_spec_for_unspecced_module` | Creates `<module>.spec.md` for an unspecced module |
| `generate_no_op_when_fully_covered` | No specs generated when coverage is complete |
| `generate_with_multiple_languages` | Mixed-language project generates per-module specs |
| `generate_uncovered_flag_accepted` | The uncovered-targeting flag is accepted |
| `generate_batch_empty_list_skips_gracefully` | Empty module list is a clean no-op |
| `generate_creates_companion_files` | tasks/context/requirements/testing companions created alongside the spec |
| `generate_creates_design_md_when_enabled` | `design.md` created only when `companions.design` is enabled |
| `companion_testing_md_has_correct_structure` | Generated testing.md carries the expected sections and `spec:` back-reference |
| `companion_files_not_overwritten_on_regenerate` | Re-running generation leaves existing companions untouched |

## Manual Testing

- [ ] Run `specsync generate` on a project with no AI provider configured — confirm template-only specs are written with no network call
- [ ] Configure an AI provider and run `generate` — confirm the spec body is AI-authored, then force a failure (bad/missing key) and confirm the warning + template fallback
- [ ] Place a `specs/_template.spec.md` and confirm it overrides the built-in template
- [ ] Enable `companions.design` and confirm `design.md` is created with `spec:`/`sources:` frontmatter; disable it and confirm it is not
- [ ] Re-run `generate` and confirm no existing spec or companion is overwritten

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Spec file already exists | Module skipped, not overwritten |
| Companion file already exists | Left untouched; only missing companions are created |
| No source files found for module | Module skipped entirely |
| AI generation fails (missing key, timeout, bad output) | Warns on stderr, falls back to template, still writes the spec |
| Cannot create the spec directory / write the spec file | Prints an error to stderr, skips the module |
| Mixed-language module | Template chosen by the most common source extension |
| Unknown/unsupported language | Falls back to `DEFAULT_TEMPLATE` |
| Hyphenated module name | Title rendered dash-to-title-case ("api-gateway" → "Api Gateway") |
| Custom template dir missing a specific companion | Falls back to the built-in template for that file only |

## Reviewer Checklist

- Run `cargo test generator::` (and the relevant `tests/integration.rs` generate/companion fixtures) before changing `src/generator.rs`.
- Reproduce one Manual Testing flow with a temp project fixture before changing user-visible output.
- If an error/warning string or template section changes, update the matching Edge Case row and test assertion in the same commit.
- Run the release checks: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
