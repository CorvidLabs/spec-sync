---
title: For AI Agents
layout: default
nav_order: 6
---

# For AI Agents
{: .no_toc }

SpecSync is designed to work well with AI coding agents and LLM-powered tooling.
{: .fs-6 .fw-300 }

<details open markdown="block">
  <summary>Table of contents</summary>
  {: .text-delta }
1. TOC
{:toc}
</details>

---

## Why It Works for AI

- **Specs are plain markdown** — any LLM can read and write them
- **The Public API format** uses backtick-quoted names that are unambiguous to parse
- **`--json` flag** outputs structured results, no terminal color codes to strip
- **Exit code 1** means something needs fixing; **0** means all clear
- **`specsync generate`** can scaffold specs automatically for an existing codebase

---

## Recommended Workflow

```bash
# 1. Check the current state
specsync check --json

# 2. Fix any errors reported (update specs or source code)

# 3. Generate specs for new modules that don't have one
specsync generate

# 4. Verify everything passes
specsync check --strict --require-coverage 100
```

---

## JSON Output Shapes

### Check result

```json
{
  "passed": false,
  "errors": [
    "auth.spec.md: Spec documents 'oldFunction' but no matching export found in source"
  ],
  "warnings": [
    "auth.spec.md: Export 'newHelper' not in spec (undocumented)"
  ],
  "specs_checked": 12
}
```

### Coverage result

```json
{
  "file_coverage": 85.33,
  "files_covered": 23,
  "files_total": 27,
  "modules": [
    {
      "name": "helpers",
      "has_spec": false
    }
  ]
}
```

---

## Parsing Tips

- **Errors** are things that must be fixed — the spec references something that doesn't exist
- **Warnings** are informational — code exports something the spec doesn't mention
- With `--strict`, warnings become errors (useful for enforcing full documentation)
- The `specs_checked` field tells you the total number of specs processed
- `file_coverage` is a float between 0 and 100

---

## Writing Specs Programmatically

When generating or updating specs, follow these rules:

1. **Frontmatter** must have `module`, `version`, `status`, and `files` fields
2. **Status** must be one of: `draft`, `review`, `stable`, `deprecated`
3. **Files** must be a non-empty list of paths relative to the project root
4. **Public API tables** must use backtick-quoted names in the first column
5. **Required sections** default to: Purpose, Public API, Invariants, Behavioral Examples, Error Cases, Dependencies, Change Log

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
| 2026-03-18 | Initial spec |
```

---

## Integration Ideas

- **Pre-commit hook**: Run `specsync check --strict` before allowing commits
- **PR review bot**: Run `specsync check --json` and post results as a PR comment
- **Spec generation**: After adding new modules, run `specsync generate` to scaffold
- **Documentation pipeline**: Use spec files as source-of-truth for API documentation
- **AI code review**: Feed `specsync check --json` output to an LLM to suggest spec updates
