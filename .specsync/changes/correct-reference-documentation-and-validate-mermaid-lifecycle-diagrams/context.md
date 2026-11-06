# Context

## Human authority

Leif's task was to make the README, docs and Mermaid diagrams accurate. A read-only audit produced 22 findings at /Users/leif/Development/_CorvidLabs/_worktrees/ss-docs-findings.md, pinned originally to e6b1ce11. t18 received that report before fixes. On 2026-11-06 t18 relayed Leif's exact decision, “Capture all three”, approving DOCS-4, DOCS-5 and DOCS-5.a as proposed. Codex captured the exact sentences with fledge hi and recorded that relay in hi/docs.md. This is a relayed claim, not authenticated identity.

Scope approval is pending: t18 explicitly requested `ready for approval <id> at <commit>` and will record Leif's approval. No native approval, test implementation, prose fix, push or PR is authorized by an agent label. Later implementation acceptance remains separate.

## Evidence and baseline

Preparation base is main 7ae7245f97e7cfd66cd80ce39b611becb1095642. The branch was clean before fast-forward. Fledge built this checkout in Herdr t130. Audit commands used the separately built e6b1ce11 binary (hash and exact receipts retained in /private/tmp/ss-docs-audit-e6b1ce11/evidence). The audit covered 71 visible help pages, hidden accept help, selected disposable command/lifecycle fixtures, 51 external links and every Mermaid fence. All 11 diagrams were reviewed manually; offline CLI parser unavailable. No parser pass, full suite pass, deployment or cross-platform parity is claimed from that audit.

## Decisions and constraints

Use one documentation-focused change; maintain the agents and github contracts through deltas. Generated skill text is corrected at its template source and regenerated, never hand-edited only in installed copies. Add parser and source-backed semantic checks to the already-required site test lane instead of duplicating README/landing workflow harnesses owned elsewhere. Test the current bad prose/diagram first and retain the red result. Current source determines facts; historical release records stay historical. No network link monitor is added to routine CI.

## Workflow recovery dependency

Chough next refuses several sibling sessions without an active pointer. Chough session new docs-accuracy refuses `.chough/session.json exists; run chough session adopt <key> first`. t18 prohibited adopting/moving those records and assigned recovery to ss-linux-arm64/#941. No sibling checkpoint has been modified. Native session creation and this change's Chough checkpoint wait for #941. Only this session's new records and evidence may be created under the declared .chough/ prefix.

## Open dependencies

#935 edits the exact HLD inherited-context row. t18 assigned its additional lifecycle omission (F22) here, but the correction waits for #935 to land; no overlapping line is edited while it is open. Its ss-contracts follow-up issue number is pending. The protected-hunk inventory and complete intended scope are in plan.md. No other agent was spawned.
