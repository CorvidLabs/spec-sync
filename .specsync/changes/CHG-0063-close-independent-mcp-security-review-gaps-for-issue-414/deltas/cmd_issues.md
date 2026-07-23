## MODIFIED

### REQUIREMENT REQ-cmd-issues-001

The issues command SHALL verify tracked GitHub references and SHALL report valid, closed, missing,
and unverifiable states predictably.

Acceptance Criteria

- References from all specs are verified in one globally deduplicated batch of at most 100 unique
  issue IDs.
- Confirmed issues are classified as valid, closed, or not_found; repository, authentication,
  transport, timeout, and malformed-provider failures are errors.
- Any batch/provider error contributes to the existing non-zero command outcome.
- Human-readable output uses gathered-reference count to distinguish an empty project from an
  all-error batch, and the latter summary includes its error count.
- Repository/provider resolution occurs only after inspection. Empty-reference projects skip Git
  auto-detection and provider access, but configured repository syntax is still validated even
  when the specs directory is missing or contains no specs.
- A present selected project config is opened through the retained project capability, must be a
  non-link regular entry, is identity-checked through one bounded 4 MiB same-handle read, and is
  parsed/applied from those exact bytes. Invalid UTF-8, malformed JSON/TOML, or wrong-shaped known
  TOML fields are structured content-free findings that exit 1 without fallback.
- Omitted source directories are detected from a bounded sparse snapshot built through the
  retained project capability and supplied to exact config parsing; ambient root replacement
  cannot alter source selection.
- Unreadable specs and malformed or missing frontmatter are retained as path-attributed,
  content-free inspection findings in every output format, suppress no-reference guidance, and
  contribute to exit 1.
- Spec discovery and reads are rooted in retained project/spec-directory capabilities, and each
  immutable snapshot keeps its discovered identity through read completion; symlink, regular-file,
  and hardlink replacement cannot authorize replacement bytes.
- The retained project capability used for spec discovery is reused for mapped-source snapshots;
  ambient project-root replacement cannot mix distinct project identities.
- Spec retention is capped at 10,000 files, 4 MiB per spec, and 64 MiB cumulatively;
  mapped-source retention is capped at 4 MiB per source and 64 MiB cumulatively.
- Recursive discovery examines at most 100,000 total entries, including non-spec entries, before
  returning an inconclusive bounded finding.
- Issue fields use maintained real-YAML checked parsing: duplicate/global malformed YAML and
  blank/null/wrong-shaped known fields fail closed; comments/trailing commas remain valid; nested
  extension and block-scalar lookalikes are ignored; LF and CRLF frontmatter delimiters are
  accepted equivalently.
- Renderer boundaries escape controls, bidi formatting characters, and Unicode Zl/Zp separators;
  Markdown/GitHub preserve valid escaped table rows and code spans, padding span content when a
  path begins or ends with a backtick.
- Windows finding paths use forward slashes; Unix literal backslashes remain filename data.
- Missing/empty specs and repository-resolution errors render through the selected output format;
  JSON remains parseable and Markdown/GitHub remain structured.
- `--create` validates retained spec and mapped-source snapshots through
  `validate_spec_content_with_sources` without reopening discovered paths or resolving
  supplied-content TypeScript wildcard imports through ambient paths, then preserves normal
  drift-issue creation for validation errors.

### SPEC SECTION Invariants

1. The command gathers all `implements` and `tracks` references before repository/provider access.
2. Project-wide verification is globally deduplicated, capped, and time-bounded by the GitHub
   module.
3. Inconclusive provider outcomes remain errors and cannot become successful not_found results.
4. No-reference guidance is emitted only when no spec references were gathered.
5. An empty reference set performs no Git auto-detection or provider access; configured repository
   syntax is still validated even when no specs were discovered.
6. A scan is empty only when every discovered spec was read and parsed successfully; unreadable or
   malformed specs are retained as safe findings and make verification inconclusive.
7. Recursive discovery and reads remain capability-rooted, and parsing consumes immutable bytes
   whose discovered identity remains binding through read, including regular/hardlink replacement.
8. Every output renderer escapes control, bidi, and Unicode line/paragraph separator characters.
9. Markdown/GitHub code spans pad content when a path begins or ends with a backtick.
10. Discovery retains at most 10,000 specs, at most 4 MiB per spec, and at most 64 MiB
    cumulatively; mapped-source retention applies a 4 MiB per-file and 64 MiB cumulative ceiling.
11. `--create` validates retained spec/source snapshots through
    `validate_spec_content_with_sources` and never reopens discovered paths or ambient wildcard
    targets.
12. Spec and mapped-source reads derive from one retained project capability, and recursive
    discovery examines no more than 100,000 total entries including non-spec entries.
13. Selected config is retained, same-handle identity-checked, bounded to 4 MiB, and parsed from
    exact bytes; malformed, wrong-shaped, linked, non-regular, replaced, or oversized input cannot
    produce fallback no-spec/no-reference success.
14. Finding paths normalize separators only on Windows; Unix literal backslashes remain data.
15. Missing/empty specs and repository-resolution failures use the selected structured renderer.
16. Omitted source directories are detected through a bounded sparse snapshot rooted in the
    retained project capability, never through a replaceable ambient root pathname.
