## ADDED

### REQUIREMENT REQ-validator-008

Coverage gates SHALL use fallible checked manifest discovery and SHALL report malformed, unreadable,
unsupported, or unconfined Gradle discovery as inconclusive instead of accepting partial coverage
or traversing outside the retained project root.

Acceptance Criteria

- `compute_coverage_checked` propagates checked manifest-discovery errors without producing a
  partial `CoverageReport`.
- CLI and MCP coverage/enforcement callers use checked coverage and fail with an inconclusive
  diagnostic.
- Raw drive-qualified module identities, dynamic/unsupported project-directory mutators, and
  symlink/reparse components in Gradle-derived directories propagate as checked errors before
  source probing, traversal, partial totals, or generation.
- Interpolated/encoded-dynamic Gradle strings and unsafe or oversized Gradle manifest endpoints
  propagate as checked errors before partial totals, outside reads, or generation.
- After checked manifest discovery, every configured or manifest-derived source tree is traversed
  through one retained project-root capability with no-follow directory opens and non-blocking,
  identity-checked regular-file reads. Post-discovery replacement, links/reparse points, and
  special files fail every coverage gate before totals, disclosure, or generated output.
- Checked coverage acquires configured source roots and source bytes through retained no-follow
  handles, binds directory/file identity before and after traversal, and derives file, LOC,
  immediate-directory, and flat-file module results from that snapshot. Post-discovery
  symlink/junction replacement fails inconclusive for every coverage gate before outside reads.
- Caller-selected spec ownership reads, manifest discovery, spec-module enumeration, source
  traversal, and final root verification share one retained project capability. Traversal is
  sorted and iterative with 8 MiB per input file, 64 MiB cumulative bytes, 100,000 entries, and
  256 path components.
- Invalid UTF-8 source names/content, special entries, links/reparse points, root/directory/file
  identity replacement, and exhausted bounds fail inconclusive before partial coverage totals.
- `compute_coverage` remains available for compatibility and returns a zero-percent report carrying
  an inconclusive module diagnostic when checked discovery fails.

## MODIFIED

### REQUIREMENT REQ-validator-001

The validator SHALL enforce bidirectional code-contract, metadata, dependency, schema, and coverage
rules while accumulating actionable findings, and SHALL support exact pre-read spec snapshots
without reopening their logical paths.

Acceptance Criteria

- Bidirectional validation reports a documented-but-missing export as an error and an undocumented
  code export as a warning.
- Missing required frontmatter fields (`module`, `version`, `status`, `files`) are errors.
- Cross-project references are recognized and skipped during local validation.
- Coverage excludes test files and configured exclude patterns.
- `find_spec_files` returns sorted results.
- Schema validation uses the configured `schema_pattern`.
- Missing source suggestions use Levenshtein distance with a maximum distance of three.
- Flat source files are detected as modules while common entry points are excluded.
- Source discovery respects configured `source_extensions`.
- Requirements companions are validated when present and remain optional for technical/internal
  modules under adaptive artifact policy.
- `validate_spec_content` applies normal single-spec validation to caller-provided spec bytes.
- `spec_path` remains the logical location for diagnostics and mapped-source resolution, but is not
  reopened to obtain spec content; adjacent companion reads are deliberately skipped for the
  pre-read spec-content API, while mapped sources retain normal path-based behavior.
- CRLF normalization and spec-size policy are computed from the supplied content.
- `validate_spec` preserves path-based compatibility by reading once and delegating the exact bytes
  to `validate_spec_content`.
- `SourceSnapshot` represents `Present`, `Missing`, `Rejected`, and `Unreadable` mapped-source
  observations.
- `validate_spec_content_with_sources` validates supplied spec bytes and supplied mapped-source
  observations without reopening either through ambient project paths.
- Supplied-content export extraction uses retained source bytes and does not resolve TypeScript
  wildcard imports through ambient paths.

### SPEC SECTION Public API

#### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `validate_spec` | `spec_path: &Path, root: &Path, schema_tables: &HashSet<String>, schema_columns: &HashMap<String, SchemaTable>, config: &SpecSyncConfig` | `ValidationResult` | Validate a single spec file: frontmatter, files, sections, API surface, and dependencies |
| `validate_spec_content` | `spec_path: &Path, content: &str, root: &Path, schema_tables: &HashSet<String>, schema_columns: &HashMap<String, SchemaTable>, config: &SpecSyncConfig` | `ValidationResult` | Validate already-read spec bytes without reopening the spec or adjacent companions; mapped sources retain normal path-based behavior |
| `find_spec_files` | `dir: &Path` | `Vec<PathBuf>` | Recursively find sorted `*.spec.md` files |
| `compute_coverage` | `root, spec_files, config` | `CoverageReport` | Compatibility file and LOC coverage computation |
| `compute_coverage_checked` | `root, spec_files, config` | `Result<CoverageReport, String>` | Checked coverage that surfaces malformed/unreadable manifest discovery |
| `get_schema_table_names` | `root, config` | `HashSet<String>` | Extract schema table names through the configured pattern |
| `is_cross_project_ref` | `dep: &str` | `bool` | Return whether a dependency is `owner/repo@module` |
| `parse_cross_project_ref` | `dep: &str` | `Option<(&str, &str)>` | Parse a cross-project reference into repository and module |
| `normalize_source_mapping` | `file: &str` | `Option<String>` | Normalize a safe portable project-relative source mapping |
| `source_within_root` | `root: &Path, file: &str` | `bool` | Return whether a source mapping remains beneath the project root |

#### Crate-Private Source-Snapshot API

| Item | Parameters / Variants | Returns | Description |
|------|------------------------|---------|-------------|
| `SourceSnapshot` | `Present(Vec<u8>)`, `Missing`, `Rejected`, `Unreadable` | — | Capability-confined observation of a mapped source |
| `validate_spec_content_with_sources` | `spec_path, content, root, schema_tables, schema_columns, config, sources: &HashMap<String, SourceSnapshot>` | `ValidationResult` | Validate supplied spec bytes and supplied mapped-source observations without ambient spec/source reopening |

### SPEC SECTION Invariants

1. Validation is bidirectional: phantom documented exports are errors and undocumented code exports
   are warnings.
2. Missing required frontmatter fields are errors.
3. Cross-project references are skipped during local validation.
4. Coverage excludes tests and configured patterns.
5. Source discovery honors configured extensions.
6. Spec discovery is sorted.
7. Schema extraction honors the configured pattern.
8. Missing-file suggestions use bounded Levenshtein distance.
9. Flat source-module detection excludes common entry points.
10. Empty required sections are reported as unfinished content.
11. Validation results retain parsed lifecycle status.
12. Requirements companions are validated when present under adaptive artifact policy.
13. Checked coverage propagates malformed, unreadable, unsupported, or unconfined manifest
    discovery; compatibility coverage remains available. Coverage source enumeration and content
    reads remain bound to one retained project-root capability after manifest discovery, and any
    replacement or non-regular endpoint makes the checked result inconclusive.
14. `validate_spec_content` validates caller-provided bytes without reopening `spec_path` or
    adjacent companions; the path remains logical diagnostic/source context and mapped sources
    retain normal path behavior.
15. `validate_spec` reads a path once and delegates the exact bytes to the shared content validator.
16. `validate_spec_content_with_sources` treats its supplied `SourceSnapshot` map as authoritative
    and does not reopen mapped sources or resolve supplied-content TypeScript wildcards through
    ambient paths.
17. Checked coverage uses retained no-follow source snapshots; symlink, reparse, or identity
    replacement fails before outside reads, partial totals, or generation.
18. Caller-selected spec ownership, manifest/spec-module/source discovery, and final verification
    share one retained project capability; deterministic iterative traversal enforces 8 MiB/file,
    64 MiB total, 100,000 entries, 256 components, strict UTF-8, and special-entry rejection.
