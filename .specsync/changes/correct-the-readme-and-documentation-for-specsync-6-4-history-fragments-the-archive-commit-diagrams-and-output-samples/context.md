---
change: correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples
artifact: context
---

# Context

Leif asked, through his planning session (hc t4c), for the README and every current documentation page, Mermaid diagrams included, to say what SpecSync 6.4.0 and 6.4.1 actually do. The audit of main 3f0985e8 with its 6.4.1 build is in defs/docs-6.4-accuracy-definition.md: 21 items wrong or missing, each with file and line, and what was checked and found accurate.

Scope confirmed by t4c under Leif's standing OK for scope approvals (2026-11-20 dialog, 'Standing OK, tell me after'): documentation only, `--no-spec-change`, no requirement wording, no refusal, the recommended fix for each item. Version and pin wording stays out: the 6.4.1 post-publication PR owns it (docs/releases/6.4.1.md, enforced by validate-release-version.py). Historical records (the 6.0 journals, older release notes, the blog) are not rewritten. Code issues found were filed, not fixed (#1139, #1140).
