## ADDED

### REQUIREMENT REQ-change-146

The trusted pre-v2 cutoff check of a workflow-v1 change (REQ-change-136) SHALL consider the change's own resolved package path, the archive directory `find_change_dir` finds by the id its state declares, alongside the active state path and the cutoff archive entries ending in `-<id>`. A workflow-v1 archive the history walk accepted SHALL NOT be refused because its directory name does not end in its id (SPEED-1.a.2). The record found at the cutoff SHALL still carry the same id and a workflow-v1 identity.

Acceptance Criteria
- A workflow-v1 archive moved before the cutoff to a directory not ending in `-<id>` is accepted by the cutoff-tree check exactly as the history walk accepts it, and the change loads.
- A record absent at the cutoff, added after it, or present at it only under another id is still refused with the existing message.
