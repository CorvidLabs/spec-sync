---
change: split-dependency-graph-import-analysis-and-reporting-into-focused-private-modules
artifact: plan
---

# Plan

## Approach

Keep DepNode, DepsReport and the existing public API paths behind a small facade. Extract complete implementations into private graph construction/validation, import extraction and ownership, and report formatting owners. Keep the entire import scanner subsystem together instead of introducing a small module for each language. Group existing tests by graph, imports, JVM ownership and reporting, with two shared fixture builders.

## Out of scope

No API, import dialect, regex, string masking, ownership resolution, source-read authority, diagnostics, cycle traversal, ordering or rendering changes. No new language support or cleanup of unrelated stale requirement prose. No dependency, runtime, unrelated module, trust-gate, browser, push, human implementation review, acceptance or archive changes.

## Steps

1. Preserve the original source and derive the 43-test baseline inventory and results from the existing main unit log.
2. Extract complete graph, imports and report implementations along their existing call boundaries; keep the public facade stable and internal seams minimal.
3. Move all existing tests into cohesive files with exact bodies, attributes and literal contents; keep every resulting file below 1000 lines.
4. Update canonical spec mappings and topology companions without changing requirements; compare original bodies, types, constants, cfg guards and test inventory.
5. Obtain independent read-only review of source-root guards, fail-loud errors, Rust masking, JVM ownership, ImportDialect authority and disclosure semantics.
6. After shared-target permission, run focused deps tests and compiled inventory, formatting, production clippy, debug and release checks, release build, strict coverage, pre-push and actual scoped lifecycle check --commit.
7. Return a clean local checkpoint awaiting human implementation review. The parent runs combined full baseline-comparison suites; report existing failures and unavailable trust tools accurately.

## Risks

An empty import set must remain distinct from a language without an analyzer. Source-root guards must precede filesystem reads, including package discovery. JVM declaration ownership takes precedence over layout and retains owned/foreign/unattributed outcomes without guessing ambiguous owners. Rust comments and all string forms must stay masked, and attribution/order/diagnostics must not drift. Move complete bodies and literals intact and review those boundaries independently.

## Constraints consulted

Read AGENTS.md, the repository spec-sync skill, Human Intent records, the deps spec and all companions. Honor specs/deps/context.md graph naming, DFS cycle collection, best-effort regex extraction, conservative warnings, fail-loud read errors and skipped cross-project references. Preserve REQ-deps-002 and REQ-deps-003, ImportDialect authority and source_within_root guards. No configured principles file exists. Older three-language and silently-skipped companion prose is stale; it does not authorize changing current behavior.
