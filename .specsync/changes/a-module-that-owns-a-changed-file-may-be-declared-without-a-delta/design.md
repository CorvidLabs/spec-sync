---
change: a-module-that-owns-a-changed-file-may-be-declared-without-a-delta
artifact: design
---

# Design

- `validate_delta_files`: a declared module with no delta file is allowed when it owns one of
  the change's declared files that exist (`module_currently_owns_path` against the acceptance
  owner evidence, the same ownership approve and finalize use). At least one module must still
  carry a delta; a delta must still name an affected spec. The refusal adds that a module may go
  without a delta only when it owns one of the declared files.
- `delta_modules(root, record)`: the affected modules whose delta file exists. Every place that
  reads a delta per affected module iterates it instead: the semantic-item check, the delta
  validation loop, approved delta digests and binding, replacing items, requirement ids,
  pending application, delta keys, effective-contract authorship, and the send-back
  applied-item and orphan checks.
- A code-only module's spec is not edited and gets no Change Log row. It is still an affected
  module everywhere else: acceptance inputs, owner resolution, verification routing, lessons.
- An approved delta deleted later is still caught: the binding reports every approved module
  whose current body is not intact, including one with no file now.
- The effective-contract gate tells the two apart by the approval: a module is skipped as
  code-only only when the effective definition approval recorded delta digests without it
  (`approved_as_code_only`). A delta missing for any other reason still makes authorship
  unknown, so draft sections stay blocking
  (`effective_contract_exempts_nothing_when_authorship_is_unknown`).

Rejected: an `--owner <module>` flag for code-only modules (Leif chose the implicit rule).
