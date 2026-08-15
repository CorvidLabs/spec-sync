---
spec: config.spec.md
---

## User Stories

- As a developer, I want zero-config source discovery and JSON/TOML compatibility.
- As a team lead, I want deterministic validation, lifecycle, and module settings.
- As a security-conscious maintainer, I want retired inference keys ignored without exposing their values.

## Constraints

- Loading is local and performs no network calls or command execution.
- Existing legacy layout formats remain readable for migration.
- Present but unreadable or malformed configuration fails loud before fallback.

### REQ-config-001

The `config` module SHALL preserve truthful user-visible behavior for the pre-6.0 product fixes landed in this change.

Acceptance Criteria
- Related tests remain green.
- No intentional regression of SpecSync 6.0 lifecycle verbs.

### REQ-config-002

Configuration source-directory autodetection SHALL recognize default measurable static files in addition to language exports.

Acceptance Criteria

- Static-only root projects resolve to `.`.
- Static-only nested projects resolve to the containing top-level directory.
- Empty projects retain the `src` fallback.

### REQ-config-003

Configuration SHALL provide a default-false `include_extensionless` option (`includeExtensionless` in legacy JSON) that adds extensionless files without changing omitted or empty `source_extensions` semantics.

Acceptance Criteria

- Canonical TOML reads `include_extensionless` and emits it only when true.
- Legacy JSON reads `includeExtensionless`.
- Omitted and explicit false values preserve existing discovery.
- Omitted and empty extension lists continue to select the default supported-language set.

### REQ-config-004

Configuration SHALL provide and document a default-false `require_draft_files` option, named `requireDraftFiles` in legacy JSON, that requires every draft mapping to exist when enabled.

Acceptance Criteria

- Omitted and explicit-false values preserve planned draft mappings.
- Canonical TOML reads and emits `require_draft_files = true` without losing the value during migration.
- Legacy JSON reads `requireDraftFiles` and recognizes it as a supported key.
- The canonical configuration structure table documents both serialized names and behavior.

### REQ-config-005

Configuration SHALL expose checked source-directory and manifest discovery that preserves malformed
or unreadable Gradle settings as errors while retaining infallible compatibility wrappers.

Acceptance Criteria

- Checked discovery returns an error before exposing partial manifest modules or source roots.
- `detect_source_dirs` remains compatible and falls back to scan-based discovery on a checked error.
- `discover_manifest_modules` remains compatible with its infallible discovery return type.
- Coverage and enforcement callers can use the checked variants to distinguish inconclusive
  discovery from successful empty discovery.

### REQ-config-006

Legacy JSON GitHub repository configuration SHALL fail closed when `github.repo` is present with a
non-string, non-null type.

Acceptance Criteria

- Number, boolean, object, and list values remain explicitly invalid instead of discarding the surrounding
  valid configuration or becoming repository auto-detection.
- Missing, null, and string repository values preserve compatibility.
- Issue inspection rejects the explicit invalid repository before no-spec/no-reference success.

### REQ-config-007

Configuration SHALL provide a checked parser for exact retained JSON/TOML bytes used by
security-sensitive callers.

Acceptance Criteria

- Parsing consumes supplied bytes without reopening the pathname.
- Leading BOM compatibility and omitted-source autodetection remain supported.
- Malformed syntax and wrong-shaped known TOML fields return an error instead of defaults.
- Checked JSON rejects a non-object `github` value and non-string/non-null `github.repo` before the
  compatibility parser can substitute a sentinel or defaults.

### REQ-config-008

Retained CLI configuration discovery SHALL preserve established compatibility while acquiring
selected bytes and omitted-source detection beneath one retained project capability.

Acceptance Criteria

- Canonical-to-legacy precedence and source-file classification are shared rather than duplicated.
- Explicit source roots are parsed before autodetection and left to normal validation instead of
  triggering unrelated manifest/source traversal.
- A nested configuration parent must remain reachable from the retained project root before and
  after its bounded read.
- Invalid-UTF-8 legacy CLI config keeps its fail-loud warning and safe-default fallback.
- Strict MCP selected-config parsing remains unchanged.

### REQ-config-009

Loading SHALL distinguish an absent config file from one that exists and could not be
loaded.

Acceptance Criteria
- An absent file yields the built-in defaults with no recorded failure.
- An unreadable file yields the built-in defaults with the failure recorded.
- A file that fails to parse yields the built-in defaults with the failure recorded.

### REQ-config-010

The default configuration loader SHALL refuse an unloadable config file.

Acceptance Criteria
- A config file that exists and cannot be used stops the command, whichever loader it reached.
- A permissive loader remains available under a name that states it bypasses the refusal, so the bypass is deliberate rather than forgotten.
- A project with no config file at all is unaffected; the built-in defaults remain a legitimate run.
