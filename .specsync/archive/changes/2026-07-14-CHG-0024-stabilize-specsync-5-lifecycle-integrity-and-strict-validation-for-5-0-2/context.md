---
change: CHG-0024-stabilize-specsync-5-lifecycle-integrity-and-strict-validation-for-5-0-2
artifact: context
---

# Context

SpecSync 5.0.1 has several fail-closed lifecycle deadlocks and two strict-validation blind spots that block truthful adoption. Current `main` also contains three records with numeric sequence `CHG-0016`: one archived record and two accepted active records created on independent branches from the same base commit.

The change allocator currently scans the local active/archive tree and chooses `maximum + 1`. Its operating-system lock serializes one checkout but cannot coordinate parallel Git branches. Validation treats the full `CHG-NNNN-slug` string as identity and therefore does not report repeated numeric sequences.

Verification currently overwrites the latest evidence and executes arbitrary safe-tokenized configured commands. A configured command that re-enters `specsync check` recursively validates the same command without a process-level cycle marker. Accepted predecessor evidence also remains stale while a valid canonical successor is attempting to govern the same module, and semantic-delta application reconstructs conventional spec paths instead of consulting the committed registry.

Coverage discovers language-backed files only, so configured static repositories can render vacuous `0/0 (100%)` output. Canonical companion files are not checked for known generated scaffold markers, allowing strict mode to report an unfinished contract as green.

Implementation now commits a sequence claim with an exact historical collision baseline, validates active and archived sequences together, prevents direct and indirect verification recursion, retains every verification attempt, recognizes only exact current canonical successors, resolves registered semantic targets safely, measures HTML/HTM/CSS sources, and diagnoses artifact-specific scaffold markers outside fenced examples. Release documentation and 5.0.2 metadata describe the new fail-closed behavior. Focused, full, strict, release-build, dependency-audit, documentation, editor-package, and local Trust/Augur verification pass; commit-keyed Attest provenance and hosted checks follow acceptance and publication.
