## ADDED

### REQUIREMENT REQ-change-145

When the trusted pre-v2 cutoff check of a workflow-v1 change (REQ-change-136) cannot read the archive tree at the cutoff, it SHALL return an error saying that tree could not be read, and SHALL NOT answer that the change was absent at the cutoff (SPEED-1.a.3). Only a cutoff whose own tree has no archive root SHALL count as having no archived candidates. The error SHALL never be cached.

Acceptance Criteria
- In an offline `--filter=tree:0` clone that never fetched the cutoff's archive tree, the check returns an error naming the unreadable archive tree, not `false`.
- A cutoff with no archive root gives the same verdict as before.
