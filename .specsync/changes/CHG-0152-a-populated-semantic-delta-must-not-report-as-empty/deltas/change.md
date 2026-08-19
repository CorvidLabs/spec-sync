## ADDED

### REQUIREMENT REQ-change-075

A semantic delta parser SHALL distinguish a file that is empty from a file that has content
but no recognized operation heading, SHALL name the allowed operation headings in that
second case, SHALL accept item headings case-insensitively, and SHALL apply the same empty
versus unrecognized wording on the historical delta path.

Acceptance Criteria
- A file whose only content is prose or unrecognized text reports that it contains no recognized operation headings and names `## Added`, `## Modified`, and `## Removed`, instead of reporting that the file is empty.
- A file that is empty or whitespace-only still reports that it is empty.
- `### requirement` and `### spec section` parse as `### REQUIREMENT` and `### SPEC SECTION`.
- An unrecognized `##` heading is still refused and names the allowed operation values.
- An unrecognized `###` heading before any item is still refused and still names both valid item forms.
- A `###` line that is not an item keyword, met while an item is open, remains that item's content.
- A valid uppercase delta still parses to the same items.
- The historical delta walk uses the same empty-versus-unrecognized distinction and does not report a populated unrecognized file as empty.

## MODIFIED

### SPEC SECTION Error Cases

| Condition | Behavior |
|-----------|----------|
| Missing acceptance criteria or affected scope | Definition approval fails |
| Missing or invalid semantic delta | Approval, verification, and unified check fail |
| Populated semantic delta with no recognized operation heading | Approval and historical validation name the allowed `## Added`, `## Modified`, and `## Removed` headings instead of reporting the file empty |
| Verification command contains shell operators | Command is rejected without execution |
| HEAD changes after verification | Acceptance requires re-verification |
| Any intervening commit changes a disallowed path, even if later reverted | Status and strict checking require re-verification in every environment |
| Accepted delivery evidence is still current | Reopen is rejected without changing lifecycle or audit state |
| Reopen actor or reason is empty | Reopen is rejected before any mutation |
| Concurrent changes edit the same semantic key | Progress requires dependency ordering or rebase |
| Ownership correction is not exact, additive, in-scope, and canonically provable | Correction is rejected transactionally |
| Covered delivery input of an accepted change changes with no covering accepted successor | Unified check names the input path, its owner, and the `change reopen` remediation |
| Covered delivery input changes while every covering successor is itself stale | Unified check names the input, the sorted covering successor IDs, and their stale evidence state |
| Covered delivery input disappears from the current inventory | Unified check names the missing path and the restore-or-reopen remediation |
| Non-inert local registry cannot be parsed while resolving a module | Canonical path resolution fails closed with `failed to parse local registry {path} while resolving `{module}`` |
| A repeated stage-zero path has a different mode or object ID | Git candidate inspection fails closed without replacing the first observation |
| Correction ledger is invalid when a definition mutation acquires the project lock | Mutation emits the safe integrity diagnostic and persists no lifecycle update |
