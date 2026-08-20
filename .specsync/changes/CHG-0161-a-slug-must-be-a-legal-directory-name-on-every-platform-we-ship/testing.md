---
change: CHG-0161-a-slug-must-be-a-legal-directory-name-on-every-platform-we-ship
artifact: testing
---

# Testing

| Test | Discriminates | Proves |
|---|---|---|
| `the_slug_cap_bounds_the_directory_component_not_the_input` | yes | punctuation-heavy input now fills the slug; the old cap counted input characters and under-filled |
| `a_truncated_slug_does_not_end_mid_word` | yes | the exact trimmed string; a raw cut would end `...sierra-ta` |
| `an_ordinary_description_slugifies_exactly_as_before` | control | an ordinary description is byte-identical on both binaries |
| `a_description_that_slugifies_to_a_reserved_device_is_not_left_as_one` | new-unit | `NUL`, `con`, `COM1`, `lpt9`, and the empty fallback all avoid reserved names |

The boundary test asserts an **exact string** rather than the property "every segment is a whole
word". The property version passed on `origin/main` — at the old 80-character cap the cut lands
somewhere else and happened to be clean, so it discriminated nothing. This is the vacuous-test
failure mode this repository has hit before, caught here by running it against the old binary
before trusting it.

The reserved-device test cannot compile against `origin/main`, because it uses the predicate
this change makes `pub(crate)`. It is therefore a unit test of new behaviour, not a
discriminator, and is labelled as such rather than counted as one.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-083 | The cap test and the boundary test both fail against a separate checkout of `origin/main` and pass here, covering the two halves of "the cap must bound the component": counting the right thing, and cutting in the right place. The reserved-device test covers every name on the shared list plus the empty-input fallback. The control asserts an ordinary description is unchanged, so none of this was achieved by mangling slugs generally |
