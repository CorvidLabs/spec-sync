---
change: CHG-0009-make-accepted-evidence-squash-safe-and-harden-the-5-0-release-path
artifact: research
---

# Research

Git commit ancestry cannot survive a squash by definition. Scoped content equality alone is too permissive because
the portable approval ledger is not a cryptographic signature. Requiring the exact accepted workspace to already
exist on the integrated remote default ref proves that the reviewed evidence crossed the repository's merge boundary.

The release workflow currently matches `v*`, which also matches the floating `v5` Action alias. The Action also
defaults its downloaded binary to `latest`, so pinning only the Action ref does not pin the executable major.
