---
change: document-shipped-specsync-6-0-0-and-set-the-action-default-to-the-stable-release
artifact: plan
---

# Plan

1. Record artifacts and semantic deltas for REQ-github-002 current default and github Purpose.
2. Flip Action default, validator constant, site table, CHANGELOG date, README/MIGRATION/ADOPTING/RELEASING/SECURITY.
3. Lead site index with check-as-product.
4. Run `python3 -S .github/scripts/validate-release-version.py` and `specsync check --spec github --spec cmd_change`.
