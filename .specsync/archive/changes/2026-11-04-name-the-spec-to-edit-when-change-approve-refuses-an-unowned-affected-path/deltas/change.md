## ADDED

### REQUIREMENT REQ-change-120

When `change approve` refuses affected paths that no declared module canonically owns, the error SHALL say which spec to edit for each refused path. Ownership itself is still decided exactly as REQ-change-053 requires.

#### Acceptance Criteria

- A refused path that some spec under the specs directory already lists in its `files:` names that spec file and its module, and says to declare it with `--spec`.
- Otherwise the refused path names the declared spec whose `files:` entries share the longest directory with the path, and says to add the path to that spec's `files:` list.
- The same paths are refused as before; only the wording of the refusal changes.
