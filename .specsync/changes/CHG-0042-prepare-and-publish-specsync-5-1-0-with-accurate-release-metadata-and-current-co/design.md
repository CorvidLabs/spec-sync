---
change: CHG-0042-prepare-and-publish-specsync-5-1-0-with-accurate-release-metadata-and-current-co
artifact: design
---

# Design

Keep release metadata minimal and explicit:

- package identity is sourced from `Cargo.toml` and synchronized into `Cargo.lock`;
- the changelog owns the human-readable release boundary and release comparison URL;
- the Trust workflow pins the released binary version independently of the package;
- comparison pages use the same capability vocabulary: solo UX, orchestration,
  deterministic enforcement, governance/evidence, and durable contract truth.

The comparison documentation presents SpecSync as a deterministic contract and
evidence layer. Spec Kit is the broad planning/workflow ecosystem, OpenSpec is the
lightweight brownfield delta experience, and BMAD is the full product-development
method. Scores and popularity claims are intentionally omitted from product docs;
only verifiable capability boundaries and official sources are retained.
