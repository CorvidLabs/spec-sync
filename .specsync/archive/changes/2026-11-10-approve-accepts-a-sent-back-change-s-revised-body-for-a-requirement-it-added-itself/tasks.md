---
change: approve-accepts-a-sent-back-change-s-revised-body-for-a-requirement-it-added-itself
artifact: tasks
---

# Tasks

- [x] `validate_delta_files` applies the send-back own-block exception before refusing an existing ADDED requirement
- [x] Test: a sent-back change is re-approved with a revised body for a requirement it added, and materialization replaces it
- [x] Test: an ADDED requirement whose canonical text someone else changed is still refused
