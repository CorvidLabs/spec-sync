---
change: CHG-0129-a-ruby-method-below-private-must-never-be-extracted-as-an-export-because-an-ass
artifact: tasks
---

# Tasks

1. Recognise a block opener wherever it appears on the line, not only as the
   first token, so push and pop stay balanced.
2. Keep the visibility region bound to the correct nesting depth.
3. Tests for both the desyncing form and the statement form that never broke.
4. Invert the #479 bug pin in sandbox drill 039, which pins the OLD behaviour.
