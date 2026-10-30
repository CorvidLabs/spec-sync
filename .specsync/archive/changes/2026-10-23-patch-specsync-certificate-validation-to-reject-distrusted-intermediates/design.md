---
change: patch-specsync-certificate-validation-to-reject-distrusted-intermediates
artifact: design
---

# Design

The dependency boundary is two Cargo.lock packages. Keep rustls0.23.46 and all Cargo.toml features unchanged. The new verifier rejects known-distrusted intermediates as required by existing TLS trust expectations. No public command, data schema, module API or declared canonical requirement changes.
