---
change: CHG-0059-tolerate-inert-5-0-1-registry-toml-stubs-so-module-resolution-falls-back-to-defa
artifact: requirements
---

# Requirements

### REQ-registry-002

Local registry loading SHALL treat inert 5.0.1-era empty registry stubs as absent while still failing closed on unparsable real registries.

### REQ-change-041

Canonical module path resolution SHALL fall back to default `specs/<module>/<module>.spec.md` paths when the local registry file is missing or an inert stub, without weakening fail-closed behavior for invalid non-inert registries.
