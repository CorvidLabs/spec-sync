---
id: publish-the-ci-confidence-guide-on-the-site-and-test-that-each-suite-has-one-owner
state: draft
type: documentation
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# Publish the CI confidence guide on the site and test that each suite has one owner

## Intent

Publish the CI confidence guide on the site and test that each suite has one owner

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- the CI confidence guide is a site page under Reference and docs/ci-confidence.md points to it; test-ci-lane-ownership.py passes on the repository and fails when test, lint, verify or a cargo test task is added to trust-lifecycle, when the Trust lifecycle runs another lane, or when verify drops its tests

## No-spec Rationale

REQ-github-006 already requires one authoritative workflow per expensive suite, a trust-lifecycle lane without cargo test, clippy or verify, and lanes.verify as the full local suite; this publishes the guide that explains it and makes the rule an executable test (#493)
