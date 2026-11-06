---
change: keep-a-passing-check-green-when-the-action-s-pr-comment-is-refused-for-a-read-only-token
artifact: requirements
---

# Requirements

- REQ-github-024 (added): a refused comment degrades to a warning plus a job-summary copy, and exits 0. Other errors still fail the step. Traces CI-3.b.
