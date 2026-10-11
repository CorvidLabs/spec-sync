## MODIFIED

### SPEC SECTION Change Log

| Date | Change |
|------|--------|
| 2026-10-11 | #813: Retain manifest source mappings for generation without rediscovery. |
| 2026-10-10 | Add `validator.3md` beside the spec. The first plane is context and later planes are lessons. `context.md` stays. |
| 2026-07-22 | v12: add checked coverage so malformed Gradle discovery fails CLI and MCP gates as inconclusive while retaining the compatibility report wrapper |
| 2026-07-22 | v13 / CHG-0063: add `validate_spec_content` so capability-rooted callers validate exact pre-read snapshots without reopening spec paths |
| 2026-07-23 | v14 / CHG-0063 independent review: keep raw drive-qualified modules, unsupported `setProjectDir` forms, and linked/reparse-backed Gradle source roots inconclusive across CLI and MCP gates |
| 2026-07-23 | v15 / CHG-0063 adversarial rereview: keep linked/reparse-backed Gradle manifests plus interpolated or encoded Gradle paths inconclusive across checked gates |
| 2026-07-23 | v16 / CHG-0063 final security rereview: snapshot coverage roots and bytes through retained no-follow handles so post-discovery symlink/junction replacement is inconclusive before outside reads |
| 2026-07-23 | v17 / CHG-0063 post-review hardening: share one retained project authority across manifest/spec/source coverage and enforce deterministic iterative byte, entry, depth, UTF-8, and identity bounds |
| 2026-07-23 | v18 / CHG-0063 acceptance remediation: read caller-selected spec ownership frontmatter and every recognized manifest through the same retained project authority before source coverage |
| 2026-07-23 | v19 / CHG-0063 exact-head review remediation: preserve nested config/manifest reachability and selected-spec identity continuity, lazily autodetect omitted source roots, and charge selected-spec/source bytes and entries within checked coverage |
| 2026-07-24 | v20 / CHG-0063 independent rereview remediation: Preserve bounded scan fallback after malformed manifest autodetection and retain selected source-directory identities from post-manifest selection through checked coverage traversal |
| 2026-07-24 | v21 / CHG-0063 exact-head rereview remediation: Replace retained sibling handles with identity records and sequential capability reopen so broad checked coverage remains descriptor-bounded |
| 2026-07-24 | v22 / CHG-0063 descriptor-breadth remediation: Select configured source-root identities without retaining all handles, then reopen and traverse roots sequentially so breadth cannot exhaust descriptors |
| 2026-07-10 | v5: keep coverage regression fixtures warning-free under current stable Clippy and document the intentionally in-file test-module layout |
| 2026-07-10 | v5: make canonical requirements companions adaptive rather than empty mandatory ceremony |
| 2026-07-02 | v4: add `source_within_root` — shared guard rejecting `files:` paths that escape the project root (absolute/`..`/symlink); applied in `validate_spec` and every export-extraction site (score, check --fix, diff, new) to close an out-of-root identifier-disclosure vector |
| 2026-06-11 | v3: `validate_spec` populates `ValidationResult.status` with the parsed lifecycle status so callers can report draft skips |
| 2026-06-07 | Update draft-only section warning wording |
| 2026-03-25 | Initial spec |
| 2026-04-06 | Document archive, compact, merge as consumers of find_spec_files; note hash_cache integration for incremental validation |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-14 | CHG-0024-stabilize-specsync-5-lifecycle-integrity-and-strict-validation-for-5-0-2: Stabilize SpecSync 5 lifecycle integrity and strict validation for 5.0.2 |
| 2026-07-14 | CHG-0025-address-all-unresolved-review-feedback-on-pr-366: Address all unresolved review feedback on PR 366 |
| 2026-07-14 | CHG-0034-support-extensionless-source-discovery-through-an-explicit-include-extensionless: Support extensionless source discovery through an explicit include_extensionless setting while preserving omitted and empty source_extensions defaults, with parser, scanner, strict file coverage, LOC coverage, and wizard regressions for extensionless-only and mixed projects |
| 2026-07-14 | CHG-0035-count-mjs-and-cjs-files-as-default-typescript-sources-so-mapped-and-uncovered-mo: Count mjs and cjs files as default TypeScript sources so mapped and uncovered module files contribute to strict file and LOC coverage denominators |
| 2026-07-14 | CHG-0039-allow-draft-specs-to-declare-planned-missing-source-mappings-without-failing-str: Allow draft specs to declare planned missing source mappings without failing strict validation while preserving path safety ownership enforcement exact coverage and complete notice contracts |
| 2026-07-27 | CHG-0063-close-independent-mcp-security-review-gaps-for-issue-414: Close independent MCP security review gaps for issue 414 |
| 2026-07-30 | CHG-0068-stabilize-specsync-6-0-with-a-low-churn-normal-workflow-preserved-audited-guara: Stabilize SpecSync 6.0 with one scope approval, same-PR finalization, lightweight archive CI, scoped review, and selected UX fixes |
| 2026-07-31 | CHG-0070-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes: Land pre-6.0 product fixes for hooks init coverage naming and exit codes |
| 2026-08-01 | CHG-0071-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes-scoped: Land pre-6.0 product fixes for hooks init coverage naming and exit codes (scoped paths) |
| 2026-08-13 | CHG-0107-fix-the-first-five-minutes-of-spec-sync-init-leaves-a-repo-that-fails-check-sc: Fix the first five minutes of spec-sync: init leaves a repo that fails check, scaffold writes prose that check rejects, and a directory in files: makes check silently green |
| 2026-08-13 | CHG-0108-stop-reporting-success-for-checks-that-did-not-happen-gate-drafts-that-document: Stop reporting success for checks that did not happen: gate drafts that document a contract over present source, drop cold-cache drift noise, and stop taking quoted frontmatter paths literally |
| 2026-08-13 | CHG-0109-a-symlink-under-a-source-directory-must-be-skipped-and-disclosed-never-abort-di: A symlink under a source directory must be skipped and disclosed, never abort discovery |
| 2026-08-14 | CHG-0117-a-config-file-that-exists-but-cannot-be-loaded-must-refuse-to-run-not-report-su: A config file that exists but cannot be loaded must refuse to run, not report success over built-in defaults |
| 2026-08-14 | CHG-0118-a-config-file-that-exists-but-cannot-be-loaded-must-refuse-to-run-not-report-su: A config file that exists but cannot be loaded must refuse to run, not report success over built-in defaults |
| 2026-08-14 | CHG-0121-coverage-over-zero-source-files-must-report-nothing-measured-everywhere-replac: Coverage over zero source files must report nothing measured, everywhere: replace the precomputed percentage fields with Option-returning accessors so no renderer can substitute 100 percent for an unasked question |
| 2026-08-14 | CHG-0124-a-source-file-or-spec-body-carrying-an-unresolved-merge-conflict-must-be-refused: A source file or spec body carrying an unresolved merge conflict must be refused, because extracting declarations from both sides of a hunk describes source that does not exist |
| 2026-08-15 | CHG-0130-gradle-module-identity-must-come-from-the-project-name-like-every-other-manifest: Gradle module identity must come from the project name like every other manifest, not from a source path segment, because both the first and last segment collapse a whole tree into one module |
| 2026-08-17 | CHG-0137-coverage-must-not-invent-a-module-over-files-that-are-all-mapped: Coverage must not invent a module over files that are all mapped |
| 2026-08-17 | CHG-0141-a-directory-named-in-files-must-score-zero-not-eighty: A directory named in files: must score zero, not eighty |
| 2026-08-27 | v37 / #723: A manifest that cannot be parsed no longer vetoes an explicitly configured `source_dirs`; it degrades to a `manifest_notices` entry, and stays fatal only when the source list itself came from discovery |
| 2026-08-27 | a-configured-source-dirs-must-survive-a-manifest-discovery-failure-and-an-in-repo-includebuild-must-be-judged-by-its: A configured source_dirs must survive a manifest discovery failure, and an in-repo includeBuild must be judged by its path rather than its token |
| 2026-08-27 | a-configured-source-dirs-must-survive-a-manifest-discovery-failure-and-an-in-repo-includebuild-must-be-judged-by-its: A configured source_dirs must survive a manifest discovery failure, and an in-repo includeBuild must be judged by its path rather than its token |
| 2026-10-11 | split-source-files-so-none-is-longer-than-5-000-lines: Split source files so none is longer than 5,000 lines |
| 2026-10-11 | bind-discovered-cargo-crate-sources-during-generation: Bind discovered Cargo crate sources during generation |

### SPEC SECTION Invariants

1. Validation is bidirectional: phantom documented exports are errors and undocumented code exports
   are warnings.
2. Missing required frontmatter fields are errors.
3. Cross-project references are skipped during local validation.
4. Coverage excludes tests and configured patterns.
5. Source discovery honors configured extensions.
6. Spec discovery is sorted.
7. Schema extraction honors the configured pattern.
8. Missing-file suggestions use bounded Levenshtein distance.
9. Flat source-module detection excludes common entry points.
10. Empty required sections are reported as unfinished content.
11. Validation results retain parsed lifecycle status.
12. Requirements companions are validated when present under adaptive artifact policy.
13. Checked coverage propagates malformed, unreadable, unsupported, or unconfined manifest
    discovery WHEN the source list it would measure came from that discovery — that is, when
    `source_dirs` was not stated. A project that stated `source_dirs` is not overruled by a failure
    to infer what it already said: discovery degrades to an empty result and the error is carried
    as a coverage notice, never as a veto. The notice is not optional, because manifest modules
    also seed module attribution, so a degraded run reports fewer modules without specs than the
    tree holds. Compatibility coverage remains available. Coverage source enumeration and content
    reads remain bound to one retained project-root capability after manifest discovery, and any
    replacement or non-regular endpoint makes the checked result inconclusive.
14. `validate_spec_content` validates caller-provided bytes without reopening `spec_path` or
    adjacent companions; the path remains logical diagnostic/source context and mapped sources
    retain normal path behavior.
15. Schema-aware validation compares quoted, qualified, and mixed-case declarations canonically;
    invalid patterns, missing configured snapshots, and declared tables without schema evidence
    remain visible findings rather than vacuous success.
15. `validate_spec` reads a path once and delegates the exact bytes to the shared content validator.
16. `validate_spec_content_with_sources` treats its supplied `SourceSnapshot` map as authoritative
    and does not reopen mapped sources or resolve supplied-content TypeScript wildcards through
    ambient paths.
17. Checked coverage uses retained no-follow source snapshots; symlink, reparse, or identity
    replacement fails before outside reads, partial totals, or generation.
18. Caller-selected spec ownership, manifest/spec-module/source discovery, and final verification
    share one retained project capability; deterministic iterative traversal enforces 8 MiB/file,
    64 MiB total, 100,000 entries, 256 components, strict UTF-8, and special-entry rejection.
19. Configuration and zero-config source detection begin after root retention; selected-spec/source
    bytes and entries share one budget, nested config/manifest parents remain reachable, explicit
    source roots avoid autodetection, and retained identities remain authoritative.
20. Early and post-discovery race checkpoints independently cover retained acquisition and later
    traversal inside checked coverage; callers propagate those failures without claiming
    command-wide retained authority.
21. Checked spec/source traversal records sibling identities and reopens children sequentially
    through retained parents, bounding live handles by depth while preserving identity and
    reachability checks around recursion.
22. Configured source-root selection stores stable identities without retaining every root handle;
    traversal reopens, identity-checks, consumes, and releases each root sequentially.


- Checked coverage retains manifest module source paths in its report from the same retained manifest observation used for module attribution. Degraded discovery leaves this mapping empty.
