---
change: split-parser-frontmatter-issue-validation-symbols-and-sections-into-focused-private-modules
artifact: plan
---

# Plan

## Approach

Retain ParsedSpec and all established public and crate-visible parser paths through a small facade. Move whole implementations into private frontmatter, checked issue-reference, API symbol, and section-analysis owners, with owner-local topical tests. Keep the strict issue path separate from the compatibility parser and share only the delimiter scan it already uses.

## Out of scope

No public behavior, API signatures, diagnostics, accepted YAML syntax, delimiter policy, byte offsets, dependency, runtime, lifecycle consumer or trust-gate changes. Do not fix the documented four-dash or lifecycle status-update gaps. No push, human implementation approval, acceptance or archive action.

## Steps

1. Snapshot original parser source and extract its 69 passing tests from the existing pristine-main unit log.
2. Move complete frontmatter implementations and their helpers into a compatibility owner; expose the existing shared scan only to its sibling.
3. Move strict YAML issue deserialization and error sanitization, API symbol parsing and fence masking, and section analysis into private owners without altering bodies.
4. Group all existing tests beside their owners, retaining test names, bodies, attributes and literal contents, with every resulting file below 1000 lines.
5. Update canonical parser coverage, version, change log and companions; compare original function bodies, declarations, literals and compiled test inventory.
6. Obtain independent review of strict-versus-compatibility parsing, delimiter agreement, byte-preserving fence masking and API visibility.
7. Run focused parser tests, formatting, production clippy, debug/release checks, strict full coverage and the pre-push gate after shared-target access is granted. Parent runs the combined full suites against the pristine-main failure baseline.
8. Run real scoped lifecycle check and check --commit for a clean local checkpoint awaiting human implementation review.

## Risks

Strict checked YAML parsing must never fall back to the compatibility parser. CRLF normalization in parse_frontmatter deliberately differs from borrowed strip_frontmatter. The regex and shared scan must retain identical delimiter decisions. Fence masking must preserve byte lengths for --fix offsets, including Unicode input. Moving tests can hide accidental literal indentation changes, so compare exact literal contents and all test bodies as well as compiled inventory. Maintain ordering and deduplication semantics.

## Constraints consulted

Read AGENTS.md, the repository spec-sync skill, all Human Intent records, parser spec and every companion. Honor two separate frontmatter contracts; content-free checked errors; duplicate-key and positive-number validation; one delimiter scan plus regex agreement; CRLF normalization versus borrowed stripping; exact first-cell symbols and ordering; byte-preserving fence masking. Keep documented four-dash and lifecycle consumer gaps unchanged. No configured principles file exists. Scope derives from the owner's explicit repository-wide large-file refactor, continuation and branch/commit authorization; this is not a new feature or a human implementation review.
