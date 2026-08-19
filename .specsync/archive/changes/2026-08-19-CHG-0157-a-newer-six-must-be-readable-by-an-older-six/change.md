---
id: CHG-0157-a-newer-six-must-be-readable-by-an-older-six
state: archived
type: feature
base_commit: ad65908f66fe42cb201e4d736431e94f534c70b0
---

# A newer six must be readable by an older six

## Intent

a newer six must be readable by an older six

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- Evidence written by a later six that carries a field this six does not know is still read rather than rejected, so a shape can be extended during six's lifetime without breaking installations already deployed; a change recording a workflow version this six does not support is reported as written by a newer spec-sync that the reader should upgrade to read, rather than as an invalid change state indistinguishable from corruption; regenerable caches keep rejecting unknown fields, because discarding and rebuilding them is correct and costs nothing; and every existing evidence file still loads and every digest is unchanged, since rejecting unknown input on read was never part of what any digest covers.

## No-spec Rationale

Not applicable
