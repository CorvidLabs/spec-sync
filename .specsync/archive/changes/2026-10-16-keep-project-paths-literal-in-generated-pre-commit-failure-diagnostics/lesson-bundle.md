# Lesson bundle — keep-project-paths-literal-in-generated-pre-commit-failure-diagnostics

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Keep project paths literal in generated pre-commit failure diagnostics
- **Kind**: BugFix
- **Specs**: hooks
- **Paths**: src/hooks.rs, specs/hooks, site/src/content/docs/cli.md, site/src/content/docs/workflow.md
- **Acceptance**: Newly generated hooks pass the exact project root to the checker and preserve success/failure exit behavior for ordinary paths, spaces and apostrophes; dollar substitutions and backticks never execute from failed-check diagnostics; executable negative controls fail on released 6.1.1 and pass on the candidate; existing-hook refresh instructions state that install alone does not replace managed blocks.

## Evidence

- Verification commit: `16e5a82ae9af7e8d912b83a8c79f04b89818688a`
- Base commit: `621a74785e3c09b2e9ebea35768e05f5772f886f`
- Verified by: `specsync check --spec hooks`

## From the change's context.md

# Context

This is a bounded repair of reported security issue #782, reproduced against released SpecSync 6.1.1. It preserves the hook API, scoped project identity, repository guard, enforcement and idempotent managed-block behavior. Leif directly approved this prepared definition on 2026-10-16, conditional on actual local testing and at least 95% engineering confidence. The native writer recorded the genuine scope approval as actor 0xLeif; the direct response is preserved in the coordinator receipt ci-cleanup-human-approval.json. Independent implementation review, current qualification, lifecycle finalization and release remain separate gates.

## From the change's design.md

# Design

Build the scoped checker command once, replace only its exact executable template line, and independently shell-quote the entire failure hint as one argument to fixed-format printf. Preserve project marker ownership, user content, worktree/core.hooksPath support and all current success/failure semantics. No dependency, executable lookup policy or public API changes.

## From the change's testing.md

# Testing

The five Unix installed-hook unit cases execute generated shell hooks using a disposable project and a fake checker that records exact argv. Both checker exit 0 and exit 1 are exercised. Assertions cover literal root, hook status, failure message and absence of shell marker effects. The baseline produced 2 passing and 3 failing cases; patched `cargo test hooks::tests` passed all 63 hook tests. The first full lane passed 2,586 tests but a separate watch Unix-socket fixture was denied by the local sandbox. The permitted full rerun passed all 2,587 unit tests, 462 integration tests, format/lint/types, release build, strict specs and release/required-CI fixture controls: receipt 20261016T033019Z-S4-fbca94, exit 0, 830.914 seconds. The newly built release candidate passed all 10 installed-hook reproduction cases, including exact roots and exit codes with no marker execution. Latest released SpecSync 6.1.1 strict/100% checks, all 62 spec quality floors (80), and the mandatory pre-push gate passed (receipt 20261016T033533Z-S4-8f6404). Cached documentation checks passed 23 tests, Astro diagnostics with zero errors, and a 43-page build (receipt 20261016T034800Z-S4-88f14f). The original draft is published as PR #818. Leif gave genuine scope approval on 2026-10-16; current qualification and implementation review/finalization remain separate evidence-bound steps. Exact-current-commit Trust and hosted review/CI remain publication gates. No human inspection, Linux qualification or released fix is claimed.

## Current lifecycle follow-up

The original product source and test bytes remain unchanged from the independently reviewed and locally qualified candidate. The genuine scope approval removes the absence-of-permission condition only; no new local test run, human inspection, native review/finalization or passing hosted gate is inferred from that approval. Fresh command results will be recorded after execution.

## Where these lessons go

- `specs/hooks/hooks.3md`
