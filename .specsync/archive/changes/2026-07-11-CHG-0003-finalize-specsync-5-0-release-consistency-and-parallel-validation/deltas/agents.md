# Agent integration truth delta

## ADDED

### REQUIREMENT REQ-agents-001

The system SHALL keep installed native agent artifacts and their canonical documentation consistent.

Acceptance Criteria
- Claude, Cursor, and Gemini receive create-spec and create-change commands.
- Codex receives the project-scoped lifecycle skill without a deprecated command file.

## MODIFIED

### SPEC SECTION Purpose

Installs native, tool-owned verified-SDD skills for Claude Code, Cursor, Codex, and Gemini CLI. Where the tool supports project commands, SpecSync installs both create-spec and create-change commands; Codex receives the project skill only because its command mechanism is deprecated/global.
