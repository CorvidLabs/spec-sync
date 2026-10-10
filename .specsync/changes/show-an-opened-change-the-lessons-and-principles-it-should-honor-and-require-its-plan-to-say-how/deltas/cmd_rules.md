## ADDED

### REQUIREMENT REQ-cmd-rules-002

The rules command SHALL name the project's principles file beside the spec-shape rules. It SHALL say that a lesson becomes a standing rule only when a person writes it into that file. When no principles file is configured, it SHALL say so. The command SHALL remain read-only.

Acceptance Criteria
- A configured `principles_file` is printed with its path.
- The output says SpecSync does not copy lessons into that file.
- An unset principles file prints that none is configured.
- Built-in rule listing is unchanged.
- The command does not write the principles file or any spec.

## MODIFIED

### SPEC SECTION Invariants

1. Built-in rules always display, showing "active" with value when configured or "off" when unset
2. Five built-in rules listed: `max_changelog_entries`, `require_behavioral_examples`, `min_invariants`, `max_spec_size_kb`, `require_depends_on`
3. Declarative custom rules display only when legacy JSON `customRules` were loaded; canonical TOML currently supports built-in `[rules]` and migration refuses unsupported custom rules rather than dropping them
4. Each custom rule displays name, severity (color-coded), type, and optional section/pattern/min_words/applies_to/message fields
5. Severity colors: error → red, warning → yellow, info → blue

6. When the SDD policy names a principles file, the command also names that file and says a lesson becomes a standing rule only when a person writes it there. When no file is configured, the command says so. The command does not create or rewrite the file.


### SPEC SECTION Behavioral Examples

**Scenario: No custom rules defined**

- **Given** the effective configuration has no declarative custom rules
- **When** `specsync rules` runs
- **Then** built-in rules are listed, followed by "No custom rules defined." with guidance text

**Scenario: Custom rules with filters**

- **Given** a custom rule with `appliesTo: { status: "stable", module: "^auth" }`
- **When** `specsync rules` runs
- **Then** the rule shows `applies_to: status=stable, module=/^auth/`

**Scenario: Principles file is named**

- **Given** `.specsync/sdd.json` sets `principles_file`
- **When** `specsync rules` runs
- **Then** the output names that file and says a lesson becomes a standing rule only when a person writes it there

**Scenario: No principles file**

- **Given** the SDD policy does not set `principles_file`
- **When** `specsync rules` runs
- **Then** the output says no standing principles file is configured and still lists the built-in rules

