---
change: complete-specsync-6-promotion-and-public-release-contracts
artifact: research
---

# Research

Confirmed source evidence: .github/scripts/validate-release-candidate.py defines REQUIRED_PLATFORMS=(ubuntu,macos). The release qualification matrix matches it. .github/workflows/release.yml contains two array-count guards requiring3; both reject the real required count2. Existing WorkflowSourceContractTests pins the matrix but does not exercise those guards.

src/cli.rs help currently says the required GitHub check authenticates the review. src/change.rs writes provider fields as constants and validates those fields; local finalization does not fetch GitHub identity evidence. The selected canonical REQ-change-046 criterion also overclaims an authenticated check result. Clarify that criterion transparently under human approval rather than leaving the public docs and canonical contract contradictory.

RC16 qualification receipts are exact-SHA-bound and passed both platforms. Runtime execution supplied to the independent reviewer was reported honestly as supplied evidence, not execution performed by Claude. No confidence target should be substituted for missing promotion evidence.

The current workflow dry_run=true dispatch selects a validation-only mode; it is not an execution of the promotion or publication jobs. Executable count-guard tests and read-only promotion-validator checks must supply that additional evidence. Do not claim a dry-run dispatch proves a tag push or package publication.
