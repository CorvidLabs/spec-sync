---
change: supersede-reads-the-predecessor-digest-says-when-the-predecessor-is-on-another-branch-and-checks-the-base-entry-when
artifact: research
---

# Research

- Coverage time already names each rejected successor and why (`stale_input_remediation_reason`).
- The reconcile-history fixture declares supersede over a base whose `src/auth.rs` changed after
  the old acceptance, so a supersede-time refusal would break it.
- `payload_digest` is the SHA-256 of the file bytes, so the change that signed the content at
  the base is found by matching it in accepted and archived manifests.
