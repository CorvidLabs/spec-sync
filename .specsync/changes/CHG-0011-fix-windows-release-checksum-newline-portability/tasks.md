---
change: CHG-0011-fix-windows-release-checksum-newline-portability
artifact: tasks
---

# Tasks

- [x] Write the Windows checksum with an explicit LF newline.
- [x] Verify every generated archive/checksum pair before artifact upload.
- [x] Add a regression check for LF acceptance and CRLF rejection.
- [x] Run strict lifecycle, workflow, and repository validation.
- [ ] Open a focused PR linked to issue #342.
