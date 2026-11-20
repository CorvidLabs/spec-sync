---
change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
artifact: research
---

# Research

- The starling report (chough#299): a truncated `requirements.md` passed every gate.
- `src/change/collect_requirement_ids.rs`: `canonical_module_paths` resolves `requirements.md` beside each spec; `ensure_approved_delta_bases_current` is called at check (`sdd_version.rs`), acceptance and audit (`historical_definition_digest_matches.rs`).
- `src/validator/requirement_ids.rs`: `requirement_heading_id` is crate-visible; its fence tracking is private, and the change module has its own `fence_mark`.
- `src/hash_cache.rs` and `src/merge_driver.rs` already treat a legacy `<module>.req.md` as a module's requirements.
- `base_commit` is HEAD at `change new` (`sdd_version.rs:2054`).
