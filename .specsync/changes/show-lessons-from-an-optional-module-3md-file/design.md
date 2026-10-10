---
change: show-lessons-from-an-optional-module-3md-file
artifact: design
---

# Design

When specs/<module>/<module>.3md exists and parses, the briefing reads every plane after the lowest one and names that file. The lowest plane is context and is not quoted as a lesson. When the file is absent, context.md is unchanged as the lesson source. A bad file yields no lesson and does not fail the command.
