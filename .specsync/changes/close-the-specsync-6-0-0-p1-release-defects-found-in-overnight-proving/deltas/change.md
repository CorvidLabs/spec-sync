## ADDED

### REQUIREMENT REQ-change-097

Accepting or finalizing a change SHALL refuse to adopt the implementation commit when verification-attempt history is missing or empty. A Verifying change SHALL refuse to recreate a missing attempts ledger.

Acceptance Criteria
- Deleting `verification-attempts.json` after a successful `change check` is the same evidence as emptying it: adopted finalization fails with `verification attempt history cannot adopt the implementation commit`.
- `record_verification_attempt` on a Verifying change whose ledger file is gone fails with `verification attempt history is missing` instead of writing a one-entry ledger.
- A present non-empty ledger is unchanged: the current verification is appended as before.

### REQUIREMENT REQ-change-098

Canonical definition-artifact payloads SHALL fold CRLF to LF before any other rewrite, so a `core.autocrlf=true` checkout hashes the same as the LF blob Git stored.

Acceptance Criteria
- `canonical_definition_artifact_payload` for a non-tasks file with `\r\n` equals the LF form of the same bytes.
- A lone `\r` is content and is preserved.
- For `tasks.md`, the CRLF fold runs before checkbox-space rewriting, so post-move archive hashing cannot disagree with the pre-move digest on line endings alone.

### REQUIREMENT REQ-change-099

Lifecycle validation limits SHALL be bundled under `src/` so the crates.io include set can compile the binary.

Acceptance Criteria
- `include_str!("lifecycle-validation-limits.json")` is the source of `lifecycle_validation_limits()`.
- `src/lifecycle-validation-limits.json` is byte-identical to `.github/scripts/lifecycle-validation-limits.json`.
- `cargo package --list` includes the bundled JSON.
