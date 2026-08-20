---
id: CHG-0162-a-change-identity-must-be-validated-for-what-it-is-not-for-how-it-starts
state: implementing
type: bug_fix
base_commit: c476cf9792366205db4e3cb7569079bc19d1012b
---

# A change identity must be validated for what it is, not for how it starts

## Intent

a change identity must be validated for what it is, not for how it starts

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- validate_change_id opens with id.starts_with('CHG-'), and it gates find_change_dir and validate_loaded_change, so the whole system is gated on a prefix. That prefix was never evidence of anything: CHG- is four characters anyone can type, so it proves neither that an ID is well-formed nor that SpecSync minted it. Meanwhile the checks that do matter for a string used as a directory component were incomplete: there is no length bound at all, and no reserved-name check, both survivable only because every ID was generated as CHG-NNNN over a capped slug. Done when: an identity carrying no ordinal is accepted; the properties that make a string a safe path component are all enforced, including a length ceiling at the filesystem component limit and the shared reserved-name predicate; and every identity shape this repository has ever minted remains legal.

## No-spec Rationale

Not applicable
