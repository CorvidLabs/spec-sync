---
module: validator
version: 30
status: stable
files:
  - src/validator.rs
db_tables: []
tracks: [119]
depends_on:
  - specs/config/config.spec.md
  - specs/exports/exports.spec.md
  - specs/parser/parser.spec.md
  - specs/schema/schema.spec.md
  - specs/types/types.spec.md
  - specs/util/util.spec.md
---

# Validator

## Purpose

Core validation engine for spec-sync. It validates specs and companions bidirectionally against
source, compares schema declarations through canonical checked-snapshot identities, computes
non-vacuous retained-capability coverage, and preserves malformed discovery as an inconclusive gate
instead of a false success.

## Public API

#### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `validate_spec` | `spec_path: &Path, root: &Path, schema_tables: &HashSet<String>, schema_columns: &HashMap<String, SchemaTable>, config: &SpecSyncConfig` | `ValidationResult` | Validate a single spec file: frontmatter, files, sections, API surface, and dependencies |
| `validate_spec_content` | `spec_path: &Path, content: &str, root: &Path, schema_tables: &HashSet<String>, schema_columns: &HashMap<String, SchemaTable>, config: &SpecSyncConfig` | `ValidationResult` | Validate already-read spec bytes without reopening the spec or adjacent companions; mapped sources retain normal path-based behavior |
| `find_spec_files` | `dir: &Path` | `Vec<PathBuf>` | Recursively find sorted `*.spec.md` files |
| `load_config_and_discover_retained` | `root: &Path` | `Result<(SpecSyncConfig, Vec<PathBuf>), String>` | Crate-private bounded retained-root configuration and spec inventory used by CLI commands |
| `compute_coverage` | `root, spec_files, config` | `CoverageReport` | Compatibility file and LOC coverage computation |
| `compute_coverage_checked` | `root, spec_files, config` | `Result<CoverageReport, String>` | Checked coverage that surfaces malformed/unreadable manifest discovery |
| `get_schema_table_names` | `root, config` | `HashSet<String>` | Extract schema table names through the configured pattern |
| `schema_table_names_from_snapshot` | `snapshot, config` | `Result<HashSet<String>, String>` | Derive canonical replay/pattern identities from the invocation snapshot |
| `schema_config_problems_for_snapshot` | `config, optional snapshot` | `Vec<String>` | Surface invalid patterns and vacuous requested schema validation |
| `is_cross_project_ref` | `dep: &str` | `bool` | Return whether a dependency is `owner/repo@module` |
| `parse_cross_project_ref` | `dep: &str` | `Option<(&str, &str)>` | Parse a cross-project reference into repository and module |
| `normalize_source_mapping` | `file: &str` | `Option<String>` | Normalize a safe portable project-relative source mapping |
| `source_within_root` | `root: &Path, file: &str` | `bool` | Return whether a source mapping remains beneath the project root |

#### Crate-Private Source-Snapshot API

| Item | Parameters / Variants | Returns | Description |
|------|------------------------|---------|-------------|
| `SourceSnapshot` | `Present(Vec<u8>)`, `Missing`, `Rejected`, `Unreadable` | — | Capability-confined observation of a mapped source |
| `validate_spec_content_with_sources` | `spec_path, content, root, schema_tables, schema_columns, config, sources: &HashMap<String, SourceSnapshot>` | `ValidationResult` | Validate supplied spec bytes and supplied mapped-source observations without ambient spec/source reopening |
| `validate_local_dependency` | Shared confined dependency verdict |

## Invariants

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
15. Schema-aware validation compares quoted, qualified, and mixed-case declarations canonically;
    invalid patterns, missing configured snapshots, and declared tables without schema evidence
    remain visible findings rather than vacuous success.
15. `validate_spec` reads a path once and delegates the exact bytes to the shared content validator.
16. `validate_spec_content_with_sources` treats its supplied `SourceSnapshot` map as authoritative
    and does not reopen mapped sources or resolve supplied-content TypeScript wildcards through
    ambient paths.
17. Checked coverage uses retained no-follow source snapshots; symlink, reparse, or identity
    replacement fails before outside reads, partial totals, or generation.
18. Caller-selected spec ownership, manifest/spec-module/source discovery, and final verification
    share one retained project capability; deterministic iterative traversal enforces 8 MiB/file,
    64 MiB total, 100,000 entries, 256 components, strict UTF-8, and special-entry rejection.
19. Configuration and zero-config source detection begin after root retention; selected-spec/source
    bytes and entries share one budget, nested config/manifest parents remain reachable, explicit
    source roots avoid autodetection, and retained identities remain authoritative.
20. Early and post-discovery race checkpoints independently cover retained acquisition and later
    traversal inside checked coverage; callers propagate those failures without claiming
    command-wide retained authority.
21. Checked spec/source traversal records sibling identities and reopens children sequentially
    through retained parents, bounding live handles by depth while preserving identity and
    reachability checks around recursion.
22. Configured source-root selection stores stable identities without retaining every root handle;
    traversal reopens, identity-checks, consumes, and releases each root sequentially.

## Behavioral Examples

### Scenario: Valid spec passes

- **Given** a spec with correct frontmatter, all required sections, and API table matching code exports
- **When** `validate_spec` is called
- **Then** returns `ValidationResult` with empty errors and warnings

### Scenario: Spec documents non-existent export

- **Given** a spec listing `` `nonExistent` `` in the Public API table
- **When** `validate_spec` is called
- **Then** errors include "Spec documents 'nonExistent' but no matching export found in source"

### Scenario: Undocumented code export

- **Given** source code exports `helperFn` but the spec does not list it
- **When** `validate_spec` is called
- **Then** warnings include "Export 'helperFn' not in spec (undocumented)"

### Scenario: Cross-project dependency reference

- **Given** a spec with `depends_on: ["corvid-labs/algochat@auth"]`
- **When** `validate_spec` is called locally
- **Then** the cross-project ref is skipped (no error or warning)

### Scenario: Malformed Gradle settings make coverage inconclusive

- **Given** a configured source tree and malformed `settings.gradle` or `settings.gradle.kts`
- **When** `compute_coverage_checked` is called by a CLI or MCP gate
- **Then** it returns an error and the caller reports coverage as inconclusive instead of accepting partial totals

### Scenario: Gradle-derived source root is not confined

- **Given** Gradle settings contain a raw drive-qualified module identity, unescaped interpolation,
  encoded traversal, an unsupported dynamic `setProjectDir`, or an effective directory with a
  symlink/reparse component
- **When** `compute_coverage_checked` is called by a CLI or MCP gate
- **Then** it returns an error before source traversal and the caller reports an inconclusive
  outcome instead of accepting partial or outside coverage

### Scenario: Gradle manifest is not confined

- **Given** a present Gradle build/settings manifest is linked, reparse-backed, non-regular, or oversized
- **When** `compute_coverage_checked` is called by a CLI or MCP gate
- **Then** it returns an error before referent reads and the caller reports an inconclusive outcome

### Scenario: Validate a retained spec snapshot

- **Given** a caller already read spec bytes through a confined capability and the ambient
  `spec_path` is later replaced
- **When** `validate_spec_content(spec_path, content, ...)` is called
- **Then** validation uses only `content` for the spec and does not reopen the replaced path or
  adjacent companion files; mapped sources retain the normal path-based behavior

### Scenario: Validate retained spec and source snapshots

- **Given** a caller retained spec bytes and mapped-source observations, then the ambient spec and
  source paths are replaced
- **When** `validate_spec_content_with_sources(spec_path, content, ..., sources)` is called
- **Then** validation uses only the supplied spec bytes and `SourceSnapshot` map, never reopens
  either path, and extracts exports from supplied source content without ambient wildcard imports

### Scenario: Coverage root is replaced after manifest discovery

- **Given** checked coverage retained the project directory, then the ambient project path or a
  discovered source directory is replaced with a symlink, junction, or different regular entry
- **When** source, spec-module, and manifest coverage discovery completes
- **Then** the operation returns an inconclusive error without reading replacement bytes or
  publishing partial file, LOC, or module totals

### Scenario: Zero-config discovery retains authority

- **Given** no source directories are configured and a recognized manifest is replaced after the
  project capability is retained
- **When** checked coverage autodetects source directories
- **Then** autodetection consumes only retained manifest/config observations and fails
  inconclusively on identity replacement instead of accepting attacker-selected source roots

### Scenario: Both checked-coverage race checkpoints are enforced

- **Given** deterministic fixtures pause once immediately after root retention and once after
  discovery
- **When** a gate caller replaces a selected input at the corresponding checked-coverage checkpoint
- **Then** checked coverage returns an inconclusive error without outside reads or partial coverage
  output, and the caller propagates that failure

### Scenario: Coverage traversal exceeds a deterministic bound

- **Given** configured sources exceed 8 MiB per file, 64 MiB cumulatively, 100,000 entries, 256
  components, or contain invalid UTF-8 in a supported source name/content
- **When** `compute_coverage_checked` runs
- **Then** it returns an inconclusive error before reporting a percentage

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Spec file unreadable | Error: "Cannot read spec" |
| Pre-read snapshot passed to `validate_spec_content` | Validates those exact bytes without opening `spec_path` or adjacent companions |
| Supplied source is `Missing`, `Rejected`, or `Unreadable` | Reports the corresponding mapped-source validation outcome without ambient fallback |
| Missing frontmatter delimiters | Error: "Missing or malformed YAML frontmatter" |
| Source file not found | Error with fix suggestion (Levenshtein-based or removal) |
| DB table not in schema | Error: "DB table not found in schema" |
| Missing required section | Error: "Missing required section: ## SectionName" |
| Dependency spec not found | Error: "Dependency spec not found" |
| Malformed, unreadable, unsupported, or unconfined Gradle discovery during checked coverage, including unsafe Gradle manifest entries | Returns `Err`; CLI/MCP gate callers report an inconclusive failure rather than coverage success, referent reads, or outside traversal |
| Coverage selected-spec/source input is linked/reparse-backed, special, replaced, invalid UTF-8, over 8 MiB, or shared traversal exceeds 64 MiB/100,000 entries/256 components | Returns `Err` from the retained project snapshot; no partial totals or ambient fallback |
| Caller-selected coverage spec is outside the retained project, missing, linked/reparse-backed, special, replaced, invalid UTF-8, or over the shared coverage input bounds | Returns `Err`; ownership mappings are never obtained through the ambient spec pathname |
| Valid checked coverage contains more sibling spec/source directories than the process descriptor limit | Children are reopened sequentially through retained parents; coverage completes with handles bounded by traversal depth |
| Valid checked coverage configures more source roots than the process descriptor limit | Root identities are selected without retaining handles, then each root is reopened, identity-checked, traversed, and released sequentially |

## Dependencies

**Consumes**

| Module | What is used |
|--------|-------------|
| parser | `parse_frontmatter`, `get_spec_symbols`, `get_missing_sections` |
| exports | `get_exported_symbols`, `get_exported_symbols_from_content`, `has_extension`, `is_test_file` |
| config | `default_schema_pattern`, `discover_manifest_modules_checked` |
| types | `CoverageReport`, `ValidationResult`, `SpecSyncConfig` |

**Consumed By**

| Module | What is used |
|--------|-------------|
| main | `validate_spec`, `validate_spec_content`, `SourceSnapshot`, `validate_spec_content_with_sources`, `find_spec_files`, `compute_coverage_checked`, `get_schema_table_names` |
| mcp | `validate_spec`, `find_spec_files`, `compute_coverage_checked`, `get_schema_table_names` |
| archive | `find_spec_files` to locate spec companion files |
| compact | `find_spec_files` to locate all spec files |
| merge | `find_spec_files` to locate all spec files when `--all` is used |

**Frontmatter Synchronization**

Implementation SHALL add these canonical dependency specs to `depends_on`: `specs/util/util.spec.md`. This YAML frontmatter update is an explicit implementation edit because semantic section deltas do not apply frontmatter.

## Change Log

| Date | Change |
|------|--------|
| 2026-07-22 | v12: add checked coverage so malformed Gradle discovery fails CLI and MCP gates as inconclusive while retaining the compatibility report wrapper |
| 2026-07-22 | v13 / CHG-0063: add `validate_spec_content` so capability-rooted callers validate exact pre-read snapshots without reopening spec paths |
| 2026-07-23 | v14 / CHG-0063 independent review: keep raw drive-qualified modules, unsupported `setProjectDir` forms, and linked/reparse-backed Gradle source roots inconclusive across CLI and MCP gates |
| 2026-07-23 | v15 / CHG-0063 adversarial rereview: keep linked/reparse-backed Gradle manifests plus interpolated or encoded Gradle paths inconclusive across checked gates |
| 2026-07-23 | v16 / CHG-0063 final security rereview: snapshot coverage roots and bytes through retained no-follow handles so post-discovery symlink/junction replacement is inconclusive before outside reads |
| 2026-07-23 | v17 / CHG-0063 post-review hardening: share one retained project authority across manifest/spec/source coverage and enforce deterministic iterative byte, entry, depth, UTF-8, and identity bounds |
| 2026-07-23 | v18 / CHG-0063 acceptance remediation: read caller-selected spec ownership frontmatter and every recognized manifest through the same retained project authority before source coverage |
| 2026-07-23 | v19 / CHG-0063 exact-head review remediation: preserve nested config/manifest reachability and selected-spec identity continuity, lazily autodetect omitted source roots, and charge selected-spec/source bytes and entries within checked coverage |
| 2026-07-24 | v20 / CHG-0063 independent rereview remediation: Preserve bounded scan fallback after malformed manifest autodetection and retain selected source-directory identities from post-manifest selection through checked coverage traversal |
| 2026-07-24 | v21 / CHG-0063 exact-head rereview remediation: Replace retained sibling handles with identity records and sequential capability reopen so broad checked coverage remains descriptor-bounded |
| 2026-07-24 | v22 / CHG-0063 descriptor-breadth remediation: Select configured source-root identities without retaining all handles, then reopen and traverse roots sequentially so breadth cannot exhaust descriptors |
| 2026-07-10 | v5: keep coverage regression fixtures warning-free under current stable Clippy and document the intentionally in-file test-module layout |
| 2026-07-10 | v5: make canonical requirements companions adaptive rather than empty mandatory ceremony |
| 2026-07-02 | v4: add `source_within_root` — shared guard rejecting `files:` paths that escape the project root (absolute/`..`/symlink); applied in `validate_spec` and every export-extraction site (score, check --fix, diff, new) to close an out-of-root identifier-disclosure vector |
| 2026-06-11 | v3: `validate_spec` populates `ValidationResult.status` with the parsed lifecycle status so callers can report draft skips |
| 2026-06-07 | Update draft-only section warning wording |
| 2026-03-25 | Initial spec |
| 2026-04-06 | Document archive, compact, merge as consumers of find_spec_files; note hash_cache integration for incremental validation |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-14 | CHG-0024-stabilize-specsync-5-lifecycle-integrity-and-strict-validation-for-5-0-2: Stabilize SpecSync 5 lifecycle integrity and strict validation for 5.0.2 |
| 2026-07-14 | CHG-0025-address-all-unresolved-review-feedback-on-pr-366: Address all unresolved review feedback on PR 366 |
| 2026-07-14 | CHG-0034-support-extensionless-source-discovery-through-an-explicit-include-extensionless: Support extensionless source discovery through an explicit include_extensionless setting while preserving omitted and empty source_extensions defaults, with parser, scanner, strict file coverage, LOC coverage, and wizard regressions for extensionless-only and mixed projects |
| 2026-07-14 | CHG-0035-count-mjs-and-cjs-files-as-default-typescript-sources-so-mapped-and-uncovered-mo: Count mjs and cjs files as default TypeScript sources so mapped and uncovered module files contribute to strict file and LOC coverage denominators |
| 2026-07-14 | CHG-0039-allow-draft-specs-to-declare-planned-missing-source-mappings-without-failing-str: Allow draft specs to declare planned missing source mappings without failing strict validation while preserving path safety ownership enforcement exact coverage and complete notice contracts |
| 2026-07-27 | CHG-0063-close-independent-mcp-security-review-gaps-for-issue-414: Close independent MCP security review gaps for issue 414 |
| 2026-07-30 | CHG-0068-stabilize-specsync-6-0-with-a-low-churn-normal-workflow-preserved-audited-guara: Stabilize SpecSync 6.0 with one scope approval, same-PR finalization, lightweight archive CI, scoped review, and selected UX fixes |
| 2026-07-31 | CHG-0070-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes: Land pre-6.0 product fixes for hooks init coverage naming and exit codes |
| 2026-08-01 | CHG-0071-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes-scoped: Land pre-6.0 product fixes for hooks init coverage naming and exit codes (scoped paths) |
| 2026-08-13 | CHG-0107-fix-the-first-five-minutes-of-spec-sync-init-leaves-a-repo-that-fails-check-sc: Fix the first five minutes of spec-sync: init leaves a repo that fails check, scaffold writes prose that check rejects, and a directory in files: makes check silently green |
| 2026-08-13 | CHG-0108-stop-reporting-success-for-checks-that-did-not-happen-gate-drafts-that-document: Stop reporting success for checks that did not happen: gate drafts that document a contract over present source, drop cold-cache drift noise, and stop taking quoted frontmatter paths literally |
| 2026-08-13 | CHG-0109-a-symlink-under-a-source-directory-must-be-skipped-and-disclosed-never-abort-di: A symlink under a source directory must be skipped and disclosed, never abort discovery |
| 2026-08-14 | CHG-0117-a-config-file-that-exists-but-cannot-be-loaded-must-refuse-to-run-not-report-su: A config file that exists but cannot be loaded must refuse to run, not report success over built-in defaults |
