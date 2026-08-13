## ADDED

### REQUIREMENT REQ-cmd-issues-002

Snapshot source validation SHALL distinguish a confined directory from a rejected path.

Acceptance Criteria

- A `files:` entry that resolves to a directory inside the project root is represented distinctly
  and reported as a mapping-shape error, not as an out-of-root security escape.
- Symbolic links and reparse points remain rejected, and that rejection is evaluated before the
  directory case.
