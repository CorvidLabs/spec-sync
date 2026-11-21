---
kind: evidence
id: 2026-11-21-documentation-only-the-21-items-of-defs-docs-6-4-accuracy
aliases: []
claim: "Documentation only, the 21 items of defs/docs-6.4-accuracy-definition.md (audit of main 3f0985e8 with its 6.4.1 build). Command check: every specsync invocation the docs show is valid against --help (80 command paths; only prose or placeholder hits left; the four check --commit hits gone). New samples are real output of the 6.4.1 build: resolve --remote against spec-sync's registry (exit 1), a three-crate Cargo workspace before and after the fixes, init in an empty project. site: bun test 48/48 (Mermaid parse, lifecycle against the code), bun run build 43 pages, lint 0 errors; validate-release-version.py passes; examples sdd-lifecycle, sdd-five-epics, sdd-concurrent-changes pass. Lane at cc4977b2 (3158 unit, 741 integration, 62/62, 100%); change audit --strict exit 0."
result: passed
evidence_kind: product-test
revision: ba4d43047925d791f94fe9cf6d02f6c0c4864d57
content: 3021035a8e81acebcbb33626a954be5504220e5cb1fc7be95b5cafc6e56c63d4
environment: local macOS 26, rust 1.89.0, branch docs-6.4-accuracy at cc4977b2
criteria: [DOCS-4, DOCS-5, DOCS-1, DOCS-2]
requirements: []
contracts: []
limitations: []
observed: 2026-11-21T01:11:53Z
actor:
  - agent: claude-code
    terminal: t81
---
**Documentation only, the 21 items of defs/docs-6.4-accuracy-definition.md (audit of main 3f0985e8 with its 6.4.1 build). Command check: every specsync invocation the docs show is valid against --help (80 command paths; only prose or placeholder hits left; the four check --commit hits gone). New samples are real output of the 6.4.1 build: resolve --remote against spec-sync's registry (exit 1), a three-crate Cargo workspace before and after the fixes, init in an empty project. site: bun test 48/48 (Mermaid parse, lifecycle against the code), bun run build 43 pages, lint 0 errors; validate-release-version.py passes; examples sdd-lifecycle, sdd-five-epics, sdd-concurrent-changes pass. Lane at cc4977b2 (3158 unit, 741 integration, 62/62, 100%); change audit --strict exit 0.**

Recorded as passed in local macOS 26, rust 1.89.0, branch docs-6.4-accuracy at cc4977b2 at revision ba4d43047925, observed 2026-11-21T01:11:53Z.
