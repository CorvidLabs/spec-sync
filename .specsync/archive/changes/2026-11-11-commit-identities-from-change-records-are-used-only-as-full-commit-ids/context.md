---
change: commit-identities-from-change-records-are-used-only-as-full-commit-ids
artifact: context
---

# Context

The same class of input handling was hardened for `files_changed_since_base` in PR #1037's review.
A read-only sweep of the other Git calls that take values from change records found the sites this
change covers. REQ-change-013 already requires persisted workspace values to be validated before
use, so this is a bug fix with no requirement text.
