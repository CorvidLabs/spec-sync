## ADDED

### REQUIREMENT REQ-change-014
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
