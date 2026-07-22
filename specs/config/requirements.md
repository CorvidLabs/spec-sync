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

Configuration loading SHALL never interpret repository or local configuration as inference credentials or executable AI commands.

Acceptance Criteria
- AI configuration fields are removed from JSON/TOML readers and writers.
- Legacy AI keys produce migration guidance without activating behavior.
- The obsolete AI-only local override merge is removed.

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
