---
change: document-shipped-specsync-6-0-0-and-set-the-action-default-to-the-stable-release
artifact: testing
---

# Testing

- `python3 -S .github/scripts/validate-release-version.py` exits 0.
- `specsync check --spec github --spec cmd_change` is clean under `--strict`.
- `rg '6.0.0-rc.14' action.yml site/src/content/docs/integrations/github-action.md` is empty except the callout that the tagged `@v6.0.0` Action still embeds that default.
- REQ-github-002 evidence: `action.yml` default `6.0.0` and the site inputs table match.
