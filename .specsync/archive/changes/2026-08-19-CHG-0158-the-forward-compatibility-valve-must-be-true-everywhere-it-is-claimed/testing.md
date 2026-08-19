# Testing

## New

| Test | Discriminates | Proves |
|---|---|---|
| `agents::tests::a_manifest_written_by_a_newer_six_is_still_usable` | yes — `unknown field \`future_record_field\`` | a committed manifest from a later 6.x is readable; a manifest missing a required field is still refused |
| `change::tests::a_policy_written_before_a_field_existed_still_loads_and_fails_closed` | yes — `missing field \`enabled\`` | an older `sdd.json` loads, and the filled defaults enforce rather than relax |
| `change::tests::a_baseline_is_still_frozen_by_its_canonical_byte_gate` | control (passes on both) | the limit is pinned rather than contradicted; starts failing if the gate moves |

## Rewritten

`change::tests::regenerable_caches_still_reject_what_they_cannot_understand` — the version it
replaces passed with `deny_unknown_fields` stripped from both `hash_cache.rs` structs, because
its payload was malformed in a different way. The rewrite fails there, and carries a control
asserting the payload is otherwise valid so a future field rename cannot re-vacuum it.

`change::tests::evidence_written_by_a_later_six_still_parses` — the `WorkflowV2Baseline` case
was true at type level and false in operation; replaced with `FinalizationRecord`, which has no
byte gate.

## Suite

`cargo test`: 2327 unit + 405 integration, 0 failures. `cargo fmt --check` clean. `cargo clippy
--all-targets` produces 19 warnings on this branch against 21 on `origin/main` with the same
local toolchain (1.89.0, newer than CI's pin); none is on a line this change touches.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-079 | The cache criterion is now true of the file it names: `regenerable_caches_still_reject_what_they_cannot_understand` is rewritten against `hashes` rather than a field `HashCache` does not have, and fails in a copy of this tree with `deny_unknown_fields` stripped from both `hash_cache.rs` structs — the version it replaces passed there. The canonical-bytes limit is pinned by `a_baseline_is_still_frozen_by_its_canonical_byte_gate`, which shows the type tolerating the unknown field and the byte round trip then rejecting it, and `evidence_written_by_a_later_six_still_parses` no longer asserts the opposite by carrying a `WorkflowV2Baseline` case; it carries `FinalizationRecord`, which has no byte gate. The digest claim is unchanged and still held by the CHG-0068 golden vector |
| REQ-change-080 | `a_policy_written_before_a_field_existed_still_loads_and_fails_closed` loads an `sdd.json` missing `enabled` and `require_change_for_meaningful_files` and asserts both fill in as enforcing. It fails with `missing field \`enabled\`` in a copy of this tree with the container-level `#[serde(default)]` reverted |
| REQ-agents-005 | `a_manifest_written_by_a_newer_six_is_still_usable` reads a manifest carrying an unknown field at both the record and the manifest level and confirms the three fields this binary needs survive; its control asserts a record missing `template_version` is still refused, so the tolerance cannot be read as accepting any shape. It fails with `unknown field \`future_record_field\`` in a copy of this tree with the two attributes restored |
