---
change: CHG-0162-a-change-identity-must-be-validated-for-what-it-is-not-for-how-it-starts
artifact: context
---

# Context

Step 3 of the change-identity work, and the one that actually opens the door: until
`validate_change_id` stops requiring `CHG-`, no slug-only identity can be loaded at all.

It is deliberately sequenced after step 2. Accepting an arbitrary name is what makes the slug
the whole path component, which is what makes `slugify("NUL")` produce a directory Windows
cannot open — so the guard had to exist first. This change is where that guard starts being
load-bearing rather than defensive.
