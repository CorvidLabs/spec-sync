---
spec: types.spec.md
---

## Tasks

- [ ] Add `SpecStatus` enum to replace raw string for `status` field validation
- [ ] Add builder pattern for `SpecSyncConfig` to simplify test construction
- [ ] Consider splitting large enums into sub-modules if type count grows significantly

## Done

- [x] Core enums: AiProvider, Language, OutputFormat, ExportLevel
- [x] Core structs: Frontmatter, ValidationResult, CoverageReport, SpecSyncConfig
- [x] Loose string parsing for AiProvider with aliases
- [x] Language detection from file extensions
- [x] Default implementations for all config types
- [x] ModuleDefinition for explicit module configuration
- [x] RegistryEntry for cross-project registry

## Gaps

- `status` field is a raw String — no type-level enforcement of valid values (draft, review, stable, deprecated)
- No validation of `version` field type at the type level (accepts any integer)

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
