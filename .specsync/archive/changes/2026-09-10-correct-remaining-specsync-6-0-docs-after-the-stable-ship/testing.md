---
change: correct-remaining-specsync-6-0-docs-after-the-stable-ship
artifact: testing
---

# Testing

- `specsync check --strict github --force` passes.
- `.github/scripts/validate-release-candidate.py` `REQUIRED_PLATFORMS = ("ubuntu", "macos")`.
- `.github/workflows/release.yml` qualify matrix is `ubuntu-latest` and `macos-14` only.

## Requirement evidence

| ID | Evidence |
|----|----------|
| REQ-github-007 | Canonical requirement, github.spec.md promote scenario, testing.md RC rows, and context.md now bind Ubuntu and macOS only. `REQUIRED_PLATFORMS` is `("ubuntu", "macos")`. Windows is refused by the Action and is not a qualify target. |
