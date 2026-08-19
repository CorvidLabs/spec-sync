---
id: CHG-0158-the-forward-compatibility-valve-must-be-true-everywhere-it-is-claimed
state: archived
type: feature
base_commit: 7bec5b3128ccf0c9265ee3863af63638979b6ace
---

# The forward-compatibility valve must be true everywhere it is claimed

## Intent

the forward-compatibility valve must be true everywhere it is claimed

## Affected Canonical Specs

- `change`
- `agents`

## Acceptance Criteria

- An adversarial pass over CHG-0157 found the valve claimed in three places it does not hold. (1) The code comment states that agents.rs discards and rebuilds an unrecognised manifest; it returns Err, and .specsync/agent-artifacts.json is git-tracked and team-shared, so deny_unknown_fields there is exactly the 6.x lockout CHG-0157 set out to remove. (2) The regenerable-cache test names the map 'entries'; HashCache calls it 'hashes' and does not default it, so the parse failed on the missing field regardless and the test passes with deny_unknown_fields stripped from both hash_cache structs. (3) The test asserts WorkflowV2Baseline parses a forward field, which is true at type level and operationally meaningless: read_workflow_v2_baseline and validate_legacy_archive_baseline_bytes require bytes_match_canonical_json, a byte round-trip gate strictly stronger than the attribute. Separately, SddPolicy has the mirror defect in the other direction: no field is optional on deserialize, so the day 6.x adds a ninth field, every sdd.json written before it becomes unreadable by the binary that added it. Done when: the manifest structs tolerate unknown fields with the three required fields still required; the comment says what the code does; the cache test fails when the attribute is stripped; the baseline limit is pinned by a test rather than contradicted by one; SddPolicy carries a container-level serde default whose values fail closed; every digest is unchanged.

## No-spec Rationale

Not applicable
