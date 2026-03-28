---
title: Cross-Project References
layout: default
nav_order: 5
---

# Cross-Project References
{: .no_toc }

Validate spec dependencies across repositories. Zero network cost by default — remote verification is opt-in.
{: .fs-6 .fw-300 }

<details open markdown="block">
  <summary>Table of contents</summary>
  {: .text-delta }
1. TOC
{:toc}
</details>

---

## Overview

Specs can declare dependencies on modules in other repositories using the `owner/repo@module` syntax in `depends_on`. This lets you verify that upstream APIs you depend on are still documented and available.

```yaml
depends_on:
  - specs/database/database.spec.md          # local ref
  - corvid-labs/algochat@messaging           # cross-project ref
```

**Local refs** are validated by `specsync check` (file must exist). **Cross-project refs** require `specsync resolve --remote` which fetches the target repo's registry from GitHub.

---

## How It Works

1. **You declare a cross-project dependency** in your spec's `depends_on`
2. **`specsync resolve --remote`** parses the `owner/repo@module` syntax
3. **Fetches `specsync-registry.toml`** from the target repo's default branch on GitHub
4. **Checks that the module exists** in the registry

No authentication required for public repos. Private repos need a `GITHUB_TOKEN` environment variable.

---

## Publishing Your Registry

For other projects to reference your modules, commit a `specsync-registry.toml` to your repo's default branch:

```bash
specsync init-registry                     # uses project folder name
specsync init-registry --name myapp        # custom name
git add specsync-registry.toml
git commit -m "chore: add spec registry for cross-project refs"
git push
```

The registry lists all modules from your specs directory:

```toml
[registry]
name = "spec-sync"
generated = "2026-03-28T00:00:00Z"

[[modules]]
name = "cli"
spec = "specs/cli/cli.spec.md"

[[modules]]
name = "parser"
spec = "specs/parser/parser.spec.md"
```

---

## Verifying References

```bash
# Local refs only (no network, runs in check too)
specsync resolve

# Local + cross-project refs (fetches registries from GitHub)
specsync resolve --remote
```

Output:

```
Cross-project references:
  ✓ CorvidLabs/spec-sync@cli — resolved
  ✓ CorvidLabs/spec-sync@parser — resolved
  ✗ CorvidLabs/spec-sync@nonexistent — module not in registry
```

---

## CI Usage

Add `resolve --remote` to your CI pipeline to catch broken cross-project refs:

```yaml
- name: Verify cross-project refs
  run: specsync resolve --remote
```

{: .note }
> `specsync check` validates local refs only and never hits the network. Use `resolve --remote` explicitly when you want cross-project verification. This keeps CI fast by default.

---

## Error Cases

| Scenario | Output |
|:---------|:-------|
| Module not in registry | `✗ module not in registry` |
| Repository not found | `! HTTP 404` + `? registry fetch failed` |
| No registry file | `? registry fetch failed` (registry not committed) |
| Network error | `? registry fetch failed` with details |
