## ADDED

### REQUIREMENT REQ-change-084

A change identity SHALL be accepted or refused on the properties that make a string a safe path component, and SHALL NOT be required to begin with any particular prefix.

Acceptance Criteria
- An identity carrying no ordinal is accepted, because a prefix is text any caller can type and is therefore evidence neither that an identity is well-formed nor that SpecSync minted it.
- An identity is refused when it is empty, is not a single path component, contains a path separator or a control character, exceeds the longest name a path component may hold, or is a name a supported platform reserves.
- Every identity shape SpecSync has previously minted remains acceptable, so relaxing what is required does not orphan history.
