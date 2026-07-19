---
id: CHG-0059-tolerate-inert-5-0-1-registry-toml-stubs-so-module-resolution-falls-back-to-defa
state: accepted
type: bug_fix
base_commit: 418235bcf78087c923db45f6a6a5e13f90b451b8
---

# Tolerate inert 5.0.1 registry.toml stubs so module resolution falls back to default specs layout without failing closed on empty legacy stubs

## Intent

Tolerate inert 5.0.1 registry.toml stubs so module resolution falls back to default specs layout without failing closed on empty legacy stubs

## Affected Canonical Specs

- `registry`
- `change`

## Acceptance Criteria

- An inert 5.0.1-era .specsync/registry.toml stub (version=1 plus empty [modules], or any file with no registry name and no [specs] mappings) is treated as absent so canonical_module_paths falls back to specs/<module>/<module>.spec.md; a non-inert unparsable registry still fails closed with the exact diagnostic failed to parse local registry {path} while resolving `{module}`; named registries continue to win over the conventional fallback; unit coverage proves inert tolerate, non-inert fail-closed, and load_local_registry discrimination.

## No-spec Rationale

Not applicable
