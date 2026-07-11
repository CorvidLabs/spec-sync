---
id: CHG-0007-harden-specsync-5-0-as-an-agent-native-secret-free-sdd-core-and-close-release-r
state: accepted
type: bug_fix
base_commit: 58dc5b92ee950e7a5c01e44381a17b52cfa7099c
---

# Harden SpecSync 5.0 as an agent-native, secret-free SDD core and close release regressions

## Intent

Harden SpecSync 5.0 as an agent-native, secret-free SDD core and close release regressions

## Affected Canonical Specs

- `exports`
- `ai`
- `cmd_generate`
- `generator`
- `types`
- `config`
- `cli`
- `mcp`
- `cli_args`
- `cmd_check`
- `cmd_comment`
- `comment`
- `change`

## Acceptance Criteria

- SpecSync detects all documented exports across multi-file Rust modules in regex and AST modes; the core binary contains no embedded LLM client or provider credentials or automatic source transmission or AI shell escape; agent skills and MCP remain supported; Astro security advisories and bounded PR-comment output are fixed; README and repository documentation describe the agent-native model; all local example security cross-platform and GitHub gates pass.

## No-spec Rationale

Not applicable
