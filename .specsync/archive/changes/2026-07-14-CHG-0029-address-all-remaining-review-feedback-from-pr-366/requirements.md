---
change: CHG-0029-address-all-remaining-review-feedback-from-pr-366
artifact: requirements
---

# Requirements

### REQ-change-030

Lifecycle enforcement SHALL preserve explicit user scope, registry authority, policy opt-out boundaries, and native verification commands while retaining fail-closed SpecSync recursion protection.

#### Acceptance Criteria

- Generated sequence bookkeeping does not satisfy or suppress the interview question for source, test, documentation, or configuration scope.
- Registry-resolved canonical specs and companions participate in meaningful-path coverage and acceptance hashing.
- The local registry is a protected lifecycle input because it controls canonical writes.
- An explicitly disabled SDD policy returns without sequence-ledger validation.
- Native `cargo run -- check` commands remain allowed unless Cargo is actually selecting the SpecSync binary.

### REQ-cli-004

The root CLI SHALL reject inherited verification re-entry before dispatching any lifecycle command handler.

#### Acceptance Criteria

- Explicit and default `check`, `change`, and `lifecycle` commands fail before handler-specific discovery, warnings, validation, or mutation.
- The process emits one contextual diagnostic and exits non-zero.
- Commands outside the lifecycle boundary preserve current dispatch behavior.
