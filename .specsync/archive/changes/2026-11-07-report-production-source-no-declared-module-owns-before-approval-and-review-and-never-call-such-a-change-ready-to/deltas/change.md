## MODIFIED

### REQUIREMENT REQ-change-053


Canonical ownership of declared paths SHALL be resolved when the definition is
approved, and a change that has never closed SHALL be able to correct an
acceptance input owner without an audited reopen event.

Acceptance Criteria

- Approving a definition rejects declared paths that no declared module
  canonically owns, naming every offending path in one error.
- Paths that do not yet exist are not rejected at approve, since the owning spec
  may claim them in the same change; they remain enforced at finalize.
- A change with justified `no_spec_change` and no declared specs is
  ownership-checked at approve as well: an existing production-source path in
  its scope has no declared owner, so it is rejected, naming the module that
  owns it, before anyone approves or reviews it (#986, #956). Empty specs
  without that justification still fail closed at definition validation and at
  ownership validation.
- A change at verifying that has never closed may correct an acceptance input
  owner under a currently valid definition approval. That substitute is for
  guided-path reachability, not audit-equivalent provenance to an
  Accepted→reopen cycle.
- A change that did close continues to require an audited reopen, unchanged.

### REQUIREMENT REQ-change-120


When `change approve` refuses affected paths that no declared module canonically owns, the error SHALL say which spec to edit for each refused path. Ownership itself is still decided exactly as REQ-change-053 requires.

#### Acceptance Criteria

- A refused path that some spec under the specs directory already lists in its `files:` names that spec file and its module, and says to declare it with `--spec`. Every such spec is named, sorted by spec path, whatever order the file system returns them in.
- A listing spec that cannot own the path says why: it has no `module:`, or it is not the canonical spec of the declared module it names.
- Otherwise the refused path names the declared spec whose `files:` entries share the longest directory with the path, comparing whole path components, and says to add the path to that spec's `files:` list. When no declared spec shares any directory with the path (depth 0), the refusal gives the generic message instead of naming a spec.
- For a draft, the refusal also names `specsync change answer <id> affected_specs <modules>`, listing the declared modules and every undeclared module that owns a refused path, so running it declares them (#986).
- Which paths are refused is decided by REQ-change-053; this requirement governs only the wording.
