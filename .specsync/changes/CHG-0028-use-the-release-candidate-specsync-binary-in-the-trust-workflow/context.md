---
change: CHG-0028-use-the-release-candidate-specsync-binary-in-the-trust-workflow
artifact: context
---

# Context

The hosted Trust workflow installs the latest released SpecSync binary. During 5.0.2 validation that is still 5.0.1, so the Trust contract gate cannot evaluate lifecycle behavior implemented by the release candidate even when the repository's ordinary `spec-check` job passes.

Trust v1.0.1 supports a fail-closed exact SpecSync version and an authority-free `file://` mirror confined beneath `runner.temp`. The workflow can package the pull request's release binary with its SHA-256 checksum and pass that runner-local mirror to the immutable Trust action. Lifecycle, contract, Augur risk, and Attest provenance gates remain enabled.
