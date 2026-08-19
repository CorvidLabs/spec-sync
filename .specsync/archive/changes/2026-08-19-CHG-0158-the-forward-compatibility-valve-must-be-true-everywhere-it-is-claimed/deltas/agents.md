## ADDED

### REQUIREMENT REQ-agents-005

The agent artifact manifest SHALL be read as committed evidence rather than as a regenerable cache: a manifest carrying a field this SpecSync does not recognise SHALL still be used, and a manifest missing a field this SpecSync requires SHALL still be refused.

Acceptance Criteria
- A manifest written by a newer SpecSync of the same major version does not stop `agents install` or `init`, because the file is committed and shared and one contributor's upgrade must not brick the command for everyone else.
- A manifest record missing a field this SpecSync requires is still refused, so tolerance of unknown fields cannot be mistaken for accepting any shape.
- The manifest is not discarded on a parse failure, because it records the digest of exactly the bytes SpecSync last generated and is the only thing distinguishing an untouched artifact from an edited one.
