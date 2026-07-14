---
change: CHG-0027-support-extensionless-source-discovery-through-an-explicit-include-extensionless
artifact: requirements
---

# Requirements

## REQ-CHG-0027-001 — Explicit extensionless discovery

SpecSync SHALL accept `include_extensionless = true` in canonical TOML and `includeExtensionless: true` in legacy JSON, serialize the canonical option when enabled, and include files with no filename extension in source discovery.

Acceptance criteria:

- The default is `false`.
- Omitting the setting preserves existing discovery.
- `include_extensionless = false` preserves existing discovery.
- Enabling the setting works with an omitted, empty, or non-empty `source_extensions` list.
- Canonical TOML migration round-trips the enabled setting without changing other fields.

## REQ-CHG-0027-002 — Non-vacuous coverage

Strict coverage SHALL count extensionless files when the option is enabled and SHALL calculate file and LOC coverage from their real contents.

Acceptance criteria:

- An extensionless-only project with one mapped source reports one covered file out of one and non-zero covered and total LOC.
- A mixed project with one extensionless file and one configured suffixed file reports two covered files out of two and non-zero covered and total LOC.
- Both projects pass `--strict --require-coverage 100` only when every discovered file is mapped.

## REQ-CHG-0027-003 — Consistent source selection

Coverage, generation, scaffold, new-spec, wizard, diff, and output source scans SHALL use the same extensionless-selection rule so a file cannot be measurable in one command and invisible in another.
