---
change: CHG-0101-add-audited-solo-maintainer-self-review-override
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-046 | `change::tests::scoped_review_requires_an_independent_passing_verdict` proves ordinary self-identity rejection, mismatched/empty self-review rejection, durable self-review provenance, and current self-review freshness. |
| REQ-cmd-change-005 | Command tests cover mode-aware review rendering and ship-status guidance; JSON status exposes the persisted review projection. |
| REQ-cli-args-009 | `cli::tests::change_check_review_and_finalize_are_plain_commands` proves independent parsing remains compatible and validates the complete self-review grammar plus missing-audit-input rejection. |

Automated coverage will exercise the following matrix:

| Case | Expected result |
|---|---|
| Ordinary `--reviewer` review | Existing independent behavior and hosted-check provenance remain unchanged. |
| `--self-review --actor <scope approver> --reason <text>` | Appends a passing audited self-review bound to current verification/digests. |
| Self-review actor differs from scope approver | Fails without writing review evidence. |
| Missing/empty/invalid actor or reason | Fails before lifecycle mutation. |
| `--self-review` combined with `--reviewer` | Fails as an ambiguous review mode. |
| Self-review after stale or failed verification | Fails with the existing re-check guidance. |
| Legacy v2 independent review record | Continues to load and validate. |
| Text/JSON status and ship status after self-review | Mark the review as self-reviewed and retain product/trust/finalization gates. |

Run the configured CHG-0101 commands (`cargo test change::`, `cargo test commands::change::`,
`cargo test cli::tests::`, full `cargo test`, and both release-pin scripts), then run
`fledge trust verify`.
