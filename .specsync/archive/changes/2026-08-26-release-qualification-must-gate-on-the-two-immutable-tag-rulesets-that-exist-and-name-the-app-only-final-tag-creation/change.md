---
id: release-qualification-must-gate-on-the-two-immutable-tag-rulesets-that-exist-and-name-the-app-only-final-tag-creation
state: archived
type: bug_fix
base_commit: e82542d19ce8d79926b144a0e38d4d620b120715
---

# Release qualification must gate on the two immutable tag rulesets that exist and name the App-only final-tag creation policy it no longer enforces

## Intent

Release qualification must gate on the two immutable tag rulesets that exist and name the App-only final-tag creation policy it no longer enforces

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- The resolve job of .github/workflows/release.yml resolves and validates exactly two repository tag rulesets — 'SpecSync immutable RC tags' and 'SpecSync immutable final tags' — and no longer resolves 'SpecSync final tag creation', queries the release deployment environment, or reads vars.SPECSYNC_RELEASE_APP_ID. The validator's 'rulesets' command accepts only --final-immutability-ruleset-json and --rc-ruleset-json, rejects --release-app-id and --final-creation-ruleset-json, and emits a non-empty 'unenforced' array naming the App-only final-tag creation policy and the unverified protected release environment. Every release run prints those unenforced items as GitHub warning annotations so a green run cannot be read as proof that App-only final-tag creation is enforced. Both immutability rulesets stay strict: any bypass actor, broadened include/exclude pattern, extra or missing rule type, inactive enforcement, or non-Repository source is still rejected. The 'environment' subcommand is removed with its tests. docs/ci-confidence.md, specs/github/requirements.md, specs/github/github.spec.md and specs/github/tasks.md describe two enforced rulesets and explicitly record the dropped App-only creation policy and the unverified release environment. python3 .github/scripts/test-validate-release-candidate.py passes, and running the 'rulesets' command against the live payloads of rulesets 21432132 and 21432148 exits 0.

## No-spec Rationale

Not applicable
