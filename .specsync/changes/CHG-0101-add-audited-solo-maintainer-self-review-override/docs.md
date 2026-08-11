---
change: CHG-0101-add-audited-solo-maintainer-self-review-override
artifact: docs
---

# Docs

Document the exception beside the ordinary lifecycle review command:

```text
specsync change review CHG-… --self-review --actor <scope-approver> --reason "solo maintainer"
```

The documentation will state that this is an audit-visible exception for a solo maintainer. It is
not an independent review, does not impersonate GitHub review provenance, and does not bypass
verification, trust, CI, or finalization checks. Team workflows continue to use
`--reviewer <independent-reviewer>`.
