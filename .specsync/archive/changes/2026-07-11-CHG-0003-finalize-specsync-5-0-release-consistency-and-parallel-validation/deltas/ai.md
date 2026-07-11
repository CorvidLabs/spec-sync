# AI compatibility truth delta

## ADDED

### REQUIREMENT REQ-ai-001

The system SHALL document deprecated AI compatibility paths according to their shipped major-version behavior.

Acceptance Criteria
- The Claude alias is described as deprecated but retained in 5.0.
- The trusted `aiCommand` escape hatch is described as deprecated but retained in 5.0.

## MODIFIED

### SPEC SECTION Purpose

Provides AI-assisted generation through the shared `corvid-ai` HTTP provider layer, with deterministic provider/model resolution and explicitly deprecated compatibility aliases and trusted command escape hatches retained for the 5.0 major release.
