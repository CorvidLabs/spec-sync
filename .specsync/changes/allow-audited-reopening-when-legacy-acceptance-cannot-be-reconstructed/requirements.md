---
change: allow-audited-reopening-when-legacy-acceptance-cannot-be-reconstructed
artifact: requirements
---

# Requirements

### REQ-change-094

The lifecycle SHALL allow audited reopening of manifest-less legacy accepted evidence when historical acceptance reconstruction fails, even if current delivery inputs match and the verification commit is anchored.

Acceptance Criteria
- Record an explicit legacy reconstruction failure cause and preserve prior closing and verification evidence.
- Reconstructible legacy evidence and current manifest-backed evidence remain non-reopenable.
- Authentication, explicit actor and reason, fresh verification, and new closing approval remain mandatory.
- Reverification and acceptance produce a modern manifest that can be archived.
