---
change: CHG-0014-route-archive-only-lifecycle-moves-through-a-minimal-specsync-ci-gate-while-pres
artifact: testing
---

# Testing

## Classifier Matrix

| Diff | Expected selection |
|------|--------------------|
| active deletion plus immutable archive addition | SpecSync only |
| active workspace edit that still exists | not archive-only |
| source, test, Cargo, workflow, Action, or release input | full CI |
| site-only | SpecSync plus site |
| VS Code-only | SpecSync plus extension |
| canonical spec or non-archive lifecycle metadata | SpecSync only |
| mixed archive and site | SpecSync plus site |
| unknown or manual dispatch | full CI |

Run:

- `.github/scripts/test-classify-ci-paths.sh`
- parse `.github/workflows/ci.yml` with the repository YAML tooling
- `fledge lanes run pre-commit`
- `fledge lanes run check`
- `fledge lanes run ci`
- `fledge lanes run repo`
- `specsync check --strict --require-coverage 100 --force`
- `augur check --staged`

## Local Results

- Classifier fixture matrix: passed.
- `actionlint .github/workflows/ci.yml`: passed.
- `shellcheck .github/scripts/*.sh`: passed.
- Ruby YAML parse: passed.
- `fledge lanes run check`: passed with 1,529 unit and 188 integration tests.
- `fledge lanes run ci`: passed.
- `fledge lanes run repo`: passed.
- Strict SpecSync validation: 62/62 specs, zero warnings, 100% file and LOC coverage.
- Current workflow and script diff classifies as `full=true`.
- `augur check --staged`: REVIEW, risk 38/100, confidence 62/100; no block verdict.
