## MODIFIED

### SPEC SECTION Dependencies

**Consumes**

| Module | What is used |
|--------|-------------|
| exports | `has_extension`, `is_test_file` |
| types | `CoverageReport` (including the symlinked entries discovery skipped and the manifests degraded rather than propagated), `SpecSyncConfig` |

**Consumed By**

| Module | What is used |
|--------|-------------|
| main | `generate_specs_for_unspecced_modules`, `generate_companion_files_for_spec` |
| mcp | `generate_specs_for_unspecced_modules_paths` |
