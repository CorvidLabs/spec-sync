---
change: extract-issues-command-integration-tests-without-changing-their-behavior
artifact: context
---

# Context

Leif asked to split oversized SpecSync files into focused, tested work and publish reviewable PRs. This candidate extracts one existing integration-test family only. The large process file shrinks from 5,962 to 4,874 lines; all 26 complete test functions, attributes and inline fixture bytes are conserved. Shared helpers, environment isolation and product behavior are unchanged.

Hosted PR #820 initially failed its strict lifecycle audit because the four meaningful changed paths lacked an active change record. The repository guide calls SDD optional, but the actual CI policy requires coverage here. The original agent-proposed draft recorded that scope without weakening CI or inventing human approval. On 2026-10-16 Leif directly approved the prepared scope, conditional on actual local testing and at least 95% engineering confidence. The supported writer recorded actor 0xLeif and the direct response is preserved in ci-cleanup-human-approval.json. Current qualification, actual independent implementation review and supported finalization remain separate evidence-bound gates.
