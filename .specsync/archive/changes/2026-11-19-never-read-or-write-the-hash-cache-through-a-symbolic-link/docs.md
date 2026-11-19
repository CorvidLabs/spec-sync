---
change: never-read-or-write-the-hash-cache-through-a-symbolic-link
artifact: docs
---

# Docs

- `CHANGELOG.md`, `[Unreleased]`, a `### Security` entry: what a cloned repository could do, which versions, and what `check` and `rehash` do now.
- `site/src/content/docs/cli.md`, the hash-cache note: the cache is never read or written through a symbolic link; `check` warns and runs without it, `rehash` fails.
- `docs/HLD.md`, the hash-cache paragraph: the same rule in one sentence.
- `specs/hash_cache` and `specs/cmd_check`: Public API row for `refusal`, invariants, error cases, `testing.md`, `tasks.md` and a lesson in each `.3md`.

The advisory text and the release notes are written when the human decides to publish.
