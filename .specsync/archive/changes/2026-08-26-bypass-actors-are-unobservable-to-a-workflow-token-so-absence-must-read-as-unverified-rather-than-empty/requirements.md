---
change: bypass-actors-are-unobservable-to-a-workflow-token-so-absence-must-read-as-unverified-rather-than-empty
artifact: requirements
---

# requirements

Absent `bypass_actors` reads as unverified rather than empty, because GitHub returns the field only to repository administrators and a workflow token is not one.
