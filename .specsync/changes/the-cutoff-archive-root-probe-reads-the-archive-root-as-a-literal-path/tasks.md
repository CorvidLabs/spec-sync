---
change: the-cutoff-archive-root-probe-reads-the-archive-root-as-a-literal-path
artifact: tasks
---

# Tasks

- [x] Red-first unit test: a project renamed to `:(exclude)app` at a cutoff with no archive root is answered, not reported unreadable
- [x] Pass the archive root to the fallback probe as `:(top,literal)<archive root>`
- [ ] Targeted cutoff tests, fmt and clippy
