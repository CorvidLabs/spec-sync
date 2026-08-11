## MODIFIED

### REQUIREMENT REQ-change-046

Agent-authored changes SHALL receive one current scoped review of implementation evidence before
finalization. Independent review remains mandatory by default; a solo maintainer may use the
explicit audited self-review exception only when its actor equals the scope approver and it records
a non-empty reason.

Acceptance Criteria

- Ordinary review binds the implementation parent commit, governed input digests, an explicit
  pass/block verdict, a stable reviewer claim distinct from the scope approver, and the exact
  required GitHub Actions check whose authenticated result is proven again by finalization.
- `--self-review --actor <scope-approver> --reason <reason>` records an explicit self-review mode,
  stable actor, non-empty reason, pass/block verdict, and the same implementation/contract/
  execution/workspace digest bindings in append-only review history.
- Self-review evidence never represents itself as an independent review or as authenticated by the
  required GitHub Actions review check; it replaces only the independent-identity requirement.
- Every persisted review attempt revalidates mode-specific identity and provenance against the
  scope approver bound to that attempt's contract digest; missing, malformed, mismatched, or
  ambiguous self-review evidence fails closed.
- Current passing review is still required for finalization; definition approval, verification,
  product CI/trust, and every-parent freshness validation remain mandatory.
- Existing v2 independent review evidence remains readable and valid.
- Status distinguishes independent and self-review evidence and gives the applicable next action.
