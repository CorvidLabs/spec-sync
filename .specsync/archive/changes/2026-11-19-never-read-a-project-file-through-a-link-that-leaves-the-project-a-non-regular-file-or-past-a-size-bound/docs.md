---
change: never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound
artifact: docs
---

# Docs

- `CHANGELOG.md`, `[Unreleased]`, `### Security`: one entry describing what a repository could make a read do, and the rule and bounds now.
- `site/src/content/docs/cli.md`: extend change 2's symbolic-link note to reads, with the bounds.
- Each affected spec gets an invariant, an error case, a `testing.md` and a `tasks.md` entry; `util.3md` gets a lesson.
