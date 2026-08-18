## ADDED

### REQUIREMENT REQ-scoring-006

The git half of the freshness dimension SHALL report itself withheld when a cited file was deleted, rather than reporting a measured zero.

Acceptance Criteria
- A spec citing a deleted file yields a withheld git-freshness verdict instead of a measured verdict at zero commits behind.
- No additional penalty is applied for the deletion, because the file-existence criterion already charges for it; the reported score is unchanged.
