---
id: "readme"
title: "Project knowledge"
type: readme
tags:
  - chough/readme
---

# Project knowledge

This folder is the project's librarian, on by default (`chough pref set librarian off
--source <who asked>` opts out). It keeps historical copies with source links and digests
(`snapshots/`), focused next-session briefs (`briefs/`), one regenerated entry per archived
native change (`archives/`), one note per module (`modules/`) and per HI file (`intent/`),
the rules, skills and decisions (`rules.md`) and a generated index (`INDEX.md`) with a
lessons index. Every note starts with YAML front matter and links with standard Markdown
links, so the folder opens as an Obsidian vault. It is
not a ledger and no gate reads it. The living sources stay where their owners keep
them: HI under `hi/`, contracts under `specs/`, native changes and archives under
`.specsync/`, prototypes beside their companions, and the checkpoint and bug backlog
under `.chough/`. Nothing here is a second current version of those sources.
