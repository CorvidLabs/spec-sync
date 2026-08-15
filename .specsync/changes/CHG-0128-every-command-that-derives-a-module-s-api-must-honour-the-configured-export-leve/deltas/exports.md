## ADDED

### REQUIREMENT REQ-exports-008

An entry point that derives exports without stating the surface SHALL warn against new use.

Acceptance Criteria
- The convenience wrappers that hard-code export level and parse mode are documented as unsafe for new callers, naming the defect they caused.
- Production code derives exports from the configured surface instead.
