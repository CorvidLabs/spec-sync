---
change: close-changes-that-will-not-ship
artifact: research
---

# Research

- The late close (src/change/doctor.rs) already moves packages out of the enumerated directories
  with a record, drops collision acknowledgements, checks symlinked destinations and rolls back a
  failed batch; its lookups (`find_change_dir`, ID reservation, status reports) are the hooks a
  closed change needs too.
- `change list` lists active changes only, so a closed change leaves it as an archived one does.
