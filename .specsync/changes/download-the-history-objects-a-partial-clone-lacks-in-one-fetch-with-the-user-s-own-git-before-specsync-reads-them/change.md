---
id: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
state: implementing
type: bug_fix
base_commit: bdbcbd8e3ee70b89cd4934e69f8e8739073fc79e
---

# Download the history objects a partial clone lacks in one fetch with the user's own Git before SpecSync reads them

## Intent

Download the history objects a partial clone lacks in one fetch with the user's own Git before SpecSync reads them

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- (1) In a blobless clone whose promisor remote answers only with the user's configuration, change review of a change created in an earlier commit succeeds, and the objects it read are present afterwards. (2) The download is one fetch, whatever the number of objects. (3) When the remote cannot be reached, the refusal says the clone is partial and names a git fetch command with the object ids that, run as printed, lets the same command succeed. (4) A full clone makes no fetch. (5) The fetch is one call bounded at 60 seconds and never prompts; one line on standard error says how many objects and from which remote. (6) change status, change audit and change ship-status in a partial clone fetch the same way.

## No-spec Rationale

Not applicable
