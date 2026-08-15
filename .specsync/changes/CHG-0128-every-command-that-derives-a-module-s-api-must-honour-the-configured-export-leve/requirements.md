---
change: CHG-0128-every-command-that-derives-a-module-s-api-must-honour-the-configured-export-leve
artifact: requirements
---

# Requirements

`REQ-scoring-00N` — the API dimension SHALL grade against the configured export
surface, so `score` and `check` cannot disagree about what a module's API is.

`REQ-generator-00N` — generated specs SHALL contain only symbols the configured
surface includes, so the tool cannot emit a spec its own validator rejects.

`REQ-exports-00N` — an entry point that derives exports without stating the
surface SHALL be documented as unsafe for new callers.

`REQ-cmd-new-00N`, `REQ-cmd-scaffold-00N`, `REQ-cmd-diff-00N` — each SHALL
derive exports from the configured surface and parse mode.

Out of scope: retiring the hard-coded wrappers from the exports contract.
