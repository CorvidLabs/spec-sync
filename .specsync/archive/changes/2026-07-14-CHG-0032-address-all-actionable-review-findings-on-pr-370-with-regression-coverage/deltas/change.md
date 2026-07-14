## MODIFIED

### REQUIREMENT REQ-change-029

Acceptance evidence SHALL preserve historical validity across valid later sequence claims without weakening current sequence-ledger integrity.

Acceptance Criteria

- Creating a later valid lifecycle record does not stale an earlier accepted record solely because the sequence ledger advanced.
- Historical reconstruction uses the earlier owner and includes only collision acknowledgements whose sequence is not later than that owner.
- The current sequence owner remains bound to the exact current ledger content.
- Malformed claims, claims without a workspace, non-maximum claims, duplicate sequences, and invalid collision acknowledgements fail closed.
- Every covered path other than a valid later-owned sequence ledger remains acceptance-digest input.

### REQUIREMENT REQ-change-030

Lifecycle enforcement SHALL preserve explicit user scope, exact registry authority, policy opt-out boundaries, and native verification commands while retaining fail-closed SpecSync recursion protection.

Acceptance Criteria

- Generated sequence bookkeeping does not satisfy or suppress the interview question for source, test, documentation, or configuration scope.
- Registry-resolved affected modules cover only their exact canonical spec and requirements companion unless broader paths are explicit.
- Both `.specsync/registry.toml` and the supported root `specsync-registry.toml` authority files are protected lifecycle inputs.
- An explicitly disabled SDD policy returns without sequence-ledger validation.
- Native `cargo run -- check` commands remain allowed unless Cargo selects the SpecSync binary by manifest identity, binary target, or package option.
- Direct SpecSync verification commands, including an implicit default check, are rejected before execution.
- Cargo package identity parsing accepts ordinary table whitespace, single- or double-quoted strings, and trailing comments.
