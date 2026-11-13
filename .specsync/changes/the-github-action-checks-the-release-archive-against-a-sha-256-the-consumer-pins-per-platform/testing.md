---
change: the-github-action-checks-the-release-archive-against-a-sha-256-the-consumer-pins-per-platform
artifact: testing
---

# Testing

| Requirement | Evidence |
|---|---|
| REQ-github-031 | `bash .github/scripts/test-verify-release-digest.sh` (red without the script, green with it): match, mismatch, no line, duplicate, malformed, padded lines |
| action input safety | `cargo test --test integration config::` action.yml checks |

CI runs the script test in the `validate-action` job.
