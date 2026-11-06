# Documentation accuracy plan

## Outcome and traceability

One documentation-focused PR resolves the audit's non-overlapping reference and diagram findings under approved DOCS-4, DOCS-5 and DOCS-5.a, with existing DOCS-2/DOCS-2.a for archive-first closeout. Leif approved intent only: “Capture all three”, relayed by orc t18 on 2026-11-06. Scope approval and implementation acceptance remain human gates. No approval has been inferred from this plan.

## Approach

- Correct README/contributor/reference prose about staging, automatic archive commits, send-back recovery, ownership-check boundaries, CI order/classification, defaults, versions, release matrices, WSL 2, hub redirects, agent integration names, planned draft paths, .3md lessons, separate deps/score commands, generated registry shape, Action-wrapper version boundaries and broken links.
- Correct all affected HLD Mermaid states, edges and labels against current code; preserve historical dated release records and distinguish future six-target builds from published five-archive v6.2.0 assets.
- Add a Bun documentation test file in the existing site test lane. Use the official Mermaid parser (a pinned development dependency and lockfile, with a DOM adapter only if the parser requires it) for every maintained Mermaid fence. Assert lifecycle nodes against ChangeState and the send-back edges against SEND_BACK_STATES, plus source-backed staging/archive/CI guards. Include mutations that demonstrate missing states/edges, bad syntax and stale behavior claims fail. No network link checks in routine CI; retain dated manual HTTP receipts.
- Update generated SpecSync skill prose in src/agents.rs and its four checked-in copies, advance only the artifact template version, regenerate the manifest with the built binary, and update REQ-agents-009 to say ship/finalize creates the archive commit. This is generated documentation and associated tests/metadata, not a lifecycle behavior change.
- Update the two copied Chough closeout references for this repository's archive-first order. Coordinate the same wording with the canonical Chough owner; no global skill or other repository files are changed by this PR.
- Add REQ-github-027 for CI validation of maintained Mermaid syntax and lifecycle correspondence, derived from DOCS-5/DOCS-5.a. Record test strategy and lessons in the agents/github companions.
- Record only this change's Chough session, cases and evidence after #941 makes native session creation possible. The .chough/ declared prefix is solely for unpredictable native event/evidence filenames; it authorizes no edits, moves or archival of sibling sessions or their evidence.

## Protected hunks and dependencies

Read-only diff inventory on main 7ae7245f is in /private/tmp/ss-docs-plan/prs. #912/#916/#910 are merged in this base. Preserve their installed behavior. Open PRs #900/#901/#903/#918/#920/#934/#935/#936/#941/#943/#944 were checked. Re-read live diffs before implementation and before review.

- README approval/authentication paragraph belongs to #900; CLI identity/relay paragraphs belong to #900 and read-timeout blocks to #918. Edits to other sections must not rewrite those lines.
- Workflow task-gate material belongs to #903; no workflow.md edits are planned.
- Landing page and its harness are #936/#917; no index.md or documented_workflows.rs edits are planned. The local-verify harness issue #906 stays with its owner.
- HLD line 791 is currently edited by #935. F22 (add lifecycle to inherited-context refusal) is assigned to this PR by t18, but its edit waits until #935 lands; preserve that PR's wording about inherited callers and no SpecSync-created verification child.
- #941 owns main's legacy/sibling Chough checkpoints. Do not adopt, archive, move or commit their recovery. Session creation has twice refused the legacy session.json; this is a recorded dependency, not authorization to bypass it.
- Action Windows diagnostic/source behavior, upstream Chough changes, Marketplace publication and third-party product changes remain with their owners. This PR corrects descriptions and links, not those products.

## Steps

1. Small: capture approved HI and prepare this exact native scope, paths, deltas and testing plan. Stop at the requested scope gate.
2. Medium: after approval and session recovery, add the failing documentation/diagram checks and generated-skill regression first. Save red output on the existing prose; do not weaken expectations to obtain green.
3. Medium: apply source-backed prose/diagram fixes outside protected hunks, regenerate skill artifacts, and run focused checks. Fix HLD F22 only after #935 is integrated.
4. Small: resolve external links against primary sources and record the dated reachability result; distinguish unavailable Marketplace listing from source/VSIX installation options actually provided by this repo.
5. Medium: run the required local gates, review the full path/diff inventory, and prepare one PR for human implementation review. Archive locally only after that acceptance, then push and wait on CI.

Chough timing could not run because there is no unambiguous active session and legacy recovery belongs to #941. History is unavailable for this session; sizes above are relative work increments, not time estimates or deadlines.

## Verification and delivery

Run targeted Bun tests and their mutations, site tests/lint/build, generated-artifact parity tests, existing release-version/runtime-pin guards, strict SpecSync check and score. Run full verify and trust verification before claiming complete. Slow commands use Herdr terminals. Before every push run both fledge lanes run pre-push and the repository-pinned specsync change audit --strict; the latter checks affected_paths coverage and is not replaced by check --commit or verify.

Native ordering: finish source/Chough records -> change check --commit -> local gates/current evidence -> human implementation acceptance -> change review -> change ship/finalize without an intervening commit -> audit/pre-push on the archive -> push -> hosted CI. No product-tip CI wait before archival. No tag, release, publication, force push, weakened bound, timeout or gate.

## Risks

- Mermaid parsing is syntax evidence, not a proof that prose or all graph semantics match code. Explicit state/edge/sequence assertions and manual source review remain separate.
- The parser may need a DOM adapter in Bun; pin only test dependencies and report exact parser coverage. Do not silently replace parser failure with a manual pass.
- New main commits and open PR merges may change facts/line numbers. Refresh against the actual head before fixes and keep the audit's original pin intact.
- Documentation tests must exercise facts or deliberate mutations rather than merely mirror arbitrary wording. Do not duplicate the protected README/landing harness work.
- Preserve archive history and actor/provenance boundaries. A green local gate is not human acceptance, hosted CI or release readiness.

## Out of scope

Executable CLI/lifecycle behavior, installer safety mechanics, Action runtime diagnostics, release/platform matrices, workflow classifier logic, protected README/landing harnesses, upstream Chough source and Marketplace publication are outside this change. No sibling Chough recovery or open-PR changed line is edited. Generated documentation text, its template version/manifest and focused checks are inside scope; other source edits are not.

## Constraints consulted

specs/agents/agents.3md planes 4–6 require generated instructions and tracked copies to stay together, their template version to advance and the manifest to come from the installer. specs/github/github.3md and REQ-github-023 require archive-first CI; immutable wrapper/version records retain their exact version boundaries. hi/docs.md carries the approved human wording. The audit findings and t18's directives require failing checks first, current protected-hunk reads, separate human gates and `specsync change audit --strict` before every push. Native Chough recovery belongs to #941.
