---
change: CHG-0015-adopt-trust-1-and-specsync-5
artifact: testing
---

# Testing

- Run the native Fledge verify lane.
- Run SpecSync strict validation at the committed threshold.
- Confirm all four agent integrations.
- Run Trust doctor and verification.
- Run the self-host policy regression to prove the lifecycle lane uses the source-built strict 100% contract gate while only this repository disables the duplicate released-binary component.
- Confirm hosted native and Trust checks pass.
