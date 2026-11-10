---
id: report-states-the-require-coverage-verdict-when-another-finding-decides-the-exit
state: implementing
type: bug_fix
base_commit: 82c724703d5f6b89fe0a68dc672d4b1dbd8e41dc
---

# Report states the require-coverage verdict when another finding decides the exit

## Intent

report states the require-coverage verdict when another finding decides the exit

## Affected Canonical Specs

- `cmd_report`

## Acceptance Criteria

- report --require-coverage N in a tree whose staleness cannot be measured still exits 1, and its text and markdown output state the coverage gate's verdict (met or not met, with the measured coverage) and name the findings that set the exit status; --require-coverage 0 reads met; a run where nothing pre-empts the gate prints exactly what it printed before

## No-spec Rationale

COVERAGE-2 says a pipeline can refuse work when coverage falls below the bar, and REQ-cmd-report-003 keeps the staleness refusal; outside git the refusal pre-empted the --require-coverage gate so it never reported. This prints the gate's own verdict beside the refusal without changing any exit code or requirement
