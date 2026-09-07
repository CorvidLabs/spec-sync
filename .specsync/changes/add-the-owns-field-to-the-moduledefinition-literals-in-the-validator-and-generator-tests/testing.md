---
change: add-the-owns-field-to-the-moduledefinition-literals-in-the-validator-and-generator-tests
artifact: testing
---

# Testing

- The existing `validator` coverage test and the two `generator` `find_files_for_module` tests compile and pass unchanged; they are the only code touched.
- `fledge run lint` (clippy, `-D warnings`) and `fledge lanes run verify` over the whole crate.
