---
module: view
version: 1
status: stable
files:
  - src/view.rs
db_tables: []
depends_on:
  - specs/parser/parser.spec.md
---

# View

## Purpose

Filters spec content to show only sections relevant to a specific role (dev, qa, product, agent). Enables focused consumption of specs by different stakeholders without information overload.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `view_spec` | `spec_path: &Path, role: &str` | `Result<String, String>` | Read a spec file and return filtered markdown containing only role-relevant sections |
| `valid_roles` | — | `&'static [&'static str]` | Returns the list of valid role names: `["dev", "qa", "product", "agent"]` |

### Role → Section Visibility

| Role | Visible Sections |
|------|-----------------|
| dev | Purpose, Public API, Invariants, Dependencies, Change Log |
| qa | Behavioral Examples, Error Cases, Invariants |
| product | Purpose, Change Log (+ companion requirements.md if present) |
| agent | Purpose, Public API, Invariants, Behavioral Examples, Error Cases |

## Invariants

1. Four roles are supported: `dev`, `qa`, `product`, `agent`
2. Unknown roles return an error — never silently fall back
3. The `agent` role includes `status` and `agent_policy` from frontmatter in the output header
4. The `product` role appends companion `requirements.md` content if the file exists
5. `agent_policy` defaults to `"full-access"` if not set in frontmatter
6. Output includes a role-specific header line (e.g., `# ModuleName (dev view)`)
7. Section matching is based on `## ` heading prefixes

## Behavioral Examples

### Scenario: Dev view

- **Given** a spec with all standard sections
- **When** `view_spec(path, "dev")` is called
- **Then** returns Purpose, Public API, Invariants, Dependencies, and Change Log sections only

### Scenario: Agent view with policy

- **Given** a spec with `agent_policy: read-only` in frontmatter
- **When** `view_spec(path, "agent")` is called
- **Then** output header includes `Status: stable` and `Agent Policy: read-only`

### Scenario: Product view with requirements

- **Given** a spec at `specs/auth/auth.spec.md` with a companion `specs/auth/requirements.md`
- **When** `view_spec(path, "product")` is called
- **Then** returns Purpose, Change Log, and appended requirements.md content

### Scenario: Invalid role

- **Given** role string `"manager"`
- **When** `view_spec(path, "manager")` is called
- **Then** returns `Err` with descriptive message listing valid roles

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Unknown role string | Returns `Err` listing valid roles |
| Spec file unreadable | Returns `Err` with read error description |
| Frontmatter parse failure | Returns `Err` with parse error |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| parser | `parse_frontmatter` for extracting module name, status, and agent_policy |

### Consumed By

| Module | What is used |
|--------|-------------|
| main | `view_spec`, `valid_roles` via `cmd_view` subcommand |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-06 | Initial spec for v3.3.0 |
