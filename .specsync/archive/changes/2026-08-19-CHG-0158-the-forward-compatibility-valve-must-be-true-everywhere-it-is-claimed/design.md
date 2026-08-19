# Design

Four edits, each the smallest one that makes a stated claim true.

## 1. `agents.rs` — the manifest is evidence

Drop `#[serde(deny_unknown_fields)]` from `AgentArtifactRecord` and `AgentArtifactManifest`.
The three fields stay required, so nothing starts accepting a manifest it cannot use; the
structs only stop refusing fields they do not need.

Not chosen: adding a container-level `#[serde(default)]` here as well. `AgentArtifactRecord`
has no `Default`, and giving it one would make `tool: ""` a legal record — a fail-open in a
file whose whole job is to say which tool wrote which artifact.

## 2. `change.rs` — the comment says what the code does

The old text asserted `agents.rs` discards and rebuilds. It does not. The replacement states
where the line falls and why for each of the three files, and names the two baselines as
getting nothing from tolerance, with a pointer to the test that pins it.

## 3. `change.rs` — `SddPolicy` takes a container-level `#[serde(default)]`

Uses the existing `Default`, so a field added in a later 6.x needs no per-field attribute to
stay readable. Fails closed by construction: the default policy is the enforcing one.

## 4. Tests

- `regenerable_caches_still_reject_what_they_cannot_understand` is rewritten against `hashes`
  and asserts the rejection names `future_cache_field`, so a malformed payload can no longer
  masquerade as the guard. A control asserts the same payload parses without the unknown field,
  so a future rename cannot make the assertion unreachable again.
- `evidence_written_by_a_later_six_still_parses` swaps its `WorkflowV2Baseline` case for
  `FinalizationRecord`, which has no byte gate, so every case in that test is now a case where
  tolerance is the operative thing.
- `a_baseline_is_still_frozen_by_its_canonical_byte_gate` is new and pins the limit: the type
  tolerates the field, the file-level round-trip does not. Its `assert_ne!` is written so that
  it starts failing if the gate ever moves.
- `a_manifest_written_by_a_newer_six_is_still_usable` and
  `a_policy_written_before_a_field_existed_still_loads_and_fails_closed` cover the two new
  tolerances, each with a control for the fail-closed half.

## Discrimination

Measured, not assumed. In a scratch copy of this tree with only the two production edits
reverted and every test kept:

```
a_manifest_written_by_a_newer_six_is_still_usable            FAILED
  unknown field `future_record_field`, expected one of `tool`, `template_version`, `digest`
a_policy_written_before_a_field_existed_still_loads…         FAILED
  missing field `enabled`
regenerable_caches_still_reject_what_they_cannot_understand  passed
a_baseline_is_still_frozen_by_its_canonical_byte_gate        passed
```

The two that pass on both binaries are the vacuity controls: they describe behaviour this
change deliberately leaves alone, so passing on both is the expected result, not a weakness.
Separately, in a copy with `deny_unknown_fields` stripped from both `hash_cache.rs` structs,
the rewritten cache test fails — which the version it replaces did not.
