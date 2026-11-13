---
change: a-sent-back-no-spec-change-can-declare-that-it-changes-specs
artifact: delta
module: change
---

## ADDED

### REQUIREMENT REQ-change-160

`change answer <id> no_spec_change no` SHALL declare that a draft change created with
`--no-spec-change` changes specs after all. It SHALL clear `no_spec_change` and its rationale,
so the next approval needs a delta for each affected spec. An approved, implementing or
verifying change SHALL be sent back with `change send-back` first. `no_spec_change yes` SHALL be
refused, naming `change new --no-spec-change --rationale`, and so SHALL `no` on a change that
already declares spec changes (#1070).

#### Acceptance Criteria

- A no-spec change, approved and then sent back, answers `no_spec_change no`, adds a delta, and
  is approved, materialized and verified with the delta applied.
- `no_spec_change yes` and a second `no` are refused, and an approved change is refused until
  it is sent back.
