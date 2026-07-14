---
change: CHG-0024-stabilize-specsync-5-lifecycle-integrity-and-strict-validation-for-5-0-2
artifact: requirements
---

# Requirements

### REQ-change-022

The lifecycle SHALL prevent parallel branches from silently merging duplicate numeric change sequences while preserving exact historical collision evidence.

#### Acceptance Criteria

- Active and archived records are scanned together by numeric `CHG-NNNN` sequence.
- Unacknowledged duplicate sequences fail with every conflicting full ID and path.
- Repository-backed sequence claims make independent next-ID claims conflict during Git integration.
- The existing `CHG-0016` collision group is baselined without rewriting accepted state or evidence.

### REQ-change-023

Verification SHALL reject recursive lifecycle checks and preserve retryable attempt history without weakening unrelated gates.

#### Acceptance Criteria

- Direct and indirect re-entry fails once with the change or command context and leaves no repeated child process.
- Native-only verification executes once.
- Failed attempts remain inspectable and a corrected retry can replace the latest projection with passed evidence.
- Other failed or stale changes continue failing closed.

### REQ-change-024

Strict lifecycle checking SHALL permit an exact current canonical successor to replace a stale accepted predecessor without hiding unrelated stale evidence.

#### Acceptance Criteria

- A current approved implementing successor governing every affected module and path can reach verification.
- A verifying successor suppresses a predecessor only with current passed evidence.
- Draft no-spec partial failed abandoned or stale-definition successors do not suppress predecessor errors.
- After successor acceptance the project is clean and prior evidence remains inspectable.

### REQ-change-025

Semantic-delta preparation and application SHALL resolve canonical spec and companion paths through the committed registry.

#### Acceptance Criteria

- A module whose registry directory differs from its module name updates the registered spec and adjacent requirements file.
- Conventional paths remain the fallback when no registry mapping exists.
- Absolute traversing malformed and symlink-escaping registry paths fail closed.

### REQ-validator-002

Coverage SHALL measure configured static content without presenting a vacuous successful percentage.

#### Acceptance Criteria

- Mapped HTML reports one covered file out of one.
- Unmapped HTML reports zero covered files out of one and fails a 100 percent gate.
- Excluded assets remain excluded and static files require no exported symbols.
- A zero-file project is reported distinctly from measured 100 percent coverage.

### REQ-validator-003

Strict validation SHALL reject known unfilled companion scaffold markers with artifact-specific line diagnostics.

#### Acceptance Criteria

- Generated context requirements testing tasks and design markers are recognized deterministically.
- Replacing a marker with concrete content passes.
- Similar prose and markers inside fenced examples are ignored.
- Diagnostics identify the companion path line and required correction.
