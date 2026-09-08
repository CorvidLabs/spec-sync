---
change: align-specsync-6-release-guidance-with-verified-lifecycle-and-validation-boundaries
artifact: docs
---

# Docs

Correct these seven files only:
- MIGRATION.md: require coordinated6.x lifecycle tooling for collaborators and CI; explain that older writers may reject or damage newer records and that6.x downgrade refusal is detection, not safe mixed-version support. Explain explicit SpecSync pinning and the validated runner-local mirror used by current Trust wrappers. Distinguish soft provenance from enforced verification.
- CHANGELOG.md: disclose validation boundaries and the content-versus-identity approval distinction in the6.0 unreleased notes.
- docs/ADOPTING.md: explain the same boundaries, replace the obsolete rc.2 installation example with clear stable-versus-candidate selection, retain the real single workflow and requirement for same-PR archive before merge.
- site/src/content/docs/architecture.md: describe ordinary check as structural/API/schema/dependency validation; place optional SDD checks under lifecycle commands/audit. Correct archive timing to before merge.
- site/src/content/docs/workflow.md: document one scope approval plus a human implementation review (same actor allowed), immediate review→ship, archive before merge; clearly label legacy recovery examples, remove unsupported general claims about numeric succession and stale-only legacy recovery.
- site/src/content/docs/why-specsync.md: distinguish what deterministic checks establish from manual semantic review; align lifecycle/approval wording with6.0.
- site/src/content/docs/companion-files.md: use slug paths, distinguish evidence mappings from executed tests, materialize during change check, and archive on the same PR before merge.

Core proposed wording: "A successful check validates configured structure, exported API names, source mappings, dependency declarations, and supported schema rules. It does not prove that arbitrary natural-language requirements describe the implementation; reviewers and product tests establish those behaviors."

Core proposed wording: "Approval digests bind recorded approval to content. The actor/reviewer label is not authenticated identity. Signed provenance and a required policy-verification check must be configured separately when identity or provenance enforcement is required; recording a signature or using soft mode alone is not that gate."

No workflow protections, lifecycle policy, implementation, API, canonical module contract, or test semantics change. No stable release is published by this documentation change.
