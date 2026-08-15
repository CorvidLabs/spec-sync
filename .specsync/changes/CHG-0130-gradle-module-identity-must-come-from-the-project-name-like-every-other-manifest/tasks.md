---
change: CHG-0130-gradle-module-identity-must-come-from-the-project-name-like-every-other-manifest
artifact: tasks
---

# Tasks

1. Read `rootProject.name` for single-project builds; fall back to the project
   directory name.
2. Stop treating children of JVM source roots as modules.
3. Leave multi-project include handling alone.
4. CHANGELOG entry.
