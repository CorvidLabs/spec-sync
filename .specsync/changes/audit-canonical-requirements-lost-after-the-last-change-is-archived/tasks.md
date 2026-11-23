---
change: audit-canonical-requirements-lost-after-the-last-change-is-archived
artifact: tasks
---

# Tasks

- [ ] Add an integration case that removes an existing requirement after the final package has been archived. Establish that both audit modes currently pass.
- [ ] Expose the branch-wide comparison in audit when there are zero active workspaces. Keep active-change diagnostics intact.
- [ ] Cover an authorized branch removal, an older archive, a moved requirement and both missing-history cases.
- [ ] Verify the archived-tip command used by the Lifecycle gate, update the change spec and run the project verification lanes.
