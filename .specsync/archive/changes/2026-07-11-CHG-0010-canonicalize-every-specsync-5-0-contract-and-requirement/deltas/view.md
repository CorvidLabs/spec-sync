## ADDED

### REQUIREMENT REQ-view-001

Role-based views SHALL expose the documented canonical sections and companion requirements for valid roles without changing source files.

Acceptance Criteria
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
