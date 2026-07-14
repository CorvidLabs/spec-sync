---
change: CHG-0025-address-all-unresolved-review-feedback-on-pr-366
artifact: requirements
---

# Requirements

### REQ-change-026

The lifecycle SHALL treat sequence claims and historical collision acknowledgements as protected exact repository evidence across arbitrarily wide numeric sequences.

#### Acceptance Criteria

- Numeric change sequences contain at least four ASCII digits and support values beyond 9999.
- The committed sequence ledger always requires lifecycle coverage even when `.specsync/` is ignored.
- An acknowledgement matches the exact currently located ID set and remains valid only when every member is accepted or archived.
- Removed IDs, added IDs, single surviving records, and draft, approved, implementing, or verifying collision members fail closed.

### REQ-change-027

Configured verification SHALL reject direct and indirect entry into every SpecSync lifecycle command surface.

#### Acceptance Criteria

- Nested `check`, `change`, and `lifecycle` commands fail before performing validation or mutation.
- Native verification commands remain unaffected and execute once.
- The diagnostic names the configured parent command.

### REQ-change-028

Effective contract and canonical-successor validation SHALL use canonical repository resolution without redundant full-project hashing.

#### Acceptance Criteria

- Effective validation reads registry-backed canonical specs through the safe project-path resolver.
- Conventional canonical paths remain the fallback when no registry mapping exists.
- Unsafe registry mappings fail closed before effective validation.
- The current project digest is computed at most once per canonical-successor candidate scan.

### REQ-validator-004

Strict validation SHALL discover default static projects and reject every unfilled marker emitted by built-in companion templates.

#### Acceptance Criteria

- Zero-config root and nested HTML, HTM, and CSS files select their containing source directory.
- Ignored directories remain excluded from static discovery.
- Every generated Layout, Components, Tokens, and Assets design marker produces an artifact-specific line diagnostic.
- Concrete replacements pass while fenced examples and similar prose remain ignored.

### REQ-config-002

Configuration source-directory autodetection SHALL recognize default measurable static files in addition to language exports.

#### Acceptance Criteria

- Static-only root projects resolve to `.`.
- Static-only nested projects resolve to the containing top-level directory.
- Empty projects retain the `src` fallback.

### REQ-cli-003

The root CLI dispatcher SHALL fail closed when a configured verification child re-enters lifecycle checking or mutation.

#### Acceptance Criteria

- `check`, `change`, and `lifecycle` command families consult the inherited verification context before dispatch.
- A blocked nested command exits non-zero with one actionable diagnostic.
- Commands outside the lifecycle boundary preserve current dispatch behavior.
