## ADDED

### REQUIREMENT REQ-types-004

SpecSync SHALL represent every `depends_on` value with one shared typed dependency-reference model
that preserves the original declaration for diagnostics and exposes a normalized identity for
deduplication.

Acceptance Criteria

- The model distinguishes a bare local module, a project-relative local spec path, and a remote
  `owner/repository@module` reference.
- Empty values, missing remote owners, repositories, or modules (including `repo@`), absolute
  paths, drive or UNC paths, backslash-separated paths, `..` traversal, invalid spec paths, and
  invalid module or repository identifiers are rejected with the original text in the diagnostic.
- Local path resolution is rooted at the canonical project root and rejects lexical escapes,
  symlink escapes, and missing-leaf paths whose nearest existing ancestor escapes.
- Bare modules resolve through an explicit local registry mapping first, then the canonical
  `specs/<module>/<module>.spec.md` location; a same-named directory without a spec is not success.
- Equivalent repeated declarations are deduplicated by normalized identity while preserving the
  first declaration order and raw spelling for output.
- Check, dependency graph, resolution, scoring, and MCP consumers use this parser and resolver; no
  consumer retains an independent delimiter, path-join, or filename-stem interpretation.

## MODIFIED

### SPEC SECTION Invariants

1. Shared types contain no inference-provider or credential-bearing configuration surface.
2. `Language::from_extension` returns `None` for unsupported extensions and never panics.
3. `SpecSyncConfig::default()` always provides sensible deterministic defaults.
4. `ValidationResult::new` initializes empty error, warning, fix, and notice vectors with
   `status: None`.
5. Frontmatter diagnostics retain a stable kind, message, optional field, and optional source line
   and remain deterministically ordered.
6. `DependencyRef` preserves exact raw spelling while distinguishing bare local modules,
   project-relative local spec paths, and `owner/repository@module` references.
7. Dependency syntax validation is platform-portable and rejects empty, malformed, absolute,
   drive/UNC, backslash, traversal, and invalid-identifier forms before filesystem or network use.
8. Normalized dependency identity is used for deterministic deduplication only; diagnostics and
   structured output retain the first declaration's raw spelling.
9. Local dependency resolution is rooted at the canonical project capability, consults explicit
   registry mappings before conventional locations, and rejects lexical, symlink, and
   existing-ancestor escapes.
