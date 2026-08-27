---
module: view
version: 5
status: stable
files:
  - src/view.rs
db_tables: []
tracks: [94]
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
8. A CRLF-authored spec renders exactly as its LF twin does. `view_spec` reads the file with no normalization of its own, so a Windows clone with `core.autocrlf=true` used to fail with "Cannot parse frontmatter" on every spec in the project — on Windows, for which SpecSync then published a binary that no ordinary CI job ever exercised. SpecSync 6.0 publishes no Windows binary, and the tolerance is unaffected by that: a CRLF checkout reaches a Linux or macOS reader through any teammate who authored on Windows. The tolerance belongs to `parser::parse_frontmatter`, and this module states the outcome it depends on rather than re-implementing it.
9. This module owns no frontmatter stripper. The companion `requirements.md` is stripped with `parser::strip_frontmatter`, the single canonical implementation. A local copy is how `view` and `change` came to disagree about CRLF and about a closing delimiter at end of file, with no error raised in either direction.

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
| Frontmatter absent or unterminated | Returns `Err` with parse error |
| CRLF line endings | Not an error condition — the spec and its companion render as they would on LF |
| Companion `requirements.md` missing or empty after stripping | Omitted; the rest of the view still renders |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| parser | `parse_frontmatter` for module name, status, and agent_policy; `strip_frontmatter` for the companion `requirements.md` |

### Consumed By

| Module | What is used |
|--------|-------------|
| main | `view_spec`, `valid_roles` via `cmd_view` subcommand |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-10 | Populated requirements.md with user stories, acceptance criteria, constraints, and out-of-scope items |
| 2026-04-06 | Initial spec for v3.3.0 |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-08-25 | one-canonical-frontmatter-reader-for-crlf-checkouts: One canonical frontmatter reader for CRLF checkouts |
| 2026-08-27 | drop-the-windows-binary-from-the-6-0-release-matrix-and-stop-claiming-windows-support-while-keeping-every-windows: Drop the Windows binary from the 6.0 release matrix and stop claiming Windows support, while keeping every Windows-content correctness guarantee |
