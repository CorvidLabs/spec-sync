---
id: the-quickstart-and-two-example-pages-read-as-published-for-6-4-2
state: archived
type: documentation
base_commit: 9caed41498939ad7fdf6d74c2dd1c4eab3413bf0
---

# The quickstart and two example pages read as published for 6.4.2

## Intent

The quickstart and two example pages read as published for 6.4.2

## Affected Canonical Specs

- None

## Acceptance Criteria

- site/src/content/docs/quickstart.md and site/src/content/examples/polyglot.mdx lose the sentence 'Use these pins after the 6.4.2 assets are published. Until then, pin @v6.4.1 and version: 6.4.1.' In site/src/content/examples/ci-gate.mdx the sentence 'After the 6.4.2 assets are published, pin both the Action ref and the binary. Until then, pass the explicit 6.4.1 ref and binary pins.' becomes 'Pin both the Action ref and the binary.', as it read before the 6.4.2 preparation. Nothing else on those pages changes, and no page under site/ or docs/ still says 'until then'.

## No-spec Rationale

Three sentences of documentation wording; the technical contract does not change
