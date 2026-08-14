---
spec: cmd_view.spec.md
---

## User Stories

- As a developer, I want `specsync view --role dev` to show only the implementation-relevant sections of a spec so I can read its API and invariants without wading through every section
- As a QA engineer, I want a `qa` role view that surfaces behavioral examples, error cases, and invariants so I can derive test cases
- As a product owner, I want a `product` role view limited to purpose and change log
- As an agent/tooling author, I want an `agent` role view that includes status, agent policy, and invariants
- As a user, I want to optionally narrow the output to a single module with `--spec <module>`

## Acceptance Criteria

- `cmd_view` discovers spec files under `config.specs_dir` via `find_spec_files`
- Each spec is rendered through `view::view_spec(path, role)`, which keeps only the sections allowed for the given role
- When `spec_filter` is provided, only the spec whose module name (file stem minus `.spec`) matches is rendered
- Rendered specs are separated by a `---` delimiter line
- A per-file render error is printed to stderr (`error:` prefix) and processing continues to the next spec
- Exits with code 1 when no spec files are found

## Constraints

- Must not panic on expected error conditions — print and exit
- Must work with the project's Clap-based CLI argument parsing
- Read-only: never writes or modifies spec or companion files
- Unknown role names are rejected by `view::view_spec` with a message listing the valid roles (dev, qa, product, agent)

## Out of Scope

- Defining which sections belong to each role (owned by the `view` module, not this command)
- GUI or web rendering
- Interactive prompts (handled by the `wizard` command)

### REQ-cmd-view-001

The view command SHALL render a validated role-specific canonical spec view and fail clearly for unknown roles or specs.

Acceptance Criteria
- `cmd_view` discovers spec files under `config.specs_dir` via `find_spec_files`
- Each spec is rendered through `view::view_spec(path, role)`, which keeps only the sections allowed for the given role
- When `spec_filter` is provided, only the spec whose module name (file stem minus `.spec`) matches is rendered
- Rendered specs are separated by a `---` delimiter line
- A per-file render error is printed to stderr (`error:` prefix) and processing continues to the next spec
- Exits with code 1 when no spec files are found

### REQ-cmd-view-002

Rendering a spec view SHALL NOT report success when it produced nothing, and SHALL name what
it could not find.

Acceptance Criteria
- A requested module that matches no spec is reported by name and exits non-zero.
- The report names a close match when one exists, and otherwise lists the modules that do exist.
- A spec that fails to render causes a non-zero exit rather than being reported and ignored.
- A requested module that exists is still rendered and exits zero.
- Running with no module filter is unchanged.
