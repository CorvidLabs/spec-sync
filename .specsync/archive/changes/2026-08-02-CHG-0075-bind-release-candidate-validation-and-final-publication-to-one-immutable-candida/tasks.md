---
change: CHG-0075-bind-release-candidate-validation-and-final-publication-to-one-immutable-candida
artifact: tasks
---

# Tasks

- [x] Characterize current per-PR matrix and tag-first release behavior.
- [x] Add the canonical release-candidate Fledge lane.
- [x] Implement exact RC marker/SHA validation with adversarial fixtures.
- [x] Move ordinary product integration to Ubuntu-only scheduling.
- [x] Add Ubuntu/macOS/Windows RC qualification for the exact candidate SHA.
- [x] Add pre-tag promotion and fail-closed upload validation.
- [x] Correct the release/archive-binding event contract and add a regression fixture.
- [x] Pin every Action and downloaded release utility used by the protected workflow.
- [x] Dogfood live RC/final-tag ruleset behavior in `CorvidLabs/spec-sync-sandbox`.
- [x] Validate the three-policy and protected-environment contract against live sandbox API
  responses, leaving dedicated-App/production provisioning as the explicit release-activation
  prerequisite tracked in `specs/github/tasks.md`.
- [x] Run independent security/release review and one final full validation.
