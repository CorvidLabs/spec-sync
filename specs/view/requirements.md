---
spec: view.spec.md
---

## User Stories

- As a developer, I want to view only technical sections of a spec (Purpose, Public API, Invariants, Dependencies, Change Log) so that I can focus on implementation details without distraction
- As a QA engineer, I want to see Behavioral Examples, Error Cases, and Invariants so that I can understand test scenarios and validation requirements
- As a product manager, I want to see the Purpose and Change Log sections so that I can understand feature intent and track changes over time
- As a product manager, I want requirements.md content automatically appended to my view so that I can see acceptance criteria without opening multiple files
- As an AI agent, I want to see all relevant sections including agent policy metadata so that I can work within my defined constraints and permissions
- As a user, I want a clear error message when I specify an invalid role so that I know which roles are available
- As an AI agent, I want to see the module status and agent_policy in my view header so that I understand my access level before processing
- As a CLI user, I want to programmatically retrieve the list of valid roles so that I can build role-aware tooling

## Acceptance Criteria

- Four roles are supported: `dev`, `qa`, `product`, and `agent`
- Unknown roles return an error — never silently fall back to a default
- Dev view includes: Purpose, Public API, Invariants, Dependencies, and Change Log sections
- QA view includes: Behavioral Examples, Error Cases, and Invariants sections
- Product view includes: Purpose and Change Log sections, plus requirements.md content if present in the same directory
- Agent view includes: Purpose, Public API, Invariants, Behavioral Examples, and Error Cases sections
- Agent view header includes `status` and `agent_policy` extracted from frontmatter (rendered as `**Status:** ...` and `**Agent Policy:** ...` lines)
- Output includes a role-specific header line formatted as `# {module} (view: {role})` when the module name is present in frontmatter
- `agent_policy` defaults to the literal `"not set (default: full-access)"` line if not specified in frontmatter
- Section filtering matches against `## ` heading prefixes
- `valid_roles()` returns a static slice of the four supported role strings
- `view_spec()` returns `Result<String, String>` with filtered markdown on success or error message on failure
- Error messages for invalid roles include the list of valid roles
- Error messages for unreadable files include the specific read error description
- Error messages for frontmatter parse failures include the parse error details

## Constraints

- Section visibility is fixed per role — no runtime configuration of which sections appear
- Section matching is based on exact `## ` heading prefix — subsections (###) are included if their parent section is visible
- Four roles are hardcoded — no support for custom or dynamic roles
- Requirements.md must be located in the same directory as the spec file to be automatically appended for product view
- Frontmatter must be parseable by the parser module for agent view to extract status and agent_policy
- Output format is always markdown string — no support for alternative formats (JSON, YAML, etc.)
- Role validation happens before file reading — invalid roles fail fast without attempting to read the spec

## Out of Scope

- Custom role definitions or user-defined role configurations
- Fine-grained section visibility (e.g., hiding specific subsections within a visible section)
- Output format options other than markdown
- Recursive inclusion of requirements.md from parent directories
- Support for multiple requirements.md files or alternative companion file names
- Role-based access control or authentication
- Caching of filtered views
- Streaming or partial rendering of large specs
- Command-line interface for view filtering (handled by consuming module)