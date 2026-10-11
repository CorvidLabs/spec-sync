## MODIFIED

### SPEC SECTION Change Log

| Date | Change |
|------|--------|
| 2026-10-11 | #813: Adapt coverage fixtures to manifest binding metadata. |
| 2026-10-10 | Add `cli.3md` beside the spec. The first plane is context and later planes are lessons. `context.md` stays. |
| 2026-07-26 | Fix #417: forward resolved output format to compact/archive-tasks so JSON shorthand and Markdown are honored |
| 2026-07-10 | v5: dispatch the verified `specsync change` SDD lifecycle |
| 2026-06-11 | v4: `--root` now errors (exit 2) for nonexistent paths; init scenario covers the v4 config layout |
| 2026-04-10 | Add Performance Requirements section with response time targets, cache requirements, resource limits, and scalability targets |
| 2026-03-25 | Initial spec |
| 2026-04-06 | Add compact, archive-tasks, view, merge, issues subcommands; add --force, --create-issues, --format flags; add hash_cache/github/archive/compact/view/merge dependencies |
| 2026-04-09 | Add scaffold, report, comment, changelog subcommands; add --enforcement and --explain flags; add --agents hook target; add comment/changelog/deps dependencies |
| 2026-07-11 | CHG-0003-finalize-specsync-5-0-release-consistency-and-parallel-validation: Finalize SpecSync 5.0 release consistency and parallel validation |
| 2026-07-11 | CHG-0007-harden-specsync-5-0-as-an-agent-native-secret-free-sdd-core-and-close-release-r: Harden SpecSync 5.0 as an agent-native, secret-free SDD core and close release regressions |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-14 | CHG-0025-address-all-unresolved-review-feedback-on-pr-366: Address all unresolved review feedback on PR 366 |
| 2026-07-14 | CHG-0029-address-all-remaining-review-feedback-from-pr-366: Address all remaining review feedback from PR 366 |
| 2026-07-22 | CHG-0062-harden-mcp-root-confinement-write-authorization-argument-validation-and-notif: Harden MCP root confinement, write authorization, argument validation, and notification semantics for issue 414 |
| 2026-07-26 | v12 / CHG-0063 Windows root-retention remediation: Preserve the requested root spelling for MCP and coverage-gating commands so junction/symlink replacement remains observable after capability retention, including through generation publication |
| 2026-07-27 | CHG-0063-close-independent-mcp-security-review-gaps-for-issue-414: Close independent MCP security review gaps for issue 414 |
| 2026-07-27 | CHG-0065-make-issue-417-changelog-compaction-idempotent-and-provide-truthful-portable-str: Make issue 417 changelog compaction idempotent and provide truthful portable structured maintenance output |
| 2026-07-30 | CHG-0068-stabilize-specsync-6-0-with-a-low-churn-normal-workflow-preserved-audited-guara: Stabilize SpecSync 6.0 with one scope approval, same-PR finalization, lightweight archive CI, scoped review, and selected UX fixes |
| 2026-07-31 | CHG-0069-scoped-change-check-change-audit-and-agent-pack-for-the-two-verb-lifecycle: Scoped change check, change audit, and agent pack for the two-verb lifecycle |
| 2026-07-31 | CHG-0070-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes: Land pre-6.0 product fixes for hooks init coverage naming and exit codes |
| 2026-08-01 | CHG-0071-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes-scoped: Land pre-6.0 product fixes for hooks init coverage naming and exit codes (scoped paths) |
| 2026-08-13 | CHG-0109-a-symlink-under-a-source-directory-must-be-skipped-and-disclosed-never-abort-di: A symlink under a source directory must be skipped and disclosed, never abort discovery |
| 2026-08-14 | CHG-0121-coverage-over-zero-source-files-must-report-nothing-measured-everywhere-replac: Coverage over zero source files must report nothing measured, everywhere: replace the precomputed percentage fields with Option-returning accessors so no renderer can substitute 100 percent for an unasked question |
| 2026-08-17 | CHG-0142-watch-must-disclose-a-dropped-directory-and-not-claim-a-pass-over-an-empty-check: Watch must disclose a dropped directory and not claim a pass over an empty check |
| 2026-08-27 | drop-the-windows-binary-from-the-6-0-release-matrix-and-stop-claiming-windows-support-while-keeping-every-windows: Drop the Windows binary from the 6.0 release matrix and stop claiming Windows support, while keeping every Windows-content correctness guarantee |
| 2026-08-27 | v23 / #723: `CoverageReport` also carries manifest discovery that was degraded rather than allowed to veto an explicitly configured `source_dirs` |
| 2026-08-27 | a-configured-source-dirs-must-survive-a-manifest-discovery-failure-and-an-in-repo-includebuild-must-be-judged-by-its: A configured source_dirs must survive a manifest discovery failure, and an in-repo includeBuild must be judged by its path rather than its token |
| 2026-08-27 | a-configured-source-dirs-must-survive-a-manifest-discovery-failure-and-an-in-repo-includebuild-must-be-judged-by-its: A configured source_dirs must survive a manifest discovery failure, and an in-repo includeBuild must be judged by its path rather than its token |
| 2026-09-09 | close-the-specsync-6-0-0-p1-release-defects-found-in-overnight-proving: Close the SpecSync 6.0.0 P1 release defects found in overnight proving |
| 2026-10-11 | bind-discovered-cargo-crate-sources-during-generation: Bind discovered Cargo crate sources during generation |

