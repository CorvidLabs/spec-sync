---
change: CHG-0016-preserve-free-text-arguments-in-generated-agent-commands
artifact: context
---

# Context

SpecSync 5.0.1 installs deterministic command and skill templates for Claude Code, Cursor, Codex, and Gemini. The
create-spec prose currently consumes the first token before deciding whether the full input is a module identifier
or a natural-language description. Gemini's create-change command also inherits Claude/Cursor's
`$ARGUMENTS` spelling, and interview guidance does not quote multi-word answers. These defects recur in every
consumer that regenerates integrations, so the installer templates are the correct repair boundary.
