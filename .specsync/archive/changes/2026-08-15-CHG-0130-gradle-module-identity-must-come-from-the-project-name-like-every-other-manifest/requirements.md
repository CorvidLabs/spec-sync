---
change: CHG-0130-gradle-module-identity-must-come-from-the-project-name-like-every-other-manifest
artifact: requirements
---

# Requirements

`REQ-manifest-00N` — a Gradle project's module identity SHALL come from its
project name, not from a source path segment.

`REQ-validator-00N` — no child of a JVM source root SHALL be treated as a
module.

Out of scope: discovery for other ecosystems, which already derive identity from
their manifests.
