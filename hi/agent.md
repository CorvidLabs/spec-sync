---
hi: 1
families: [AGENT]
---

# Working with a coding agent

## Intent

Coding agents are the heaviest readers and writers of these documents, and they should use exactly the tools a person uses — no agent mode, no second source of truth. SpecSync stays the deterministic part: it never becomes the place credentials live and never sends source to a model, so whatever an agent enriches, it enriches under its own permissions. It is also the thing that stops an agent short of approving its own work.

## Criteria

- **AGENT-1**  My coding agent drives the same commands I do, so there is no second source of truth to keep aligned.
- **AGENT-2**  Every command can return structured data, so an agent never has to scrape a terminal.
- **AGENT-3**  SpecSync never becomes the place my model credentials live.
  - **AGENT-3.a**  My source is never sent to a model by the tool itself.
  - **AGENT-3.b**  Whatever an agent enriches, it enriches under its own credentials and permissions.
- **AGENT-4**  I can expose SpecSync to an agent as a tool server it calls directly.
  - **AGENT-4.a**  That server only reads until I explicitly allow it to write.
  - **AGENT-4.b**  Even when writing, it cannot reach outside the project root I pointed it at.
  - **AGENT-4.c**  The set of tools an agent is offered is pinned, so an upgrade cannot quietly change what it can do.
- **AGENT-5**  I can install SpecSync's instructions into whichever coding tools I actually use, in each one's native form.
- **AGENT-6**  An agent working in my repository is handed the rules of the house before it starts.
  - **AGENT-6.a**  It reads a module's contract before it changes that module's code.
  - **AGENT-6.b**  It brings the contract along in the same change as the code.
  - **AGENT-6.c**  It never approves its own work.
- **AGENT-7**  A commit hook can stop drift from being committed in the first place.
