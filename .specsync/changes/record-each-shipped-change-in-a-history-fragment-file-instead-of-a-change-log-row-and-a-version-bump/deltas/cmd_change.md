## ADDED

### REQUIREMENT REQ-cmd-change-037

`specsync history <module>` SHALL print the module's history (REQ-change-160) as one Change Log table: the spec's existing table, header and rows unchanged, followed by one row per fragment written in that table's own columns, as `change check` used to append rows, with the description `<change-id>: <summary>` and a Version column numbered on from the spec's frozen `version:` (the frozen value plus the fragment's position). A spec with no Change Log table gets a `| Date | Change |` table. With `--format json` it SHALL print the module, the frozen `version:`, the table rows as written and the fragments. An unknown module, or a fragment that fails REQ-validator-048, SHALL make it exit 1 naming the module or the file.

Acceptance Criteria
- For a spec at `version: 4` whose table is `| Version | Date | Change |`, with two fragments, `specsync history <module>` prints the existing rows unchanged, then rows numbered 5 and 6 in date order.
- `--format json` lists the table rows and the fragments separately.
- An unknown module and a malformed fragment each exit 1 and name the module or the file.
