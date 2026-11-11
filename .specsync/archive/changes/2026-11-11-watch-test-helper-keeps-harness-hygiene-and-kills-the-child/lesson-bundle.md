# Lesson bundle: watch-test-helper-keeps-harness-hygiene-and-kills-the-child

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Watch test helper keeps harness hygiene and kills the child
- **Kind**: BugFix
- **Paths**: tests/integration/watch.rs, tests/integration/helpers.rs
- **Acceptance**: watch_until inherits the specsync() env hygiene and kills the child on assert/panic paths too

## Evidence

- Verification commit: `fad23cd3a18720c6627a54dc462284fbd62643ed`
- Base commit: `184bf8893a20289a2e044a137a11d4828a558761`
- Verified by: `specsync check (no spec in scope)`

## From the change's context.md

# Context

corvid-agent on #1053: watch_until dropped the harness env hygiene (SPECSYNC_*, GITHUB_*) and leaked the child on panic paths. helpers.rs gains specsync_std() as the single hygiene source (specsync() wraps it); watch_until uses it and a KillOnDrop guard.

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-watch-003 | the five watch integration tests |

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
