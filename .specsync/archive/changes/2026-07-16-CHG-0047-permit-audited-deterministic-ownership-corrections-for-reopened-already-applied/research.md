---
change: CHG-0047-permit-audited-deterministic-ownership-corrections-for-reopened-already-applied
artifact: research
---

# Research

The existing alternatives are unsafe or incomplete:

- Editing `affected_specs` changes the approved semantic definition and can require a new delta;
  the already-applied acceptance guard correctly rejects it.
- Adding a source file to an unrelated canonical spec fabricates shared ownership and stales other
  accepted evidence.
- A successor cannot repair a predecessor that is already reopened and therefore not accepted.
- Treating production source as generic delivery metadata would weaken every acceptance manifest.
- Extending boolean metadata corrections conflates semantic interview state with exact file
  provenance.

An exact, additive, manifest-only ownership correction preserves all existing fail-closed rules. It
can prove legitimacy from the current canonical spec, is bounded to an original affected path, and
is signed by the ordinary definition and closing gates. Removing the correction vector reproduces
the prior definition, giving the reacceptance guard a deterministic compatibility proof.
