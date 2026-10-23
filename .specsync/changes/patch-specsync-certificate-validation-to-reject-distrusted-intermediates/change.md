---
id: patch-specsync-certificate-validation-to-reject-distrusted-intermediates
state: implementing
type: operations
base_commit: 9366edcff596e4607b3ebb26ae5e59d4ebbbfd25
---

# Patch SpecSync certificate validation to reject distrusted intermediates

## Intent

Patch SpecSync certificate validation to reject distrusted intermediates

## Affected Canonical Specs

- None

## Acceptance Criteria

- Update only locked rustls-webpki 0.103.15 to 0.103.16 and webpki-roots 1.0.7 to 1.0.10, with their registry checksums, to fix RUSTSEC-2026-0382 and RUSTSEC-2026-0381. Keep all other packages, dependency edges, enabled features, compiler/version pins, existing audit policy and the approved policy-audit implementation unchanged. Re-run required product/Trust/scoped/hosted checks, including the nine audit regressions and unchanged Algonest #48 candidate guard. No new ignores, fabricated approval, release, merge, deployment or consumer CI change.

## No-spec Rationale

Patch-level certificate-verifier dependency repair; existing TLS security expectations and CLI/API contracts are preserved. No application source, algorithms, features or canonical contract changes.
