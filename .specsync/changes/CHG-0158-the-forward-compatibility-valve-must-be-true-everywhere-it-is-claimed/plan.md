# Plan

1. `src/agents.rs` — remove `deny_unknown_fields` from `AgentArtifactRecord` and
   `AgentArtifactManifest`; record why the manifest is evidence and not a cache.
2. `src/change.rs` — replace the comment paragraph that misclassified `agents.rs`; state the
   canonical-bytes limit on the two baselines.
3. `src/change.rs` — add container-level `#[serde(default)]` to `SddPolicy`.
4. `src/change_tests.rs` — rewrite the cache test against `hashes`, swap the baseline case for
   `FinalizationRecord`, add the baseline-limit test and the `SddPolicy` test.
5. `src/agents.rs` — add `a_manifest_written_by_a_newer_six_is_still_usable` with its control.
6. Amend `REQ-change-079` so its cache criterion says which files it covers; add
   `REQ-change-080` for the policy direction and `REQ-agents-005` for the manifest.
7. Discriminate: build a scratch copy with only the production edits reverted; confirm the two
   new tests fail there and the two controls pass.
8. Full suite, `clippy`, `fmt`, then `change check --commit`, `review`, `ship`, `archive`.
