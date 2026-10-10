---
change: split-configuration-loading-parsing-and-discovery-into-focused-private-modules
artifact: plan
---

# Plan

## Approach

Keep the current configuration API behind a small facade. Extract complete implementations into private discovery, checked validation, filesystem loading and legacy diagnostics, JSON compatibility, TOML compatibility, shared TOML syntax, and serialization owners. Group existing regressions beside their owners. Preserve complete function bodies and literal contents rather than rewriting parsing.

## Out of scope

No API, defaults, accepted syntax, validation order, error wording, warning output, precedence, fallback, filesystem authority, or migration behavior changes. No dependency, runtime, unrelated module, trust-gate, browser, push, implementation acceptance or archive changes.

## Steps

1. Capture pristine focused configuration results and compiled inventory.
2. Extract discovery, exact-byte/type checking, file loading and format-specific parsing along their existing call boundaries.
3. Place shared string and array syntax helpers beneath a private TOML syntax owner and keep serializers and parsers unchanged.
4. Move all existing tests into topical owner-local files, preserving names, bodies, assertions and attributes, and keeping every file below 1000 lines.
5. Update canonical spec file mappings and companions; compare original bodies, types, constants, cfg guards and compiled test inventory.
6. Obtain independent review of capability, refusal, diagnostics, sentinel and migration boundaries.
7. Run focused, full baseline-comparison, formatting, debug/release, production clippy, release build, strict coverage and pre-push checks; record unavailable tools and pre-existing failures accurately.
8. Run real scoped change check and check --commit for a clean local checkpoint awaiting human implementation review.

## Risks

The checked parser and compatibility parsers intentionally differ: checked parsing rejects wrong-shaped GitHub values while legacy loading preserves a refusal sentinel. Supplied source discovery must remain lazy and authoritative. File loaders must retain strict refusal and identical diagnostics. TOML string helpers must preserve escaped literals and malformed-array behavior. Move bodies intact and verify these seams independently.

## Constraints consulted

Honor specs/config/context.md and requirements: canonical-to-legacy precedence; present unloadable config refuses by default; retained callers parse exact bytes with real syntax and known-type validation before compatibility parsing; supplied source lists never trigger ambient rediscovery; JSON compatibility retains an invalid GitHub repo sentinel while checked JSON rejects wrong shapes; retired AI keys emit names only; explicit settings and module owns survive migration. No configured principles file exists. Read AGENTS.md, the repository spec-sync skill, config spec and all companions, and the Human Intent records. This is the owner-authorized existing-behavior refactor, not a new feature or self-approved implementation review.
