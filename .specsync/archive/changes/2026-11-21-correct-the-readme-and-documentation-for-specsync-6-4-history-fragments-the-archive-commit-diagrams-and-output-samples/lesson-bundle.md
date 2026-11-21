# Lesson bundle: correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Correct the README and documentation for SpecSync 6.4: history fragments, the archive commit, diagrams and output samples
- **Kind**: Documentation
- **Paths**: README.md, AGENTS.md, docs/HLD.md, docs/ADOPTING.md, site/src/content/docs/, site/src/content/examples/rust-workspace.mdx
- **Acceptance**: (1) The 21 items in defs/docs-6.4-accuracy-definition.md are corrected as written, with no other wording changed. (2) The command check script reports nothing new. (3) Each output sample changed is the output of a real run. (4) cd site && bun test passes, Mermaid and lifecycle checks included, and bun run build passes. (5) validate-release-version.py and specsync check --strict pass.

## Evidence

- Verification commit: `8d45da4c1a2dd53294f4252c6fa1fd60d508f8cc`
- Base commit: `3f0985e80acab0fe4559fe322cf929be40057414`
- Verified by: `specsync check --spec cmd_change`

## From the change's context.md

# Context

Leif asked, through his planning session (hc t4c), for the README and every current documentation page, Mermaid diagrams included, to say what SpecSync 6.4.0 and 6.4.1 actually do. The audit of main 3f0985e8 with its 6.4.1 build is in defs/docs-6.4-accuracy-definition.md: 21 items wrong or missing, each with file and line, and what was checked and found accurate.

Scope confirmed by t4c under Leif's standing OK for scope approvals (2026-11-20 dialog, 'Standing OK, tell me after'): documentation only, `--no-spec-change`, no requirement wording, no refusal, the recommended fix for each item. Version and pin wording stays out: the 6.4.1 post-publication PR owns it (docs/releases/6.4.1.md, enforced by validate-release-version.py). Historical records (the 6.0 journals, older release notes, the blog) are not rewritten. Code issues found were filed, not fixed (#1139, #1140).

## From the change's design.md

# Design

Each correction says what the code does, in the page's own style, at the place the item names. Output samples are replaced by output from real runs of the 6.4.1 build. Mermaid diagrams gain only the jobs and steps the workflows have, and keep their shape, so `site/scripts/documentation-accuracy.test.ts` still parses them and matches the lifecycle against the code.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
