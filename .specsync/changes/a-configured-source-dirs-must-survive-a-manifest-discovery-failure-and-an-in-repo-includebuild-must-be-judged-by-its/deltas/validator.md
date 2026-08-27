## MODIFIED

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
    discovery WHEN the source list it would measure came from that discovery — that is, when
    `source_dirs` was not stated. A project that stated `source_dirs` is not overruled by a failure
    to infer what it already said: discovery degrades to an empty result and the error is carried
    as a coverage notice, never as a veto. The notice is not optional, because manifest modules
    also seed module attribution, so a degraded run reports fewer modules without specs than the
    tree holds. Compatibility coverage remains available. Coverage source enumeration and content
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

### SPEC SECTION Error Cases

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
| Malformed, unreadable, unsupported, or unconfined Gradle discovery during checked coverage with `source_dirs` unstated, including unsafe Gradle manifest entries | Returns `Err`; CLI/MCP gate callers report an inconclusive failure rather than coverage success, referent reads, or outside traversal |
| The same discovery failure with `source_dirs` explicitly stated | Coverage runs over the stated list and returns `Ok`; the error is reported as a `manifest_notices` entry beside the figures, and the manifest contributes no modules |
| Coverage selected-spec/source input is linked/reparse-backed, special, replaced, invalid UTF-8, over 8 MiB, or shared traversal exceeds 64 MiB/100,000 entries/256 components | Returns `Err` from the retained project snapshot; no partial totals or ambient fallback |
| Caller-selected coverage spec is outside the retained project, missing, linked/reparse-backed, special, replaced, invalid UTF-8, or over the shared coverage input bounds | Returns `Err`; ownership mappings are never obtained through the ambient spec pathname |
| Valid checked coverage contains more sibling spec/source directories than the process descriptor limit | Children are reopened sequentially through retained parents; coverage completes with handles bounded by traversal depth |
| Valid checked coverage configures more source roots than the process descriptor limit | Root identities are selected without retaining handles, then each root is reopened, identity-checked, traversed, and released sequentially |
