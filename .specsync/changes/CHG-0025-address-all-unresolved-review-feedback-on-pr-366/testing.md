---
change: CHG-0025-address-all-unresolved-review-feedback-on-pr-366
artifact: testing
---

# Testing

| Requirement | Focused Evidence |
|-------------|------------------|
| `REQ-change-026` | Unit fixtures cover five-digit IDs, protected ledger paths, archived exact baselines, removed IDs, singleton remnants, and active duplicates. |
| `REQ-change-027` | Integration fixtures run configured wrappers that re-enter `check`, `change`, and `lifecycle`, assert one non-zero exit, and preserve native command execution. |
| `REQ-change-028` | Effective-contract fixtures verify a non-conventional registry path and unsafe mappings; a digest probe or equivalent deterministic seam proves one project hash per scan. |
| `REQ-validator-004` | Static-only root and nested fixtures verify source discovery; design fixtures exercise every generated marker plus concrete and fenced negatives. |
| `REQ-config-002` | Focused config tests cover root HTML, nested CSS/HTM, ignored directories, and empty fallback. |
| `REQ-cli-003` | End-to-end CLI tests assert nested lifecycle-family dispatch fails before command execution. |

Run the narrow `change`, `validator`, `config`, and CLI tests first. Completion requires the full Fledge repository lane, `fledge spec check --strict`, `fledge trust verify`, Augur, Attest, and all hosted PR checks.

Focused implementation evidence: 106 lifecycle tests, six source-directory detection tests, the complete companion-marker fixture, and two recursive CLI integration cases pass. The first full repository run passed formatting, Clippy, type checking, 1,562 unit tests, 193 integration tests, release build, and dependency audit before the intentionally stale definition gate stopped the lane.
