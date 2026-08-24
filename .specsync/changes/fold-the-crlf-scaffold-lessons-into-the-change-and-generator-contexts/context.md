---
change: fold-the-crlf-scaffold-lessons-into-the-change-and-generator-contexts
artifact: context
---

# Context

The fold-back step `finalize` instructs, performed for the CRLF scaffold fix from the bundle it
left in the archive.

Two lessons were worth carrying, and neither is a restatement of what the change did.

**For `change`:** this module now diverges from the repository's normalize-then-parse convention.
That is a deliberate trade — keeping a borrowed `&str` return instead of allocating — but it means
the module owns a parser with its own CRLF dialect, and that its stripper is no longer
interchangeable with `view`'s. Both facts are the kind that decay silently; an earlier comment
claiming interchangeability was already false by the time anyone read it.

**For `generator`:** comparing against a raw template rather than an expanded one is a defect that
stays invisible behind an unrelated working step, and surfaces only when that step fails.
