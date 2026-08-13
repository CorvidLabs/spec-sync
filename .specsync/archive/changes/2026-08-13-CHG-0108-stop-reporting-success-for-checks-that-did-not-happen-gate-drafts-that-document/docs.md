---
change: CHG-0108-stop-reporting-success-for-checks-that-did-not-happen-gate-drafts-that-document
artifact: docs
---

# Docs

## CHANGELOG

Four `Fixed` entries under the unreleased 6.0.0 heading, covering the quoted-path fix, the
draft gate, the cold-cache drift noise, and the bounded remediation. The drift entry
includes a note explaining why `check --strict` had been reporting *fewer* warnings than
bare `check`, since that difference looks like suppression and was reported as such.

## Behavior change adopters must know about

`specsync check --strict` now fails on a spec that is `status: draft`, has source files
present, and documents a Public API. The remedy is `status: active`, which turns validation
on. Bare `specsync check` is unchanged.

Two shapes explicitly keep passing `--strict`:

- a draft whose mapped files do not exist yet — spec-first authoring
- a draft whose Public API names no symbol — an honest stub

## New public API

| Symbol | Spec |
|---|---|
| `HashCache::has_baseline` | `specs/hash_cache` |
| `ChangeClassification::reportable` / `::baseline_known` | `specs/hash_cache` |
| `ValidationResult::had_present_source` / `::documents_contract` | `specs/types` |

## Known and deferred

A benign symlink under a source directory still aborts `check`, `coverage`, `score`, and
`generate` (CorvidLabs/spec-sync#546). Filed with the analysis; the guard is a
capability-confinement property and the fix needs its own change. Affected projects can
exclude the linked entry via `exclude_dirs` in the meantime.
