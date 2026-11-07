---
change: a-warm-status-of-an-archived-change-does-not-digest-the-whole-working-tree-to-report-its-review-currency
artifact: requirements
---

# Requirements

- REQ-change-147: an archived change's `scoped_review_current` does not take the whole-tree project-input digest once its archive is committed, so a warm archived status does not grow with the files a project tracks.

Traces SPEED-1.a and SPEED-1.d.
