---
change: pin-the-github-action-version-default-to-published-release-6-0-0-rc-14-so-consumers-without-an-explicit-version-do-not
artifact: testing
---

# Testing

## Automated

| Command | Covers |
|---|---|
| `python3 -S .github/scripts/validate-release-version.py` | Action default + docs default accept `6.0.0-rc.14`; README pins and CI/Trust mirrors still match package `6.0.0` |
| `specsync check --spec github` | Living github module still coherent after REQ-github-002 amendment |
| `specsync change check` | Materialize deltas + scoped verification for this change |

## Manual / dogfood

- `gh release view v6.0.0-rc.14` lists the five archives + sha256 sidecars
- `gh release view v6.0.0` → not found
- Default URL shape `.../releases/download/v6.0.0-rc.14/specsync-linux-x86_64.tar.gz` resolves
  (HTTP 200 or redirect to asset), while `.../v6.0.0/...` does not

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-github-002 | action.yml default `6.0.0-rc.14`; docs table matches; validator passes; live release view shows assets |
