---
change: pin-the-github-action-version-default-to-published-release-6-0-0-rc-14-so-consumers-without-an-explicit-version-do-not
artifact: tasks
---

# Tasks

- [x] `action.yml`: set `version` default to `6.0.0-rc.14`; update description example
- [x] `site/src/content/docs/integrations/github-action.md`: inputs table default → `6.0.0-rc.14`
- [x] `.github/scripts/validate-release-version.py`: accept candidate-window Action/docs default
      `6.0.0-rc.14` while package version remains `6.0.0`; keep README pin / CI mirror checks on
      package version
- [x] `specs/github`: amend REQ-github-002 (+ Purpose/testing/context/tasks companions) for the
      candidate-window published-asset default rule
- [x] Verify: `python3 -S .github/scripts/validate-release-version.py`;
      `gh release view v6.0.0-rc.14`; `specsync change check`
