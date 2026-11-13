---
change: document-change-status-json-as-the-supported-ci-check-of-an-archived-change-with-its-trust-model
artifact: docs
---

# Docs

- New: `site/src/content/docs/integrations/ci-archive-check.md` (section Integrations, order 3).
  It holds the check, the passing and failing fields, `--timeout` and exit 124, a GitHub Actions
  example with `fetch-depth: 0`, and the trust model: re-derived, trusted, remembered, ignored.
- Changed: `site/src/content/docs/integrations/github-action.md` gains one sentence after the
  archive gate, linking the new page (relative `.md` link, which the hub rewrites).
- Follow-up outside this repository: a CorvidLabs/site navigation entry, so the hub lists the page.
