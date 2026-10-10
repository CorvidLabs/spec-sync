---
change: keep-3md-files-from-being-classified-as-roff
artifact: context
---

# Context

GitHub Linguist reads .3* as a man page and a leading dot as a troff macro. The 3md repo forces *.3md to plain text. spec-sync init writes that same rule into every project.
