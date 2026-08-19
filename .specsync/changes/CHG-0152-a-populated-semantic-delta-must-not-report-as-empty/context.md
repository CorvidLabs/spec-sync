---
change: CHG-0152-a-populated-semantic-delta-must-not-report-as-empty
artifact: context
---

# Context

GitHub #537. `change approve` is a hard gate, and a populated delta whose headings the parser
does not recognize is reported as empty. The author then looks for a missing file instead of
a malformed one.

Measured on current `main` (`cf38520e`) and independently reviewed by Claude (session
`2a308b14`) and Kimi K3 (`kimi-537`). Five delta variants; exactly one lie: prose with no
`## Added|Modified|Removed` heading reports `semantic delta for \`{module}\` is empty`. The
same wording exists on the historical path (`historical semantic delta is empty`). Other
failure paths already name the grammar. A valid uppercase delta still approves.

## Scope kept narrow

Parser-only. Two call sites that emit the lie (`src/change.rs` around the live
`validate_delta_files` empty check and the historical tombstone walk). Item headings become
case-insensitive to match operation headings. Invalid headings name the allowed values.

Dropped on purpose:

- Scaffolding a commented `deltas/<module>.md` at `change new` — any file in `deltas/`
  defeats `is_untrackable_husk` (#536, landed hours ago).
- `change approve --help` and generated `SKILL.md` — an error that names the grammar at the
  moment of failure is the discoverability surface this issue needs.
- `agents` / `cli_args` specs — unused modules would require empty deltas and fail approve
  with the bug being fixed.

## Constraints

- `#564` / REQ-change-065: a `###` that is not an item keyword, met while an item is open,
  stays content. Case-insensitive item keywords must not turn body lines into items unless
  they actually are `requirement` / `spec section`.
- Archived deltas were approved under the stricter grammar. Widening acceptance cannot change
  their parse. Kimi grepped 409 archived delta files; none use case-variant item headings.
- Digests hash delta file bytes, not parse output.
- Do not edit `site/src/content/docs/deltas.md`. Claude's "sibling site" is the historical
  code path, not the docs site. Leave that file out of `affected_paths`.
