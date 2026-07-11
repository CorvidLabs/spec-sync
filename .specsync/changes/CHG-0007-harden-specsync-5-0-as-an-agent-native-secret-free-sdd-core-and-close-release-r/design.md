---
change: CHG-0007-harden-specsync-5-0-as-an-agent-native-secret-free-sdd-core-and-close-release-r
artifact: design
---

# Design

SpecSync 5.0 has one trust boundary: deterministic local SDD state and validation. Inference belongs to the coding agent invoking SpecSync, not to the SpecSync process.

- Remove `src/ai.rs`, `corvid-ai`, provider enums, credential/model/base-URL/command config, and provider/model CLI or MCP arguments.
- Simplify generator APIs to template-only calls and remove AI fallback/error bookkeeping.
- Preserve `specsync mcp`, `specsync agents install`, installed skills, and agent-oriented JSON/Markdown output.
- Treat `pub(crate)` as visible to a module spec because a SpecSync contract can intentionally describe crate-internal collaboration across multiple files; keep narrower restricted visibility private.
- Keep `specsync comment` stdout protocol-clean by suppressing configured verification-command output during report collection, and defensively truncate the GitHub mascot context.
- Upgrade the documentation toolchain to a non-vulnerable Astro line and adapt first-party integrations together.

Removed 4.x AI keys remain accepted as unknown/deprecated input only long enough to print migration guidance; they never activate network, credential, or shell behavior.
