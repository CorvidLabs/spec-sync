---
change: CHG-0162-a-change-identity-must-be-validated-for-what-it-is-not-for-how-it-starts
artifact: testing
---

# Testing

| Test | Discriminates | Proves |
|---|---|---|
| `a_change_id_without_an_ordinal_is_accepted` | yes | a slug-only identity loads; both shapes are legal |
| `an_unsafe_or_unbounded_change_id_is_still_refused` | yes | traversal, separators, control characters, empty, over-length, and reserved names all refused |
| `every_historical_identity_shape_remains_legal` | control | the longest, oldest, and five-digit IDs in the archive stay legal on both binaries |

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-084 | The acceptance test fails against a separate checkout of `origin/main`, where the prefix is mandatory, and passes here. The refusal test also fails there — for the opposite reason, that two of its cases were unenforced — so it covers both halves: the prefix stops being required, and the properties that were only implied by the prefix start being checked. The control asserts every identity shape this repository has minted, including the 90-byte longest and the five-digit `CHG-10000`, so acceptance was not achieved by loosening and refusal was not achieved by rejecting |
