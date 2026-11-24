---
change: keep-archived-foreign-change-directories-outside-adoption-validation-and-import
artifact: context
---

# Context

Issue #432 established that adoption previews validate the same source as an import. The shared import helper lives in `src/change/validate_loaded_change.rs`, outside the original change’s named paths. Both workflows inspect active changes and preserve the foreign archive directory in place.
