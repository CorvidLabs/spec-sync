---
id: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
state: implementing
type: feature
base_commit: 65864de8194c10a26f674a7a171c500ae7ba5792
---

# Record each shipped change in a history fragment file instead of a Change Log row and a version bump

## Intent

Record each shipped change in a history fragment file instead of a Change Log row and a version bump

## Affected Canonical Specs

- `change`
- `cmd_change`
- `cli_args`
- `cli`
- `validator`
- `hash_cache`

## Acceptance Criteria

- After change check materializes a change, each affected module has specs/<module>/history/<change-id>.md with the change id, date and title, and the spec's version: and Change Log are byte for byte unchanged; two changes to one module shipped on separate branches merge with plain git (no merge driver) without a conflict in that spec; specsync check refuses a malformed fragment and names it; specsync history <module> prints the frozen table rows unchanged followed by one row per fragment

## No-spec Rationale

Not applicable
