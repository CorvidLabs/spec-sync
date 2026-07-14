## ADDED

### REQUIREMENT REQ-validator-005

Configuration-driven source discovery SHALL include paths without a filename extension when `include_extensionless` is true and SHALL apply that rule consistently across validation and generation commands.

Acceptance Criteria

- Extensionless-only strict coverage measures one mapped file and non-zero LOC.
- Mixed strict coverage measures both extensionless and explicitly configured suffixed files with non-zero LOC.
- Coverage, generation, scaffold, new-spec, wizard, diff, and output scans share the extensionless rule.
- Omitted or false configuration preserves existing source selection.
