## ADDED

### REQUIREMENT REQ-change-122

A definition approval SHALL record, for every `## MODIFIED` and `## REMOVED` delta item, a digest of the canonical block it was approved against. When such an item is not already applied and its canonical block no longer matches that recorded base, `change check` SHALL refuse before writing any canonical file and `change audit` SHALL report the same refusal. The refusal SHALL name the module and block and SHALL name rewriting the delta body onto the current canonical text, `change approve` and `change check` as the remedy. An approval that recorded no bases is unknown, not violated, and keeps materializing as before.

#### Acceptance Criteria

- A stale MODIFIED item whose block main rewrote after approval is refused by `change check`, both before and after the first materialization, and the canonical file keeps main's bytes.
- `change audit` reports the same refusal for that change.
- The refusal names the module, the requirement or section, and `specsync change approve <id>`.
- A rebased and re-approved delta materializes, keeping the other change's text it now carries.
- A block main changed that this change does not touch is not refused.
- An approval without recorded bases keeps materializing as before.
