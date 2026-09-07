# Lesson bundle — pin-the-trust-gate-to-v1-2-0-rc-4

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Pin the Trust gate to v1.2.0-rc.4
- **Kind**: BugFix
- **Specs**: github
- **Paths**: .github/workflows/trust.yml
- **Acceptance**: The Trust workflow pins CorvidLabs/trust to e0272543 (v1.2.0-rc.4) while specsync-version stays 6.0.0 against the runner-local file:// mirror.

## Evidence

- Verification commit: `08b8e7d3035e6ba0a1562dac406e246708e00bbd`
- Base commit: `ac796b8eadd3092283093bbea331ec2d3494b527`
- Verified by: `specsync check --spec github`

## From the change's context.md

# Context

PR 750 pinned the hosted Trust action from CorvidLabs/trust@a239f786 (v1.1.1) to CorvidLabs/trust@e0272543 (v1.2.0-rc.4) in .github/workflows/trust.yml. The dogfood SpecSync 6.0.0 runner-local file:// mirror, specsync-version, and workflow shape are unchanged.

Lifecycle gate and Trust both require an active change covering that meaningful path. This package records the pin only; canonical spec text is not changing.

## From the change's testing.md

# Testing

- specsync change audit --strict covers .github/workflows/trust.yml with this active change.
- Hosted trust job runs the same audit and the pinned Trust action against the existing 6.0.0 file:// mirror.
- No product spec or binary contract change to re-prove beyond the pin itself.

## Where these lessons go

- `specs/github/context.md`
