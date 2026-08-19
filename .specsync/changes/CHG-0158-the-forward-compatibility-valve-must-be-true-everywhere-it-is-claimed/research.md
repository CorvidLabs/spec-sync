# Research

Every claim below was measured against the tree at `7bec5b31`, not inferred.

## The three overclaims in CHG-0157

**`.specsync/agent-artifacts.json` is committed, and reading it hard-fails.**

```
$ git ls-files .specsync/ | grep agent-artifacts
.specsync/agent-artifacts.json
$ git check-ignore -v .specsync/hashes.json
.specsync/.gitignore:7:hashes.json	.specsync/hashes.json
```

`load_agent_artifact_manifest` (`src/agents.rs:423`) returns `Err` on a parse failure.
`HashCache::load` (`src/hash_cache.rs:88`) returns `Self::default()`. Only the second is a
cache. The comment in `src/change.rs` grouped them, which is how both got the same attribute.

Planting one unknown field and running `specsync agents install --claude` fails identically on
the CHG-0157 binary and the one before it:

```
✗ Claude Code SDD skill + specsync commands: Failed to parse .specsync/agent-artifacts.json:
  unknown field `source_template`, expected one of `tool`, `template_version`, `digest`
```

`specsync init` routes through the same `cmd_install`. So a teammate on a later 6.x adding a
field stops `init` and `agents install` for every teammate still on the older binary, in a file
they all share — precisely the lockout CHG-0157 existed to remove.

The manifest is not recomputable: it records the digest of exactly the bytes SpecSync last
generated, which is the only thing distinguishing "unchanged since we wrote it" from "the user
edited it". Discarding it turns every managed artifact into an unmanaged one.

**The cache test guarded nothing.** It fed `{"format_version":1,"entries":{},…}`. The field is
`hashes`, and it has no `#[serde(default)]`, so the parse failed with ``missing field `hashes` ``
whether or not `deny_unknown_fields` was present. Confirmed by copying the tree, stripping the
attribute from both `hash_cache.rs` structs, and re-running: the test still passed.

**The two baselines gain nothing.** `read_workflow_v2_baseline` (`src/change.rs:16795`) and
`validate_legacy_archive_baseline_bytes` (`:14340`) both re-serialize what they parsed and
require `bytes_match_canonical_json` against the bytes on disk. An added field survives
`from_slice` and is then dropped by the re-serialization, so the comparison fails. The gate is
deliberate — these files anchor history — but it makes the CHG-0157 test's `WorkflowV2Baseline`
case true at type level and false in operation, and liable to convince a future maintainer that
those files are extensible.

## The mirror defect

`deny_unknown_fields` is the old-reads-new door. `SddPolicy` (`src/change.rs:619`) has the
new-reads-old one: none of its eight fields is optional on deserialize. That works today only
because SpecSync writes all of them, so the day 6.x adds a ninth, every `sdd.json` written
before it becomes unreadable by the binary that added it.

`impl Default for SddPolicy` (`:645`) already exists and is the safe policy — `enabled: true`,
`require_change_for_meaningful_files: true` — so a container-level default fails closed: a
policy that loses a field enforces more, not less.

## Digest impact

None. `#[serde(default)]` and `deny_unknown_fields` are both deserialize-only; neither appears
in any `Serialize` impl. The CHG-0157 pass proved this at codegen level by diffing
`-Zunpretty=expanded` across the two revisions: 411 changed lines, zero matching
`Serialize`/`serialize`. The same argument covers this change, and the CHG-0068 golden-vector
constants are unchanged.

## What is deliberately NOT fixed here

`record_scoped_review` reads `review-attempts.json`, appends, and rewrites. Once a field is
tolerated rather than preserved, that read-modify-write silently strips a field a newer 6.x
wrote. Preserving unknowns needs `#[serde(flatten)] BTreeMap<String, Value>`, which changes the
`Deserialize` path for the whole struct — including number handling on digest preimages — so it
is not a change to make beside a comment fix. The identical hazard already exists on `main` for
`verification-attempts.json`, which never carried the attribute. Recorded as an issue, not
smuggled in here.
