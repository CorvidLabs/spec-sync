# Documentation

No user-facing documentation changes. The husk is an artifact of git's inability to represent
an empty directory; after this change it is neither produced nor surfaced, so there is nothing
for a user to learn or work around.

The two behaviours worth recording live in code comments at the sites that enforce them:
`prune_empty_package_directories` explains why pruning happens after validation, and
`is_untrackable_husk` explains why a directory holding files is refused rather than skipped.
