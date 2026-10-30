---
change: patch-specsync-certificate-validation-to-reject-distrusted-intermediates
artifact: research
---

# Research

Primary advisories: https://rustsec.org/advisories/RUSTSEC-2026-0382 (rustls-webpki patched at 0.103.16) and https://rustsec.org/advisories/RUSTSEC-2026-0381 (webpki-roots patched at 1.0.10). External preview: ../specsync-cert-roots-preview.patch changes only both versions and registry checksums. Cargo resolution and fresh audit pass without other package or dependency-edge changes. Product results on the original graph are distinct from updated-graph verification, which has not run. The earlier verifier-only question is superseded by the paired scope.
