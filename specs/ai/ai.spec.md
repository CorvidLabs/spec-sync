---
module: ai
version: 4
status: archived
files:
  - specs/ai/retired.md
db_tables: []
tracks: []
depends_on: []
---

# Embedded AI (Retired)

## Purpose

Archived historical tombstone for SpecSync's removed embedded inference subsystem. SpecSync 5.0 is a deterministic, agent-native SDD core: it does not select providers or models, store inference credentials, transmit source to a model, or execute an AI shell command.

## Public API

No runtime API. The listed file is a historical retirement marker; the former `src/ai.rs` module and `corvid-ai` dependency were removed in 5.0.

## Invariants

1. SpecSync generation is deterministic and local.
2. Provider, model, credential, endpoint, timeout, and AI-command settings are not part of the runtime contract.
3. Legacy AI configuration key names may produce migration guidance, but their values are never retained, printed, transmitted, or executed.
4. Coding-agent enrichment remains available through native skills and MCP under the invoking agent's trust boundary.

## Behavioral Examples

### Scenario: Enrich a deterministic scaffold

- **Given** a developer runs `specsync generate`
- **When** the local template is created
- **Then** a configured coding agent may refine the markdown and `specsync check` validates the result deterministically

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Legacy provider/model CLI flag | Clap rejects the unknown flag and migration docs point to agent integrations |
| Legacy AI MCP argument | Tool request fails explicitly without echoing its value |
| Legacy AI config key | Key name is ignored with migration guidance; value is never interpreted |

## Dependencies

None. This module is retained only as an auditable historical contract tombstone.

## Change Log

| Date | Change |
|------|--------|
| 2026-07-11 | CHG-0007-harden-specsync-5-0-as-an-agent-native-secret-free-sdd-core-and-close-release-r: Harden SpecSync 5.0 as an agent-native, secret-free SDD core and close release regressions |
