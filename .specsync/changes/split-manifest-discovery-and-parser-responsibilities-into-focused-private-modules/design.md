---
change: split-manifest-discovery-and-parser-responsibilities-into-focused-private-modules
artifact: design
---

# Design

Keep manifest.rs as the stable public/type/discovery facade with unchanged fixed parse order and merge priority. Private access.rs owns ManifestAccess and retained/ambient implementations, cumulative limits and retained directory listings. identity.rs owns shared native identity and no-follow primitives. cargo.rs, swift.rs, node.rs and simple_manifests.rs own ecosystem discovery. gradle.rs retains discovery, root naming and effective-directory interpretation; gradle/access.rs owns bounded retained reads/checkpoints; gradle/syntax.rs owns comment, literal, parenthesis and argument helpers; gradle/guards.rs owns executable/directive/control-flow checks. Prefer restricted parent visibility and existing API wrappers; no additional crate APIs. Topical tests cover ecosystem discovery, Gradle includes/composites, projectDir/control flow, confinement/races and retained non-Gradle descriptors.
