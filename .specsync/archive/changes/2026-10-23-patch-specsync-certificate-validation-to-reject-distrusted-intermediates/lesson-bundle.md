# Lesson bundle — patch-specsync-certificate-validation-to-reject-distrusted-intermediates

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Patch SpecSync certificate validation to reject distrusted intermediates
- **Kind**: Operations
- **Paths**: Cargo.lock
- **Acceptance**: Update only locked rustls-webpki 0.103.15 to 0.103.16 and webpki-roots 1.0.7 to 1.0.10, with their registry checksums, to fix RUSTSEC-2026-0382 and RUSTSEC-2026-0381. Keep all other packages, dependency edges, enabled features, compiler/version pins, existing audit policy and the approved policy-audit implementation unchanged. Re-run required product/Trust/scoped/hosted checks, including the nine audit regressions and unchanged Algonest #48 candidate guard. No new ignores, fabricated approval, release, merge, deployment or consumer CI change.

## Evidence

- Verification commit: `8176e3c51e4bca5bbe6d03402bf53b949c36e51a`
- Base commit: `9366edcff596e4607b3ebb26ae5e59d4ebbbfd25`
- Verified by: `specsync check (no spec in scope)`

## From the change's context.md

# Context

SpecSync #821 fixes Algonest #48 policy-audit coverage, but hosted audit rejects existing rustls-webpki 0.103.15 and webpki-roots 1.0.7. RUSTSEC-2026-0382 and RUSTSEC-2026-0381 require 0.103.16 and 1.0.10 respectively. The external preview changes only these two version/checksum pairs; all dependency edges are unchanged. A fresh audit finds zero vulnerabilities in the preview and two in the original graph under the same policy and database. Actual Cargo.lock remains untouched. The original policy-audit approval is separate; the paired dependency scope awaits actual approval.

## From the change's design.md

# Design

The dependency boundary is two Cargo.lock packages. Keep rustls0.23.46 and all Cargo.toml features unchanged. The new verifier rejects known-distrusted intermediates as required by existing TLS trust expectations. No public command, data schema, module API or declared canonical requirement changes.

## From the change's testing.md

# Testing

Verify parsed Cargo.lock packages are identical except the version/checksum pairs for rustls-webpki and webpki-roots. Run current cargo audit under the unchanged policy, all required native lane steps, strict canonical coverage, scoped checks for both active scopes, native Trust and exact-head hosted checks. Repeat the nine policy-audit tests and actual unchanged Algonest48 gate with the rebuilt candidate. External lock resolution passed; no updated-graph test pass is claimed before implementation.


## Observed preview evidence

The paired external preview passes a fresh security audit under the unchanged existing policy (zero vulnerabilities); the actual branch lockfile reproduces both vulnerabilities against the same database. Parsed package records and all dependency edges are identical except the two specified version/checksum pairs. This is a lock-resolution/audit preview, not updated-graph compilation or product-test evidence. Actual Cargo.lock remains unchanged pending the updated paired-scope approval. The earlier verifier-only question is superseded; its definition and preview remain preserved separately.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
