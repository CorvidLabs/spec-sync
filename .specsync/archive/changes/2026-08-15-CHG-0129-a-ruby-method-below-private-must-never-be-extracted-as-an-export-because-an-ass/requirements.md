---
change: CHG-0129-a-ruby-method-below-private-must-never-be-extracted-as-an-export-because-an-ass
artifact: requirements
---

# Requirements

`REQ-exports-00N` — Ruby visibility SHALL survive block forms that do not begin
a line. A method below `private` is never an export, and documenting one is an
orphan error rather than an accepted export.

Out of scope: Ruby constructs the extractor does not model, recorded in testing.
