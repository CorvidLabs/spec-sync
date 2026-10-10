---
change: split-manifest-discovery-and-parser-responsibilities-into-focused-private-modules
artifact: plan
---

# Plan

## Approach

Separate stable discovery/type entry points from ecosystem parsers, retained access and Gradle's existing syntax/guard/access responsibilities. Preserve complete function bodies and explicit data flow; split files by ownership, not arbitrary line intervals. Keep existing external API paths through a small facade and parent-private seams.

## Out of scope

No parser grammar, accepted/rejected input, process execution, result ordering, merging, module naming, diagnostic, serialization, path confinement, identity check, budget or caching change. Do not add language support, fix historical parser limitations or change validator/MCP implementations. No browser publication, human implementation acceptance, archive or merge is implied.

## Steps

1. Record baseline focused manifest tests and existing pristine-main full-suite failures.
2. Extract shared retained access and identity primitives, preserving the capability-owned cache/budget and bounded handle lifetime.
3. Extract Cargo, Swift, Node and simple Dart/Go/Python discovery while preserving fixed discovery order and first-name-wins merging in the facade.
4. Extract Gradle discovery/effective-directory orchestration, retained manifest access, syntax and executable-directive guards. Preserve the entire existing semantic pipeline and complete bodies.
5. Move all existing tests and platform fixtures into topical owner-local files, preserving names, assertions and cfg guards. Target every source/test file below 1000 lines without fragmenting cohesive functions.
6. Prepare canonical spec mappings and companion documentation, mechanically compare bodies and tests, and independently review capability/grammar seams.
7. Run focused and consumer regressions, full unit/integration baseline comparisons, formatting, debug/release checks, clippy, release build, strict coverage, pre-push and real scoped lifecycle verification.
8. Commit a local reviewable result with exact gate limitations; later human review and shipping remain explicit.

## Risks

Gradle syntax and guards are interleaved and easy to reorder incorrectly. Keep their function bodies and invocation order, especially comment masking, escaped literals, includeBuild block bounds and separate guards over its unchanged contents. Retained listing/access extraction must not duplicate budgets, relax identity checks or keep sibling handles alive. Tests belong beneath owners to avoid expanding crate-visible internals.

## Constraints consulted

Read AGENTS.md and specs/manifest requirements, context and tasks. Honor the recorded lessons for #723 and #725: includeBuild safety comes from its literal argument, balanced configuration blocks contribute no project, and other guards still inspect their content. Keep bounded process-free parsing, raw Gradle identity checks before colon conversion, all present variant preflight, one retained capability, shared workspace charging before deduplication, memoized complete nodes and sequential listing verification/release. No configured principles file exists. hi/plan.md requires these substantive sections and hi/agent.md forbids self-approved implementation review.
