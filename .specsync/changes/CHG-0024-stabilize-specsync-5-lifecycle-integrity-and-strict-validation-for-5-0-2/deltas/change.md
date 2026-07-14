## ADDED

### REQUIREMENT REQ-change-022

The lifecycle SHALL prevent parallel branches from silently merging duplicate numeric change sequences while preserving exact historical collision evidence.

Acceptance Criteria

- Active and archived records are scanned together by numeric `CHG-NNNN` sequence.
- Unacknowledged duplicate sequences fail with every conflicting full ID and path.
- Repository-backed sequence claims make independent next-ID claims conflict during Git integration.
- Existing accepted collisions can be baselined exactly without rewriting accepted state or evidence.

### REQUIREMENT REQ-change-023

Verification SHALL reject recursive lifecycle checks and preserve retryable attempt history without weakening unrelated gates.

Acceptance Criteria

- Direct and indirect re-entry fails once before repeated child execution.
- Native-only verification executes once.
- Failed attempts remain inspectable and a corrected retry can record passed latest evidence.
- Other failed or stale changes continue failing closed.

### REQUIREMENT REQ-change-024

Strict lifecycle checking SHALL permit an exact current canonical successor to replace a stale accepted predecessor without hiding unrelated stale evidence.

Acceptance Criteria

- An implementing successor with current approved definition evidence can govern every affected module and path while reaching verification.
- A verifying successor requires current passed evidence.
- Draft, no-spec, partial, failed, abandoned, and stale-definition successors never suppress predecessor errors.
- Accepted successors leave strict validation clean while preserving predecessor history.

### REQUIREMENT REQ-change-025

Semantic-delta preparation and application SHALL resolve canonical spec and companion paths through the committed registry.

Acceptance Criteria

- Registry-backed non-conventional module paths receive semantic spec and requirements updates.
- Conventional paths remain the fallback when no mapping exists.
- Unsafe registry paths fail closed.
