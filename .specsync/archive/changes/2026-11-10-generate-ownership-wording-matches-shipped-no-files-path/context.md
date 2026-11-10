---
change: generate-ownership-wording-matches-shipped-no-files-path
artifact: context
---

# Context






Round-1 review of #611 (PR #1025): the requirement text said "produces no spec" while the code
(and the pre-existing no-files path) writes the spec with files: []. This change rewords the
requirements to the shipped behavior and adds the missing `new` coverage. The archived round-1
workspace bullet about config.modules ("but generated specs still never double-map") contradicted
REQ-generator-009 AC3; the canonical wording stands: explicit mappings are the author's
declaration and are not filtered.
