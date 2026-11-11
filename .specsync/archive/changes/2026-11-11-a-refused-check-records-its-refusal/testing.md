---
change: a-refused-check-records-its-refusal
artifact: testing
---

# Testing

Integration: an approved change with an open task gets checked; the refusal
exits 1, and verification-attempts.json holds a passed:false entry naming the
reason while no verification.json appears.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-154 | integration test driving the tasks-gate refusal |
