---
change: pin-the-trust-gate-to-v1-2-0-rc-4
artifact: testing
---

# Testing

- specsync change audit --strict covers .github/workflows/trust.yml with this active change.
- Hosted trust job runs the same audit and the pinned Trust action against the existing 6.0.0 file:// mirror.
- No product spec or binary contract change to re-prove beyond the pin itself.
