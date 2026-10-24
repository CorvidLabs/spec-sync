---
change: patch-specsync-certificate-validation-to-reject-distrusted-intermediates
artifact: context
---

# Context

SpecSync #821 fixes Algonest #48 policy-audit coverage, but hosted audit rejects existing rustls-webpki 0.103.15 and webpki-roots 1.0.7. RUSTSEC-2026-0382 and RUSTSEC-2026-0381 require 0.103.16 and 1.0.10 respectively. The external preview changes only these two version/checksum pairs; all dependency edges are unchanged. A fresh audit finds zero vulnerabilities in the preview and two in the original graph under the same policy and database. Actual Cargo.lock remains untouched. The original policy-audit approval is separate; the paired dependency scope awaits actual approval.
