---
id: CHG-0097-break-text-mode-next-action-guidance-from-correction-digests-codeql-58-clearte
state: implementing
type: bug_fix
base_commit: a4dfa9999398020c305b495e3656ff4306b188cc
---

# Break text-mode next-action guidance from correction digests (CodeQL #58 cleartext-logging)

## Intent

Break text-mode next-action guidance from correction digests (CodeQL #58 cleartext-logging)

## Affected Canonical Specs

- None

## Acceptance Criteria

- Text-mode change status/show/list next-action never loads correction digests or validate_trusted_correction_history; JSON mode still surfaces effective_definition and corrections; draft text surfaces require complete artifacts before approval; CodeQL cleartext-logging taint path from digests into println is broken

## No-spec Rationale

No public API surface change: artifacts_complete_for_guidance already documented; implementation stops loading correction digests for human text sinks only
