---
change: CHG-0040-support-audited-append-only-correction-of-accepted-interview-metadata-without-re
artifact: research
---

# Research

## Existing behavior

`ChangeRecord.answers` is currently part of the definition digest. Accepted delivery recovery uses
`ReopenRecord` snapshots and `canonical_applied` to preserve evidence and avoid reapplying deltas.
Approving a canonically applied change in `verifying` already keeps it in that state, which provides
the correct gate sequence for a metadata correction.

## Alternatives considered

### Rewrite `state.json` in place

Rejected because it destroys the historically accepted answer, invalidates approval attribution,
and makes tampering indistinguishable from a supported correction.

### Use only a successor change

Rejected because a successor can define future behavior but cannot accurately correct the accepted
change's own historical interview record. Issue #360 specifically requires that audit relationship.

### Broaden `change reopen`

Rejected because reopen has a deliberately narrow delivery-evidence invariant. Mixing definition
mutation into it would obscure user intent and weaken the non-replay guard.

### Allow arbitrary interview keys

Rejected for the first release. Free-form corrections could change scope, semantic intent, or
artifact policy without a stable validation model. A closed typed allowlist is easier to audit and
can be extended through later requirements.

## Compatibility and portability

- Absence of a correction ledger means the original effective definition and existing digest rules.
- Correction metadata digests use framed, domain-separated content and no absolute checkout path.
- Complete definition approval continues using repository-relative artifact paths and Git modes.
- Squash compatibility follows the existing accepted-workspace integration proofs rather than
  weakening history checks.
