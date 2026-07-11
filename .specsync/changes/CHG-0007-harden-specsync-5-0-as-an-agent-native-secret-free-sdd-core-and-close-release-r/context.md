---
change: CHG-0007-harden-specsync-5-0-as-an-agent-native-secret-free-sdd-core-and-close-release-r
artifact: context
---

# Context

CHG-0007 follows accepted CHG-0006 on the same 5.0 branch and PR. It is required because issue #334 and repository security alerts were discovered after the prior closing evidence was recorded. Existing accepted workspaces stay active until the delivery diff merges; this change owns all additional source, dependency, workflow, and documentation edits.

The product decision is explicit: SpecSync itself is the trustworthy deterministic SDD engine. Coding agents consume its CLI, JSON/Markdown output, MCP resources/tools, installed skills, and change workspaces, but credentials and model execution remain entirely outside the core binary.

The repository README and `site/` documentation are in scope. The separate CorvidLabs/site repository is intentionally deferred until after this branch is release-ready.
