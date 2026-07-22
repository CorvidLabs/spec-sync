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
- Generated files and rollback cleanup resolve through retained parent capabilities; publication
  verifies staged identities and rollback preserves replacements at public transaction paths.
  Failed batches may leave empty parents rather than claim ownership across a non-atomic
  create/open interval. Same-user mutation of private stage/quarantine names is outside the MCP
  caller and project-path confinement boundary.
- Windows absolute read roots derive their relative suffix with the same case-insensitive,
  extended-drive/UNC-aware normalization used for containment.

### REQ-mcp-003

The MCP server SHALL validate JSON-RPC envelopes and arguments before dispatch and SHALL fail closed
when protocol output or deterministic generation cannot complete safely.

Acceptance Criteria

- Missing/invalid `jsonrpc`, method, ID, params, or top-level request shapes return `-32600` before
  dispatch and cannot mutate.
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
- Issue listing follows strict encoded pagination for at most 100 pages of 100 issues and fails on
  malformed links, duplicate IDs, or a continuing next page at the cap. Every next link retains
  the requested repository issues endpoint and exact open-state, page-size, label, and page query
  semantics.

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
- Batch import follows strict pagination for at most 100 pages of 100 issues and fails on malformed
  links, duplicate IDs, or cap truncation rather than reporting partial success.

### REQ-cmd-issues-001

The issues command SHALL verify tracked GitHub references and SHALL report valid, closed, missing,
and unverifiable states predictably.

Acceptance Criteria

- The command gathers all spec references and verifies them in one project-wide batch.
- Confirmed issue states remain attributed to each spec after global deduplication.
- Provider/batch errors contribute to the existing non-zero command outcome.
- Text output distinguishes a project with no references from an all-error batch and includes the
  error count in its summary.
- Projects with no references return no-reference guidance before repository or provider
  resolution, with or without repository configuration.

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

### REQ-validator-008

Coverage gates SHALL use fallible checked manifest discovery and SHALL report malformed or unreadable
Gradle settings as inconclusive instead of accepting partial coverage.

Acceptance Criteria

- `compute_coverage_checked` propagates discovery errors without a partial report.
- CLI and MCP enforcement callers use checked coverage.
- `compute_coverage` remains a compatibility wrapper carrying an inconclusive diagnostic.

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
