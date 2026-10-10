---
change: split-remaining-mcp-server-responsibilities-into-focused-private-modules
artifact: testing
---

# Testing

Reference is the frozen first split: 124 MCP unit tests and 72 integration tests pass. Preserve test function names, assertions and cfg guards even when qualified module paths change. Record inventory before and after. Mechanically compare moved production bodies where possible; separately review thin public facade wiring, imports and minimally widened parent interfaces.

After each coherent extraction run cargo fmt --check, cargo check --locked and relevant cargo test --locked mcp:: coverage. At completion run all MCP unit/integration tests, cargo clippy --locked -- -D warnings, cargo build --release --locked, catalog-lock comparison, strict 100% spec coverage, the mandatory pre-push script and scoped lifecycle verification. Run independent source/security review against the final combined patch.

Run full available validation and distinguish new failures from known baseline failures. Do not claim full trust if fledge/Attest remain unavailable, do not hide Ruby/Psych prerequisites, and do not weaken sandbox/Git detection to obtain green tests. Native non-Linux runtime tests require an appropriate executor and are not implied by a Linux pass.

## Executed Evidence

- All 173 original production function/method bodies preserve their complete token sequences, including literal bytes. Only the public catalog and confinement manifest-preflight forwarding facades were added.
- All 108 relocated test declarations preserve bodies and cfg guards. Shared fixtures, the publication failure enum and quarantine helper remain present. Focused execution passes 124 MCP unit tests and 72 integration tests.
- Debug and release cargo check, cargo fmt --check, cargo clippy --locked -- -D warnings, and cargo build --release --locked passed. The final focused rerun again passed 124 MCP unit tests and 72 integration tests. The mandatory pre-push gate passed.
- Strict source validation passes 62 specs with zero warnings and 122/122 covered source files. MCP quality score is 100/A.
- Full unit suite: 2327 passed and 184 failed, with exactly the same 184 failure names as the initial baseline and first split. Most directly report the sandbox's ancestor Git markers. No detection or security check was changed.
- Independent review checked production preservation, ownership, cfg boundaries, final tests and file mappings. Its Windows-only missing fs import, Unix-only test-module imports and fixture-location documentation findings were fixed. No non-Linux runtime qualification is claimed.
- Missing Ruby/Psych and fledge still block their corresponding full CI/trust checks. Scoped validation and local commits do not replace human implementation review or authorize merge.

## Lifecycle Blocker

The approved continuation is a local implementation checkpoint, not a verified delivery. The real `change check --commit` command refused because its declared first-split dependency is still Verifying and must be Accepted before this change can start. No human review or acceptance was fabricated, and the dependency was not removed. Canonical MCP documentation is synchronized to the approved internal-organization delta manually; no canonical_applied flag or verification ledger is forged. Retry the normal lifecycle once the prerequisite review/finalization is legitimately satisfied.
