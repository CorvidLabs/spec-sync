## ADDED

### REQUIREMENT REQ-cmd-scaffold-004

`scaffold --dir` SHALL remain beneath the project root. Absolute paths outside the root and `..` components are refused.

Acceptance Criteria
- `--dir /tmp` exits 1 naming `scaffold --dir must remain beneath the project root`.
- `--dir ../escape` exits 1 via `confined_generation_path` and writes nothing outside the project.
- A relative directory under the root is joined to the root and used as the specs directory.
