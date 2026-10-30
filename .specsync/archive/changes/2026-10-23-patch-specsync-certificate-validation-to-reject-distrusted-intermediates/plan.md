---
change: patch-specsync-certificate-validation-to-reject-distrusted-intermediates
artifact: plan
---

# Plan

## Approach

Apply the exact external lockfile-only preview for rustls-webpki0.103.16 and webpki-roots1.0.10. Leave manifests/features, all other packages and policy-audit code unchanged.

## Out of scope

No new advisory ignores, dependency-wide refresh, version release, merge, deployment, consumer pin or prior lifecycle-history edits.

## Steps

Obtain actual definition approval; apply only the four lockfile fields; verify package graph equality otherwise; run required full product/Trust and scoped/hosted gates; qualify the unchanged Algonest48 gate with the rebuilt candidate.

## Risks

The verifier runs on certificate paths, so existing tests for the old dependency do not qualify the replacement. Test the actual rebuilt candidate and current audit.

## Constraints consulted

The existing change-module lessons distinguish authentic immutable evidence from unavailable Git objects and require selective archive authentication. This refresh changes only a TLS dependency and does not alter those rules.
