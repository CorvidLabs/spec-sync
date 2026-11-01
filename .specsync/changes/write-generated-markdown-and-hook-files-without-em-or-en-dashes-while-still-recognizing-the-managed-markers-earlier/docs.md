---
change: write-generated-markdown-and-hook-files-without-em-or-en-dashes-while-still-recognizing-the-managed-markers-earlier
artifact: docs
---

# Docs

- No page under `site/src/content/docs/` quotes the hook begin sentinel, the lesson-bundle
  heading, the `.gitignore` headers or a compaction summary row. A grep of `site/src`, `docs/`
  and `README.md` found none, so no site page changes.
- Requirement text arrives through the approved deltas.
- Companion updates (after verification):
  - `specs/compact/requirements.md`: the free acceptance list above REQ-compact-001 names the
    dash form, and is brought in line with the modified requirement.
  - `specs/compact/compact.3md`: the Key Decisions and Notes planes that mention dash
    placeholders get a later lesson plane naming the two accepted forms.
  - `specs/hooks/hooks.3md`: a lesson plane saying that the begin sentinel is read back, so the
    6.1.1 form is a frozen literal that the reader accepts. The pre-commit body is found by its
    project-keyed markers and needs no frozen copy; the unmarked legacy comparison has matched no
    released install since #774.
  - `specs/agents/agents.3md`: template version 6.
- `CHANGELOG.md` is written by the release change. The PR body states:
  - the observable file-format changes listed in `design.md`;
  - that existing installed blocks keep their old text until `hooks uninstall` and
    `hooks install` are run;
  - that downgrade is one-way.
  The release notes can then carry all three.
