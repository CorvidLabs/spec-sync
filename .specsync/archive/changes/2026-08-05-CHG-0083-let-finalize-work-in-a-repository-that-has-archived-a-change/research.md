---
change: CHG-0083-let-finalize-work-in-a-repository-that-has-archived-a-change
artifact: research
---

# Research

`finalize` failed with `Git returned an out-of-scope index path` naming an
archived change from a previous release — a path unrelated to the change being
finalized.

Reproduced directly against git: a `:(top,literal)` pathspec naming a directory
expands to every tracked file beneath it.

    git ls-files --stage -z -- ":(top,literal).specsync/archive/changes/<dir>"
    -> 12 files, the first being approvals.json

The scope guards compared each returned path against the candidate set by exact
membership. The set holds only the directory, so the first expansion was
rejected.

This fires only where an archive exists. Every sandbox drill and Rust fixture
builds from an empty temp repo, so the entire suite was blind to it.
