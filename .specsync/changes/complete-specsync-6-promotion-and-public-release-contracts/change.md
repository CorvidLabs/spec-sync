---
id: complete-specsync-6-promotion-and-public-release-contracts
state: implementing
type: bug_fix
base_commit: ffba9a32b664a7b3308350c162f0ac808f24efe3
---

# Complete SpecSync 6 promotion and public release contracts

## Intent

Complete SpecSync 6 promotion and public release contracts

## Affected Canonical Specs

- `cli_args`
- `change`

## Acceptance Criteria

- Promotion and final publication accept exactly the current required Ubuntu and macOS evidence set, and reject missing, duplicate, extra, failed, or mixed-identity evidence without bypassing release protections.
- Executable regression controls exercise both workflow evidence-count guards and fail when the obsolete three-record expectation is restored.
- CLI help and canonical review contracts distinguish a digest-bound reviewer claim from separately configured authenticated provenance, with no runtime identity or release gate weakened.
- README and CLI examples use supported slug workflows and valid supersede flags; security support and release notes describe the 6.0 rollout without claiming an unpublished release already shipped.
- The exact final tree passes required local and hosted verification, Trust, provenance, independent review, and a fresh RC qualification before stable publication.

## No-spec Rationale

Not applicable
