# Lesson bundle — classify-the-archive-commit-that-finalize-records

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Classify the archive commit that finalize records
- **Kind**: BugFix
- **Paths**: tests/integration/change.rs
- **Acceptance**: The integration test classifies the archive commit that change finalize records, and that commit is the child of the implementation commit.

## Evidence

- Verification commit: `af08729f59a667f1bed5f467a5e7f8db6e58cb14`
- Base commit: `efa8046bc3bb3d70f006304b7ceef3fa285725cb`
- Verified by: `specsync check (no spec in scope)`

## From the change's context.md

# Context

`change finalize` commits the archive move. The integration test still staged `.specsync` and diffed that index against HEAD. After the commit, that diff is empty, so the classifier reported `archive_only=false` and CI failed the test job.

The test now classifies `HEAD^..HEAD` for `.specsync` and checks that the archive commit's parent is the implementation commit.

## From the change's testing.md

# Testing

`cargo test --test integration -- eight_workflow_v2_changes_finalize` passed. Each of the eight finalizations classified `archive_only=true`, and the archive commit's parent was the implementation commit.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
