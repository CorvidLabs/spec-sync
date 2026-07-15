---
change: CHG-0043-make-accepted-change-validity-successor-aware-with-exact-per-input-evidence-rec
artifact: docs
---

# Docs

Update the change canonical contract to expose the optional per-input evidence carried by `VerificationRecord` and to define terminal successor validity, legacy reconstruction, archive discovery, and shared lifecycle conclusions.

Update the command contract with supported pre-approval `change supersede` adoption. Public status documentation distinguishes active accepted current-input validity (`exact`, `successor-covered`, `stale`) from archived historical integrity (`authenticated-history`, `corrupt-history`). Public documentation must state that archive integrity does not require immutable equality with today's exact-only delivery files, but an archive selected as a semantic successor must additionally pass recursive current-input validation. Only accepted or candidate-valid archived semantic successors with explicit approved predecessor/path/module/digest bindings can govern changed historical inputs; in-progress work remains visible but cannot make a stale accepted gate green.

Document the baseline's two authority phases: definition-approved exact-ledger bootstrap before CHG43 acceptance, followed by mandatory manifest-backed closing/history authority after acceptance. This compatibility path authenticates immutable stored history only; it never proves current inputs or semantic succession, never trusts PR-only or post-cutoff commits, never applies to modern manifest archives, and fails on any ledger digest, ordering, authority, subtree, mode, type, or trust-root ambiguity.

No version, release note, tag, or mutable channel is created by CHG43.
