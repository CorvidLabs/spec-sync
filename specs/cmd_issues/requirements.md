---
spec: cmd_issues.spec.md
---

## User Stories

- As a maintainer, I want `specsync issues` to verify every GitHub issue referenced in spec frontmatter so that I catch specs pointing at closed, deleted, or wrong issues.
- As a CI operator, I want the command to exit non-zero when references are broken so that the pipeline fails on stale links.
- As a maintainer, I want `--create` to open drift issues for specs that fail validation so that drift gets tracked in the issue tracker.
- As a developer, I want machine-readable JSON/Markdown output so that I can post results in CI summaries.

## Acceptance Criteria

- Reads each spec's `implements:` and `tracks:` only as lists of numeric issue IDs; wrong shapes or
  invalid entries are inspection findings, while specs with neither field are skipped.
- After gathering references, a configured repository is syntax-validated in every case; without a
  configured repository, Git auto-detection occurs only when at least one reference exists. An
  unresolvable required repo prints an error and exits 1.
- All references are verified in one globally deduplicated batch of at most 100 unique issue IDs;
  confirmed issues are classified as valid/closed/not-found while repository, authentication,
  transport, timeout, or malformed-provider failures are errors.
- Per-format output: Text/Table/Csv print per-spec details, safe inspection findings, and a one-line
  summary; Json emits totals plus `inspection_findings`, a content-free `findings` array, and the
  `specs` array; Markdown/Github emit a metric table plus a findings table when needed.
- Text/Table/Csv no-reference guidance appears only when no references were gathered; all-error
  batches include their error count in the summary.
- An empty reference set performs no Git auto-detection or provider access. If `github.repo` is
  configured, its exact `owner/repository` syntax is validated even when no references or no specs
  exist; malformed configured identity fails instead of producing no-spec/no-reference success.
- Spec discovery and reads remain rooted in retained project/spec-directory capabilities, and each
  snapshot is read and identity-checked through the same verified file handle from discovery
  through read completion, rejecting symlink, regular-file, and hardlink replacement.
- The retained project capability used to open the specs directory is also the sole authority for
  mapped-source snapshots, so replacing the ambient project-root name cannot mix different project
  identities in one validation result.
- Discovery retains at most 10,000 spec snapshots, at most 4 MiB per spec, and at most 64 MiB of
  spec bytes cumulatively; mapped-source retention for `--create` is independently capped at
  4 MiB per source and 64 MiB cumulatively.
- Recursive discovery examines at most 100,000 total directory entries, including non-spec files
  and directories, before returning an inconclusive bounded finding.
- Checked real-YAML parsing rejects duplicate/global malformed YAML and blank/null/wrong-shaped
  top-level issue fields, accepts comments/trailing commas, and ignores nested extension or
  block-scalar lookalikes; LF and CRLF frontmatter delimiters are accepted equivalently.
- With `--create`, validation passes retained spec bytes and capability-confined mapped-source
  observations to `validate_spec_content_with_sources` and creates drift issues for resulting
  errors; spec/source paths are not reopened and supplied-content TypeScript wildcard exports do
  not resolve through ambient paths.
- Unreadable specs and malformed or missing frontmatter are retained as path-attributed,
  content-free inspection findings; they never disappear into the empty-reference success path.
- Recursive spec discovery is checked and traversal failures are retained as inconclusive findings.
- Finding paths are project-relative, content-free, and safe for each renderer: terminal controls
  plus bidirectional formatting controls and Unicode Zl/Zp separators are not emitted raw, JSON
  remains parseable, and Markdown/GitHub escapes table syntax and uses a valid code-span delimiter
  for filenames containing backticks, padding span content when a filename begins or ends with a
  backtick. Windows path separators render as forward slashes; literal backslashes in Unix
  filenames remain escaped data rather than path separators.
- Exits 1 when any reference is not found (404), any verification error occurred, or any spec
  inspection finding exists; otherwise exits 0.

## Constraints

- Closed issues are reported as a warning ("spec may need updating") but do **not** by themselves cause a non-zero exit.
- Issue verification depends on the GitHub REST API (via the `github` module); an unavailable or
  invalid `GITHUB_TOKEN` surfaces as "error" entries without consulting authenticated `gh` state.
- Must not panic or disclose spec bytes on unreadable specs or unparseable frontmatter; such specs
  are reported as inconclusive inspection findings.
- Must not reopen a discovered spec or mapped-source path for issue parsing or `--create`
  validation.
- When selected config omits source directories, detection must run through the retained project
  capability with deterministic file/byte bounds; it must not consult a replaceable ambient root
  pathname.

## Out of Scope

- Editing specs to fix stale references (read-only verification).
- Creating non-drift issues or syncing issue state back into specs.
- Interactive prompts, GUI, or web output.

### REQ-cmd-issues-001

The issues command SHALL verify tracked GitHub references and SHALL report valid, closed, missing, and unverifiable states predictably.

Acceptance Criteria
- Reads each spec's `implements:` and `tracks:` only as lists of numeric issue IDs; wrong shapes or
  invalid entries are inspection findings, while specs with neither field are skipped.
- Validates configured repository syntax even with zero references or missing/empty specs;
  otherwise resolves through Git auto-detection only after at least one reference is gathered.
- All references are verified in one globally deduplicated batch of at most 100 unique issue IDs;
  confirmed issues are classified as valid/closed/not-found while repository, authentication,
  transport, timeout, or malformed-provider failures are errors.
- Per-format output: Text/Table/Csv print per-spec details, safe inspection findings, and a one-line
  summary; Json emits totals plus `inspection_findings`, a content-free `findings` array, and the
  `specs` array; Markdown/Github emit a metric table plus a findings table when needed.
- Text/Table/Csv no-reference guidance appears only when no references were gathered; all-error
  batches include their error count in the summary.
- An empty reference set skips Git auto-detection and provider access, but any configured repository
  identity is syntax-validated even with zero references or no discovered specs.
- Discovery and reads use retained capability-rooted directory/file handles, and checked issue
  parsing consumes immutable snapshots whose discovered identity remains unchanged through read,
  including regular-file and hardlink replacement checks.
- The retained project capability that authorizes spec discovery also authorizes every
  mapped-source snapshot; later ambient root replacement cannot combine project generations.
- Spec retention is capped at 10,000 files, 4 MiB per spec, and 64 MiB cumulatively; mapped-source
  retention is capped at 4 MiB per source and 64 MiB cumulatively.
- At most 100,000 total directory entries are examined, including non-spec entries.
- Checked issue parsing rejects duplicate/global malformed YAML and blank/null/wrong-shaped known
  fields, accepts comments/trailing commas, and ignores nested extension/block-scalar lookalikes.
- With `--create`, validation consumes retained spec/source snapshots through
  `validate_spec_content_with_sources` and never reopens discovered paths before deciding whether
  to create drift issues.
- Unreadable specs and malformed or missing frontmatter are retained as path-attributed,
  content-free inspection findings and suppress no-reference guidance.
- Checked traversal failures are retained as findings, and every rendered finding path is
  project-relative, content-free, control/bidi/Zl/Zp-safe, and valid in its output format.
- Windows finding paths use forward slashes while Unix literal backslashes remain filename data.
- A present selected project configuration must be readable UTF-8 and syntactically valid JSON or
  TOML before discovery. It is opened through the retained project capability, must be a regular
  non-link entry, is identity-checked through one bounded 4 MiB same-handle read, and is parsed and
  applied from those exact bytes. Invalid syntax, UTF-8, or known TOML field types is a
  content-free configuration finding that exits 1 without ambient/default-path fallback.
- Missing/empty specs and repository-resolution failures render through the selected format:
  JSON stays parseable and Markdown/GitHub retain their structured report.
- Markdown/GitHub code spans pad content when a path starts or ends with a backtick.
- Exits 1 when any reference is not found (404), any verification error occurred, or any spec
  inspection finding exists; otherwise exits 0.
- Omitted source-directory discovery is derived from a bounded sparse snapshot built through the
  retained project capability, and a post-config ambient root replacement cannot alter it.
