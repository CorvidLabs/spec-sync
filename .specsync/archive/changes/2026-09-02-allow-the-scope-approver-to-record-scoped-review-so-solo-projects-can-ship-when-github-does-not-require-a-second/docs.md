---
change: allow-the-scope-approver-to-record-scoped-review-so-solo-projects-can-ship-when-github-does-not-require-a-second
artifact: docs
---

# Docs

ADOPTING.md "Things that will bite you" currently tells solo adopters to pick a second identity. Replace that with: scoped review may be the same actor as the approver; GitHub required reviews remain the two-person gate when a repo wants one. AGENTS.md shipping path drops `<other-than-approver>`. Generated agent skill text in `src/agents.rs` matches.
