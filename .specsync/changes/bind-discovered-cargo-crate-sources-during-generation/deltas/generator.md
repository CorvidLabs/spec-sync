## MODIFIED

### SPEC SECTION Change Log

| Date | Change |
|------|--------|
| 2026-10-11 | #813: Bind discovered Cargo sources through coverage while preserving more specific modules and existing ownership. |
| 2026-10-10 | Add `generator.3md` beside the spec. The first plane is context and later planes are lessons. `context.md` stays. |
| 2026-07-26 | v7 / CHG-0063: Bind CLI generation templates, directories, and no-overwrite file publication to one retained project-root capability |
| 2026-07-10 | v5: keep module-discovery test fixtures warning-free under current stable Clippy |
| 2026-03-25 | Initial spec |
| 2026-04-07 | Document find_files_for_module, generate_spec, generate_spec_from_custom_template, generate_companion_files_from_template |
| 2026-04-12 | Update companion files list to include requirements.md, testing.md, and opt-in design.md; add design_enabled parameter |
| 2026-04-13 | Fix generate_companion_files_from_template signature to include design_enabled; update scenario for conditional design.md |
| 2026-06-07 | Replace unfinished-marker built-in template content with guided starter content |
| 2026-06-11 | Return `GenerationOutcome` (count, paths, AI errors) from both generation entry points so AI failures surface with a non-zero exit |
| 2026-06-11 | Add `find_single_source_fallback` so `new`/`scaffold` auto-detect the source in single-source-file projects (e.g. a fresh cargo crate with only `src/lib.rs`) |
| 2026-07-11 | CHG-0007-harden-specsync-5-0-as-an-agent-native-secret-free-sdd-core-and-close-release-r: Harden SpecSync 5.0 as an agent-native, secret-free SDD core and close release regressions |
| 2026-07-27 | CHG-0063-close-independent-mcp-security-review-gaps-for-issue-414: Close independent MCP security review gaps for issue 414 |
| 2026-08-01 | CHG-0071-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes-scoped: Land pre-6.0 product fixes for hooks init coverage naming and exit codes (scoped paths) |
| 2026-08-13 | CHG-0107-fix-the-first-five-minutes-of-spec-sync-init-leaves-a-repo-that-fails-check-sc: Fix the first five minutes of spec-sync: init leaves a repo that fails check, scaffold writes prose that check rejects, and a directory in files: makes check silently green |
| 2026-08-13 | CHG-0109-a-symlink-under-a-source-directory-must-be-skipped-and-disclosed-never-abort-di: A symlink under a source directory must be skipped and disclosed, never abort discovery |
| 2026-08-14 | CHG-0121-coverage-over-zero-source-files-must-report-nothing-measured-everywhere-replac: Coverage over zero source files must report nothing measured, everywhere: replace the precomputed percentage fields with Option-returning accessors so no renderer can substitute 100 percent for an unasked question |
| 2026-08-15 | CHG-0128-every-command-that-derives-a-module-s-api-must-honour-the-configured-export-leve: Every command that derives a module's API must honour the configured export level and parse mode, so check, score, new, generate, scaffold and diff cannot disagree about what the API is |
| 2026-08-24 | close-the-lessons-loop-surface-what-a-module-already-learned-at-proposal-name-where-a-lesson-goes-when-a-build-fails: Close the lessons loop: surface what a module already learned at proposal, name where a lesson goes when a build fails, and assemble the archived bundle at finalize |
| 2026-08-24 | frontmatter-stripping-and-scaffold-detection-must-survive-crlf-and-an-unexpanded-module-placeholder: Frontmatter stripping and scaffold detection must survive CRLF and an unexpanded module placeholder |
| 2026-08-27 | v16 / #723: `CoverageReport` also carries manifest discovery that was degraded rather than allowed to veto an explicitly configured `source_dirs` |
| 2026-08-27 | a-configured-source-dirs-must-survive-a-manifest-discovery-failure-and-an-in-repo-includebuild-must-be-judged-by-its: A configured source_dirs must survive a manifest discovery failure, and an in-repo includeBuild must be judged by its path rather than its token |
| 2026-08-27 | a-configured-source-dirs-must-survive-a-manifest-discovery-failure-and-an-in-repo-includebuild-must-be-judged-by-its: A configured source_dirs must survive a manifest discovery failure, and an in-repo includeBuild must be judged by its path rather than its token |
| 2026-09-09 | close-the-specsync-6-0-0-p1-release-defects-found-in-overnight-proving: Close the SpecSync 6.0.0 P1 release defects found in overnight proving |

### SPEC SECTION Invariants

- CLI generation confines template reads, directory creation, and no-overwrite publication to one
  retained project-root capability so public-root replacement cannot redirect output.


- Manifest fallback binds only uncovered files observed by coverage; configured mappings and more specific named modules take precedence.

### SPEC SECTION Behavioral Examples

### Scenario: Generate spec for unspecced module

- **Given** a module "auth" with source files in `src/auth/` and no existing spec
- **When** `generate_specs_for_unspecced_modules` is called
- **Then** creates `specs/auth/auth.spec.md`, `specs/auth/tasks.md`, `specs/auth/context.md`, `specs/auth/requirements.md`, `specs/auth/testing.md`, and `specs/auth/design.md` if `companions.design` is enabled in config

### Scenario: Skip existing spec

- **Given** a module "auth" that already has `specs/auth/auth.spec.md`
- **When** `generate_specs_for_unspecced_modules` is called
- **Then** skips the module, returns an outcome with `generated == 0`

### Scenario: Design companion opt-in

- **Given** `companions.design` is enabled in config
- **When** `generate_companion_files_for_spec` is called for module "dashboard"
- **Then** creates design.md with YAML frontmatter (`spec: dashboard.spec.md`, `sources: []`) and sections for Layout, Components, Tokens, Assets

### Scenario: Design companion disabled by default

- **Given** no `companions.design` config (default: false)
- **When** `generate_companion_files_for_spec` is called
- **Then** creates tasks.md, context.md, requirements.md, testing.md but NOT design.md


### Scenario: Generate a fresh Cargo library

- **Given** package `demo` with `src/lib.rs` exporting `add`, initialized with `specsync init`
- **When** generation runs in text, JSON, or batch mode
- **Then** `specs/demo/demo.spec.md` binds `src/lib.rs` and documents `add`. Existing spec owners retain their files.

## ADDED

### REQUIREMENT REQ-generator-006

The generator SHALL bind a manifest-discovered Cargo module to its measured uncovered source files when conventional module-name paths do not resolve, preserving explicit configuration, existing spec ownership, and more specific named module bindings.

Acceptance Criteria

- The issue #813 init then generate fixture creates specs/demo/demo.spec.md with src/lib.rs and add.
- Text, JSON, batch, and library generation entry points use the same binding.
- Library/binary roots and workspace members stay scoped to their declared sources.
- Files already referenced by specs and files belonging to other named modules are not claimed by the fallback.
- Missing configured modules still report skipped_no_files.

