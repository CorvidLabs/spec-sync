---
change: extract-issues-command-integration-tests-without-changing-their-behavior
artifact: context
---

# Context

Leif asked to split oversized SpecSync files into focused, tested work and publish reviewable PRs. This candidate extracts one existing integration-test family only. The large process file shrinks from 5,962 to 4,874 lines; all 26 complete test functions, attributes and inline fixture bytes are conserved. Shared helpers, environment isolation and product behavior are unchanged.

Hosted PR #820 initially failed its strict lifecycle audit because the four meaningful changed paths lacked an active change record. The repository guide calls SDD optional, but the actual CI policy requires coverage here. This agent-proposed draft records that scope without weakening CI or inventing human approval. Human scope approval, review, acceptance and finalization remain pending.
