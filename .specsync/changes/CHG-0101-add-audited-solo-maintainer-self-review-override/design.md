---
change: CHG-0101-add-audited-solo-maintainer-self-review-override
artifact: design
---

# Design

`change review` has two exclusive modes:

| Mode | Required inputs | Identity rule | Provenance |
|---|---|---|---|
| Independent (default) | `--reviewer` | Must differ from the scope approver | Required GitHub Actions review check |
| Audited self-review | `--self-review --actor --reason` | Actor must equal the scope approver | Explicit local audited exception; never a hosted-review claim |

Review records evolve additively within the established v2 schema: historical evidence defaults to
independent mode, while new records carry an explicit mode and, for self-review only, a validated
reason. Validators validate every record according to its declared/defaulted mode. The append-only
attempt ledger still protects prior block/pass events.

The text status line calls out `self-review` rather than `independent review`; JSON includes a
stable mode plus actor/reason fields. Ship guidance continues to require a green product tip and
trust before finalization, but does not instruct a solo maintainer to wait for an unavailable
independent reviewer once a valid self-review is recorded.
