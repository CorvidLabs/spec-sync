## MODIFIED

### SPEC SECTION Public API

**Exported Enums**

| Type | Description |
|------|-------------|
| `WarningCategory` | 13-variant enum representing classifiable warning types: RequirementsCompanion, StubSection, UndocumentedExport, Deprecated, UnknownStatus, UnknownAgentPolicy, SchemaColumn, SchemaTypeMismatch, ConsumedBy, ChangelogEntries, SpecSize, MinInvariants, RequireDependsOn |

**Exported Structs**

| Type | Description |
|------|-------------|
| `IgnoreRules` | Container holding global suppression set and per-spec suppression map, loaded from `.specsyncignore` |

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `from_str` | Alias for `WarningCategory::from_str` | `Option<WarningCategory>` | Parse category name string |
| `classify` | Alias for `WarningCategory::classify` | `Option<WarningCategory>` | Classify warning text |
| `load` | Alias for `IgnoreRules::load` | `IgnoreRules` | Load from `.specsyncignore` |
| `parse_inline` | Alias for `IgnoreRules::parse_inline` | `HashSet<WarningCategory>` | Parse inline directives |
| `is_suppressed` | Alias for `IgnoreRules::is_suppressed` | `bool` | Check suppression |
