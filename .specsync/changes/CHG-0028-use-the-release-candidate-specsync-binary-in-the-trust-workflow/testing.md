---
change: CHG-0028-use-the-release-candidate-specsync-binary-in-the-trust-workflow
artifact: testing
---

# Testing

- Validate the workflow structure with the repository's action validation task.
- Run the local Trust gate against the release-candidate SpecSync binary.
- Confirm hosted `trust` installs the checksum-verified runner-local candidate and passes lifecycle, contract, risk, and provenance gates.
- Confirm all other pull-request checks remain green.
