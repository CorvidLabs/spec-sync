---
title: For AI Agents
layout: default
nav_order: 6
---

# For AI Agents
{: .no_toc }

SpecSync is built to work well with LLM-powered coding tools.
{: .fs-6 .fw-300 }

<details open markdown="block">
  <summary>Table of contents</summary>
  {: .text-delta }
1. TOC
{:toc}
</details>

---

## Why It Works

- Specs are plain markdown — any LLM can read and write them
- `--json` outputs structured data, no terminal color codes
- Exit code 1 = needs fixing, 0 = all clear
- `specsync generate` bootstraps specs for existing codebases
- Public API tables use backtick-quoted names, unambiguous to parse

---

## Workflow

```bash
specsync check --json                       # 1. assess current state
# fix errors in specs or source             # 2. resolve issues
specsync generate                           # 3. scaffold missing specs
specsync check --strict --require-coverage 100  # 4. verify
```

---

## JSON Shapes

### Check

```json
{
  "passed": false,
  "errors": ["auth.spec.md: phantom export `oldFunction` not found in source"],
  "warnings": ["auth.spec.md: undocumented export `newHelper`"],
  "specs_checked": 12
}
```

- **Errors**: spec references something that doesn't exist in code — must fix
- **Warnings**: code exports something the spec doesn't mention — informational
- **`--strict`**: promotes warnings to errors

### Coverage

```json
{
  "file_coverage": 85.33,
  "files_covered": 23,
  "files_total": 27,
  "modules": [{ "name": "helpers", "has_spec": false }]
}
```

---

## Writing Specs Programmatically

1. Frontmatter requires `module`, `version`, `status`, `files`
2. Status: `draft`, `review`, `stable`, `deprecated`
3. Files: non-empty list, paths relative to project root
4. Public API tables: backtick-quoted names in first column
5. Default required sections: Purpose, Public API, Invariants, Behavioral Examples, Error Cases, Dependencies, Change Log

### Minimal valid spec

```markdown
---
module: mymodule
version: 1
status: draft
files:
  - src/mymodule.ts
---

# MyModule

## Purpose
TODO

## Public API

| Export | Description |
|--------|-------------|
| `myFunction` | Does something |

## Invariants
TODO

## Behavioral Examples
TODO

## Error Cases
TODO

## Dependencies
None

## Change Log

| Date | Change |
|------|--------|
| 2026-03-19 | Initial spec |
```

---

## Integration Ideas

- **Pre-commit hook**: `specsync check --strict`
- **PR review bot**: parse `specsync check --json` output, post as PR comment
- **Spec generation**: run `specsync generate` after adding modules
- **AI code review**: feed JSON output to an LLM for spec update suggestions
