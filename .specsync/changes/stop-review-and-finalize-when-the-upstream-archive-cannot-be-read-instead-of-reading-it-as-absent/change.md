---
id: stop-review-and-finalize-when-the-upstream-archive-cannot-be-read-instead-of-reading-it-as-absent
state: draft
type: bug_fix
base_commit: ae3d7bcad107701057bc43667422c5abbed7ce55
---

# Stop review and finalize when the upstream archive cannot be read instead of reading it as absent

## Intent

Stop review and finalize when the upstream archive cannot be read instead of reading it as absent

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- SHIP-3.c: when a remote default exists and its archived state records cannot be listed or read (a failed or truncated ls-tree or cat-file --batch, or a failed lookup of a matching package's addition or ancestry), `change review` and `change finalize` refuse before acceptance and write no archive.
- The refusal names the remote default reference, says the tool cannot tell whether the change is already archived there, and names how to find out: fetch the remote default, then list its archived packages for this id.
- `change status` and `change ship-status` report the same refusal, never `safe` or ready to finalize.
- A project with no remote default, or a remote default whose archive is read and holds no copy of the change, is unchanged: nothing is reported and review and finalize proceed.
- A unit test and an integration test that remove the archived record's blob from the clone fail on the base first and pass after.

## No-spec Rationale

Not applicable
