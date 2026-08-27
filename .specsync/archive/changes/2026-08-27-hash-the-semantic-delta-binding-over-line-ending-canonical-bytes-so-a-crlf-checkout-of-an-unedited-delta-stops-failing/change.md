---
id: hash-the-semantic-delta-binding-over-line-ending-canonical-bytes-so-a-crlf-checkout-of-an-unedited-delta-stops-failing
state: archived
type: bug_fix
base_commit: d6f266a4fd683246469eb15a8f632061dd5cfbb4
---

# Hash the semantic delta binding over line-ending-canonical bytes so a CRLF checkout of an unedited delta stops failing the approval gate, and fold nothing else

## Intent

Hash the semantic delta binding over line-ending-canonical bytes so a CRLF checkout of an unedited delta stops failing the approval gate, and fold nothing else

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- the approved-delta digest is computed over the delta body with CRLF folded to LF, so a delta a checkout rewrote from LF to CRLF with no other edit still materializes into the canonical spec instead of being refused as changed after approval; nothing beyond line endings is folded, so a real wording change delivered in CRLF is still refused, and a body differing only by a trailing blank line, a leading blank line, trailing spaces, a tab, or a lone carriage return is still refused even though the applier would treat it as equal; and the digest recorded for an LF delta is byte-identical to the digest the raw-bytes binding recorded, so no approval already written since the binding shipped stops verifying

## No-spec Rationale

Not applicable
