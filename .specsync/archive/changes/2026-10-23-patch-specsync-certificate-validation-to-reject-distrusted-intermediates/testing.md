---
change: patch-specsync-certificate-validation-to-reject-distrusted-intermediates
artifact: testing
---

# Testing

Verify parsed Cargo.lock packages are identical except the version/checksum pairs for rustls-webpki and webpki-roots. Run current cargo audit under the unchanged policy, all required native lane steps, strict canonical coverage, scoped checks for both active scopes, native Trust and exact-head hosted checks. Repeat the nine policy-audit tests and actual unchanged Algonest48 gate with the rebuilt candidate. External lock resolution passed; no updated-graph test pass is claimed before implementation.


## Observed preview evidence

The paired external preview passes a fresh security audit under the unchanged existing policy (zero vulnerabilities); the actual branch lockfile reproduces both vulnerabilities against the same database. Parsed package records and all dependency edges are identical except the two specified version/checksum pairs. This is a lock-resolution/audit preview, not updated-graph compilation or product-test evidence. Actual Cargo.lock remains unchanged pending the updated paired-scope approval. The earlier verifier-only question is superseded; its definition and preview remain preserved separately.
