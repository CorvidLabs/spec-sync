---
module: cmd_rules
version: 1
status: stable
files:
  - src/commands/rules.rs
db_tables: []
tracks: []
depends_on:
  - specs/commands/commands.spec.md
  - specs/config/config.spec.md
  - specs/types/types.spec.md
---

# Cmd Rules

## Purpose

Implements the `specsync rules` command. Lists all active validation rules — both built-in (from `specsync.json` `rules` section) and custom declarative rules (from `customRules` array). Shows configuration status, severity, rule type, and filter criteria.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_rules` | `root: &Path` | `()` | Load config and display all built-in and custom validation rules |

### Internal Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `print_builtin` | `name: &str, description: &str, value: Option<String>` | `()` | Print a built-in rule with its active/off status |

## Invariants

1. Built-in rules always display, showing "active" with value when configured or "off" when unset
2. Five built-in rules listed: `max_changelog_entries`, `require_behavioral_examples`, `min_invariants`, `max_spec_size_kb`, `require_depends_on`
3. Custom rules section only displays when `customRules` is non-empty; otherwise shows guidance to add them
4. Each custom rule displays name, severity (color-coded), type, and optional section/pattern/min_words/applies_to/message fields
5. Severity colors: error → red, warning → yellow, info → blue

## Behavioral Examples

### Scenario: No custom rules defined

- **Given** `specsync.json` has no `customRules` array
- **When** `specsync rules` runs
- **Then** built-in rules are listed, followed by "No custom rules defined." with guidance text

### Scenario: Custom rules with filters

- **Given** a custom rule with `appliesTo: { status: "stable", module: "^auth" }`
- **When** `specsync rules` runs
- **Then** the rule shows `applies_to: status=stable, module=/^auth/`

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Missing `specsync.json` | Config loader handles this (not this module's concern) |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| config | `load_config` |
| types | `CustomRuleType`, `RuleSeverity` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync rules` |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-10 | Initial spec |
