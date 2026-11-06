---
hi: 1
families: [DOCS]
---

# Documentation you can follow

## Intent

The first thing a new user does is copy what the README and the landing page show them. Those examples should work when followed in the order written, with the hand-done steps named rather than skipped, and they should agree with what the pipeline and Chough actually enforce. When an example stops matching the tool, the pipeline should find out before a reader does.

DOCS-1 through DOCS-3.a were drafted from audit findings D5 (#881) and D11 (#883). Leif approved them as drafted on 2026-11-05 at 20:22 MST in the orc tab ("Approve all"), relayed by orc t18.

DOCS-4, DOCS-5 and DOCS-5.a were proposed in the documentation accuracy audit.
Leif approved their exact wording on 2026-11-06: "Capture all three", relayed by
orc t18 to the docs agent in Herdr. Recorded by Codex; this records the relayed
human decision, not authenticated identity. Scope approval and implementation
acceptance remain separate decisions.

## Criteria

- **DOCS-1**  What the README and the landing page show me runs as written, once I do the steps they tell me to do by hand.
  - **DOCS-1.a**  CI fails when one of those examples stops working, so a new user is not the one who finds out.
- **DOCS-2**  The documented closeout has me finish it locally (verify, implementation review, archive) before I push and wait for CI, because CI refuses a pull request whose change is still open.
  - **DOCS-2.a**  It says the same thing Chough teaches.
- **DOCS-3**  The landing page shows what I must do between creating a change and approving it, so approval does not refuse me for a step it never mentioned.
  - **DOCS-3.a**  If I skip one of those steps anyway, approval's refusal names the command that finishes it.
- **DOCS-4**  I can rely on the reference and contributor documentation for the current SpecSync version: its commands, settings, defaults, paths, platform guidance and links are accurate.
- **DOCS-5**  The diagrams show the states, transitions and steps that SpecSync actually uses.
  - **DOCS-5.a**  A diagram that no longer parses or disagrees with the lifecycle is caught before it reaches readers.
