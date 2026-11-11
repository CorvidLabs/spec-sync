---
change: the-ownership-recovery-narrows-a-scope-over-files-the-change-never-touched
artifact: tasks
---

# Tasks

- [x] Split unowned inputs into modified and untouched since the change's base commit
- [x] Narrowing recovery for untouched inputs; declare recovery only for modified ones
- [x] Unit test with one modified and one untouched unowned file under one directory scope
- [x] REQ-cmd-change-008 and REQ-change-053 name the narrowing recovery
