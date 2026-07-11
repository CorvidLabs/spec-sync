---
change: CHG-0014-route-archive-only-lifecycle-moves-through-a-minimal-specsync-ci-gate-while-pres
artifact: research
---

# Research

GitHub top-level `paths` filters can decide whether a workflow starts, but they
cannot distinguish an archive move from an edit because an archive commit
contains both deleted active-workspace paths and added archive paths.

Job-level classification is required. Skipped jobs are acceptable dependencies
only when the aggregate gate explicitly treats `skipped` as neutral.

CodeQL is configured through GitHub default setup rather than a repository
workflow file, so this change optimizes the repository CI workflow. Default
CodeQL may still appear separately until its GitHub settings receive equivalent
path filtering.
