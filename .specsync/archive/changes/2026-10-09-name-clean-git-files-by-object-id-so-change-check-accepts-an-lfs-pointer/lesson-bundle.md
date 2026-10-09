# Lesson bundle — name-clean-git-files-by-object-id-so-change-check-accepts-an-lfs-pointer

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Name clean Git files by object id so change check accepts an LFS pointer
- **Kind**: Feature
- **Specs**: change
- **Paths**: src/change.rs, src/change_tests.rs, docs/HLD.md
- **Acceptance**: A clean filter=lfs file is named by its Git object id and is not read from the worktree. A dirty copy of that file changes the workspace digest. Other content filters still fail. Scoped acceptance still hashes the bytes of the paths the change covers. The 256 MiB bound still applies to bytes that are read.

## Evidence

- Verification commit: `f213f03454aeb7c596635bd7f2b9dd68b4f744ae`
- Base commit: `6a47f2cd0c6dc5dfa4168b79ac6dda960966bcfc`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

`change check` records one workspace digest over every Git-visible project file. Published 6.0.0 rejects any Git content filter, including `filter=lfs`, and a pointer-aware build still loads every clean blob and sums those bytes against the 256 MiB payload bound.

A clean tracked file is already a Git object. The workspace digest names that object id. It reads bytes only for dirty or untracked paths, and those reads stay inside the existing bound. `filter=lfs` is the pointer object. Every other content filter still fails closed. A change's own acceptance evidence still hashes the bytes of the paths it covers.

The digest domain is `specsync.project-input-digest.v3`. A recorded v2 digest does not match a v3 recompute, so an in-flight verification is stale until `change check` records a new one. Archives stay history.

Peck keeps LFS enabled and does not edit its ledger. This change does not raise the 256 MiB cap.

## From the change's design.md

# Design

`PROJECT_DIGEST_DOMAIN` is `specsync.project-input-digest.v3`. `project_input_digest` calls discovered-evidence capture with `identify_clean_blobs` set. That flag is part of the evidence cache key.

When the flag is set, a clean regular file (`100644` or `100755`) contributes its object id and is left out of `git cat-file`. A clean symlink is still read so its target can be checked. Dirty files, untracked files, and non-Git reads still count against `MAX_GIT_EVIDENCE_PAYLOAD_BYTES` (256 MiB).

`validate_git_attribute_output` continues only for `filter=lfs`. `ident`, `working-tree-encoding`, and any other filter still fail closed.

Acceptance discovery, definition snapshots, and archive snapshots call the same capture with `identify_clean_blobs` clear, so they still hash bytes. `git_evidence_with_policy` and `git_worktree_state` do the same.

A stored v2 workspace digest will not match a v3 recompute. `change check` records the new digest. Archived changes are not re-hashed as the product check.

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-103 | `git_lfs_clean_file_is_named_by_its_object_id` names a clean `filter=lfs` file by its Git object id, shows that a dirty copy changes the workspace digest, and leaves other content filters fail-closed. Scoped acceptance tests still hash covered path bytes. |

`git_lfs_clean_file_is_named_by_its_object_id` builds a repository whose clean and smudge filters are `cat`, commits a pointer, and checks that a clean `filter=lfs` file is named by its object id. It then rewrites the worktree file and checks that the workspace digest changes.

`custom_content_attributes_fail_before_index_substitution` and `fsmonitor_valid_and_ident_conversion_fail_closed` keep every other content filter fail-closed. Scoped acceptance tests still compare payload digests to file bytes. The payload bound stays 256 MiB for bytes that are read.

## Where these lessons go

- `specs/change/context.md`
