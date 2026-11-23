## ADDED

### REQUIREMENT REQ-change-archived-requirement-loss-001

On a SpecSync-managed branch with zero active change workspaces, `change audit` SHALL compare the tracked canonical requirement IDs with their IDs at the merge base of HEAD and the default branch. It SHALL use REQ-change-169 to find canonical files, compare IDs across files, and validate removals from changes archived on this branch after the merge base. For each unaccounted loss it SHALL report the missing ID, its original file and the command to display it for restoration. The same loss SHALL fail default audit, strict audit and archive audit. The comparison SHALL be read-only. When the default branch is known but its merge base is unavailable, audit SHALL report the incomplete history and the fetch needed for the comparison. With no default branch reference, audit SHALL disclose that the comparison was skipped.

Acceptance Criteria

- A merge resolution drops a requirement after finalization. The full-history required Lifecycle gate runs strict audit and rejects the archived tip with the missing ID and its restoration command.
- The branch archives a change with an approved REMOVED item for the missing ID. The removal is accounted for. An archive that already existed at the base cannot authorize the deletion on this branch.
- The branch moves a requirement between canonical files while retaining its ID. Audit passes this comparison.
- The checkout has a default ref but too little history to find the merge base. Audit refuses with fetch guidance. A checkout without a default ref reports that the comparison was skipped.
