## ADDED

### REQUIREMENT REQ-agents-004

Generated agent artifacts SHALL be tracked by a versioned digest manifest so upgrades preserve
customized files and report conflicts.

Acceptance Criteria

- Installation records artifact path, tool, template version, and digest in a project-local manifest.
- Unchanged generated artifacts update idempotently.
- Customized artifacts are never overwritten or deleted and produce an actionable conflict.
- Uninstall removes only digest-matching managed artifacts and preserves shared directories.
- Legacy installations are adopted only when their bytes match a known generated template.
