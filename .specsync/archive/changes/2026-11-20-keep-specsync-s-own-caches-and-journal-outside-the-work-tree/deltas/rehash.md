## ADDED

### REQUIREMENT REQ-rehash-003

`rehash` SHALL save the rebuilt cache only in the project's local state folder (REQ-hash-cache-009), SHALL print the path it wrote, and SHALL exit 1 with the reason when no local state folder can be found. It SHALL never write `.specsync/hashes.json` in the work tree.

Acceptance Criteria
- After `rehash` the work tree has no `.specsync/hashes.json` and the printed path exists.
