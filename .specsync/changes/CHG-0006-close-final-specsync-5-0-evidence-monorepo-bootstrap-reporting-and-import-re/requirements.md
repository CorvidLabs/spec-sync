---
change: CHG-0006-close-final-specsync-5-0-evidence-monorepo-bootstrap-reporting-and-import-re
artifact: requirements
---

# Requirements

## REQ-change-014

The lifecycle SHALL preserve evidence, canonical truth, project-root isolation, bootstrap usability, and import safety through acceptance and archival.

Acceptance Criteria
- Accepted changes remain valid only while verification matches current delivery inputs, and archive revalidates the same evidence.
- Archive eligibility is attributable to the specific accepted change rather than overlapping path coverage from another change.
- Trusted policy lookup and meaningful changed paths are relative to the requested project root.
- Canonical specs require lifecycle coverage and adoption covers its protected policy bootstrap.
- A no-spec declaration cannot accompany a declared public-contract change.
- OpenSpec and Spec Kit imports reject symlinked files and directories.
- Rejected foreign imports leave no partial adoption policy, report, or imported content.
- The exact schema-v1 self-adoption record is the sole migration exception to the no-spec/public-contract rule.

## REQ-cmd-comment-002

Generated pull-request comments SHALL report SDD lifecycle failures even when a project has no canonical spec files.

Acceptance Criteria
- Empty canonical discovery does not bypass SDD checking in comment mode.
- SDD-only errors render a failing comment with actionable detail.

## REQ-cmd-init-004

Initialization SHALL enable Git-dependent SDD coverage only when the project can provide Git comparison evidence.

Acceptance Criteria
- Git repositories receive normal strict SDD defaults.
- Non-Git directories initialize successfully without an immediately impossible changed-path gate.
