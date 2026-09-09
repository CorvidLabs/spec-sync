---
change: pin-the-github-action-version-default-to-published-release-6-0-0-rc-14-so-consumers-without-an-explicit-version-do-not
artifact: docs
---

# Docs

## Position to state

**The Action's default `version` downloads a published release.** During the 6.0 candidate
window that is `6.0.0-rc.14`, not the unpublished package version `6.0.0`.

## Pages changed

| Page | Change |
|---|---|
| `action.yml` `version` description | Example cites `6.0.0-rc.14`; drop implication that stable `6.0.0` is downloadable |
| `site/.../github-action.md` inputs table | Default column `6.0.0` → `6.0.0-rc.14` |

## Pages deliberately not changed

- README / site YAML **pin examples** that set `version: '6.0.0'` with `@v6.0.0` — those are
  release-gate pin illustrations kept in sync with `Cargo.toml` by the validator, not the
  Action's omitted-input default.
- `docs/RELEASING.md`, overnight journals, confidence reports — historical / runbook prose.
