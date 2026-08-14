---
change: CHG-0126-the-config-refusal-must-guard-both-loaders-because-load-config-is-a-second-door
artifact: docs
---

# Docs

CHANGELOG under Unreleased → Fixed, leading with `rehash`: the failure was not
only a wrong report but a written cache derived from configuration that never
loaded.
