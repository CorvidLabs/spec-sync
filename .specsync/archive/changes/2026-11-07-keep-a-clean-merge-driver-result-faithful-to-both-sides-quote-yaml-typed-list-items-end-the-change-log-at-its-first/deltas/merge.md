## ADDED

### REQUIREMENT REQ-merge-006

The git merge driver SHALL report a merge clean only when the result keeps everything both sides wrote and means what they meant; when it cannot, it SHALL leave a conflict. A union of `files`, `db_tables` or `depends_on` SHALL write an item YAML would read as a number, boolean or null in double quotes, so it stays a string. The first blank line after the change-log rows SHALL end the table, and what follows it SHALL be merged as text, never as change-log rows. When both sides bump a numeric `version:`, the text after the number SHALL merge three ways, with the same result in either merge order. Renumbering a new change-log `Version` cell SHALL leave the table a conflict when no higher `u64` exists.

#### Acceptance Criteria

- `files: ["123"]`, `db_tables: ["true"]` and `depends_on: ["null"]` stay quoted strings after a union, in either merge order, while a plain path stays plain (#872).
- Prose or a second table after a blank line that follows the change log comes out byte for byte after a row union, in either order (#875).
- A version comment added on one side survives; the same comment on both sides is kept; two different comments are a conflict; in either order (#876).
- An ancestor change-log row at `u64::MAX` plus a new row on each side is a conflict, with no panic and no `0` row (#870).
