# Retire the change contract clauses that promise configured verification commands run

## MODIFIED

### REQUIREMENT REQ-change-019

Verification SHALL recognize a non-removed requirement or spec-section delta item as semantic acceptance evidence when observable acceptance criteria are present.

Acceptance Criteria

- A section-only modified delta can pass with an empty requirement-ID list.
- Requirement evidence mapping remains mandatory for every collected requirement ID.
- Missing semantic acceptance evidence and missing requirement evidence produce distinct diagnostics, and neither is reported as a configured-command failure, because verification executes no configured command (REQ-change-023).

### REQUIREMENT REQ-change-027

A SpecSync lifecycle command SHALL refuse to run under an inherited verification context, at every lifecycle command surface.

Acceptance Criteria

- `check`, `change`, and `lifecycle` commands that inherit `SPECSYNC_VERIFICATION_CONTEXT` fail once, before performing validation or mutation.
- The diagnostic names the parent command recorded in `SPECSYNC_VERIFICATION_CONTEXT`.
- SpecSync sets that marker on no child of its own: verification executes no configured command (REQ-change-023), so the refusal applies only to a context a caller outside SpecSync supplies.

### REQUIREMENT REQ-change-030

Lifecycle enforcement SHALL preserve explicit user scope, precise canonical companion coverage, registry authority, and policy opt-out boundaries while retaining fail-closed SpecSync recursion protection.

Acceptance Criteria

- Generated sequence bookkeeping does not satisfy or suppress the interview question for source, test, documentation, or configuration scope.
- Registry-resolved modules cover only their exact canonical spec and the standard `requirements.md`, `tasks.md`, `context.md`, `testing.md`, and `design.md` companions; unrelated siblings and the containing directory are not implicitly covered.
- Both registry files remain protected lifecycle inputs because they control canonical writes.
- An explicitly disabled SDD policy returns without sequence-ledger validation.
- `change check` does not execute configured commands (REQ-change-023), so a policy listing `cargo run --bin specsync -- check` cannot recurse through verification.
- Direct SpecSync lifecycle commands remain rejected when invoked under `SPECSYNC_VERIFICATION_CONTEXT`.
- The retained Cargo identity classifier is consulted by no lifecycle command, because no configured command is executed. As a helper it still classifies `cargo run -- check` as non-SpecSync unless Cargo is actually selecting SpecSync by manifest identity, `default-run`, binary, or package.
- In that classifier, both `--manifest-path <path>` and `--manifest-path=<path>` participate in Cargo identity detection, unsafe explicit manifest paths fail closed, and argument parsing tokenizes ordinary whitespace, quoted values, and trailing comments without shell execution.

### REQUIREMENT REQ-change-061

The digest recorded for a bootstrapped policy SHALL pin the enforcement surface rather than the
file's bytes.

Acceptance Criteria

- Every field that determines whether the coverage gate applies is covered by the digest.
- Verification commands are excluded, so editing the list does not revoke the bootstrap. Nothing
  executes the list (REQ-change-023), and fresh initialization writes it empty.
- A policy file that cannot be parsed falls back to a digest of its bytes.

### REQUIREMENT REQ-change-069

Declaring an additional affected module SHALL never remove a verification command from the set the retained routing helper selects for a change. Routing selects commands and executes none of them (REQ-change-023).

Acceptance Criteria
- The command set selected for a scope is a superset of the set selected for any subset of that scope, so widening declared scope can only add commands.
- A declared module with no component routing entry contributes the project-wide verification commands, because a module nobody routed is not a module that needs no verification.
- A change scoped entirely to routed modules is still routed only its component commands, so targeted routing remains available.
- A change declaring no affected module is still routed the project-wide verification commands.
- Strict escalation continues to append its own commands without removing any already selected.
- No lifecycle command executes a routed command; `change check` records only its in-process spec↔code evidence.

## REMOVED

### REQUIREMENT REQ-change-015

Retired (#848). It required a protocol-clean reporting mode that executed every configured
verification command while suppressing the children's stdout and stderr. REQ-change-058 says that
quiet-output variant SHALL NOT exist, and REQ-change-023, REQ-change-049, REQ-change-058 and
REQ-change-091 say verification executes no configured command, so there is no child output to
suppress (change spec invariant 14). Its last criterion, that normal check and change verification
keep their diagnostics, only contrasted them with the removed mode.
