# Design

The prefix test is replaced by the properties it was standing in for. Every check that remains
asks what the string *is*; none asks how it begins.

| Check | Status |
|---|---|
| single path component | kept |
| no `/` or `\` | kept |
| no control characters | kept |
| non-empty | **added** |
| at most `MAX_CHANGE_ID_BYTES` (255) | **added** |
| not a reserved directory name | **added** |
| begins with `CHG-` | **removed** |

`.` and `..` were already rejected by the component check and are now pinned by test.

## Discrimination

Against a separate checkout of `origin/main`:

```
a_change_id_without_an_ordinal_is_accepted        FAILED   (prefix required)
an_unsafe_or_unbounded_change_id_is_still_refused FAILED   (no bound, no reserved check)
every_historical_identity_shape_remains_legal     passed   (control)
```

The control is what stops the refusal test being satisfied by rejecting everything. It asserts
the longest ID in the archive (90 bytes), the oldest, and `CHG-10000` — the five-digit shape a
CI fixture already exercises.
