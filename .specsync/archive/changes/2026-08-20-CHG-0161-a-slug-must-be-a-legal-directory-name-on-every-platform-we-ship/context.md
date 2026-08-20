---
change: CHG-0161-a-slug-must-be-a-legal-directory-name-on-every-platform-we-ship
artifact: context
---

# Context

Step 2 of the change-identity work, and a prerequisite for step 3 rather than an independent
tidy-up. Relaxing `validate_change_id` to accept an ID without the `CHG-NNNN` prefix is what
makes the slug the whole path component — and therefore what makes `slugify("NUL")` produce a
directory Windows cannot open. The guard has to exist before the prefix goes, not after.

The reserved-name predicate already exists, extracted, with the full list, in
`src/commands/mod.rs`. `src/importer.rs:195` already routes its slug through the module-name
validator for exactly this reason. `change.rs` was the one place minting directory names from
free text without it — the sibling-site pattern this release keeps paying for, so this reuses
the predicate rather than restating the list.
