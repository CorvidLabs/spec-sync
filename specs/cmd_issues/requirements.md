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
- Retained detection skips the same ignored directory names before inspecting links. Selected
  config and recognized manifests use no-follow, non-blocking opens whose retained-handle identity
  remains authoritative; non-regular entries and replacement/FIFO races become inconclusive
  configuration findings without blocking.

## Out of Scope

- Editing specs to fix stale references (read-only verification).
- Creating non-drift issues or syncing issue state back into specs.
- Interactive prompts, GUI, or web output.

### REQ-cmd-issues-001

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
- Retained source detection applies the shared ignored-directory policy before metadata
  inspection. Ignored symlinks are skipped, while recognized non-regular manifests produce a
  structured inconclusive configuration finding.
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


### REQ-cmd-issues-002

Snapshot source validation SHALL distinguish a confined directory from a rejected path.

Acceptance Criteria

- A `files:` entry that resolves to a directory inside the project root is represented distinctly
  and reported as a mapping-shape error, not as an out-of-root security escape.
- Symbolic links and reparse points remain rejected, and that rejection is evaluated before the
  directory case.

### REQ-cmd-issues-003

Issue discovery SHALL classify a directory encountered through confined metadata the same way the path-based predicate does.

Acceptance Criteria
- A directory maps to the directory source-snapshot variant rather than to an unreadable or empty one.
- The classification agrees with `check` and with confined MCP validation, so one path cannot be a directory to one caller and an unreadable file to another.

