---
change: add-the-owns-field-to-the-moduledefinition-literals-in-the-validator-and-generator-tests
artifact: design
---

# Design

- No design: fixture literals gain a field with its empty default. `..Default::default()` was not used because the neighbouring literals spell every field out, and matching them keeps the diff to one line per site.
