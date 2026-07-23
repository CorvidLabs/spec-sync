---
change: CHG-0063-close-independent-mcp-security-review-gaps-for-issue-414
artifact: requirements
---

# Requirements

### REQ-mcp-002

The MCP server SHALL confine every filesystem operation to its configured project root and SHALL
bound project-controlled inputs before downstream parsing.

Acceptance Criteria

- Absolute outside roots are rejected lexically before metadata or symlink resolution.
- The canonical server root is retained as a directory capability; reads use bounded snapshots and
  writes resolve only through that retained capability.
- Every case variant of `.git`, plus Git files, symlinks, junctions, and config includes, is rejected
  as configuration input and cannot redirect MCP issue checks; an explicit `github.repo` is required.
- Project inputs are bounded to 8 MiB per file and 64 MiB of actual file/config bytes cumulatively
  per operation; explicitly configured roots remain eligible even when normally ignored.
- Unix symlinks and Windows junction/reparse-point escapes are rejected with outside bytes intact.
- Startup opens and identity-binds the requested root before canonicalization, then reopens the
  canonical path and requires the same identity, so replacement during acquisition cannot redirect
  authority.
- Root-wide and manifest-derived project inputs remain in bounded snapshots even beneath normally
  ignored names; Cargo workspaces are parsed as real TOML; checked Gradle discovery handles
  comments, escapes, and standard Groovy/Kotlin forms; malformed discovery is inconclusive for
  gates; and every manifest read is copied from the exact bytes charged to the shared cumulative
  budget so omission, growth, or discovery cannot produce a false-green result.
- Cargo path discovery follows only semantic target, dependency, workspace-dependency,
  target-specific dependency, patch, and replacement tables. Unrelated metadata `path` keys are
  ignored. Confined Windows-native backslashes normalize from the declaring manifest, while
  drive, UNC, rooted, traversal, symlink, and junction escapes remain rejected.
- Generated files and rollback cleanup resolve through retained parent capabilities; publication
  verifies staged identities and rollback preserves replacements at public transaction paths.
  Failed batches may leave empty parents rather than claim ownership across a non-atomic
  create/open interval. Same-user mutation of private stage/quarantine names is outside the MCP
  caller and project-path confinement boundary.
- Windows absolute read roots derive their relative suffix with the same case-insensitive,
  extended-drive/UNC-aware normalization used for containment.
- On Windows, an absolute child may use either original or canonical startup spelling after both
  spellings are bound to the same root identity; only the suffix is opened through the retained
  canonical capability, and sibling-prefix lookalikes fail.
- MCP validates the exact bounded selected-config snapshot before compatibility loading. Invalid
  UTF-8, malformed JSON/TOML, and wrong-typed specs/source path selectors make tools and resources
  inconclusive rather than substituting an empty/default project.

### REQ-mcp-003

The MCP server SHALL validate JSON-RPC envelopes and arguments before dispatch and SHALL fail closed
when protocol output or deterministic generation cannot complete safely.

Acceptance Criteria

- Missing/invalid `jsonrpc`, missing/invalid method, invalid present ID, wrongly typed present
  params, or invalid top-level request shapes return `-32600` before dispatch and cannot mutate;
  valid notifications may omit ID and params and remain silent.
- `resources/read` rejects non-object, missing, wrongly typed, and unknown parameters with `-32602`.
- Responses are bounded to 1 MiB; oversized responses become compact `-32603` errors preserving a
  bounded request ID and safely using `null` when an attacker-controlled ID cannot fit; stdin/stdout
  transport failures are surfaced.
- Generation destination collisions and incomplete writes return `isError`, never false success;
  generation is bounded to 1,000 specs and 64 MiB, its result is preflighted before mutation, and
  staged/synced files are atomically published without overwrite. Rollback preserves public
  replacements and removes matching files; it may retain empty parents created by the failed
  batch when portable identity-safe removal is unavailable.
- Request IDs over 4 KiB are rejected with `-32600` before dispatch.
- GitHub issue reads, listing, and verification require an explicit `GITHUB_TOKEN`, use in-process
  REST, and never spawn a `gh` provider process. Verification globally deduplicates no more than
  100 issue IDs and bounds repository preflight, REST requests, and whole-batch duration; `gh`
  remains only the explicit issue-creation write path.
- Repository, authentication, transport, timeout, and malformed-provider failures return an
  inconclusive tool error; only a confirmed absent issue is represented as not_found.
- A missing issue result is revalidated against repository access within the same deadline before
  it can be represented as not_found.
- MCP issue verification treats failed spec discovery, unreadable specs, and malformed or missing
  frontmatter as inconclusive tool errors rather than silently omitting them from a successful
  zero-reference result.
- MCP issue verification uses the shared maintained real-YAML parser: duplicate/global malformed
  YAML and blank/null/wrong-shaped known fields fail closed; comments/trailing commas remain valid;
  nested extension and block-scalar lookalikes are ignored; LF and CRLF frontmatter delimiters are
  parsed equivalently.
- MCP issue diagnostic paths normalize separators only on Windows; literal Unix filename
  backslashes remain data and cannot be misreported as nested-path separators.
- Release notes and public MCP guidance document all compatibility changes.

### REQ-github-001

GitHub helpers SHALL resolve repositories and issue state predictably while redacting credentials
from surfaced failures.

Acceptance Criteria

- Repository access is preflighted once and revalidated after a missing issue response before
  absence can be classified as not_found.
- One batch globally deduplicates at most 100 issue IDs and bounds repository preflight, REST
  requests, and whole-batch duration.
- Authentication, repository, transport, timeout, and malformed-provider failures are errors.
- Read/list/verify paths require `GITHUB_TOKEN` and spawn no `gh` process; only explicit issue
  creation uses `gh`.
- Issue listing follows strict encoded pagination for at most 100 pages of 100 provider entries and fails on
  malformed links, duplicate IDs, or a continuing next page at the cap. Every raw provider page
  is rejected above 100 entries before item parsing, including pull-request entries. Before PR
  filtering, every raw item is validated for marker shape, positive identity, nonempty title,
  nonempty names for any labels, exact open state, canonical repository/resource/number URL
  identity including exact canonical decimal number spelling without leading zeros, and duplicate
  raw identity within/across pages. Every next link retains the requested repository issues
  endpoint and exact open-state, page-size, label, and page query semantics.

### REQ-parser-001

The parser SHALL deterministically parse the supported frontmatter and Public API Markdown subset,
SHALL identify required and stub sections, and SHALL provide fail-closed real-YAML parsing for
security-sensitive GitHub issue references.

Acceptance Criteria

- Compatibility `parse_frontmatter` behavior remains available for the established metadata subset.
- `parse_checked_issue_references` parses the complete frontmatter with maintained `serde-saphyr`
  and rejects duplicate keys globally.
- Malformed YAML anywhere and blank/null/scalar/mapping/mixed/non-positive/overflowing top-level
  `implements`/`tracks` values fail the complete checked parse.
- Comments and valid trailing commas are accepted.
- LF and CRLF frontmatter delimiters are accepted equivalently.
- Nested extension mappings/sequences and block-scalar text do not contribute issue references.
- Surfaced checked-parser errors are stable and content-free.

### REQ-importer-001

The importer SHALL normalize supported external content into safe local spec drafts while
sanitizing paths, secrets, markup, and oversized input.

Acceptance Criteria

- GitHub single-issue reads delegate to the shared typed REST contract, require explicit
  `GITHUB_TOKEN`, and never launch `gh issue view`.
- An ambiguous issue 404 is classified only after repository access is revalidated.
- Missing tokens, malformed payloads, inaccessible repositories, transport failures, and timeouts
  fail without producing a partial imported item.

### REQ-cmd-import-001

The import command SHALL create non-overwriting draft specs from supported single and batch sources
with deterministic companion generation.

Acceptance Criteria

- Single and batch GitHub imports require explicit `GITHUB_TOKEN` and execute no provider read
  subprocess.
- Every GitHub REST operation is bounded to 10 seconds.
- Batch import follows strict pagination for at most 100 pages of 100 provider entries, rejects an
  oversized page before item parsing, and fails on malformed links, duplicate IDs, or cap
  truncation rather than reporting partial success.

### REQ-cmd-issues-001

The issues command SHALL verify tracked GitHub references and SHALL report valid, closed, missing,
and unverifiable states predictably.

Acceptance Criteria

- The command gathers all spec references and verifies them in one project-wide batch.
- Confirmed issue states remain attributed to each spec after global deduplication.
- Provider/batch errors contribute to the existing non-zero command outcome.
- Text output distinguishes a project with no references from an all-error batch and includes the
  error count in its summary.
- Projects with no references skip Git auto-detection and provider access. A configured
  `github.repo` is still syntax-validated before no-reference success, including when the specs
  directory is missing or contains no specs.
- A present selected project config must be readable UTF-8 and syntactically valid JSON or TOML;
  it is opened through the retained project capability, rejected when linked or non-regular,
  identity-checked through one bounded 4 MiB same-handle read, and parsed/applied from those exact
  bytes. Malformed, invalid-UTF-8, wrong-shaped, replaced, or oversized config is a structured
  content-free finding that exits 1 without default-path fallback, no-spec success, or
  no-reference success.
- Unreadable specs and malformed or missing frontmatter are retained as path-attributed,
  content-free inspection findings in text, JSON, Markdown, and GitHub output; they suppress
  no-reference guidance and make the command exit 1.
- Discovery and reads are rooted in retained project/spec-directory capabilities; each immutable
  spec snapshot keeps its discovered identity through read completion, rejecting symlink,
  regular-file, and hardlink replacement.
- One retained project capability supplies both spec discovery and mapped-source snapshots, so
  ambient root replacement cannot mix project identities.
- Spec snapshots are capped at 4 MiB each, 64 MiB cumulatively, and 10,000 files; mapped-source
  snapshots used by `--create` are capped at 4 MiB each and 64 MiB cumulatively.
- Recursive discovery examines at most 100,000 total entries, including non-spec entries, before
  returning an inconclusive bounded finding.
- Issue inspection uses the shared checked real-YAML parser.
- Every renderer escapes controls, bidi formatting characters, and Unicode Zl/Zp separators;
  Markdown/GitHub additionally preserve valid escaped table/code-span structure and pad code-span
  content when a path begins or ends with a backtick.
- Windows finding paths use forward slashes, while literal Unix backslashes remain filename data.
- Missing/empty specs and repository-resolution failures render through the selected format:
  JSON remains parseable and Markdown/GitHub preserve their structured report.
- `--create` performs normal drift validation from retained spec and mapped-source snapshots
  through `validate_spec_content_with_sources`, without reopening discovered paths or resolving
  supplied-content TypeScript wildcard imports through ambient paths.

### REQ-commands-003

Drift-issue creation SHALL render untrusted text safely at both the command terminal and GitHub
issue boundaries.

Acceptance Criteria

- Repository-resolution failures, spec paths, returned issue URLs, and provider failures pass
  through the shared safe diagnostic renderer before terminal output.
- Terminal output does not preserve raw control characters, bidirectional formatting controls, or
  Unicode line/paragraph separators from untrusted values.
- The explicit GitHub creation helper sanitizes spec paths and validation errors separately for
  title text and Markdown body text.
- Sanitization preserves one attempted issue per spec and continuation after individual failures.
- Public `run_validation` and `create_drift_issues` retain rendered `Vec<String>` compatibility;
  private structured attribution and longest exact discovered-path matching preserve legal paths
  containing `": "` without exporting new command types.

### REQ-exports-005

Snapshot export extraction SHALL parse caller-supplied source content without reopening logical
paths or resolving TypeScript wildcard imports through ambient filesystem authority.

Acceptance Criteria

- `get_exported_symbols_from_content` accepts a logical path and caller-supplied UTF-8 content.
- The logical path supplies language/type context but is never opened.
- Regex and AST TypeScript snapshot extraction pass no wildcard resolver.
- Local supplied-content exports retain normal ordering, deduplication, parse-mode, and export-level
  behavior.

### REQ-cmd-check-001

Unified JSON checking SHALL preserve the documented top-level check schema when SDD validation or
coverage discovery fails.

Acceptance Criteria

- Failed SDD JSON output includes `passed`, `errors`, `warnings`, `stale`, and `specs_checked`.
- Structured SDD detail remains additive.
- Malformed manifest discovery exits nonzero with parseable inconclusive JSON.

### REQ-cmd-comment-003

The comment command SHALL emit a bounded markdown protocol on stdout.

Acceptance Criteria

- Configured SDD command output remains off stdout in comment mode.
- Only the final bounded markdown report is printed or posted.
- Malformed manifest discovery exits nonzero before misleading markdown is rendered.

### REQ-cmd-coverage-001

The coverage command SHALL report trustworthy file and LOC coverage and SHALL fail closed when
manifest discovery is inconclusive.

Acceptance Criteria

- Coverage uses `compute_coverage_checked`.
- Trustworthy zero-denominator coverage remains 100 percent.
- Malformed discovery exits 1 with valid inconclusive JSON and null percentages.

### REQ-cmd-generate-001

The generate command SHALL create deterministic local specs only from trustworthy discovery.

Acceptance Criteria

- Every generation mode uses checked coverage discovery before selecting output.
- Malformed discovery exits nonzero before mutation.
- JSON remains parseable with an empty `generated` collection.

### REQ-cmd-report-001

The report command SHALL provide a trustworthy project/module health view and SHALL fail closed when
manifest discovery is inconclusive.

Acceptance Criteria

- Overall coverage uses `compute_coverage_checked`.
- Malformed discovery exits before partial report rendering.
- JSON remains parseable with null coverage, zero counts, and empty modules.

### REQ-cmd-score-001

The score command SHALL produce deterministic quality scores while honoring filters, formats, and
release gates.

Acceptance Criteria

- Checked coverage discovery succeeds before scoring gates are evaluated.
- Trustworthy warn-mode scoring remains advisory.
- Malformed discovery exits nonzero with parseable inconclusive JSON and no score.

### REQ-config-005

Configuration SHALL expose checked source-directory and manifest discovery that preserves malformed
or unreadable Gradle settings as errors while retaining infallible compatibility wrappers.

Acceptance Criteria

- Checked discovery fails before exposing partial modules or source roots.
- Compatibility source-directory and manifest wrappers retain their existing return types.
- Enforcement callers can distinguish inconclusive discovery from successful empty discovery.

### REQ-config-006

Legacy JSON GitHub repository configuration SHALL fail closed when `github.repo` is present with a
non-string, non-null type.

Acceptance Criteria

- Number, boolean, object, and list values remain explicitly invalid instead of discarding valid
  surrounding configuration or becoming repository auto-detection.
- Missing, null, and string repository values preserve compatibility.
- Issue inspection rejects the explicit invalid repository before no-spec/no-reference success.

### REQ-config-007

Configuration SHALL provide a checked parser for exact retained JSON/TOML bytes used by
security-sensitive callers.

Acceptance Criteria

- Parsing consumes caller-supplied bytes without reopening the configuration pathname.
- Leading UTF-8 BOM compatibility, selected-format behavior, and omitted-source autodetection are
  preserved.
- Malformed JSON/TOML and wrong-shaped known TOML fields return an error rather than silently
  accepting compatibility defaults.

### REQ-validator-008

Coverage gates SHALL use fallible checked manifest discovery and SHALL report malformed or unreadable
Gradle settings as inconclusive instead of accepting partial coverage.

Acceptance Criteria

- `compute_coverage_checked` propagates discovery errors without a partial report.
- CLI and MCP enforcement callers use checked coverage.
- `compute_coverage` remains a compatibility wrapper carrying an inconclusive diagnostic.

### REQ-validator-001

The validator SHALL enforce bidirectional code-contract, metadata, dependency, schema, and coverage
rules while accumulating actionable findings, and SHALL support exact pre-read spec snapshots
without reopening their logical paths.

Acceptance Criteria

- Bidirectional validation reports a documented-but-missing export as an error and an undocumented
  code export as a warning.
- Missing required frontmatter fields are errors.
- Cross-project references are skipped during local validation.
- Coverage excludes test files and configured exclude patterns.
- Spec discovery is sorted.
- Schema validation uses the configured schema regex.
- Missing-source suggestions use bounded Levenshtein distance.
- Flat source files are detected while common entry points are excluded.
- Source discovery respects configured extensions.
- Requirements companions are validated when present and remain optional under adaptive policy.
- `validate_spec_content` validates caller-provided spec bytes through the normal single-spec
  validation core.
- The logical `spec_path` remains diagnostic/source context, but neither it nor adjacent companions
  are opened by pre-read spec-content validation; mapped sources retain normal path behavior.
- `validate_spec` preserves path-based compatibility by reading once and delegating exact bytes.
- CRLF normalization and size policy use the supplied content.
- `SourceSnapshot` represents present, missing, rejected, and unreadable mapped-source
  observations.
- `validate_spec_content_with_sources` validates supplied spec bytes and supplied mapped-source
  observations without reopening either through ambient project paths.
- Supplied-source export extraction does not resolve TypeScript wildcard imports through ambient
  paths.

### REQ-manifest-001

Manifest discovery SHALL identify supported project modules and source roots deterministically
without claiming unsupported workspace expansion.

Acceptance Criteria

- One checked parser handles Groovy/Kotlin single and double quotes, parenthesized and bare
  multiline Gradle includes, comments, escapes, nested colon names, and supported `projectDir`
  overrides.
- General module discovery and MCP snapshot preflight use the same effective Gradle module paths.
- MCP Cargo workspace discovery parses bounded manifests as real TOML.
- Malformed MCP Cargo TOML/workspace shapes make MCP operations inconclusive; malformed Gradle
  comments, escapes, strings, parentheses, and overrides make every checked coverage gate
  inconclusive without returning partial module results.
