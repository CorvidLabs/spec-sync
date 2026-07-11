## ADDED

### REQUIREMENT REQ-cmd-rules-001

The rules command SHALL display effective built-in and declarative validation rules from the loaded configuration without mutating project state.

Acceptance Criteria
- All exported functions perform their documented purpose
- Built-in rules are always listed with their active/off status
- Custom rules display name, type, severity, and filter criteria when defined
- Error conditions produce clear, actionable messages
- Module follows the project's established patterns for config loading and output formatting

## MODIFIED

### SPEC SECTION Purpose

Implements the read-only `specsync rules` command. It lists built-in rules from canonical TOML `[rules]` or legacy JSON `rules`, plus declarative `customRules` when loaded from legacy JSON; TOML migration refuses unsupported custom rules rather than dropping them.

### SPEC SECTION Invariants

1. Built-in rules always display, showing "active" with value when configured or "off" when unset
2. Five built-in rules listed: `max_changelog_entries`, `require_behavioral_examples`, `min_invariants`, `max_spec_size_kb`, `require_depends_on`
3. Declarative custom rules display only when legacy JSON `customRules` were loaded; canonical TOML currently supports built-in `[rules]` and migration refuses unsupported custom rules rather than dropping them
4. Each custom rule displays name, severity (color-coded), type, and optional section/pattern/min_words/applies_to/message fields
5. Severity colors: error → red, warning → yellow, info → blue

### SPEC SECTION Behavioral Examples

**Scenario: No custom rules defined**

- **Given** the effective configuration has no declarative custom rules
- **When** `specsync rules` runs
- **Then** built-in rules are listed, followed by "No custom rules defined." with guidance text

**Scenario: Custom rules with filters**

- **Given** a custom rule with `appliesTo: { status: "stable", module: "^auth" }`
- **When** `specsync rules` runs
- **Then** the rule shows `applies_to: status=stable, module=/^auth/`

### SPEC SECTION Error Cases

| Condition | Behavior |
|-----------|----------|
| No configuration file | Loads defaults and reports all built-in rules as off |
| Legacy JSON has no `customRules` | Reports no custom rules without failing |
