---
change: address-every-review-comment-on-the-archive-and-pull-request-commands
artifact: testing
---

# Testing

- `cargo test --bin specsync` covers the plan-section, lesson-comment, and archive-ranking tests for REQ-change-105 and REQ-change-106.
- `.github/scripts/test-archive-readiness.sh` covers accepted ranking, a blocking review, and an archived state left in the active tree for REQ-cmd-change-019.
- `python3 .github/scripts/test-required-ci-gate.py` covers Archive failing the required gate without failing implementation ready for REQ-cmd-change-019.
- REQ-cmd-change-020 is the `change pr` contract: one JSON result, an open pull request is reused, a closed one is not, and merge is not said while a change is open.
- REQ-cmd-rules-002 is the principles listing, including a malformed policy.
