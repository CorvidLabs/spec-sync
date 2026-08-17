---
id: CHG-0137-coverage-must-not-invent-a-module-over-files-that-are-all-mapped
state: implementing
type: bug_fix
base_commit: 7309c17f774467183f75d4063311d6ceb8b62bf0
---

# Coverage must not invent a module over files that are all mapped

## Intent

coverage must not invent a module over files that are all mapped

## Affected Canonical Specs

- `validator`
- `manifest`

## Acceptance Criteria

- coverage reports no module for a directory or stem whose every discovered file is already mapped by a spec, in text and in JSON; a module owning at least one unmapped file is still reported; a module owning no discovered file at all is still reported, because owning nothing measurable is absence of input rather than evidence of coverage; the file and LOC coverage percentages are unchanged by the fix, so the phantom disappears without moving the measurement; running coverage on this repository goes from modules ['specsync','change_tests'] to [] while files_covered stays 106/106; all four derivation sites are fixed, not only the flat-file-stem site named in the report; sandbox gate 061 goes rc=1 to rc=0 with its not-a-mute control still green.

## No-spec Rationale

Not applicable
