---
id: the-release-lane-must-be-able-to-read-the-tag-that-triggered-it
state: implementing
type: bug_fix
base_commit: 89886855c6df280075582e332553744dce76d7a9
---

# The release lane must be able to read the tag that triggered it

## Intent

the release lane must be able to read the tag that triggered it

## Affected Canonical Specs

- None

## Acceptance Criteria

- The release lane refuses every annotated tag, including correct ones, and therefore has never released. resolve checks out with fetch-depth zero, which fetches history but not tag objects because fetch-tags defaults to false. A tag-triggered run then holds a lightweight local ref at the tag name even when the server tag is annotated, and resolve_annotated_rc_tag runs git cat-file against that local ref, sees commit rather than tag, and refuses with must be an annotated tag not a lightweight tag. Measured on v6.0.0-rc.1: the GitHub API reports the tag as type tag with a tagger and a message, the tag object was created at 00:05:07Z, the run it triggered started at 00:05:15Z, and that run refused it. Done when: resolve fetches tag objects so the annotation check reads the tag it was given, and the reason is recorded beside the setting so nobody removes it as redundant.

## No-spec Rationale

Workflow checkout configuration; no module contract changes and no production source in scope
