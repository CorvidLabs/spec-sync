---
change: CHG-0007-harden-specsync-5-0-as-an-agent-native-secret-free-sdd-core-and-close-release-r
artifact: docs
---

# Docs

Update README and `site/src/content/docs/` to present SpecSync as a secret-free deterministic engine used directly or through a coding agent. Remove provider lists, API-key examples, embedded AI claims, `--provider`/`--model`, and `ai_*` configuration. Add migration guidance: run template generation, then ask the connected coding agent to complete and review artifacts through the SpecSync lifecycle.

Document that MCP and installed native agent skills remain supported and require no credentials inside SpecSync. Record the separate CorvidLabs/site update as a post-release-repository follow-up rather than modifying that repository from this change.
