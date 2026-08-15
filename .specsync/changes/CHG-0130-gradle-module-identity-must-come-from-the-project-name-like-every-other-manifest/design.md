---
change: CHG-0130-gradle-module-identity-must-come-from-the-project-name-like-every-other-manifest
artifact: design
---

# Design

Name a single-project Gradle build from `rootProject.name`, falling back to the
project directory when unset — which is Gradle's own default, not a spec-sync
invention. Multi-project builds keep using include names, which already worked.

Children of a JVM source root (`src/main/kotlin` and siblings) are not modules.
That is the line that removes the whole class: no segment of a package
hierarchy is a module name, so there is no "which segment" question left to get
wrong.

Why the fix lands in `manifest.rs` rather than the path-splitting code: the
rejected attempt edited how a segment was CHOSEN, which is why it produced a
different wrong answer. This moves Gradle onto the manifest-identity path the
other seven ecosystems already use. The file it touches is the tell that the
diagnosis changed.
