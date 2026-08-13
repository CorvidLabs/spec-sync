---
id: CHG-0110-stop-printing-green-result-lines-for-checks-that-could-not-run-when-frontmatter
state: implementing
type: bug_fix
base_commit: 7bd6c0ac75ecf83bf680a303d3146709021423f1
---

# Stop printing green result lines for checks that could not run when frontmatter is invalid

## Intent

Stop printing green result lines for checks that could not run when frontmatter is invalid

## Affected Canonical Specs

- `commands`

## Acceptance Criteria

- When a spec's frontmatter cannot be parsed, `specsync check` reports the source-file, required-section, and dependency checks as skipped rather than printing a green result line for each. A spec whose frontmatter is valid continues to report those three checks normally, and the same spec body with valid frontmatter still reports every genuinely missing required section, proving the previous green line was false rather than merely vacuous.

## No-spec Rationale

Not applicable
