---
change: CHG-0133-extract-the-change-module-s-tests-into-their-own-file-so-the-file-that-manufactu
artifact: research
---

# Research

The plan for this warned: "Verify deletions by counting `#[test]` markers before
and after rather than trusting the removal output — a brace-matching bug
silently ate five unrelated functions earlier because these files contain `{`
inside string literals."

That warning shaped the method. Boundaries were established by printing the
exact lines rather than by matching braces, and the result was checked by count
at three levels: test functions, passing tests, and stripped line content.
