---
change: bind-semantic-delta-bodies-to-the-approval-that-signed-them
artifact: context
---

# Context

#704 demonstrated, end to end, that a semantic delta body can be swapped between `approve` and
materialization and that the swapped wording lands in the living spec with no error and no warning.
The approval ledger goes on asserting that a human signed a definition; the canonical spec carries
text that definition never covered.

Nothing covered the region, and each mechanism misses it for its own reason:

- `definition_digest` under workflow v2 is `scope_digest`, a projection of intent and boundary. It
  deliberately excludes wording so that editing a delta stales verification instead of demanding a
  fresh human approval — but nothing was left holding the wording.
- `validate_delta_files` reads `entry.file_name()`. It checks the module set, never a byte.
- `project_input_digest` excludes `.specsync/changes/` by design (`project_input_is_volatile`).
- the descendant walk would notice, and passes 0 of 107 archived reviews (#694).

Workflow v1 did NOT have this hole: `definition_artifact_snapshot` hashes each delta file's payload
into the v1 definition digest. The binding existed and was dropped at the v2 boundary. Spec
invariant 3 still claimed it, which is why the gap read as covered.

The threat model is the one #704 states, not a larger one: this needs local write access to the
workspace between approve and materialize. It is not remote. What it breaks is evidence integrity —
and the same window is reached without malice by a bad merge, a rebase that resurrects an older
delta, an agent editing the wrong file, or two changes racing on one workspace.

The hard constraint is compatibility. 183 archived changes carry no such digest and never could. An
absent digest must read as "this approval made no claim about wording", never as "the wording was
tampered with". This repository has shipped the opposite reading three times — #672 read an
unparseable schema as every table missing, #684 read a missing config as a gating warning, and
#689's first design would have reported "ready" from absent evidence — so the absent case is the
part of this change that got the most care, and it has its own test.

Ruled out along the way:

- Adding the digests to `ApprovedScopeV1`. That struct is the `scope_digest` preimage; a new field
  changes every existing scope digest and invalidates every live approval.
- Adding a field to `ChangeRecord`. The workflow-v1 definition digest serializes the whole record,
  so a field there is only digest-safe while it is omitted — a trap for the next person.
- Putting the check inside `prepare_delta_application`. It is the one choke point both application
  paths share, but four existing tests call it directly on draft fixtures with no definition
  approval at all, so the check would have had to treat "no approval" as "proceed" — a second
  absent-evidence rule with a much weaker justification than the one this change is built on.
