## MODIFIED

### REQUIREMENT REQ-change-120

When `change approve` refuses affected paths that no declared module canonically owns, the error SHALL say which spec to edit for each refused path. Ownership itself is still decided exactly as REQ-change-053 requires.

#### Acceptance Criteria

- A refused path that some spec under the specs directory already lists in its `files:` names that spec file and its module, and says to declare it with `--spec`. Every such spec is named, sorted by spec path, whatever order the file system returns them in.
- A listing spec that cannot own the path says why: it has no `module:`, or it is not the canonical spec of the declared module it names.
- Otherwise the refused path names the declared spec whose `files:` entries share the longest directory with the path, comparing whole path components, and says to add the path to that spec's `files:` list. When no declared spec shares any directory with the path (depth 0), the refusal gives the generic message instead of naming a spec.
- The same paths are refused as before; only the wording of the refusal changes.
