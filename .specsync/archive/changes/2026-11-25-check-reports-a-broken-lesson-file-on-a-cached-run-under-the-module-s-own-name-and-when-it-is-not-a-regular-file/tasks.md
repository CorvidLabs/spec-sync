---
change: check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file
artifact: tasks
---

# Tasks

- [x] Failing tests first: a cached `check` after the file breaks, a spec file not named for its module, a directory named `<module>.3md`.
- [x] The hash cache binds the `.3md` entries beside a spec; the validator names the lesson file from the module and reports a non-regular entry.
- [x] REQ-cmd-check-023's wording and the changelog entry say what a strict lifecycle check does.
- [x] Targeted tests, the full native Verify lane, `change audit` and `check --strict` before review.
