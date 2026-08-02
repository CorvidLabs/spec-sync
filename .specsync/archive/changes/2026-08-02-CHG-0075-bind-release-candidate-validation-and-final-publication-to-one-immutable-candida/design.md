---
change: CHG-0075-bind-release-candidate-validation-and-final-publication-to-one-immutable-candida
artifact: design
---

# Design

## One release identity

```text
release/vX.Y.Z branch
        |
        +-- immutable vX.Y.Z-rc.N marker --> candidate SHA
                                                |
                         +----------------------+----------------------+
                         |                      |                      |
                    Ubuntu RC lane         macOS RC lane        Windows RC lane
                         |                      |                      |
                         +---------- exact-SHA green evidence --------+
                                                |
                                      explicit promotion gate
                                                |
                              create final vX.Y.Z tag at same SHA
                                                |
                                      build/upload release assets
```

The RC branch is a staging surface; the immutable RC marker is the authority. Any branch movement is
irrelevant until a new RC marker is created.

## Ordinary development

Ordinary development and product pull requests run the named Ubuntu integration lane. macOS and
Windows are release-qualification platforms, not per-PR integration platforms.

## RC execution

An annotated `vX.Y.Z-rc.N` marker captures the full candidate SHA. Every matrix job checks out that
SHA explicitly, runs the same named Fledge release-candidate lane, and reports a result whose identity
includes platform, marker, candidate SHA, and workflow revision.

## Promotion and publication

Promotion receives the RC marker, resolves it again, and queries the required Ubuntu/macOS/Windows
results for the resolved SHA. It fails closed unless every result is successful and bound to the same
marker and SHA. Only that successful promotion creates the final `vX.Y.Z` tag at the candidate SHA.
The release upload path independently revalidates the final tag, RC marker, checkout, and artifact
manifest before publishing.

No final tag is used as a speculative test trigger. A candidate byte change requires a new RC marker
and fresh platform evidence.

## Implementation boundary

Use one small validator with deterministic fixtures rather than duplicating SHA/path logic in YAML.
Workflow YAML should orchestrate named Fledge lanes and the validator. Required-workflow pin updates
remain part of the protected delivery procedure.
